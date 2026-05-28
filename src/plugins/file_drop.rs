use bevy::prelude::*;
use bevy::window::FileDragAndDrop;

use crate::resources::ModelLoadRequest;
use crate::utils::file_utils::is_supported_format;

/// Plugin that handles file drag-and-drop onto the application window.
pub struct FileDropPlugin;

impl Plugin for FileDropPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, file_drop_system);
    }
}

fn file_drop_system(
    mut dnd_events: MessageReader<FileDragAndDrop>,
    mut load_requests: MessageWriter<ModelLoadRequest>,
) {
    for event in dnd_events.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            if is_supported_format(path_buf) {
                info!("File dropped: {:?}", path_buf);
                load_requests.write(ModelLoadRequest {
                    path: path_buf.clone(),
                });
            } else {
                warn!("Unsupported file format: {:?}", path_buf);
            }
        }
    }
}
