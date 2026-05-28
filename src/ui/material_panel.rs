use bevy_egui::egui;
use std::collections::HashSet;

use crate::resources::{Locale, UiTextKey, t};
use crate::resources::material_data::{MaterialData, MaterialInfo, MaterialPanelState};

/// Material tab content inside the right panel.
pub fn material_tab_ui(
    ui: &mut egui::Ui,
    state: &mut MaterialPanelState,
    data: &MaterialData,
    locale: Locale,
) {
    // --- Stats grid ---
    egui::Grid::new("mat_stats")
        .spacing(egui::vec2(8.0, 4.0))
        .show(ui, |ui| {
            let total = data.materials.len();
            let used = data
                .mesh_material_map
                .iter()
                .flat_map(|e| &e.material_indices)
                .collect::<HashSet<_>>()
                .len();

            ui.label(format!("{}:", t(locale, UiTextKey::LabelMeshes)));
            ui.label(format!("{}", data.mesh_material_map.len()));
            ui.label(format!("{}:", t(locale, UiTextKey::LabelMaterials)));
            ui.label(format!("{}", total));
            ui.end_row();
            ui.label(t(locale, UiTextKey::MaterialUsed));
            ui.label(format!("{}", used));
            ui.end_row();
        });

    ui.separator();

    // --- Search ---
    ui.horizontal(|ui| {
        ui.label("🔍");
        ui.text_edit_singleline(&mut state.search_query);
    });

    ui.separator();

    // --- Mesh-Material tree ---
    let available_height = ui.available_height();
    let tree_height = available_height * state.tree_section_ratio;

    egui::ScrollArea::vertical()
        .max_height(tree_height)
        .show(ui, |ui| {
            let search_lower = state.search_query.to_lowercase();

            for (idx, entry) in data.mesh_material_map.iter().enumerate() {
                // Filter by search
                if !search_lower.is_empty() && !entry.mesh_name.to_lowercase().contains(&search_lower) {
                    continue;
                }

                let is_expanded = state.expanded_meshes.contains(&idx);
                let header = format!(
                    "{} {} ({} {})",
                    if is_expanded { "▼" } else { "▶" },
                    entry.mesh_name,
                    entry.material_indices.len(),
                    t(locale, UiTextKey::MaterialCountSuffix),
                );

                if ui.selectable_label(false, &header).clicked() {
                    if is_expanded {
                        state.expanded_meshes.remove(&idx);
                    } else {
                        state.expanded_meshes.insert(idx);
                    }
                }

                if is_expanded {
                    for &mat_idx in &entry.material_indices {
                        if let Some(mat) = data.materials.get(mat_idx) {
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                // Color swatch
                                let color = egui::Color32::from_rgba_unmultiplied(
                                    (mat.base_color[0] * 255.0) as u8,
                                    (mat.base_color[1] * 255.0) as u8,
                                    (mat.base_color[2] * 255.0) as u8,
                                    255,
                                );
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(12.0, 12.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 2.0, color);

                                let selected = state.selected_material_index == Some(mat_idx);
                                if ui.selectable_label(selected, &mat.name).clicked() {
                                    state.selected_material_index = Some(mat_idx);
                                }
                            });
                        }
                    }
                }
            }
        });

    ui.separator();

    // --- Material detail ---
    if let Some(idx) = state.selected_material_index {
        if let Some(mat) = data.materials.get(idx) {
            material_detail_ui(ui, mat, locale);
        }
    }
}

fn material_detail_ui(ui: &mut egui::Ui, mat: &MaterialInfo, locale: Locale) {
    ui.heading(&mat.name);
    ui.separator();

    egui::Grid::new("mat_detail")
        .spacing(egui::vec2(8.0, 6.0))
        .show(ui, |ui| {
            // Base Color (read-only display)
            ui.label(t(locale, UiTextKey::MaterialBaseColor));
            let mut color = [mat.base_color[0], mat.base_color[1], mat.base_color[2]];
            ui.color_edit_button_rgb(&mut color);
            ui.end_row();

            // Metallic
            ui.label(t(locale, UiTextKey::SettingsMetalness));
            let mut metallic = mat.metallic;
            ui.add(egui::Slider::new(&mut metallic, 0.0..=1.0));
            ui.end_row();

            // Roughness
            ui.label(t(locale, UiTextKey::SettingsRoughness));
            let mut roughness = mat.roughness;
            ui.add(egui::Slider::new(&mut roughness, 0.0..=1.0));
            ui.end_row();

            // Texture info
            if mat.has_base_color_texture {
                ui.label(t(locale, UiTextKey::MaterialAlbedoMap));
                ui.label("✓");
                ui.end_row();
            }
            if mat.has_normal_texture {
                ui.label(t(locale, UiTextKey::MaterialNormalMap));
                ui.label("✓");
                ui.end_row();
            }
            if mat.has_roughness_texture {
                ui.label(t(locale, UiTextKey::MaterialRoughnessMap));
                ui.label("✓");
                ui.end_row();
            }
        });
}
