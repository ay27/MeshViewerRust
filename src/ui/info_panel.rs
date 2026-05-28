use bevy_egui::egui;

use crate::resources::{ComponentColorMap, Locale, ModelStats, RenderMode, RenderModeState, UiState, UiTextKey, t};
use crate::utils::format_number;

/// Floating stats overlay, anchored to top-left of the viewport.
/// `sidebar_right_edge` is the pixel x-coordinate where the sidebar ends (0 if hidden).
pub fn stats_overlay_ui(
    ctx: &egui::Context,
    stats: &ModelStats,
    ui_state: &UiState,
    sidebar_right_edge: f32,
    locale: Locale,
) {
    if !ui_state.show_stats_overlay {
        return;
    }

    // Offset the overlay so it sits to the right of the sidebar
    let x_offset = sidebar_right_edge + 10.0;

    egui::Window::new(t(locale, UiTextKey::ModelInfoTitle))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(x_offset, 50.0))
        .resizable(false)
        .collapsible(true)
        .title_bar(false)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_black_alpha(180))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::same(10)),
        )
        .show(ctx, |ui| {
            let text_color = egui::Color32::from_rgb(200, 200, 200);

            egui::Grid::new("stats_grid")
                .spacing(egui::vec2(12.0, 4.0))
                .show(ui, |ui| {
                    stat_row(ui, t(locale, UiTextKey::LabelMeshes), &format!("{}", stats.num_meshes), text_color);
                    stat_row(ui, t(locale, UiTextKey::LabelVertices), &format_number(stats.num_vertices), text_color);
                    stat_row(ui, t(locale, UiTextKey::LabelFaces), &format_number(stats.num_faces), text_color);
                    stat_row(
                        ui,
                        t(locale, UiTextKey::LabelBbox),
                        &format_bbox(&stats.bbox_size),
                        text_color,
                    );
                    stat_row(
                        ui,
                        t(locale, UiTextKey::LabelMaterials),
                        &format!("{}", stats.num_materials),
                        text_color,
                    );
                });
        });
}

/// Info tab content inside the right panel.
pub fn info_tab_ui(
    ui: &mut egui::Ui,
    stats: &ModelStats,
    render_mode: &RenderModeState,
    color_map: &ComponentColorMap,
    locale: Locale,
) {
    ui.heading(t(locale, UiTextKey::ModelStatistics));
    ui.separator();

    let text_color = egui::Color32::from_rgb(229, 231, 235);

    egui::Grid::new("info_tab_grid")
        .spacing(egui::vec2(12.0, 6.0))
        .show(ui, |ui| {
            stat_row(ui, t(locale, UiTextKey::LabelMeshes), &format!("{}", stats.num_meshes), text_color);
            stat_row(ui, t(locale, UiTextKey::LabelVertices), &format_number(stats.num_vertices), text_color);
            stat_row(ui, t(locale, UiTextKey::LabelFaces), &format_number(stats.num_faces), text_color);
            stat_row(
                ui,
                t(locale, UiTextKey::LabelBbox),
                &format_bbox(&stats.bbox_size),
                text_color,
            );
            stat_row(
                ui,
                t(locale, UiTextKey::LabelMaterials),
                &format!("{}", stats.num_materials),
                text_color,
            );
            stat_row(
                ui,
                t(locale, UiTextKey::LabelTextures),
                &format!("{}", stats.num_textures),
                text_color,
            );
        });

    // Show component color legend when in Component mode
    if render_mode.current_mode == RenderMode::Component && !color_map.entries.is_empty() {
        ui.add_space(8.0);
        ui.separator();
        ui.heading(t(locale, UiTextKey::ComponentColors));
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for entry in &color_map.entries {
                    ui.horizontal(|ui| {
                        // Color swatch
                        let color = egui::Color32::from_rgba_unmultiplied(
                            (entry.color[0] * 255.0) as u8,
                            (entry.color[1] * 255.0) as u8,
                            (entry.color[2] * 255.0) as u8,
                            255,
                        );
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(14.0, 14.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(rect, 2.0, color);

                        ui.label(
                            egui::RichText::new(&entry.mesh_name)
                                .color(egui::Color32::from_rgb(229, 231, 235))
                                .small(),
                        );
                    });
                }
            });
    }
}

/// Format bounding box size with auto unit (m/cm/mm).
fn format_bbox(size: &[f32; 3]) -> String {
    let max_dim = size[0].max(size[1]).max(size[2]);
    if max_dim == 0.0 {
        return "0 m".to_string();
    }
    // glTF spec: 1 unit = 1 meter. Auto-scale the display unit.
    if max_dim >= 1.0 {
        format!("{:.2} x {:.2} x {:.2} m", size[0], size[1], size[2])
    } else if max_dim >= 0.01 {
        format!(
            "{:.1} x {:.1} x {:.1} cm",
            size[0] * 100.0,
            size[1] * 100.0,
            size[2] * 100.0
        )
    } else {
        format!(
            "{:.1} x {:.1} x {:.1} mm",
            size[0] * 1000.0,
            size[1] * 1000.0,
            size[2] * 1000.0
        )
    }
}

fn stat_row(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    ui.label(
        egui::RichText::new(label)
            .color(egui::Color32::from_rgb(156, 163, 175))
            .small(),
    );
    ui.label(egui::RichText::new(value).color(color).small().strong());
    ui.end_row();
}
