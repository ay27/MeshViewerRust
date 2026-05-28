use bevy_egui::egui;

use crate::plugins::file_dialog::{spawn_blender_browse_dialog, FileDialogState};
use crate::resources::{AppSettings, Locale, StartupRenderMode, UiState, UiTextKey, t};

/// Render the settings dialog as a floating window.
pub fn settings_dialog_ui(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    settings: &mut AppSettings,
    dialog_state: &mut FileDialogState,
) {
    if !ui_state.show_settings_dialog {
        return;
    }

    let locale = settings.locale;

    egui::Window::new(t(locale, UiTextKey::SettingsTitle))
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .default_height(600.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // === Blender 集成 ===
                egui::CollapsingHeader::new(t(locale, UiTextKey::SettingsBlenderIntegration))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(t(locale, UiTextKey::SettingsPath));
                            let mut path = settings.blender_path.clone().unwrap_or_default();
                            ui.text_edit_singleline(&mut path);
                            settings.blender_path = if path.is_empty() { None } else { Some(path) };
                            if ui.button(t(locale, UiTextKey::SettingsBrowse)).clicked() {
                                spawn_blender_browse_dialog(dialog_state, locale);
                            }
                        });
                    });

                ui.separator();

                // === 通用设置 ===
                egui::CollapsingHeader::new(t(locale, UiTextKey::SettingsCommon))
                    .default_open(true)
                    .show(ui, |ui| {
                        egui::Grid::new("common_settings")
                            .spacing(egui::vec2(8.0, 6.0))
                            .show(ui, |ui| {
                                ui.label(t(locale, UiTextKey::SettingsBackground));
                                ui.color_edit_button_rgb(&mut settings.common.background_color);
                                ui.end_row();

                                ui.label(t(locale, UiTextKey::SettingsLightIntensity));
                                ui.add(egui::Slider::new(
                                    &mut settings.common.light_intensity,
                                    0.0..=10.0,
                                ));
                                ui.end_row();

                                ui.label("");
                                ui.checkbox(
                                    &mut settings.common.show_stats,
                                    t(locale, UiTextKey::SettingsShowStats),
                                );
                                ui.end_row();

                                ui.label("");
                                ui.checkbox(
                                    &mut settings.common.normalize_imported_model,
                                    t(locale, UiTextKey::SettingsNormalizeImportedModel),
                                );
                                ui.end_row();

                                ui.label(t(locale, UiTextKey::SettingsDefaultStartupMode));
                                egui::ComboBox::from_id_salt("default_startup_mode")
                                    .selected_text(startup_mode_label(locale, settings.default_startup_mode))
                                    .show_ui(ui, |ui| {
                                        for mode in [
                                            StartupRenderMode::Wireframe,
                                            StartupRenderMode::Solid,
                                            StartupRenderMode::Component,
                                            StartupRenderMode::Material,
                                            StartupRenderMode::Skeleton,
                                        ] {
                                            ui.selectable_value(
                                                &mut settings.default_startup_mode,
                                                mode,
                                                startup_mode_label(locale, mode),
                                            );
                                        }
                                    });
                                ui.end_row();

                                ui.label(t(locale, UiTextKey::SettingsLanguage));
                                egui::ComboBox::from_id_salt("settings_locale")
                                    .selected_text(match settings.locale {
                                        Locale::Zh => "中文",
                                        Locale::En => "English",
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut settings.locale, Locale::Zh, "中文");
                                        ui.selectable_value(&mut settings.locale, Locale::En, "English");
                                    });
                                ui.end_row();
                            });
                    });

                ui.separator();

                // === 场景辅助 ===
                egui::CollapsingHeader::new(t(locale, UiTextKey::SettingsSceneHelpers))
                    .default_open(true)
                    .show(ui, |ui| {
                        egui::Grid::new("scene_helpers_settings")
                            .spacing(egui::vec2(8.0, 6.0))
                            .show(ui, |ui| {
                                ui.label("");
                                ui.checkbox(
                                    &mut settings.common.show_axis_gizmos,
                                    t(locale, UiTextKey::SettingsShowAxis),
                                );
                                ui.end_row();

                                ui.label("");
                                ui.checkbox(
                                    &mut settings.common.show_ground_grid,
                                    t(locale, UiTextKey::SettingsShowGroundGrid),
                                );
                                ui.end_row();
                            });
                    });

                ui.separator();

                // === Solid 模式 ===
                egui::CollapsingHeader::new(t(locale, UiTextKey::SettingsSolidMode))
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::Grid::new("solid_settings")
                            .spacing(egui::vec2(8.0, 6.0))
                            .show(ui, |ui| {
                                ui.label(t(locale, UiTextKey::SettingsColor));
                                ui.color_edit_button_rgb(&mut settings.solid_mode.single_color);
                                ui.end_row();

                                ui.label("");
                                ui.checkbox(
                                    &mut settings.solid_mode.backface_culling,
                                    t(locale, UiTextKey::SettingsBackfaceCulling),
                                );
                                ui.end_row();

                                ui.label("");
                                ui.checkbox(
                                    &mut settings.solid_mode.specular_highlights,
                                    t(locale, UiTextKey::SettingsSpecularHighlights),
                                );
                                ui.end_row();

                                ui.label(t(locale, UiTextKey::SettingsRoughness));
                                ui.add(egui::Slider::new(
                                    &mut settings.solid_mode.roughness,
                                    0.0..=1.0,
                                ));
                                ui.end_row();
                            });
                    });

                ui.separator();

                // === 线框模式 ===
                egui::CollapsingHeader::new(t(locale, UiTextKey::SettingsWireframeMode))
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::Grid::new("wire_settings")
                            .spacing(egui::vec2(8.0, 6.0))
                            .show(ui, |ui| {
                                ui.label(t(locale, UiTextKey::SettingsWireColor));
                                ui.color_edit_button_rgb(
                                    &mut settings.wireframe_mode.wireframe_color,
                                );
                                ui.end_row();

                                ui.label("");
                                ui.checkbox(
                                    &mut settings.wireframe_mode.show_points,
                                    t(locale, UiTextKey::SettingsShowPoints),
                                );
                                ui.end_row();

                                ui.label(t(locale, UiTextKey::SettingsPointColor));
                                ui.color_edit_button_rgb(&mut settings.wireframe_mode.point_color);
                                ui.end_row();

                                ui.label(t(locale, UiTextKey::SettingsPointSize));
                                ui.add(egui::Slider::new(
                                    &mut settings.wireframe_mode.point_size,
                                    0.01..=5.0,
                                ));
                                ui.end_row();

                                ui.label("");
                                ui.checkbox(
                                    &mut settings.wireframe_mode.use_vertex_colors,
                                    t(locale, UiTextKey::SettingsUseVertexColors),
                                );
                                ui.end_row();

                                ui.label("");
                                ui.checkbox(
                                    &mut settings.wireframe_mode.xray,
                                    t(locale, UiTextKey::SettingsWireframeXray),
                                );
                                ui.end_row();
                            });
                    });

                ui.separator();

                // === 骨骼模式 ===
                egui::CollapsingHeader::new(t(locale, UiTextKey::SettingsSkeletonMode))
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::Grid::new("skeleton_settings")
                            .spacing(egui::vec2(8.0, 6.0))
                            .show(ui, |ui| {
                                ui.label(t(locale, UiTextKey::SettingsMeshAlpha));
                                ui.add(egui::Slider::new(
                                    &mut settings.skeleton_mode.mesh_alpha,
                                    0.0..=1.0,
                                ));
                                ui.end_row();

                                ui.label(t(locale, UiTextKey::SettingsBoneColor));
                                ui.color_edit_button_rgb(&mut settings.skeleton_mode.bone_color);
                                ui.end_row();

                                ui.label(t(locale, UiTextKey::SettingsBoneLineWidth));
                                ui.add(egui::Slider::new(
                                    &mut settings.skeleton_mode.bone_line_width,
                                    0.1..=4.0,
                                ));
                                ui.end_row();

                                ui.label("");
                                ui.checkbox(
                                    &mut settings.skeleton_mode.always_visible,
                                    t(locale, UiTextKey::SettingsAlwaysVisibleXray),
                                );
                                ui.end_row();
                            });
                    });

                ui.separator();
            });

            ui.separator();

            // --- Bottom actions ---
            ui.horizontal(|ui| {
                if ui.button(t(locale, UiTextKey::SettingsResetDefaults)).clicked() {
                    *settings = AppSettings::default();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t(locale, UiTextKey::SettingsClose)).clicked() {
                        ui_state.show_settings_dialog = false;
                    }
                });
            });
        });
}

fn startup_mode_label(locale: Locale, mode: StartupRenderMode) -> &'static str {
    match mode {
        StartupRenderMode::Wireframe => t(locale, UiTextKey::StartupModeWireframe),
        StartupRenderMode::Solid => t(locale, UiTextKey::StartupModeSolid),
        StartupRenderMode::Component => t(locale, UiTextKey::StartupModeComponent),
        StartupRenderMode::Material => t(locale, UiTextKey::StartupModeMaterial),
        StartupRenderMode::Skeleton => t(locale, UiTextKey::StartupModeSkeleton),
    }
}
