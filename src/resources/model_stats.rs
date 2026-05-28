use bevy::prelude::*;

/// Aggregate statistics for the currently loaded model.
#[derive(Resource, Default, Debug, Clone)]
pub struct ModelStats {
    pub num_meshes: u32,
    pub num_vertices: u64,
    pub num_faces: u64,
    /// Bounding box size [x, y, z].
    pub bbox_size: [f32; 3],
    pub num_materials: u32,
    pub num_textures: u32,
}
