use bevy::prelude::*;
use std::collections::HashSet;

/// Information about a single material extracted from the model.
#[derive(Debug, Clone)]
pub struct MaterialInfo {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub has_base_color_texture: bool,
    pub has_normal_texture: bool,
    pub has_roughness_texture: bool,
}

impl Default for MaterialInfo {
    fn default() -> Self {
        Self {
            name: String::from("Unnamed"),
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            has_base_color_texture: false,
            has_normal_texture: false,
            has_roughness_texture: false,
        }
    }
}

/// Maps a mesh to its material indices.
#[derive(Debug, Clone, Default)]
pub struct MeshMaterialEntry {
    pub mesh_name: String,
    pub mesh_index: usize,
    pub material_indices: Vec<usize>,
}

/// All material data extracted from the current model.
#[derive(Resource, Default, Debug, Clone)]
pub struct MaterialData {
    pub materials: Vec<MaterialInfo>,
    pub mesh_material_map: Vec<MeshMaterialEntry>,
}

/// UI state for the material panel.
#[derive(Resource)]
pub struct MaterialPanelState {
    pub selected_material_index: Option<usize>,
    pub expanded_meshes: HashSet<usize>,
    pub search_query: String,
    /// Ratio of the tree section height to the total panel height (0.0..=1.0).
    pub tree_section_ratio: f32,
}

impl Default for MaterialPanelState {
    fn default() -> Self {
        Self {
            selected_material_index: None,
            expanded_meshes: HashSet::new(),
            search_query: String::new(),
            tree_section_ratio: 0.5,
        }
    }
}
