use bevy::prelude::*;
use bevy_egui::egui;

use crate::resources::{
    AppSettings, FileState, RenderMode, RenderModeState, RightPanelTab, UiState, UiTextKey, t,
};
use crate::resources::events::UiAction;
use crate::resources::ui_state::toggle_right_panel;

/// Render the top toolbar inside a `TopBottomPanel`.
///
/// Returns `Some(RenderMode)` if the user clicked a render mode button,
/// `None` otherwise. The caller is responsible for writing the new mode
/// into the `RenderModeState` resource so that change-detection only
/// fires when an actual switch happens.
pub fn toolbar_ui(
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    render_mode: &RenderModeState,
    file_state: &FileState,
    settings: &AppSettings,
    actions: &mut MessageWriter<UiAction>,
) -> Option<RenderMode> {
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;

        // === Left button group ===
        let sidebar_icon = if ui_state.show_sidebar { "◀" } else { "▶" };
        if ui
            .button(sidebar_icon)
            .on_hover_text(t(settings.locale, UiTextKey::ToolbarTooltipToggleSidebar))
            .clicked()
        {
            ui_state.show_sidebar = !ui_state.show_sidebar;
        }

        ui.separator();

        if ui.button("📂").on_hover_text(t(settings.locale, UiTextKey::ToolbarTooltipOpenDirectory)).clicked() {
            actions.write(UiAction::OpenDirectoryDialog);
        }

        if ui.button("📄").on_hover_text(t(settings.locale, UiTextKey::ToolbarTooltipOpenFile)).clicked() {
            actions.write(UiAction::OpenFileDialog);
        }

        if ui.button("⚙").on_hover_text(t(settings.locale, UiTextKey::ToolbarTooltipSettings)).clicked() {
            ui_state.show_settings_dialog = true;
        }

        ui.separator();

        // === Center: render mode buttons ===
        let new_mode = render_mode_buttons(ui, render_mode, settings.locale);

        // === Right button group ===
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // UV panel toggle
            let uv_active =
                ui_state.show_right_panel && ui_state.right_panel_tab == RightPanelTab::Uv;
            if toggle_button(
                ui,
                t(settings.locale, UiTextKey::ToolbarButtonUv),
                uv_active,
                t(settings.locale, UiTextKey::ToolbarTooltipUv),
            ) {
                toggle_right_panel(ui_state, RightPanelTab::Uv);
            }

            // Material panel toggle
            let mat_active =
                ui_state.show_right_panel && ui_state.right_panel_tab == RightPanelTab::Material;
            if toggle_button(
                ui,
                t(settings.locale, UiTextKey::ToolbarButtonMaterial),
                mat_active,
                t(settings.locale, UiTextKey::ToolbarTooltipMaterialPanel),
            ) {
                toggle_right_panel(ui_state, RightPanelTab::Material);
            }

            // Info panel toggle
            let info_active =
                ui_state.show_right_panel && ui_state.right_panel_tab == RightPanelTab::Info;
            if toggle_button(
                ui,
                t(settings.locale, UiTextKey::ToolbarButtonInfo),
                info_active,
                t(settings.locale, UiTextKey::ToolbarTooltipInfoPanel),
            ) {
                toggle_right_panel(ui_state, RightPanelTab::Info);
            }

            // Blender button (conditional)
            if settings.blender_path.is_some() && file_state.current_file_path.is_some() {
                if ui.button("B").on_hover_text(t(settings.locale, UiTextKey::ToolbarTooltipOpenInBlender)).clicked() {
                    actions.write(UiAction::OpenInBlender);
                }
            }
        });

        new_mode
    })
    .inner
}

fn render_mode_buttons(
    ui: &mut egui::Ui,
    state: &RenderModeState,
    locale: crate::resources::Locale,
) -> Option<RenderMode> {
    let modes = [
        (
            RenderMode::Wireframe,
            if locale == crate::resources::Locale::Zh {
                "线框(1)"
            } else {
                "Wire(1)"
            },
            t(locale, UiTextKey::ToolbarTooltipWireframe),
        ),
        (
            RenderMode::Solid,
            if locale == crate::resources::Locale::Zh {
                "实体(2)"
            } else {
                "Solid(2)"
            },
            t(locale, UiTextKey::ToolbarTooltipSolid),
        ),
        (
            RenderMode::Component,
            if locale == crate::resources::Locale::Zh {
                "部件(3)"
            } else {
                "Comp(3)"
            },
            t(locale, UiTextKey::ToolbarTooltipComponent),
        ),
        (
            RenderMode::Material,
            if locale == crate::resources::Locale::Zh {
                "材质(4)"
            } else {
                "Mat(4)"
            },
            t(locale, UiTextKey::ToolbarTooltipMaterial),
        ),
        (
            RenderMode::Skeleton,
            if locale == crate::resources::Locale::Zh {
                "骨骼(5)"
            } else {
                "Bone(5)"
            },
            t(locale, UiTextKey::ToolbarTooltipSkeleton),
        ),
    ];

    let mut selected = None;
    for (mode, label, tooltip) in &modes {
        let is_active = state.current_mode == *mode;
        let btn = egui::Button::new(*label).fill(if is_active {
            egui::Color32::from_rgb(37, 99, 235) // #2563eb
        } else {
            egui::Color32::TRANSPARENT
        });
        if ui.add(btn).on_hover_text(*tooltip).clicked() && !is_active {
            selected = Some(*mode);
        }
    }
    selected
}

fn toggle_button(ui: &mut egui::Ui, label: &str, active: bool, tooltip: &str) -> bool {
    let btn = egui::Button::new(label).fill(if active {
        egui::Color32::from_rgb(37, 99, 235)
    } else {
        egui::Color32::TRANSPARENT
    });
    ui.add(btn).on_hover_text(tooltip).clicked()
}
