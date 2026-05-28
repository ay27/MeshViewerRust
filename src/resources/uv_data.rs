use bevy::prelude::*;

/// UI state for the UV preview panel.
#[derive(Resource)]
pub struct UvPanelState {
    /// Selected mesh index, or `None` for all meshes.
    pub selected_mesh_index: Option<usize>,
    /// Active UV channel (0, 1, 2, ...).
    pub selected_uv_channel: usize,
    /// Optionally filter by material index.
    pub selected_material_index: Option<usize>,
    /// Zoom level for the UV canvas.
    pub zoom: f32,
    /// Pan offset [x, y] in screen pixels.
    pub pan_offset: [f32; 2],
    /// Whether to draw the UV grid.
    pub show_grid: bool,
}

impl Default for UvPanelState {
    fn default() -> Self {
        Self {
            selected_mesh_index: None,
            selected_uv_channel: 0,
            selected_material_index: None,
            zoom: 1.0,
            pan_offset: [0.0, 0.0],
            show_grid: true,
        }
    }
}
