use bevy::prelude::*;
use std::collections::HashMap;

/// One bone's static information in the skeleton hierarchy.
#[derive(Clone, Debug)]
pub struct BoneInfo {
    pub name: String,
    pub parent: Option<usize>,
    pub inverse_bind: Mat4,
}

/// Per-vertex bone influences (up to 4).
#[derive(Clone, Debug)]
pub struct VertexBoneInfluence {
    pub indices: [u16; 4],
    pub weights: [f32; 4],
}

impl Default for VertexBoneInfluence {
    fn default() -> Self {
        Self {
            indices: [0; 4],
            weights: [0.0; 4],
        }
    }
}

/// Skinning data for one mesh entity.
#[derive(Clone, Debug)]
pub struct MeshSkinningData {
    pub entity: Entity,
    pub vertex_influences: Vec<VertexBoneInfluence>,
    /// World transform of the mesh node at load time.
    pub mesh_world_transform: Mat4,
}

/// Bone transform keyframes.
#[derive(Clone, Debug)]
pub struct BoneTrack {
    pub translation_times: Vec<f32>,
    pub translations: Vec<Vec3>,
    pub rotation_times: Vec<f32>,
    pub rotations: Vec<Quat>,
    pub scale_times: Vec<f32>,
    pub scales: Vec<Vec3>,
}

/// One animation clip with named tracks.
#[derive(Clone, Debug)]
pub struct AnimationClipData {
    pub name: String,
    pub duration: f32,
    pub tracks: HashMap<String, BoneTrack>,
}

/// Runtime animation state and parsed skeleton/skinning data.
#[derive(Resource, Default, Debug)]
pub struct AnimationData {
    pub skeleton: Vec<BoneInfo>,
    pub bone_lookup: HashMap<String, usize>,
    pub rest_global_transforms: Vec<Mat4>,
    pub rest_local_transforms: Vec<Mat4>,
    pub clips: Vec<AnimationClipData>,
    pub mesh_skinning: Vec<MeshSkinningData>,
    pub mesh_original_positions: HashMap<Entity, Vec<[f32; 3]>>,
    pub mesh_original_normals: HashMap<Entity, Vec<[f32; 3]>>,
    pub current_time: f32,
    pub active_clip: usize,
    pub loop_playback: bool,
    pub selected_bone: Option<usize>,
    /// Additional transform applied to model entities for display normalization.
    pub model_display_transform: Mat4,
}

impl AnimationData {
    /// Clear all runtime data before loading a new model.
    pub fn reset(&mut self) {
        self.skeleton.clear();
        self.bone_lookup.clear();
        self.rest_global_transforms.clear();
        self.rest_local_transforms.clear();
        self.clips.clear();
        self.mesh_skinning.clear();
        self.mesh_original_positions.clear();
        self.mesh_original_normals.clear();
        self.current_time = 0.0;
        self.active_clip = 0;
        self.loop_playback = true;
        self.selected_bone = None;
        self.model_display_transform = Mat4::IDENTITY;
    }
}
