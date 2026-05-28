use bevy::mesh::{Mesh, VertexAttributeValues};
use crate::resources::ModelStats;

/// Calculate aggregate statistics from a set of Bevy meshes.
pub fn calculate_mesh_stats_from_scene(meshes: &[&Mesh]) -> ModelStats {
    let mut total_vertices: u64 = 0;
    let mut total_faces: u64 = 0;
    let mut bbox_min = [f32::MAX; 3];
    let mut bbox_max = [f32::MIN; 3];
    let mut has_positions = false;

    for mesh in meshes {
        // Count vertices from position attribute
        if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            let count = positions.len();
            total_vertices += count as u64;

            // Update bounding box
            if let VertexAttributeValues::Float32x3(pos_data) = positions {
                for pos in pos_data {
                    has_positions = true;
                    bbox_min[0] = bbox_min[0].min(pos[0]);
                    bbox_min[1] = bbox_min[1].min(pos[1]);
                    bbox_min[2] = bbox_min[2].min(pos[2]);
                    bbox_max[0] = bbox_max[0].max(pos[0]);
                    bbox_max[1] = bbox_max[1].max(pos[1]);
                    bbox_max[2] = bbox_max[2].max(pos[2]);
                }
            }
        }

        // Count faces from indices
        if let Some(indices) = mesh.indices() {
            total_faces += (indices.len() / 3) as u64;
        }
    }

    let bbox_size = if has_positions {
        [
            bbox_max[0] - bbox_min[0],
            bbox_max[1] - bbox_min[1],
            bbox_max[2] - bbox_min[2],
        ]
    } else {
        [0.0, 0.0, 0.0]
    };

    ModelStats {
        num_meshes: meshes.len() as u32,
        num_vertices: total_vertices,
        num_faces: total_faces,
        bbox_size,
        num_materials: 0,
        num_textures: 0,
    }
}
