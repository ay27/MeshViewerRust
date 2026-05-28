use bevy::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

/// Sort column in the file browser.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum SortField {
    #[default]
    Name,
    Modified,
}

/// A node in the file/directory tree.
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_directory: bool,
    pub size: u64,
    pub modified: SystemTime,
    pub depth: u32,
    pub children: Vec<TreeNode>,
    /// Lazy-load flag: `false` means children have not been scanned yet.
    pub is_loaded: bool,
}

impl Default for TreeNode {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: PathBuf::new(),
            is_directory: false,
            size: 0,
            modified: SystemTime::UNIX_EPOCH,
            depth: 0,
            children: Vec::new(),
            is_loaded: false,
        }
    }
}

/// State of the file browser sidebar.
#[derive(Resource)]
pub struct FileBrowserState {
    pub root_path: Option<PathBuf>,
    pub tree_nodes: Vec<TreeNode>,
    pub search_query: String,
    pub show_only_3d_files: bool,
    pub show_directories: bool,
    pub sort_field: SortField,
    pub sort_ascending: bool,
    pub selected_path: Option<PathBuf>,
    pub expanded_dirs: HashSet<PathBuf>,
}

impl Default for FileBrowserState {
    fn default() -> Self {
        Self {
            root_path: None,
            tree_nodes: Vec::new(),
            search_query: String::new(),
            show_only_3d_files: true,
            show_directories: true,
            sort_field: SortField::Name,
            sort_ascending: true,
            selected_path: None,
            expanded_dirs: HashSet::new(),
        }
    }
}
