use bevy::prelude::*;
use bevy_egui::egui;
use std::path::PathBuf;

use crate::resources::events::UiAction;
use crate::resources::file_browser_state::{FileBrowserState, SortField, TreeNode};
use crate::resources::{Locale, UiTextKey, t};
use crate::utils::{is_3d_file, truncate_path};

/// Render the file browser sidebar content (called inside a `SidePanel`).
pub fn file_browser_ui(
    ui: &mut egui::Ui,
    state: &mut FileBrowserState,
    actions: &mut MessageWriter<UiAction>,
    locale: Locale,
) {
    // --- Search bar ---
    ui.horizontal(|ui| {
        ui.label("🔍");
        ui.text_edit_singleline(&mut state.search_query);
        if !state.search_query.is_empty() && ui.small_button("✕").clicked() {
            state.search_query.clear();
        }
    });

    // --- Filter options ---
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.show_directories, t(locale, UiTextKey::FileBrowserFolders));
        ui.checkbox(&mut state.show_only_3d_files, t(locale, UiTextKey::FileBrowserOnly3d));
    });

    ui.separator();

    // --- Breadcrumb navigation ---
    if let Some(ref root) = state.root_path.clone() {
        ui.horizontal(|ui| {
            if ui
                .small_button("↑")
                .on_hover_text(t(locale, UiTextKey::FileBrowserParentDirectory))
                .clicked()
            {
                if let Some(parent) = root.parent() {
                    state.root_path = Some(parent.to_path_buf());
                    state.tree_nodes.clear();
                }
            }
            let path_str = root.to_string_lossy();
            ui.label(
                egui::RichText::new(truncate_path(&path_str, 30))
                    .small()
                    .color(egui::Color32::from_rgb(156, 163, 175)),
            )
            .on_hover_text(path_str.as_ref());
        });
    }

    // --- Sort controls ---
    ui.horizontal(|ui| {
        let name_arrow = sort_arrow(state.sort_field == SortField::Name, state.sort_ascending);
        if ui
            .small_button(format!(
                "{} {name_arrow}",
                t(locale, UiTextKey::FileBrowserSortName)
            ))
            .clicked()
        {
            toggle_sort(state, SortField::Name);
        }
        let mod_arrow = sort_arrow(state.sort_field == SortField::Modified, state.sort_ascending);
        if ui
            .small_button(format!(
                "{} {mod_arrow}",
                t(locale, UiTextKey::FileBrowserSortModified)
            ))
            .clicked()
        {
            toggle_sort(state, SortField::Modified);
        }
    });

    ui.separator();

    // --- File tree ---
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if state.tree_nodes.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(t(locale, UiTextKey::FileBrowserNoFilesFound))
                            .color(egui::Color32::from_rgb(107, 114, 128)),
                    );
                });
                return;
            }

            let search = state.search_query.to_lowercase();
            let show_only_3d = state.show_only_3d_files;
            let show_dirs = state.show_directories;

            // We need to clone the nodes to iterate while mutating state
            let nodes = state.tree_nodes.clone();
            for node in &nodes {
                // When "Folders" is off, hide the root directory row itself but still
                // render files directly under the current root directory.
                if node.is_directory && !show_dirs {
                    if state.expanded_dirs.contains(&node.path) {
                        for child in &node.children {
                            if should_show_node(child, &search, show_only_3d, show_dirs) {
                                render_tree_node(ui, child, state, actions, 0);
                            }
                        }
                    }
                } else if should_show_node(node, &search, show_only_3d, show_dirs) {
                    render_tree_node(ui, node, state, actions, 0);
                }
            }
        });
}

fn should_show_node(node: &TreeNode, search: &str, show_only_3d: bool, show_dirs: bool) -> bool {
    if node.is_directory {
        return show_dirs;
    }
    if show_only_3d && !is_3d_file(&node.name) {
        return false;
    }
    if !search.is_empty() && !node.name.to_lowercase().contains(search) {
        return false;
    }
    true
}

fn render_tree_node(
    ui: &mut egui::Ui,
    node: &TreeNode,
    state: &mut FileBrowserState,
    actions: &mut MessageWriter<UiAction>,
    depth: u32,
) {
    let indent = depth as f32 * 16.0;
    let is_selected = state.selected_path.as_ref() == Some(&node.path);
    let is_expanded = state.expanded_dirs.contains(&node.path);

    ui.horizontal(|ui| {
        ui.add_space(indent);

        if node.is_directory {
            let arrow = if is_expanded { "▼" } else { "▶" };
            if ui.small_button(arrow).clicked() {
                if is_expanded {
                    state.expanded_dirs.remove(&node.path);
                } else {
                    state.expanded_dirs.insert(node.path.clone());
                }
            }
            ui.label("📁");
            let resp = ui.add(
                egui::SelectableLabel::new(is_selected, &node.name)
                    .truncate(),
            );
            if resp.clicked() {
                state.selected_path = Some(node.path.clone());
            }
        } else {
            ui.add_space(16.0); // align with directories (no arrow)
            let icon = if is_3d_file(&node.name) {
                "🗋"
            } else {
                "📄"
            };
            ui.label(icon);

            let resp = ui.add(
                egui::SelectableLabel::new(is_selected, &node.name)
                    .truncate(),
            );
            if resp.clicked() {
                state.selected_path = Some(node.path.clone());
                actions.write(UiAction::OpenFile(node.path.clone()));
            }
        }
    });

    // Recursively render children
    if node.is_directory && is_expanded {
        for child in &node.children {
            let search = state.search_query.to_lowercase();
            if should_show_node(child, &search, state.show_only_3d_files, state.show_directories) {
                render_tree_node(ui, child, state, actions, depth + 1);
            }
        }
    }
}

fn sort_arrow(is_active: bool, ascending: bool) -> &'static str {
    if !is_active {
        " "
    } else if ascending {
        "↑"
    } else {
        "↓"
    }
}

fn toggle_sort(state: &mut FileBrowserState, field: SortField) {
    if state.sort_field == field {
        state.sort_ascending = !state.sort_ascending;
    } else {
        state.sort_field = field;
        state.sort_ascending = true;
    }
}

// ---------------------------------------------------------------------------
// Directory lazy-load Bevy system
// ---------------------------------------------------------------------------

/// Bevy system that scans directories when they are expanded.
pub fn directory_lazy_load_system(mut browser: ResMut<FileBrowserState>) {
    // Collect dirs that need loading
    let dirs_to_load: Vec<PathBuf> = browser
        .expanded_dirs
        .iter()
        .filter(|dir| {
            find_node(&browser.tree_nodes, dir)
                .map(|n| !n.is_loaded)
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    for dir in dirs_to_load {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut children: Vec<TreeNode> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let metadata = e.metadata().ok()?;
                    let name = e.file_name().to_string_lossy().to_string();
                    Some(TreeNode {
                        name,
                        path: e.path(),
                        is_directory: metadata.is_dir(),
                        size: metadata.len(),
                        modified: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                        depth: 0,
                        children: Vec::new(),
                        is_loaded: false,
                    })
                })
                .collect();

            // Sort directories first, then by name
            children.sort_by(|a, b| {
                b.is_directory
                    .cmp(&a.is_directory)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });

            if let Some(node) = find_node_mut(&mut browser.tree_nodes, &dir) {
                node.children = children;
                node.is_loaded = true;
            }
        }
    }
}

fn find_node<'a>(nodes: &'a [TreeNode], path: &PathBuf) -> Option<&'a TreeNode> {
    for node in nodes {
        if &node.path == path {
            return Some(node);
        }
        if let Some(found) = find_node(&node.children, path) {
            return Some(found);
        }
    }
    None
}

fn find_node_mut<'a>(nodes: &'a mut [TreeNode], path: &PathBuf) -> Option<&'a mut TreeNode> {
    for node in nodes.iter_mut() {
        if &node.path == path {
            return Some(node);
        }
        if let Some(found) = find_node_mut(&mut node.children, path) {
            return Some(found);
        }
    }
    None
}
