use bevy::prelude::*;
use std::path::PathBuf;

use super::material_data::MaterialInfo;
use super::model_stats::ModelStats;
use super::render_mode::RenderMode;
use super::ui_state::RightPanelTab;

/// High-level UI actions emitted by toolbar buttons, shortcuts, etc.
#[derive(Message, Debug)]
pub enum UiAction {
    OpenFile(PathBuf),
    OpenDirectory(PathBuf),
    OpenFileDialog,
    OpenDirectoryDialog,
    SetRenderMode(RenderMode),
    ToggleSidebar,
    ToggleRightPanel(RightPanelTab),
    OpenInBlender,
    OpenSettings,
    ResetCamera,
    FitToView,
}

/// Request to load a model from the given path.
#[derive(Message, Debug)]
pub struct ModelLoadRequest {
    pub path: PathBuf,
}

/// Fired after a model has been loaded successfully.
#[derive(Message, Debug)]
pub struct ModelLoaded {
    pub stats: ModelStats,
    pub materials: Vec<MaterialInfo>,
}

/// Fired when model loading fails.
#[derive(Message, Debug)]
pub struct ModelLoadError {
    pub error: String,
}
