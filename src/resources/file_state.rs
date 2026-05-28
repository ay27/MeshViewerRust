use bevy::prelude::*;
use std::path::PathBuf;

/// Tracks the currently opened file and directory.
#[derive(Resource, Default)]
pub struct FileState {
    /// Full path to the currently loaded model file.
    pub current_file_path: Option<PathBuf>,
    /// Directory currently shown in the file browser.
    pub current_directory: Option<PathBuf>,
    /// Display name of the currently loaded file.
    pub file_name: Option<String>,
}
