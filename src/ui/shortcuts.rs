use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::resources::{
    RenderMode, RenderModeState, RightPanelTab, UiState,
};
use crate::resources::events::UiAction;
use crate::resources::ui_state::toggle_right_panel;

/// Bevy system that processes keyboard shortcuts.
pub fn keyboard_shortcut_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<UiState>,
    mut render_mode: ResMut<RenderModeState>,
    mut actions: MessageWriter<UiAction>,
    mut contexts: EguiContexts,
) {
    // Skip when egui wants keyboard input (e.g. user typing in a text field)
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_keyboard_input() {
        return;
    }

    let ctrl = keyboard.pressed(KeyCode::SuperLeft)
        || keyboard.pressed(KeyCode::SuperRight)
        || keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);
    let shift = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    // Ctrl+O / Ctrl+Shift+O
    if ctrl && keyboard.just_pressed(KeyCode::KeyO) {
        if shift {
            actions.write(UiAction::OpenDirectoryDialog);
        } else {
            actions.write(UiAction::OpenFileDialog);
        }
    }

    // Cmd+, / Ctrl+, - open settings dialog
    if ctrl && keyboard.just_pressed(KeyCode::Comma) {
        ui_state.show_settings_dialog = true;
    }

    // B - toggle sidebar
    if keyboard.just_pressed(KeyCode::KeyB) && !ctrl {
        ui_state.show_sidebar = !ui_state.show_sidebar;
    }

    

    // Number keys - render modes
    let mode_keys = [
        (KeyCode::Digit1, RenderMode::Wireframe),
        (KeyCode::Digit2, RenderMode::Solid),
        (KeyCode::Digit3, RenderMode::Component),
        (KeyCode::Digit4, RenderMode::Material),
        (KeyCode::Digit5, RenderMode::Skeleton),
    ];
    for (key, mode) in &mode_keys {
        if keyboard.just_pressed(*key) && !ctrl {
            render_mode.current_mode = *mode;
        }
    }

    // F - fit to view
    if keyboard.just_pressed(KeyCode::KeyF) && !ctrl {
        actions.write(UiAction::FitToView);
    }

    // I - toggle right info panel
    if keyboard.just_pressed(KeyCode::KeyI) && !ctrl {
        toggle_right_panel(&mut ui_state, RightPanelTab::Info);
    }

    // M - toggle material panel
    if keyboard.just_pressed(KeyCode::KeyM) && !ctrl {
        toggle_right_panel(&mut ui_state, RightPanelTab::Material);
    }

    // U - toggle UV panel
    if keyboard.just_pressed(KeyCode::KeyU) && !ctrl {
        toggle_right_panel(&mut ui_state, RightPanelTab::Uv);
    }
}
