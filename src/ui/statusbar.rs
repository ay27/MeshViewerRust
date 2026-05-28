use bevy_egui::egui;

use crate::resources::{FileState, Locale, ModelStats, RenderModeState, UiTextKey, render_mode_label, t};
use crate::utils::format_number;

/// Render the bottom status bar.
pub fn statusbar_ui(
    ctx: &egui::Context,
    file_state: &FileState,
    stats: &ModelStats,
    render_mode: &RenderModeState,
    locale: Locale,
) {
    egui::TopBottomPanel::bottom("statusbar")
        .exact_height(24.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // Status indicator
                ui.label(
                    egui::RichText::new(t(locale, UiTextKey::StatusReady))
                        .small()
                        .color(egui::Color32::from_rgb(100, 200, 100)),
                );

                ui.separator();

                // File name
                if let Some(ref name) = file_state.file_name {
                    ui.label(egui::RichText::new(name).small());
                    ui.separator();

                    // Vertex count
                    ui.label(
                        egui::RichText::new(format!(
                            "{} {}",
                            format_number(stats.num_vertices),
                            t(locale, UiTextKey::StatusVertsSuffix)
                        ))
                            .small()
                            .color(egui::Color32::from_rgb(156, 163, 175)),
                    );

                    ui.separator();

                    // Face count
                    ui.label(
                        egui::RichText::new(format!(
                            "{} {}",
                            format_number(stats.num_faces),
                            t(locale, UiTextKey::StatusFacesSuffix)
                        ))
                            .small()
                            .color(egui::Color32::from_rgb(156, 163, 175)),
                    );
                }

                // Right side: render mode
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(render_mode_label(locale, render_mode.current_mode))
                        .small(),
                    );
                });
            });
        });
}
