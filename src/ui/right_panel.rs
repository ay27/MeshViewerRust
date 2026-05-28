use bevy_egui::egui;

use crate::resources::{
    ComponentColorMap, Locale, MaterialData, ModelStats, RenderModeState, RightPanelTab, UiState,
    UiTextKey, t,
};
use crate::resources::material_data::MaterialPanelState;
use crate::resources::uv_data::UvPanelState;

use super::info_panel::info_tab_ui;
use super::material_panel::material_tab_ui;
use super::uv_panel::uv_tab_ui;

/// Render the right panel with Info / Material / UV tabs.
pub fn right_panel_ui(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    stats: &ModelStats,
    material_state: &mut MaterialPanelState,
    material_data: &MaterialData,
    uv_state: &mut UvPanelState,
    render_mode: &RenderModeState,
    color_map: &ComponentColorMap,
    locale: Locale,
) {
    if !ui_state.show_right_panel {
        return;
    }

    egui::SidePanel::right("right_panel")
        .default_width(ui_state.right_panel_width)
        .width_range(250.0..=600.0)
        .resizable(true)
        .show(ctx, |ui| {
            // --- Tab bar ---
            ui.horizontal(|ui| {
                let tabs = [
                    (RightPanelTab::Info, t(locale, UiTextKey::RightPanelInfoTab)),
                    (RightPanelTab::Material, t(locale, UiTextKey::RightPanelMaterialTab)),
                    (RightPanelTab::Uv, t(locale, UiTextKey::RightPanelUvTab)),
                ];
                for (tab, label) in &tabs {
                    let is_active = ui_state.right_panel_tab == *tab;
                    if ui.selectable_label(is_active, *label).clicked() {
                        ui_state.right_panel_tab = *tab;
                    }
                }

                // Close button
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").clicked() {
                        ui_state.show_right_panel = false;
                    }
                });
            });

            ui.separator();

            // --- Tab content ---
            match ui_state.right_panel_tab {
                RightPanelTab::Info => info_tab_ui(ui, stats, render_mode, color_map, locale),
                RightPanelTab::Material => material_tab_ui(ui, material_state, material_data, locale),
                RightPanelTab::Uv => uv_tab_ui(ui, uv_state, locale),
            }
        });
}
