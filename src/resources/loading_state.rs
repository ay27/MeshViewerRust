use bevy::prelude::*;

/// Stages during model loading. `Error` carries a description string,
/// so this enum cannot derive `Copy`.
#[derive(PartialEq, Clone, Debug, Default)]
pub enum LoadingStage {
    #[default]
    Idle,
    Reading,
    Parsing,
    Building,
    Ready,
    Error(String),
}

/// Progress state while a model is being loaded.
#[derive(Resource)]
pub struct LoadingState {
    pub is_loading: bool,
    /// Progress in the range 0.0..=1.0.
    pub progress: f32,
    /// Name of the file being loaded (display only).
    pub file_name: String,
    pub stage: LoadingStage,
}

impl Default for LoadingState {
    fn default() -> Self {
        Self {
            is_loading: false,
            progress: 0.0,
            file_name: String::new(),
            stage: LoadingStage::Idle,
        }
    }
}
