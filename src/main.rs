mod resources;
mod utils;
mod plugins;
mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use std::path::PathBuf;

use resources::*;
use utils::file_utils::is_supported_format;

fn main() {
    // Install macOS Finder open-file hooks as early as possible.
    plugins::install_macos_open_hooks_early();

    let args: Vec<String> = std::env::args().collect();

    // Check command-line args for a file path (macOS may open files this way from Finder)
    let cli_file: Option<PathBuf> = args
        .iter()
        .skip(1) // skip binary name
        .map(PathBuf::from)
        .find(|p| p.exists() && is_supported_format(p));

    let settings = AppSettings::load();
    let initial_mode = settings.default_startup_mode.to_render_mode();

    let mut app = App::new();
    app
        // --- Bevy default plugins with custom window settings ---
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "MeshViewerRust".to_string(),
                resolution: (1400, 900).into(),
                ..default()
            }),
            ..default()
        }))
        // --- egui integration ---
        .add_plugins(EguiPlugin::default())
        // --- Resources (all initialised to Default) ---
        .init_resource::<UiState>()
        .init_resource::<FileState>()
        .init_resource::<ModelStats>()
        .insert_resource(RenderModeState {
            current_mode: initial_mode,
        })
        .init_resource::<LoadingState>()
        .init_resource::<FileBrowserState>()
        .init_resource::<MaterialPanelState>()
        .init_resource::<MaterialData>()
        .init_resource::<UvPanelState>()
        .init_resource::<AnimationData>()
        .insert_resource(settings)
        // --- Messages (events) ---
        .add_message::<UiAction>()
        .add_message::<ModelLoadRequest>()
        .add_message::<ModelLoaded>()
        .add_message::<ModelLoadError>()
        // --- Plugins ---
        .add_plugins(plugins::CameraPlugin)
        .add_plugins(plugins::ModelLoaderPlugin)
        .add_plugins(plugins::RenderModePlugin)
        .add_plugins(plugins::FileDropPlugin)
        .add_plugins(plugins::FileDialogPlugin)
        .add_plugins(plugins::MacOsOpenFilePlugin)
        .add_plugins(ui::MeshViewerUiPlugin);

    // If a file was passed via CLI, schedule it for loading on startup
    if let Some(path) = cli_file {
        app.add_systems(PostStartup, move |mut load_requests: MessageWriter<ModelLoadRequest>| {
            info!("Opening file from CLI: {:?}", path);
            load_requests.write(ModelLoadRequest { path: path.clone() });
        });
    }

    app.run();
}
