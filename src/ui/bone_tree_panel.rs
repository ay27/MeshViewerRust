use bevy_egui::egui;
use std::collections::HashSet;

use crate::resources::{AnimationData, Locale, RenderMode, RenderModeState, UiTextKey, t};

/// Floating bone hierarchy panel shown in Skeleton mode.
/// `sidebar_right_edge` is the x-coordinate where the left sidebar ends.
pub fn bone_tree_overlay_ui(
    ctx: &egui::Context,
    render_mode: &RenderModeState,
    animation_data: &mut AnimationData,
    sidebar_right_edge: f32,
    locale: Locale,
) {
    if render_mode.current_mode != RenderMode::Skeleton {
        return;
    }

    let x_offset = sidebar_right_edge + 10.0;
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); animation_data.skeleton.len()];
    let mut roots: Vec<usize> = Vec::new();

    for (idx, bone) in animation_data.skeleton.iter().enumerate() {
        if let Some(parent) = bone.parent {
            if parent < children.len() {
                children[parent].push(idx);
            } else {
                roots.push(idx);
            }
        } else {
            roots.push(idx);
        }
    }

    let mut animated_bones: HashSet<usize> = HashSet::new();
    if !animation_data.clips.is_empty() {
        let clip_idx = animation_data.active_clip.min(animation_data.clips.len() - 1);
        for bone_name in animation_data.clips[clip_idx].tracks.keys() {
            if let Some(bone_idx) = animation_data.bone_lookup.get(bone_name) {
                animated_bones.insert(*bone_idx);
            }
        }
    }

    egui::Window::new(egui::RichText::new("骨骼树").size(11.0))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(x_offset, 180.0))
        .default_width(220.0)
        .resizable(true)
        .collapsible(true)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_black_alpha(180))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::same(10)),
        )
        .show(ctx, |ui| {
            // Reduce tree indentation and keep the panel visually compact.
            ui.spacing_mut().indent = 10.0;

            ui.label(
                egui::RichText::new(format!(
                    "{}: {}",
                    t(locale, UiTextKey::BoneCount),
                    animation_data.skeleton.len()
                ))
                    .small()
                    .color(egui::Color32::from_rgb(156, 163, 175)),
            );
            ui.separator();

            if animation_data.skeleton.is_empty() {
                ui.label(
                    egui::RichText::new(t(locale, UiTextKey::BoneModelNoSkeleton))
                        .small()
                        .color(egui::Color32::from_rgb(209, 213, 219)),
                );
                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(360.0)
                .show(ui, |ui| {
                    for &root in &roots {
                        draw_bone_node(ui, root, &children, animation_data, &animated_bones, 0);
                    }
                });
        });
}

fn draw_bone_node(
    ui: &mut egui::Ui,
    idx: usize,
    children: &[Vec<usize>],
    animation_data: &mut AnimationData,
    animated_bones: &HashSet<usize>,
    depth: usize,
) {
    if idx >= animation_data.skeleton.len() {
        return;
    }
    let bone = &animation_data.skeleton[idx];
    let has_children = idx < children.len() && !children[idx].is_empty();
    let is_selected = animation_data.selected_bone == Some(idx);
    let is_animated = animated_bones.contains(&idx);
    let text_color = if is_selected {
        egui::Color32::from_rgb(250, 204, 21)
    } else if is_animated {
        egui::Color32::from_rgb(74, 222, 128)
    } else {
        egui::Color32::from_rgb(229, 231, 235)
    };
    let title = egui::RichText::new(&bone.name).small().color(text_color);

    if has_children {
        let resp = egui::CollapsingHeader::new(title)
            .id_salt(("bone_node", idx))
            .default_open(depth < 1)
            .show(ui, |ui| {
                for &child_idx in &children[idx] {
                    draw_bone_node(ui, child_idx, children, animation_data, animated_bones, depth + 1);
                }
            });
        if resp.header_response.clicked() {
            animation_data.selected_bone = Some(idx);
        }
    } else {
        let resp = ui.selectable_label(is_selected, title);
        if resp.clicked() {
            animation_data.selected_bone = Some(idx);
        }
    }
}
