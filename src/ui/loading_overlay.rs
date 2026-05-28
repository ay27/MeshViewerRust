use bevy_egui::egui;

use crate::resources::loading_state::{LoadingStage, LoadingState};
use crate::resources::{Locale, UiTextKey, t};

/// Render a loading overlay with progress bar.
pub fn loading_overlay_ui(ctx: &egui::Context, loading: &LoadingState, locale: Locale) {
    if !loading.is_loading {
        return;
    }

    // Semi-transparent backdrop
    egui::Area::new(egui::Id::new("loading_backdrop"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen = ctx.content_rect();
            ui.painter().rect_filled(
                screen,
                0.0,
                egui::Color32::from_black_alpha(150),
            );
        });

    // Loading window
    egui::Window::new(t(locale, UiTextKey::LoadingTitle))
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .fixed_size(egui::vec2(300.0, 120.0))
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgb(40, 40, 40))
                .corner_radius(egui::CornerRadius::same(8)),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                // Stage text
                let stage_text = match &loading.stage {
                    LoadingStage::Reading => t(locale, UiTextKey::LoadingReading),
                    LoadingStage::Parsing => t(locale, UiTextKey::LoadingParsing),
                    LoadingStage::Building => t(locale, UiTextKey::LoadingBuilding),
                    LoadingStage::Error(e) => {
                        // Show error inline; we handle it below
                        let _ = e;
                        t(locale, UiTextKey::LoadingError)
                    }
                    _ => t(locale, UiTextKey::LoadingGeneric),
                };
                ui.label(
                    egui::RichText::new(stage_text)
                        .size(16.0)
                        .color(egui::Color32::from_rgb(200, 200, 200)),
                );

                ui.add_space(4.0);

                // File name
                ui.label(
                    egui::RichText::new(&loading.file_name)
                        .small()
                        .color(egui::Color32::from_rgb(130, 130, 130)),
                );

                ui.add_space(8.0);

                // Progress bar
                ui.add(egui::ProgressBar::new(loading.progress).show_percentage());

                // Error message if applicable
                if let LoadingStage::Error(msg) = &loading.stage {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(msg)
                            .small()
                            .color(egui::Color32::from_rgb(239, 68, 68)),
                    );
                }
            });
        });
}
