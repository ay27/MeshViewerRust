use bevy_egui::egui;

use crate::resources::{Locale, UiTextKey, t};
use crate::resources::uv_data::UvPanelState;

/// UV tab content inside the right panel.
pub fn uv_tab_ui(ui: &mut egui::Ui, state: &mut UvPanelState, locale: Locale) {
    // --- Mesh selection ---
    ui.horizontal(|ui| {
        ui.label(t(locale, UiTextKey::UvMesh));
        egui::ComboBox::from_id_salt("uv_mesh_select")
            .selected_text(match state.selected_mesh_index {
                None => t(locale, UiTextKey::UvAllMeshes).to_string(),
                Some(i) => format!("{} {}", t(locale, UiTextKey::UvMeshN), i),
            })
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(state.selected_mesh_index.is_none(), t(locale, UiTextKey::UvAllMeshes))
                    .clicked()
                {
                    state.selected_mesh_index = None;
                }
                for i in 0..8 {
                    if ui
                        .selectable_label(
                            state.selected_mesh_index == Some(i),
                            format!("{} {}", t(locale, UiTextKey::UvMeshN), i),
                        )
                        .clicked()
                    {
                        state.selected_mesh_index = Some(i);
                    }
                }
            });
    });

    // --- UV channel tabs ---
    ui.horizontal(|ui| {
        ui.label(t(locale, UiTextKey::UvChannel));
        for ch in 0..4 {
            if ui
                .selectable_label(state.selected_uv_channel == ch, format!("UV{}", ch))
                .clicked()
            {
                state.selected_uv_channel = ch;
            }
        }
    });

    ui.separator();

    // --- UV Canvas ---
    let available_size = ui.available_size();
    let canvas_size = available_size.x.min(available_size.y - 60.0).max(50.0);

    let (response, painter) = ui.allocate_painter(
        egui::vec2(canvas_size, canvas_size),
        egui::Sense::click_and_drag(),
    );

    let rect = response.rect;

    // Background
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 20, 20));

    // Grid
    if state.show_grid {
        draw_uv_grid(&painter, rect, state.zoom, state.pan_offset);
    }

    // Placeholder text
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        t(locale, UiTextKey::UvPreviewPlaceholder),
        egui::FontId::proportional(12.0),
        egui::Color32::from_rgb(100, 100, 100),
    );

    // Handle pan
    if response.dragged() {
        state.pan_offset[0] += response.drag_delta().x;
        state.pan_offset[1] += response.drag_delta().y;
    }

    // Handle zoom via scroll
    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            state.zoom *= 1.0 + scroll * 0.001;
            state.zoom = state.zoom.clamp(0.1, 50.0);
        }
    }

    ui.separator();

    // --- Bottom controls ---
    ui.horizontal(|ui| {
        if ui.button(t(locale, UiTextKey::UvReset)).clicked() {
            state.zoom = 1.0;
            state.pan_offset = [0.0, 0.0];
        }
        if ui.button(t(locale, UiTextKey::UvFit)).clicked() {
            state.zoom = 1.0;
            state.pan_offset = [0.0, 0.0];
        }
        ui.label(t(locale, UiTextKey::UvZoom));
        ui.add(egui::Slider::new(&mut state.zoom, 0.1..=50.0).logarithmic(true));
    });
    ui.checkbox(&mut state.show_grid, t(locale, UiTextKey::UvGrid));
}

fn draw_uv_grid(painter: &egui::Painter, rect: egui::Rect, zoom: f32, offset: [f32; 2]) {
    let grid_color = egui::Color32::from_rgb(50, 50, 50);
    let grid_count = 10;

    for i in 0..=grid_count {
        let t = i as f32 / grid_count as f32;
        // Vertical line
        let x = rect.left() + t * rect.width() * zoom + offset[0];
        if x >= rect.left() && x <= rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(1.0, grid_color),
            );
        }
        // Horizontal line
        let y = rect.top() + t * rect.height() * zoom + offset[1];
        if y >= rect.top() && y <= rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.0, grid_color),
            );
        }
    }

    // UV space boundary
    let border_color = egui::Color32::from_rgb(80, 80, 80);
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(2.0, border_color),
        egui::StrokeKind::Inside,
    );
}
