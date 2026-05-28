use bevy::prelude::*;

/// Which tab is active in the right panel.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum RightPanelTab {
    #[default]
    Info,
    Material,
    Uv,
}

/// UI layout state — panel visibility, sizes, active tab.
#[derive(Resource)]
pub struct UiState {
    /// Left-side file browser panel visible.
    pub show_sidebar: bool,
    /// Right-side properties panel visible.
    pub show_right_panel: bool,
    /// Active tab inside the right panel.
    pub right_panel_tab: RightPanelTab,
    /// Stats overlay floating window visible.
    pub show_stats_overlay: bool,
    /// Settings dialog visible.
    pub show_settings_dialog: bool,
    /// Loading overlay visible.
    pub show_loading_overlay: bool,
    /// Remembered sidebar width (default 280.0).
    pub sidebar_width: f32,
    /// Remembered right panel width (default 320.0).
    pub right_panel_width: f32,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_sidebar: false,
            show_right_panel: false,
            right_panel_tab: RightPanelTab::Info,
            show_stats_overlay: true,
            show_settings_dialog: false,
            show_loading_overlay: false,
            sidebar_width: 280.0,
            right_panel_width: 320.0,
        }
    }
}

/// Toggle the right panel. If the requested tab is already active and the panel
/// is visible, hide the panel; otherwise show the panel with the requested tab.
pub fn toggle_right_panel(state: &mut UiState, tab: RightPanelTab) {
    if state.show_right_panel && state.right_panel_tab == tab {
        state.show_right_panel = false;
    } else {
        state.show_right_panel = true;
        state.right_panel_tab = tab;
    }
}
