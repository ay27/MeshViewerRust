use bevy::prelude::*;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use std::collections::HashMap;

use crate::plugins::model_loader::{LoadedModel, MeshName};
use crate::resources::{
    AnimationData, AppSettings, BoneInfo, ComponentColorEntry, ComponentColorMap, RenderMode,
    RenderModeState,
};

/// Plugin that handles switching between render modes, background color, and gizmos.
pub struct RenderModePlugin;

impl Plugin for RenderModePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin::default())
            .init_resource::<ComponentColorMap>()
            .add_systems(
                Update,
                capture_original_materials,
            )
            .add_systems(
                Update,
                apply_render_mode.run_if(
                    resource_changed::<RenderModeState>.or(
                        |query: Query<(), Added<OriginalMaterial>>| !query.is_empty()
                    )
                ),
            )
            .add_systems(Update, sync_background_color.run_if(resource_changed::<AppSettings>))
            .add_systems(Startup, init_background_color)
            .add_systems(
                Update,
                (
                    draw_ground_grid,
                    draw_axis_gizmos,
                    draw_skeleton_cubes,
                ),
            );
    }
}

/// Stores the original material properties so we can restore them on mode switch.
#[derive(Component, Clone)]
pub struct OriginalMaterial {
    pub base_color: Color,
    pub metallic: f32,
    pub perceptual_roughness: f32,
    pub alpha_mode: AlphaMode,
    pub unlit: bool,
    pub double_sided: bool,
    pub cull_mode: Option<bevy::render::render_resource::Face>,
}

#[derive(Component)]
struct BoneSegmentVisual;

/// System that captures original material properties for newly spawned model entities.
fn capture_original_materials(
    mut commands: Commands,
    std_materials: Res<Assets<StandardMaterial>>,
    new_models: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        (With<LoadedModel>, Without<OriginalMaterial>),
    >,
) {
    for (entity, mat_handle) in new_models.iter() {
        if let Some(material) = std_materials.get(&mat_handle.0) {
            commands.entity(entity).insert(OriginalMaterial {
                base_color: material.base_color,
                metallic: material.metallic,
                perceptual_roughness: material.perceptual_roughness,
                alpha_mode: material.alpha_mode,
                unlit: material.unlit,
                double_sided: material.double_sided,
                cull_mode: material.cull_mode,
            });
        }
    }
}

/// Set the initial ClearColor from settings on startup.
fn init_background_color(mut commands: Commands, settings: Res<AppSettings>) {
    let bg = &settings.common.background_color;
    commands.insert_resource(ClearColor(Color::srgb(bg[0], bg[1], bg[2])));
}

/// Keep ClearColor in sync when AppSettings changes.
fn sync_background_color(settings: Res<AppSettings>, mut clear_color: ResMut<ClearColor>) {
    let bg = &settings.common.background_color;
    let new_color = Color::srgb(bg[0], bg[1], bg[2]);
    clear_color.0 = new_color;
}

/// Draw XYZ axis gizmos at the world origin.
fn draw_axis_gizmos(mut gizmos: Gizmos, settings: Res<AppSettings>) {
    if !settings.common.show_axis_gizmos {
        return;
    }

    let length = 100000.0;
    gizmos.line(Vec3::new(-length, 0.0, 0.0), Vec3::new(length, 0.0, 0.0), Color::linear_rgb(1.0, 0.1, 0.1));

    gizmos.line(Vec3::new(0.0, -length, 0.0), Vec3::new(0.0, length, 0.0), Color::linear_rgb(0.1, 1.0, 0.1));

    gizmos.line(Vec3::new(0.0, 0.0, -length), Vec3::new(0.0, 0.0, length), Color::linear_rgb(0.1, 0.1, 1.0));
}

/// Draw a grid on the XZ plane (Y=0) similar to Blender's ground grid.
fn draw_ground_grid(mut gizmos: Gizmos, settings: Res<AppSettings>) {
    if !settings.common.show_ground_grid {
        return;
    }

    let grid_size = 1000;
    let step = 100.0_f32;
    let y = 0.0;
    let color = Color::linear_rgba(0.2, 0.2, 0.2, 0.5);
    let center_color = Color::linear_rgba(0.4, 0.4, 0.4, 0.7);

    for i in -grid_size..=grid_size {
        if i == 0 {
            continue;
        }
        let offset = i as f32 * step;
        let c = if i == 0 { center_color } else { color };
        gizmos.line(
            Vec3::new(-grid_size as f32 * step, y, offset),
            Vec3::new(grid_size as f32 * step, y, offset),
            c,
        );
        gizmos.line(
            Vec3::new(offset, y, -grid_size as f32 * step),
            Vec3::new(offset, y, grid_size as f32 * step),
            c,
        );
    }
}

/// Restore a material to its original state.
fn restore_material(material: &mut StandardMaterial, original: &OriginalMaterial) {
    material.base_color = original.base_color;
    material.metallic = original.metallic;
    material.perceptual_roughness = original.perceptual_roughness;
    material.alpha_mode = original.alpha_mode;
    material.unlit = original.unlit;
    material.double_sided = original.double_sided;
    material.cull_mode = original.cull_mode;
}

/// Generate a visually distinct color for the given index using golden ratio hue distribution.
fn component_color(index: usize) -> Color {
    let golden_ratio: f32 = 0.618033988749895;
    let hue: f32 = (index as f32 * golden_ratio) % 1.0;
    let saturation: f32 = 0.7;
    let lightness: f32 = 0.6;

    // HSL to RGB conversion
    let c: f32 = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let h: f32 = hue * 6.0;
    let x: f32 = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m: f32 = lightness - c / 2.0;

    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::srgb(r + m, g + m, b + m)
}

fn sample_vec3(times: &[f32], values: &[Vec3], t: f32, fallback: Vec3) -> Vec3 {
    if times.is_empty() || values.is_empty() {
        return fallback;
    }
    if times.len() == 1 || values.len() == 1 {
        return values[0];
    }

    let mut i = 0usize;
    while i + 1 < times.len() && times[i + 1] < t {
        i += 1;
    }
    let next = (i + 1).min(times.len() - 1);
    let t0 = times[i];
    let t1 = times[next];
    if (t1 - t0).abs() < f32::EPSILON {
        values[i.min(values.len() - 1)]
    } else {
        let f = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
        values[i.min(values.len() - 1)].lerp(values[next.min(values.len() - 1)], f)
    }
}

fn sample_quat(times: &[f32], values: &[Quat], t: f32, fallback: Quat) -> Quat {
    if times.is_empty() || values.is_empty() {
        return fallback;
    }
    if times.len() == 1 || values.len() == 1 {
        return values[0];
    }

    let mut i = 0usize;
    while i + 1 < times.len() && times[i + 1] < t {
        i += 1;
    }
    let next = (i + 1).min(times.len() - 1);
    let t0 = times[i];
    let t1 = times[next];
    if (t1 - t0).abs() < f32::EPSILON {
        values[i.min(values.len() - 1)]
    } else {
        let f = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
        values[i.min(values.len() - 1)].slerp(values[next.min(values.len() - 1)], f)
    }
}

fn draw_skeleton_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    render_mode: Res<RenderModeState>,
    settings: Res<AppSettings>,
    animation_data: Res<AnimationData>,
    existing_visuals: Query<Entity, With<BoneSegmentVisual>>,
    mut cone_mesh_handle: Local<Option<Handle<Mesh>>>,
    mut start_cap_mesh_handle: Local<Option<Handle<Mesh>>>,
    mut normal_mat_handle: Local<Option<Handle<StandardMaterial>>>,
    mut selected_mat_handle: Local<Option<Handle<StandardMaterial>>>,
) {
    if render_mode.current_mode != RenderMode::Skeleton {
        for entity in existing_visuals.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }
    if animation_data.skeleton.is_empty() || animation_data.rest_global_transforms.is_empty() {
        for entity in existing_visuals.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    for entity in existing_visuals.iter() {
        commands.entity(entity).despawn();
    }

    let cone_mesh = cone_mesh_handle
        .get_or_insert_with(|| meshes.add(Mesh::from(Cone::new(0.5, 1.0))))
        .clone();
    let start_cap_mesh = start_cap_mesh_handle
        .get_or_insert_with(|| meshes.add(Mesh::from(Sphere::new(0.5))))
        .clone();

    let c = settings.skeleton_mode.bone_color;
    let always_visible = settings.skeleton_mode.always_visible;
    let color = if always_visible {
        Color::linear_rgba(c[0], c[1], c[2], 1.0)
    } else {
        Color::linear_rgba(c[0], c[1], c[2], 0.95)
    };
    let selected_color = if always_visible {
        Color::linear_rgba(0.2, 1.0, 0.2, 1.0)
    } else {
        Color::linear_rgba(0.1, 0.9, 0.1, 1.0)
    };
    let selected_bone = animation_data.selected_bone;
    let width_scale = settings.skeleton_mode.bone_line_width.max(0.1);
    let emissive_strength = if always_visible { 2.0 } else { 0.8 };
    let alpha_mode = if always_visible {
        AlphaMode::Opaque
    } else {
        AlphaMode::Blend
    };
    let depth_bias = if always_visible {
        -8.0
    } else {
        0.0
    };

    let normal_mat = normal_mat_handle
        .get_or_insert_with(|| {
            std_materials.add(StandardMaterial {
                base_color: color,
                emissive: LinearRgba::rgb(c[0] * emissive_strength, c[1] * emissive_strength, c[2] * emissive_strength),
                unlit: true,
                alpha_mode,
                depth_bias,
                cull_mode: None,
                ..default()
            })
        })
        .clone();
    if let Some(mat) = std_materials.get_mut(&normal_mat) {
        mat.base_color = color;
        mat.emissive = LinearRgba::rgb(c[0] * emissive_strength, c[1] * emissive_strength, c[2] * emissive_strength);
        mat.alpha_mode = alpha_mode;
        mat.depth_bias = depth_bias;
    }

    let selected_mat = selected_mat_handle
        .get_or_insert_with(|| {
            std_materials.add(StandardMaterial {
                base_color: selected_color,
                emissive: LinearRgba::rgb(0.2 * emissive_strength, 1.0 * emissive_strength, 0.2 * emissive_strength),
                unlit: true,
                alpha_mode,
                depth_bias,
                cull_mode: None,
                ..default()
            })
        })
        .clone();
    if let Some(mat) = std_materials.get_mut(&selected_mat) {
        mat.base_color = selected_color;
        mat.emissive = LinearRgba::rgb(0.2 * emissive_strength, 1.0 * emissive_strength, 0.2 * emissive_strength);
        mat.alpha_mode = alpha_mode;
        mat.depth_bias = depth_bias;
    }

    for (idx, bone) in animation_data.skeleton.iter().enumerate() {
        if let Some(parent_idx) = bone.parent {
            if idx < animation_data.rest_global_transforms.len()
                && parent_idx < animation_data.rest_global_transforms.len()
            {
                let a = animation_data.model_display_transform.transform_point3(
                    animation_data.rest_global_transforms[parent_idx].transform_point3(Vec3::ZERO),
                );
                let b = animation_data.model_display_transform.transform_point3(
                    animation_data.rest_global_transforms[idx].transform_point3(Vec3::ZERO),
                );
                let dir = b - a;
                let len = dir.length();
                if len < 1e-5 {
                    continue;
                }

                let rotation = Quat::from_rotation_arc(Vec3::Y, dir / len);
                let thickness = (len * 0.06 * width_scale).clamp(0.01, 2.5);
                let arrow_len = len;
                let mat = if selected_bone == Some(idx) || selected_bone == Some(parent_idx) {
                    selected_mat.clone()
                } else {
                    normal_mat.clone()
                };

                // One cone per bone: full parent->child direction arrow.
                let arrow_center = b - (dir / len) * (arrow_len * 0.5);
                let arrow_radius = (thickness * (2.8 + width_scale * 0.35)).clamp(0.06, 1.8);
                let arrow_scale = Vec3::new(
                    arrow_radius / 0.5,
                    arrow_len,
                    arrow_radius / 0.5,
                );
                commands.spawn((
                    Mesh3d(cone_mesh.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform {
                        translation: arrow_center,
                        rotation,
                        scale: arrow_scale,
                    },
                    BoneSegmentVisual,
                ));

                // Emphasize the start joint so direction is easier to read.
                let cap_radius = (arrow_radius * 0.9).clamp(0.05, 1.5);
                commands.spawn((
                    Mesh3d(start_cap_mesh.clone()),
                    MeshMaterial3d(mat),
                    Transform {
                        translation: a,
                        rotation: Quat::IDENTITY,
                        scale: Vec3::splat(cap_radius / 0.5),
                    },
                    BoneSegmentVisual,
                ));
            }
        }
    }
}

fn animate_skinning_system(
    time: Res<Time>,
    render_mode: Res<RenderModeState>,
    settings: Res<AppSettings>,
    mut animation_data: ResMut<AnimationData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<(Entity, &Mesh3d), With<LoadedModel>>,
) {
    if render_mode.current_mode != RenderMode::Animation {
        return;
    }
    if animation_data.clips.is_empty()
        || animation_data.skeleton.is_empty()
        || animation_data.mesh_skinning.is_empty()
    {
        return;
    }

    let clip_index = animation_data.active_clip.min(animation_data.clips.len() - 1);
    let duration = animation_data.clips[clip_index].duration.max(0.001);
    let speed = settings.animation_mode.playback_speed.max(0.0);
    animation_data.current_time += time.delta_secs() * speed;
    if animation_data.loop_playback {
        animation_data.current_time = animation_data.current_time.rem_euclid(duration);
    } else {
        animation_data.current_time = animation_data.current_time.min(duration);
    }
    let current_time = animation_data.current_time;
    let clip = &animation_data.clips[clip_index];

    let bone_count = animation_data.skeleton.len();
    let mut bone_globals = vec![Mat4::IDENTITY; bone_count];
    let mut skin_mats = vec![Mat4::IDENTITY; bone_count];

    // Build topological order so parents are always computed before children.
    let topo_order = {
        let mut order = Vec::with_capacity(bone_count);
        let mut visited = vec![false; bone_count];
        fn visit(
            idx: usize,
            skeleton: &[BoneInfo],
            visited: &mut Vec<bool>,
            order: &mut Vec<usize>,
        ) {
            if visited[idx] {
                return;
            }
            if let Some(p) = skeleton[idx].parent {
                visit(p, skeleton, visited, order);
            }
            visited[idx] = true;
            order.push(idx);
        }
        for i in 0..bone_count {
            visit(i, &animation_data.skeleton, &mut visited, &mut order);
        }
        order
    };

    for &idx in &topo_order {
        let bone = &animation_data.skeleton[idx];
        let rest_local = animation_data
            .rest_local_transforms
            .get(idx)
            .copied()
            .unwrap_or(Mat4::IDENTITY);
        let (mut s, mut r, mut t) = rest_local.to_scale_rotation_translation();
        if let Some(track) = clip.tracks.get(&bone.name) {
            t = sample_vec3(&track.translation_times, &track.translations, current_time, t);
            r = sample_quat(&track.rotation_times, &track.rotations, current_time, r);
            s = sample_vec3(&track.scale_times, &track.scales, current_time, s);
        }
        let local = Mat4::from_scale_rotation_translation(s, r, t);
        bone_globals[idx] = if let Some(parent_idx) = bone.parent {
            bone_globals[parent_idx] * local
        } else {
            local
        };
    }

    // FBX files often have a coordinate-system rotation on the scene root
    // (e.g. -90° around X). rest_global_transforms include this rotation, but
    // assimp's offset_matrix (inverse_bind) does not. Detect and compensate:
    //   R = rest_global[k] * inverse_bind[k]   (should be IDENTITY but is root_rot)
    //   corrected skin_mat = R⁻¹ * bone_global * inverse_bind
    let bind_correction = {
        let mut correction = Mat4::IDENTITY;
        for (idx, bone) in animation_data.skeleton.iter().enumerate() {
            if bone.inverse_bind == Mat4::IDENTITY {
                continue;
            }
            if let Some(&rest_g) = animation_data.rest_global_transforms.get(idx) {
                let r = rest_g * bone.inverse_bind;
                correction = r.inverse();
                break;
            }
        }
        correction
    };

    for idx in 0..bone_count {
        skin_mats[idx] =
            bind_correction * bone_globals[idx] * animation_data.skeleton[idx].inverse_bind;
    }

    let entity_to_mesh: HashMap<Entity, Handle<Mesh>> = mesh_query
        .iter()
        .map(|(entity, mesh3d)| (entity, mesh3d.0.clone()))
        .collect();

    for mesh_skin in &animation_data.mesh_skinning {
        let Some(mesh_handle) = entity_to_mesh.get(&mesh_skin.entity) else {
            continue;
        };
        let Some(mesh) = meshes.get_mut(mesh_handle) else {
            continue;
        };
        let Some(original_positions) = animation_data.mesh_original_positions.get(&mesh_skin.entity) else {
            continue;
        };
        let original_normals = animation_data
            .mesh_original_normals
            .get(&mesh_skin.entity)
            .filter(|n| n.len() == original_positions.len());
        if original_positions.len() != mesh_skin.vertex_influences.len() {
            continue;
        }

        let mut skinned_positions = vec![[0.0; 3]; original_positions.len()];
        let mut skinned_normals = original_normals.map(|_| vec![[0.0; 3]; original_positions.len()]);

        for (i, influence) in mesh_skin.vertex_influences.iter().enumerate() {
            let src_p = Vec3::new(
                original_positions[i][0],
                original_positions[i][1],
                original_positions[i][2],
            );
            let src_n = original_normals.map(|n| Vec3::new(n[i][0], n[i][1], n[i][2]));
            let mut out_p = Vec3::ZERO;
            let mut out_n = Vec3::ZERO;

            for j in 0..4 {
                let w = influence.weights[j];
                if w <= 0.0 {
                    continue;
                }
                let bone_idx = influence.indices[j] as usize;
                if bone_idx >= skin_mats.len() {
                    continue;
                }
                let m = skin_mats[bone_idx];
                out_p += m.transform_point3(src_p) * w;
                if let Some(n) = src_n {
                    out_n += m.transform_vector3(n) * w;
                }
            }

            skinned_positions[i] = [out_p.x, out_p.y, out_p.z];
            if let Some(normals) = &mut skinned_normals {
                let n = out_n.normalize_or_zero();
                normals[i] = [n.x, n.y, n.z];
            }
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, skinned_positions);
        if let Some(normals) = skinned_normals {
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        }
    }
}

fn restore_original_mesh_geometry(
    meshes: &mut Assets<Mesh>,
    animation_data: &AnimationData,
    model_query: &Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>, Option<&OriginalMaterial>, Option<&MeshName>),
        With<LoadedModel>,
    >,
) {
    for (entity, mesh3d, _, _, _) in model_query.iter() {
        let Some(mesh) = meshes.get_mut(&mesh3d.0) else {
            continue;
        };
        let original_positions = animation_data.mesh_original_positions.get(&entity);
        if let Some(positions) = original_positions {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
        }
        if let (Some(positions), Some(original_normals)) = (
            original_positions,
            animation_data.mesh_original_normals.get(&entity),
        ) {
            if original_normals.len() == positions.len() {
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, original_normals.clone());
            }
        }
    }
}

fn apply_render_mode(
    render_mode: Res<RenderModeState>,
    settings: Res<AppSettings>,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    animation_data: Res<AnimationData>,
    model_query: Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>, Option<&OriginalMaterial>, Option<&MeshName>),
        With<LoadedModel>,
    >,
    mut color_map: ResMut<ComponentColorMap>,
) {
    // First: restore all materials to their original state
    for (_, _, mat_handle, original, _) in model_query.iter() {
        if let (Some(material), Some(orig)) = (std_materials.get_mut(&mat_handle.0), original) {
            restore_material(material, orig);
        }
    }
    // Leaving animation mode should restore mesh to bind/rest shape.
    if render_mode.current_mode != RenderMode::Animation {
        restore_original_mesh_geometry(&mut meshes, &animation_data, &model_query);
    }

    match render_mode.current_mode {
        RenderMode::Solid => {
            wireframe_config.global = false;

            let solid_settings = &settings.solid_mode;
            let color = Color::linear_rgb(
                solid_settings.single_color[0],
                solid_settings.single_color[1],
                solid_settings.single_color[2],
            );
            let roughness = if solid_settings.specular_highlights {
                solid_settings.roughness.clamp(0.0, 1.0)
            } else {
                1.0
            };

            for (_, _, mat_handle, _, _) in model_query.iter() {
                if let Some(material) = std_materials.get_mut(&mat_handle.0) {
                    material.base_color = color;
                    material.alpha_mode = AlphaMode::Opaque;
                    material.metallic = 0.0;
                    material.perceptual_roughness = roughness;
                    material.double_sided = !solid_settings.backface_culling;
                    material.unlit = false;
                    material.cull_mode = if solid_settings.backface_culling {
                        Some(bevy::render::render_resource::Face::Back)
                    } else {
                        None
                    };
                }
            }
            color_map.entries.clear();
            info!("Switched to Solid mode");
        }
        RenderMode::Skeleton => {
            wireframe_config.global = false;
            let solid_settings = &settings.solid_mode;
            let skeleton_settings = &settings.skeleton_mode;
            let color = Color::linear_rgba(
                solid_settings.single_color[0],
                solid_settings.single_color[1],
                solid_settings.single_color[2],
                skeleton_settings.mesh_alpha.clamp(0.0, 1.0),
            );
            let roughness = if solid_settings.specular_highlights {
                solid_settings.roughness.clamp(0.0, 1.0)
            } else {
                1.0
            };

            for (_, _, mat_handle, _, _) in model_query.iter() {
                if let Some(material) = std_materials.get_mut(&mat_handle.0) {
                    material.base_color = color;
                    material.alpha_mode = AlphaMode::Blend;
                    material.metallic = 0.0;
                    material.perceptual_roughness = roughness;
                    material.double_sided = !solid_settings.backface_culling;
                    material.unlit = false;
                    material.cull_mode = if solid_settings.backface_culling {
                        Some(bevy::render::render_resource::Face::Back)
                    } else {
                        None
                    };
                }
            }
            color_map.entries.clear();
            info!("Switched to Skeleton mode");
        }
        RenderMode::Animation => {
            // Animation mode is temporarily disabled.
            // Keep a safe fallback so stale configs or external mode switches won't break rendering.
            wireframe_config.global = false;
            let solid_settings = &settings.solid_mode;
            let color = Color::linear_rgb(
                solid_settings.single_color[0],
                solid_settings.single_color[1],
                solid_settings.single_color[2],
            );
            let roughness = if solid_settings.specular_highlights {
                solid_settings.roughness.clamp(0.0, 1.0)
            } else {
                1.0
            };
            for (_, _, mat_handle, _, _) in model_query.iter() {
                if let Some(material) = std_materials.get_mut(&mat_handle.0) {
                    material.base_color = color;
                    material.alpha_mode = AlphaMode::Opaque;
                    material.metallic = 0.0;
                    material.perceptual_roughness = roughness;
                    material.double_sided = !solid_settings.backface_culling;
                    material.unlit = false;
                    material.cull_mode = if solid_settings.backface_culling {
                        Some(bevy::render::render_resource::Face::Back)
                    } else {
                        None
                    };
                }
            }
            color_map.entries.clear();
            warn!("Animation mode is disabled; fallback to Solid-like rendering");
        }
        RenderMode::Wireframe => {
            wireframe_config.global = true;
            let wire_settings = &settings.wireframe_mode;
            let xray = wire_settings.xray;
            wireframe_config.default_color = if xray {
                Color::linear_rgba(
                    wire_settings.wireframe_color[0],
                    wire_settings.wireframe_color[1],
                    wire_settings.wireframe_color[2],
                    0.95,
                )
            } else {
                Color::linear_rgb(
                    wire_settings.wireframe_color[0],
                    wire_settings.wireframe_color[1],
                    wire_settings.wireframe_color[2],
                )
            };

            for (_, _, mat_handle, _, _) in model_query.iter() {
                if let Some(material) = std_materials.get_mut(&mat_handle.0) {
                    material.base_color = if xray {
                        Color::linear_rgba(0.8, 0.8, 0.8, 0.08)
                    } else {
                        Color::linear_rgba(0.8, 0.8, 0.8, 0.35)
                    };
                    material.alpha_mode = AlphaMode::Blend;
                    material.metallic = if xray { 0.0 } else { 0.5 };
                    material.perceptual_roughness = if xray { 1.0 } else { 0.5 };
                    material.double_sided = true;
                    material.unlit = xray;
                    material.cull_mode = None;
                }
            }
            color_map.entries.clear();
            info!("Switched to Wireframe mode");
        }
        RenderMode::Material => {
            wireframe_config.global = false;
            // Original materials already restored above — nothing else to do.
            color_map.entries.clear();
            info!("Switched to Material mode (using original materials)");
        }
        RenderMode::Normal => {
            wireframe_config.global = false;

            for (_, _, mat_handle, _, _) in model_query.iter() {
                if let Some(material) = std_materials.get_mut(&mat_handle.0) {
                    material.base_color = Color::linear_rgb(0.5, 0.5, 1.0);
                    material.alpha_mode = AlphaMode::Opaque;
                    material.unlit = false;
                    material.metallic = 0.0;
                    material.perceptual_roughness = 1.0;
                }
            }
            color_map.entries.clear();
            info!("Switched to Normal mode (stub — using placeholder material)");
        }
        RenderMode::Component => {
            wireframe_config.global = false;

            // Build color map and assign unique color per mesh entity
            color_map.entries.clear();
            for (idx, (_, _, mat_handle, _, mesh_name)) in model_query.iter().enumerate() {
                let color = component_color(idx);
                let srgba = color.to_srgba();
                let name = mesh_name
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|| format!("Mesh_{}", idx));
                color_map.entries.push(ComponentColorEntry {
                    mesh_name: name,
                    color: [srgba.red, srgba.green, srgba.blue, 1.0],
                });
                if let Some(material) = std_materials.get_mut(&mat_handle.0) {
                    material.base_color = color;
                    material.alpha_mode = AlphaMode::Opaque;
                    material.unlit = false;
                    material.metallic = 0.0;
                    material.perceptual_roughness = 0.8;
                    material.double_sided = true;
                    material.cull_mode = None;
                }
            }
            info!("Switched to Component mode");
        }
    }
}
