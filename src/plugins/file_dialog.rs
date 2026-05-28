use bevy::prelude::*;
use std::path::PathBuf;
use std::sync::{mpsc, Mutex};

use crate::resources::events::{ModelLoadRequest, UiAction};
use crate::resources::file_browser_state::{FileBrowserState, TreeNode};
use crate::resources::{AppSettings, Locale, UiState, UiTextKey, t};
use crate::utils::SUPPORTED_3D_EXTENSIONS;

/// Plugin that handles file / directory dialogs spawned from the UI.
pub struct FileDialogPlugin;

impl Plugin for FileDialogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileDialogState>()
            .add_systems(Update, handle_ui_actions)
            .add_systems(Update, poll_dialog_results);
    }
}

/// What kind of dialog did we open?
#[derive(Debug)]
enum DialogKind {
    OpenFile,
    OpenDirectory,
    BrowseBlender,
}

/// Stores the channel receiver for any currently-open dialog.
/// Wrapped in Mutex to satisfy Sync requirement for Bevy Resource.
#[derive(Resource, Default)]
pub struct FileDialogState {
    pending: Option<(DialogKind, Mutex<mpsc::Receiver<Option<PathBuf>>>)>,
}

/// Reads UiAction messages and spawns file dialogs when requested.
fn handle_ui_actions(
    mut actions: MessageReader<UiAction>,
    mut dialog_state: ResMut<FileDialogState>,
    mut load_requests: MessageWriter<ModelLoadRequest>,
    mut browser_state: ResMut<FileBrowserState>,
    mut ui_state: ResMut<UiState>,
    settings: Res<AppSettings>,
) {
    for action in actions.read() {
        match action {
            UiAction::OpenFileDialog => {
                if dialog_state.pending.is_some() {
                    continue; // another dialog already open
                }
                let (tx, rx) = mpsc::channel();
                let dialog_title = t(settings.locale, UiTextKey::FileDialogOpenModel).to_string();
                let filter_label = t(settings.locale, UiTextKey::FileDialogFilter3dModels).to_string();
                std::thread::spawn(move || {
                    let dialog = rfd::FileDialog::new()
                        .set_title(&dialog_title)
                        .add_filter(&filter_label, SUPPORTED_3D_EXTENSIONS);
                    let result = dialog.pick_file();
                    let _ = tx.send(result);
                });
                dialog_state.pending = Some((DialogKind::OpenFile, Mutex::new(rx)));
            }
            UiAction::OpenDirectoryDialog => {
                if dialog_state.pending.is_some() {
                    continue;
                }
                let (tx, rx) = mpsc::channel();
                let dialog_title = t(settings.locale, UiTextKey::FileDialogOpenDirectory).to_string();
                std::thread::spawn(move || {
                    let result = rfd::FileDialog::new()
                        .set_title(&dialog_title)
                        .pick_folder();
                    let _ = tx.send(result);
                });
                dialog_state.pending = Some((DialogKind::OpenDirectory, Mutex::new(rx)));
            }
            UiAction::OpenFile(path) => {
                load_requests.write(ModelLoadRequest { path: path.clone() });
            }
            UiAction::OpenDirectory(path) => {
                browser_state.root_path = Some(path.clone());
                browser_state.tree_nodes.clear();
                browser_state.expanded_dirs.clear();
                ui_state.show_sidebar = true;
                // Create the root TreeNode so directory_lazy_load_system can populate it
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());
                browser_state.tree_nodes.push(TreeNode {
                    name,
                    path: path.clone(),
                    is_directory: true,
                    is_loaded: false,
                    ..Default::default()
                });
                browser_state.expanded_dirs.insert(path.clone());
            }
            _ => {}
        }
    }
}

/// Non-blocking poll for the result of an open dialog.
fn poll_dialog_results(
    mut dialog_state: ResMut<FileDialogState>,
    mut load_requests: MessageWriter<ModelLoadRequest>,
    mut browser_state: ResMut<FileBrowserState>,
    mut settings: ResMut<AppSettings>,
    mut ui_state: ResMut<UiState>,
) {
    let Some((kind, rx_mutex)) = dialog_state.pending.as_ref() else {
        return;
    };

    let rx = rx_mutex.lock().unwrap();
    match rx.try_recv() {
        Ok(Some(path)) => {
            match kind {
                DialogKind::OpenFile => {
                    info!("Opening file: {:?}", path);
                    load_requests.write(ModelLoadRequest { path });
                }
                DialogKind::OpenDirectory => {
                    info!("Opening directory: {:?}", path);
                    browser_state.root_path = Some(path.clone());
                    browser_state.tree_nodes.clear();
                    browser_state.expanded_dirs.clear();
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.to_string_lossy().to_string());
                    browser_state.tree_nodes.push(TreeNode {
                        name,
                        path: path.clone(),
                        is_directory: true,
                        is_loaded: false,
                        ..Default::default()
                    });
                    browser_state.expanded_dirs.insert(path);
                    ui_state.show_sidebar = true;
                }
                DialogKind::BrowseBlender => {
                    settings.blender_path = Some(path.to_string_lossy().to_string());
                }
            }
            drop(rx);
            dialog_state.pending = None;
        }
        Ok(None) => {
            // User cancelled
            drop(rx);
            dialog_state.pending = None;
        }
        Err(mpsc::TryRecvError::Empty) => {
            // Still waiting — do nothing
        }
        Err(mpsc::TryRecvError::Disconnected) => {
            // Thread died, clean up
            drop(rx);
            dialog_state.pending = None;
        }
    }
}

/// Public helper: spawn a "Browse for Blender" dialog. Call from settings UI.
pub fn spawn_blender_browse_dialog(dialog_state: &mut FileDialogState, locale: Locale) {
    if dialog_state.pending.is_some() {
        return;
    }
    let (tx, rx) = mpsc::channel();
    let dialog_title = t(locale, UiTextKey::FileDialogSelectBlender).to_string();
    std::thread::spawn(move || {
        let result = rfd::FileDialog::new()
            .set_title(&dialog_title)
            .pick_file();
        let _ = tx.send(result);
    });
    dialog_state.pending = Some((DialogKind::BrowseBlender, Mutex::new(rx)));
}
