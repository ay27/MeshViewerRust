use bevy::prelude::*;

/// Available render modes for the 3D viewport.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum RenderMode {
    /// Blender-like Solid shading: uniform color + simple PBR + directional light.
    #[default]
    Solid,
    Wireframe,
    Material,
    Normal,
    /// Color each mesh node with a unique color for visual distinction.
    Component,
    /// Show mesh semi-transparent with highlighted skeleton lines.
    Skeleton,
    /// Show mesh rendering while continuously playing animation.
    Animation,
}

/// Tracks the currently active render mode.
#[derive(Resource)]
pub struct RenderModeState {
    pub current_mode: RenderMode,
}

impl Default for RenderModeState {
    fn default() -> Self {
        Self {
            current_mode: RenderMode::Component,
        }
    }
}

/// Entry in the component color map: mesh name + assigned color.
#[derive(Debug, Clone)]
pub struct ComponentColorEntry {
    pub mesh_name: String,
    pub color: [f32; 4],
}

/// Stores the mapping from mesh entities to their assigned component colors.
/// Populated when switching to Component render mode.
#[derive(Resource, Default, Debug, Clone)]
pub struct ComponentColorMap {
    pub entries: Vec<ComponentColorEntry>,
}
