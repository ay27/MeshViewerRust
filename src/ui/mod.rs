pub mod theme;
pub mod toolbar;
pub mod file_browser;
pub mod info_panel;
pub mod right_panel;
pub mod material_panel;
pub mod uv_panel;
pub mod bone_tree_panel;
pub mod settings_dialog;
pub mod loading_overlay;
pub mod statusbar;
pub mod shortcuts;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::plugins::file_dialog::FileDialogState;
use crate::resources::{
    AnimationData, AppSettings, ComponentColorMap, FileState, LoadingState, MaterialData, ModelStats,
    RenderModeState, UiState,
};
use crate::resources::file_browser_state::FileBrowserState;
use crate::resources::material_data::MaterialPanelState;
use crate::resources::uv_data::UvPanelState;
use crate::resources::events::UiAction;

/// System set ordering: UI root runs first, then shortcuts.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum UiSet {
    Root,
    Shortcuts,
}

/// The main UI plugin. Registers all UI systems.
pub struct MeshViewerUiPlugin;

impl Plugin for MeshViewerUiPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            EguiPrimaryContextPass,
            UiSet::Shortcuts.after(UiSet::Root),
        );
        app.add_systems(
            EguiPrimaryContextPass,
            ui_root_system.in_set(UiSet::Root),
        );
        app.add_systems(
            EguiPrimaryContextPass,
            shortcuts::keyboard_shortcut_system.in_set(UiSet::Shortcuts),
        );
        // File system I/O can stay in Update.
        app.add_systems(
            Update,
            (
                file_browser::directory_lazy_load_system,
                persist_settings.run_if(resource_changed::<AppSettings>),
            ),
        );
    }
}

/// Root UI system that draws all egui panels each frame.
fn ui_root_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    file_state: Res<FileState>,
    mut render_mode: ResMut<RenderModeState>,
    stats: Res<ModelStats>,
    loading: Res<LoadingState>,
    mut browser_state: ResMut<FileBrowserState>,
    mut material_state: ResMut<MaterialPanelState>,
    material_data: Res<MaterialData>,
    mut uv_state: ResMut<UvPanelState>,
    mut settings: ResMut<AppSettings>,
    mut animation_data: ResMut<AnimationData>,
    mut actions: MessageWriter<UiAction>,
    mut dialog_state: ResMut<FileDialogState>,
    color_map: Res<ComponentColorMap>,
    mut theme_applied: Local<bool>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Apply theme once on first frame
    if !*theme_applied {
        theme::apply_dark_theme(ctx);
        *theme_applied = true;
    }

    // Stats overlay visibility follows settings.
    ui_state.show_stats_overlay = settings.common.show_stats;

    // 1. Top toolbar
    let new_mode = egui::TopBottomPanel::top("toolbar")
        .exact_height(40.0)
        .show(ctx, |ui| {
            toolbar::toolbar_ui(
                ui,
                &mut ui_state,
                &render_mode,
                &file_state,
                &settings,
                &mut actions,
            )
        })
        .inner;

    // Only trigger change detection when a render mode button was actually clicked
    if let Some(mode) = new_mode {
        render_mode.current_mode = mode;
    }

    // 2. Bottom status bar
    statusbar::statusbar_ui(ctx, &file_state, &stats, &render_mode, settings.locale);

    // 3. Left panel (file browser) — conditional
    let sidebar_right_edge = if ui_state.show_sidebar {
        let resp = egui::SidePanel::left("file_browser")
            .default_width(ui_state.sidebar_width)
            .width_range(200.0..=500.0)
            .resizable(true)
            .show(ctx, |ui| {
                file_browser::file_browser_ui(ui, &mut browser_state, &mut actions, settings.locale);
            });
        resp.response.rect.right()
    } else {
        0.0
    };

    // 4. Right panel — conditional
    right_panel::right_panel_ui(
        ctx,
        &mut ui_state,
        &stats,
        &mut material_state,
        &material_data,
        &mut uv_state,
        &render_mode,
        &color_map,
        settings.locale,
    );

    // 5. Floating overlays — offset stats overlay by sidebar width
    info_panel::stats_overlay_ui(ctx, &stats, &ui_state, sidebar_right_edge, settings.locale);
    bone_tree_panel::bone_tree_overlay_ui(
        ctx,
        &render_mode,
        &mut animation_data,
        sidebar_right_edge,
        settings.locale,
    );
    loading_overlay::loading_overlay_ui(ctx, &loading, settings.locale);
    settings_dialog::settings_dialog_ui(ctx, &mut ui_state, &mut settings, &mut *dialog_state);
}

fn persist_settings(settings: Res<AppSettings>) {
    settings.save();
}
