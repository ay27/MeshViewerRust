use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};

use std::collections::HashMap;
use std::rc::Rc;

use crate::resources::{
    AnimationClipData, AnimationData, AppSettings, BoneInfo, BoneTrack, FileState, LoadingStage, LoadingState,
    MaterialData, MaterialInfo, MeshMaterialEntry, MeshSkinningData, ModelLoadError,
    ModelLoadRequest, ModelLoaded, ModelStats, UiAction, VertexBoneInfluence,
};

/// Marker component for entities that belong to the currently loaded model.
#[derive(Component)]
pub struct LoadedModel;

/// Stores the mesh name from the source file for display purposes.
#[derive(Component, Clone)]
pub struct MeshName(pub String);

/// Plugin that handles loading 3D model files via russimp.
pub struct ModelLoaderPlugin;

impl Plugin for ModelLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_load_requests);
    }
}

// NOTE: Lighting (Studio key/fill/rim + AmbientLight) lives in
// camera.rs::setup_camera, spawned as CHILDREN of the camera entity so that
// the lights track the camera's view space — matching Blender's Solid mode
// Studio Lighting (view-space anchored, not world-space).

fn handle_load_requests(
    mut commands: Commands,
    mut load_requests: MessageReader<ModelLoadRequest>,
    mut loaded_events: MessageWriter<ModelLoaded>,
    mut error_events: MessageWriter<ModelLoadError>,
    mut ui_actions: MessageWriter<UiAction>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut model_stats: ResMut<ModelStats>,
    mut material_data: ResMut<MaterialData>,
    mut animation_data: ResMut<AnimationData>,
    settings: Res<AppSettings>,
    mut file_state: ResMut<FileState>,
    mut loading_state: ResMut<LoadingState>,
    existing_models: Query<Entity, With<LoadedModel>>,
) {
    for request in load_requests.read() {
        let path = &request.path;
        info!("Loading model: {:?}", path);

        // Update loading state
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        loading_state.is_loading = true;
        loading_state.file_name = file_name.clone();
        loading_state.stage = LoadingStage::Reading;
        loading_state.progress = 0.1;
        animation_data.reset();

        // Despawn old model entities
        for entity in existing_models.iter() {
            commands.entity(entity).despawn();
        }

        // Attempt to load with russimp
        loading_state.stage = LoadingStage::Parsing;
        loading_state.progress = 0.3;

        let path_str = path.to_string_lossy().to_string();
        let scene_result = russimp::scene::Scene::from_file(
            &path_str,
            vec![
                russimp::scene::PostProcess::Triangulate,
                // Critical for PBR shading: if the source file has no normals
                // (common in OBJ / many CAD exports), N dot L is 0 and the mesh
                // renders as a flat colored blob. JoinIdenticalVertices first
                // so smooth normals get averaged across shared vertices.
                russimp::scene::PostProcess::JoinIdenticalVertices,
                russimp::scene::PostProcess::GenerateSmoothNormals,
            ],
        );

        match scene_result {
            Ok(scene) => {
                loading_state.stage = LoadingStage::Building;
                loading_state.progress = 0.6;

                let mut total_vertices: u64 = 0;
                let mut total_faces: u64 = 0;
                let mut total_renderable_triangles: u64 = 0;
                let mut total_skipped_non_tri_faces: u64 = 0;
                let mut material_infos: Vec<MaterialInfo> = Vec::new();
                let mut mesh_material_map: Vec<MeshMaterialEntry> = Vec::new();

                // Extract materials
                for (mat_idx, russimp_mat) in scene.materials.iter().enumerate() {
                    let mat_info = russimp_to_material_info(russimp_mat, mat_idx);
                    material_infos.push(mat_info);
                }

                // Step 1: Traverse node tree for mesh world transforms + node hierarchy.
                let scene_graph = collect_scene_graph_data(&scene);
                let (bbox_min, bbox_max) = calculate_scene_bbox_world(&scene, &scene_graph);
                let display_normalization = calculate_display_normalization(
                    settings.common.normalize_imported_model,
                    bbox_min,
                    bbox_max,
                );
                let mut bone_lookup: HashMap<String, usize> = HashMap::new();
                let mut skeleton: Vec<BoneInfo> = Vec::new();
                let mut rest_global_transforms: Vec<Mat4> = Vec::new();
                let mut rest_local_transforms: Vec<Mat4> = Vec::new();
                let mut mesh_skinning_data: Vec<MeshSkinningData> = Vec::new();

                // Step 2: Convert and spawn meshes. Keep mesh vertices in local space;
                // apply node world transform to entity Transform.
                for (mesh_idx, russimp_mesh) in scene.meshes.iter().enumerate() {
                    // Get the world transform for this mesh (by index)
                    let source_world_transform = scene_graph
                        .mesh_to_transform
                        .get(&mesh_idx)
                        .copied()
                        .unwrap_or(Mat4::IDENTITY);
                    let display_world_transform = display_normalization * source_world_transform;

                    let positions: Vec<[f32; 3]> = russimp_mesh
                        .vertices
                        .iter()
                        .map(|v| [v.x, v.y, v.z])
                        .collect();

                    let normals: Vec<[f32; 3]> = if !russimp_mesh.normals.is_empty() {
                        russimp_mesh
                            .normals
                            .iter()
                            .map(|n| [n.x, n.y, n.z])
                            .collect()
                    } else {
                        Vec::new()
                    };

                    // Build Bevy mesh in local space.
                    let (bevy_mesh, tri_face_count, skipped_face_count) = build_bevy_mesh_transformed(
                        russimp_mesh,
                        &positions,
                        &normals,
                    );
                    total_renderable_triangles += tri_face_count as u64;
                    total_skipped_non_tri_faces += skipped_face_count as u64;

                    total_vertices += russimp_mesh.vertices.len() as u64;
                    total_faces += russimp_mesh.faces.len() as u64;

                    // Get material for this mesh
                    let mat_index = russimp_mesh.material_index as usize;
                    let bevy_material = if mat_index < material_infos.len() {
                        let info = &material_infos[mat_index];
                        StandardMaterial {
                            base_color: Color::linear_rgba(
                                info.base_color[0],
                                info.base_color[1],
                                info.base_color[2],
                                info.base_color[3],
                            ),
                            metallic: info.metallic,
                            perceptual_roughness: info.roughness,
                            double_sided: true,
                            cull_mode: None,
                            ..default()
                        }
                    } else {
                        StandardMaterial {
                            base_color: Color::linear_rgb(0.8, 0.8, 0.8),
                            metallic: 0.0,
                            perceptual_roughness: 0.5,
                            double_sided: true,
                            cull_mode: None,
                            ..default()
                        }
                    };

                    info!(
                        "Mesh[{}] '{}': {} vertices, {} faces (triangles={}, skipped_non_tri={}), display_world_transform={:?}",
                        mesh_idx, russimp_mesh.name,
                        russimp_mesh.vertices.len(),
                        russimp_mesh.faces.len(),
                        tri_face_count,
                        skipped_face_count,
                        display_world_transform
                    );

                    let mesh_handle = meshes.add(bevy_mesh);
                    let material_handle = materials.add(bevy_material);

                    // Spawn with identity transform since vertices are already
                    // world-space transformed (matching Python behavior)
                    let mesh_name = if russimp_mesh.name.is_empty() {
                        format!("Mesh_{}", mesh_idx)
                    } else {
                        russimp_mesh.name.clone()
                    };
                    let entity = commands.spawn((
                        Mesh3d(mesh_handle),
                        MeshMaterial3d(material_handle),
                        Transform::from_matrix(display_world_transform),
                        LoadedModel,
                        MeshName(mesh_name),
                    )).id();
                    info!("Spawned model entity: {:?}", entity);

                    animation_data
                        .mesh_original_positions
                        .insert(entity, positions.clone());
                    if !normals.is_empty() {
                        animation_data
                            .mesh_original_normals
                            .insert(entity, normals.clone());
                    }

                    // Parse skinning weights for this mesh.
                    let mut vertex_influences =
                        vec![VertexBoneInfluence::default(); russimp_mesh.vertices.len()];
                    for bone in &russimp_mesh.bones {
                        let bone_name = bone.name.clone();
                        let bone_index = *bone_lookup.entry(bone_name.clone()).or_insert_with(|| {
                            let idx = skeleton.len();
                            skeleton.push(BoneInfo {
                                name: bone_name.clone(),
                                parent: None,
                                inverse_bind: russimp_matrix_to_mat4(&bone.offset_matrix),
                            });
                            rest_global_transforms.push(Mat4::IDENTITY);
                            rest_local_transforms.push(Mat4::IDENTITY);
                            idx
                        });
                        skeleton[bone_index].inverse_bind = russimp_matrix_to_mat4(&bone.offset_matrix);

                        for vw in &bone.weights {
                            let vertex_id = vw.vertex_id as usize;
                            if vertex_id < vertex_influences.len() {
                                insert_vertex_influence(
                                    &mut vertex_influences[vertex_id],
                                    bone_index as u16,
                                    vw.weight,
                                );
                            }
                        }
                    }
                    normalize_vertex_influences(&mut vertex_influences);
                    mesh_skinning_data.push(MeshSkinningData {
                        entity,
                        vertex_influences,
                        mesh_world_transform: display_world_transform,
                    });

                    // Track mesh-material mapping
                    mesh_material_map.push(MeshMaterialEntry {
                        mesh_name: russimp_mesh.name.clone(),
                        mesh_index: mesh_idx,
                        material_indices: vec![mat_index],
                    });
                }

                // Some rigs keep non-deforming root/controller nodes outside mesh.bones.
                // Pull in ancestor nodes so the skeleton tree/lines include true roots.
                let deform_bone_names: Vec<String> = bone_lookup.keys().cloned().collect();
                for bone_name in deform_bone_names {
                    ensure_ancestor_bones(
                        &bone_name,
                        &scene_graph,
                        &mut bone_lookup,
                        &mut skeleton,
                        &mut rest_global_transforms,
                        &mut rest_local_transforms,
                    );
                }

                for bone in &mut skeleton {
                    if let Some(global) = scene_graph.node_world.get(&bone.name) {
                        let idx = *bone_lookup.get(&bone.name).unwrap_or(&0);
                        rest_global_transforms[idx] = *global;
                    }
                    if let Some(local) = scene_graph.node_local.get(&bone.name) {
                        let idx = *bone_lookup.get(&bone.name).unwrap_or(&0);
                        rest_local_transforms[idx] = *local;
                    }
                    bone.parent = find_parent_bone_index(
                        &bone.name,
                        &scene_graph.node_parent,
                        &bone_lookup,
                    );
                }

                let clips = parse_animation_clips(&scene);
                if !clips.is_empty() {
                    info!("Parsed {} animation clip(s)", clips.len());
                }

                animation_data.skeleton = skeleton;
                animation_data.bone_lookup = bone_lookup;
                animation_data.rest_global_transforms = rest_global_transforms;
                animation_data.rest_local_transforms = rest_local_transforms;
                animation_data.mesh_skinning = mesh_skinning_data;
                animation_data.clips = clips;
                animation_data.current_time = 0.0;
                animation_data.active_clip = 0;
                animation_data.loop_playback = true;
                animation_data.model_display_transform = display_normalization;

                // Calculate bounding box size
                let bbox_size = if bbox_min.x <= bbox_max.x {
                    let size = bbox_max - bbox_min;
                    [size.x / 100.0, size.y / 100.0, size.z / 100.0]
                } else {
                    [0.0, 0.0, 0.0]
                };

                // Update ModelStats
                let stats = ModelStats {
                    num_meshes: scene.meshes.len() as u32,
                    num_vertices: total_vertices,
                    num_faces: total_faces,
                    bbox_size,
                    num_materials: scene.materials.len() as u32,
                    num_textures: 0, // TODO: count textures
                };
                *model_stats = stats.clone();

                // Update MaterialData
                *material_data = MaterialData {
                    materials: material_infos.clone(),
                    mesh_material_map,
                };

                // Update FileState
                file_state.current_file_path = Some(path.clone());
                file_state.file_name = Some(file_name.clone());
                if let Some(parent) = path.parent() {
                    file_state.current_directory = Some(parent.to_path_buf());
                }

                // Update loading state
                loading_state.is_loading = false;
                loading_state.progress = 1.0;
                loading_state.stage = LoadingStage::Ready;

                // Send loaded event
                loaded_events.write(ModelLoaded {
                    stats,
                    materials: material_infos,
                });

                info!(
                    "Model loaded: {} meshes, {} vertices, {} faces (triangles={}, skipped_non_tri={})",
                    scene.meshes.len(),
                    total_vertices,
                    total_faces,
                    total_renderable_triangles,
                    total_skipped_non_tri_faces
                );

                if total_vertices > 0 && total_renderable_triangles == 0 {
                    warn!(
                        "No renderable triangle faces after import. \
This file may be point-cloud/line-only data (common in some OBJ/PLY). \
Current pipeline renders triangle meshes only. file={:?}, vertices={}, faces={}",
                        path,
                        total_vertices,
                        total_faces
                    );
                }

                // Auto fit-to-view after loading
                ui_actions.write(UiAction::FitToView);
            }
            Err(err) => {
                let error_msg = format!("Failed to load model {:?}: {}", path, err);
                error!("{}", error_msg);

                loading_state.is_loading = false;
                loading_state.stage = LoadingStage::Error(error_msg.clone());
                loading_state.progress = 0.0;

                error_events.write(ModelLoadError { error: error_msg });
            }
        }
    }
}

#[derive(Default)]
struct SceneGraphData {
    mesh_to_transform: HashMap<usize, Mat4>,
    node_world: HashMap<String, Mat4>,
    node_local: HashMap<String, Mat4>,
    node_parent: HashMap<String, Option<String>>,
}

/// Traverse scene nodes and collect world transforms for meshes and named nodes.
fn collect_scene_graph_data(scene: &russimp::scene::Scene) -> SceneGraphData {
    let mut data = SceneGraphData::default();

    fn traverse_node(
        node: &Rc<russimp::node::Node>,
        parent_name: Option<String>,
        parent_transform: Mat4,
        data: &mut SceneGraphData,
    ) {
        let local_transform = russimp_matrix_to_mat4(&node.transformation);
        let world_transform = parent_transform * local_transform;
        data.node_world.insert(node.name.clone(), world_transform);
        data.node_local.insert(node.name.clone(), local_transform);
        data.node_parent.insert(node.name.clone(), parent_name);

        for &mesh_index in &node.meshes {
            data.mesh_to_transform.insert(mesh_index as usize, world_transform);
        }

        for child in node.children.borrow().iter() {
            traverse_node(
                child,
                Some(node.name.clone()),
                world_transform,
                data,
            );
        }
    }

    if let Some(ref root) = scene.root {
        traverse_node(root, None, Mat4::IDENTITY, &mut data);
    }
    data
}

fn calculate_scene_bbox_world(scene: &russimp::scene::Scene, scene_graph: &SceneGraphData) -> (Vec3, Vec3) {
    let mut bbox_min = Vec3::splat(f32::MAX);
    let mut bbox_max = Vec3::splat(f32::MIN);

    for (mesh_idx, russimp_mesh) in scene.meshes.iter().enumerate() {
        let world_transform = scene_graph
            .mesh_to_transform
            .get(&mesh_idx)
            .copied()
            .unwrap_or(Mat4::IDENTITY);
        for v in &russimp_mesh.vertices {
            let local = Vec3::new(v.x, v.y, v.z);
            let world = world_transform.transform_point3(local);
            bbox_min = bbox_min.min(world);
            bbox_max = bbox_max.max(world);
        }
    }

    (bbox_min, bbox_max)
}

fn calculate_display_normalization(
    enabled: bool,
    bbox_min: Vec3,
    bbox_max: Vec3,
) -> Mat4 {
    if !enabled || bbox_min.x > bbox_max.x {
        return Mat4::IDENTITY;
    }

    let size = bbox_max - bbox_min;
    let max_dim = size.max_element();
    if max_dim <= f32::EPSILON {
        return Mat4::IDENTITY;
    }

    let center = (bbox_min + bbox_max) * 0.5;
    // Scene unit is centimeter; 10m target size => 1000cm.
    let target_extent = 1000.0;
    let uniform_scale = target_extent / max_dim;
    Mat4::from_scale_rotation_translation(
        Vec3::splat(uniform_scale),
        Quat::IDENTITY,
        -center * uniform_scale,
    )
}

fn find_parent_bone_index(
    bone_name: &str,
    node_parent: &HashMap<String, Option<String>>,
    bone_lookup: &HashMap<String, usize>,
) -> Option<usize> {
    let mut current = node_parent.get(bone_name).and_then(|v| v.clone());
    while let Some(parent_name) = current {
        if let Some(idx) = bone_lookup.get(&parent_name) {
            return Some(*idx);
        }
        current = node_parent.get(&parent_name).and_then(|v| v.clone());
    }
    None
}

fn insert_vertex_influence(influence: &mut VertexBoneInfluence, bone_index: u16, weight: f32) {
    let mut min_slot = 0usize;
    let mut min_weight = influence.weights[0];
    for i in 1..4 {
        if influence.weights[i] < min_weight {
            min_weight = influence.weights[i];
            min_slot = i;
        }
    }
    if weight > min_weight {
        influence.indices[min_slot] = bone_index;
        influence.weights[min_slot] = weight;
    }
}

fn normalize_vertex_influences(influences: &mut [VertexBoneInfluence]) {
    for influence in influences {
        let sum: f32 = influence.weights.iter().sum();
        if sum > 1e-6 {
            for w in &mut influence.weights {
                *w /= sum;
            }
        }
    }
}

fn ensure_ancestor_bones(
    bone_name: &str,
    scene_graph: &SceneGraphData,
    bone_lookup: &mut HashMap<String, usize>,
    skeleton: &mut Vec<BoneInfo>,
    rest_global_transforms: &mut Vec<Mat4>,
    rest_local_transforms: &mut Vec<Mat4>,
) {
    let mut chain: Vec<String> = Vec::new();
    let mut current = scene_graph
        .node_parent
        .get(bone_name)
        .and_then(|parent| parent.clone());

    while let Some(parent_name) = current {
        if bone_lookup.contains_key(&parent_name) {
            break;
        }
        chain.push(parent_name.clone());
        current = scene_graph
            .node_parent
            .get(&parent_name)
            .and_then(|parent| parent.clone());
    }

    for name in chain.into_iter().rev() {
        let idx = skeleton.len();
        bone_lookup.insert(name.clone(), idx);
        skeleton.push(BoneInfo {
            name: name.clone(),
            parent: None,
            inverse_bind: Mat4::IDENTITY,
        });
        rest_global_transforms.push(*scene_graph.node_world.get(&name).unwrap_or(&Mat4::IDENTITY));
        rest_local_transforms.push(*scene_graph.node_local.get(&name).unwrap_or(&Mat4::IDENTITY));
    }
}

/// Convert russimp Matrix4x4 to Bevy Mat4.
/// russimp stores the matrix in row-major order:
///   row 0: a1 a2 a3 a4
///   row 1: b1 b2 b3 b4
///   row 2: c1 c2 c3 c4
///   row 3: d1 d2 d3 d4
/// Bevy Mat4::from_cols_array expects column-major, so we transpose.
fn russimp_matrix_to_mat4(m: &russimp::Matrix4x4) -> Mat4 {
    // Column-major: each column is (row0_col, row1_col, row2_col, row3_col)
    Mat4::from_cols_array(&[
        m.a1, m.b1, m.c1, m.d1, // column 0
        m.a2, m.b2, m.c2, m.d2, // column 1
        m.a3, m.b3, m.c3, m.d3, // column 2
        m.a4, m.b4, m.c4, m.d4, // column 3
    ])
}

/// Build a Bevy Mesh from a russimp mesh using pre-transformed vertices and normals.
fn build_bevy_mesh_transformed(
    russimp_mesh: &russimp::mesh::Mesh,
    transformed_positions: &[[f32; 3]],
    transformed_normals: &[[f32; 3]],
) -> (Mesh, usize, usize) {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Positions (already world-space transformed)
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, transformed_positions.to_vec());

    // Normals (already world-space transformed)
    if !transformed_normals.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, transformed_normals.to_vec());
    }

    // UVs (first channel only — not affected by spatial transform)
    if let Some(Some(tex_coords)) = russimp_mesh.texture_coords.first() {
        let uvs: Vec<[f32; 2]> = tex_coords.iter().map(|uv| [uv.x, uv.y]).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }

    // Indices
    let mut indices: Vec<u32> = Vec::new();
    let mut tri_face_count = 0usize;
    let mut skipped_face_count = 0usize;
    for face in &russimp_mesh.faces {
        // Only add triangulated faces (3 indices)
        if face.0.len() == 3 {
            indices.extend_from_slice(&face.0);
            tri_face_count += 1;
        } else {
            skipped_face_count += 1;
        }
    }
    if !indices.is_empty() {
        mesh.insert_indices(Indices::U32(indices));
    }

    // Fallback: if the source had no normals AND russimp's GenerateSmoothNormals
    // didn't kick in for some reason, compute them on the Bevy side so PBR
    // shading produces visible surface detail instead of a flat blob.
    // compute_smooth_normals requires indexed geometry, so guard on that.
    if mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_none() && mesh.indices().is_some() {
        mesh.compute_smooth_normals();
    }

    (mesh, tri_face_count, skipped_face_count)
}

fn parse_animation_clips(scene: &russimp::scene::Scene) -> Vec<AnimationClipData> {
    let mut clips = Vec::new();
    for (idx, anim) in scene.animations.iter().enumerate() {
        let ticks_per_second = if anim.ticks_per_second.abs() < f64::EPSILON {
            25.0
        } else {
            anim.ticks_per_second
        };
        let duration = (anim.duration / ticks_per_second) as f32;
        let mut tracks: HashMap<String, BoneTrack> = HashMap::new();

        for channel in &anim.channels {
            let translation_times: Vec<f32> = channel
                .position_keys
                .iter()
                .map(|k| (k.time / ticks_per_second) as f32)
                .collect();
            let translations: Vec<Vec3> = channel
                .position_keys
                .iter()
                .map(|k| Vec3::new(k.value.x, k.value.y, k.value.z))
                .collect();

            let rotation_times: Vec<f32> = channel
                .rotation_keys
                .iter()
                .map(|k| (k.time / ticks_per_second) as f32)
                .collect();
            let rotations: Vec<Quat> = channel
                .rotation_keys
                .iter()
                .map(|k| Quat::from_xyzw(k.value.x, k.value.y, k.value.z, k.value.w))
                .collect();

            let scale_times: Vec<f32> = channel
                .scaling_keys
                .iter()
                .map(|k| (k.time / ticks_per_second) as f32)
                .collect();
            let scales: Vec<Vec3> = channel
                .scaling_keys
                .iter()
                .map(|k| Vec3::new(k.value.x, k.value.y, k.value.z))
                .collect();

            tracks.insert(
                channel.name.clone(),
                BoneTrack {
                    translation_times,
                    translations,
                    rotation_times,
                    rotations,
                    scale_times,
                    scales,
                },
            );
        }

        clips.push(AnimationClipData {
            name: if anim.name.is_empty() {
                format!("Anim_{}", idx)
            } else {
                anim.name.clone()
            },
            duration: duration.max(0.001),
            tracks,
        });
    }
    clips
}

/// Extract material information from a russimp material.
fn russimp_to_material_info(
    russimp_mat: &russimp::material::Material,
    index: usize,
) -> MaterialInfo {
    let mut info = MaterialInfo::default();
    info.name = format!("Material_{}", index);

    // Extract properties from the russimp material
    for prop in &russimp_mat.properties {
        match prop.key.as_str() {
            "?mat.name" => {
                if let russimp::material::PropertyTypeInfo::String(ref name) = prop.data {
                    info.name = name.clone();
                }
            }
            "$clr.diffuse" => {
                if let russimp::material::PropertyTypeInfo::FloatArray(ref values) = prop.data {
                    if values.len() >= 3 {
                        info.base_color[0] = values[0];
                        info.base_color[1] = values[1];
                        info.base_color[2] = values[2];
                        if values.len() >= 4 {
                            info.base_color[3] = values[3];
                        }
                    }
                }
            }
            "$mat.roughnessFactor" | "$mat.roughness" => {
                if let russimp::material::PropertyTypeInfo::FloatArray(ref values) = prop.data {
                    if !values.is_empty() {
                        info.roughness = values[0];
                    }
                }
            }
            "$mat.metallicFactor" | "$mat.metallic" => {
                if let russimp::material::PropertyTypeInfo::FloatArray(ref values) = prop.data {
                    if !values.is_empty() {
                        info.metallic = values[0];
                    }
                }
            }
            _ => {}
        }
    }

    // Check for textures
    info.has_base_color_texture = russimp_mat
        .textures
        .contains_key(&russimp::material::TextureType::Diffuse);
    info.has_normal_texture = russimp_mat
        .textures
        .contains_key(&russimp::material::TextureType::Normals);
    info.has_roughness_texture = russimp_mat
        .textures
        .contains_key(&russimp::material::TextureType::Shininess);

    info
}
