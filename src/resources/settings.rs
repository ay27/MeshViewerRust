use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::i18n::Locale;
use super::render_mode::RenderMode;

/// Color theme for the application.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

/// Common rendering settings (shared across render modes).
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct CommonSettings {
    /// Background color as [r, g, b] in 0.0..=1.0.
    pub background_color: [f32; 3],
    /// Directional light intensity.
    pub light_intensity: f32,
    /// Show ground grid helper.
    #[serde(default = "default_true", alias = "show_grid")]
    pub show_ground_grid: bool,
    /// Show world-axis gizmos helper.
    #[serde(default = "default_true", alias = "show_axis")]
    pub show_axis_gizmos: bool,
    /// Show stats overlay by default.
    pub show_stats: bool,
    /// Normalize imported model to centered 10m x 10m x 10m bbox.
    #[serde(default = "default_false")]
    pub normalize_imported_model: bool,
}

impl Default for CommonSettings {
    fn default() -> Self {
        Self {
            // Near-black background — gives the mid-grey Solid mesh a clean,
            // high-contrast stage and matches Blender's default viewport
            // gradient bottom. Not pure black so the ground grid is still
            // faintly visible.
            background_color: [0.08, 0.08, 0.08],
            light_intensity: 4.0,
            show_ground_grid: true,
            show_axis_gizmos: true,
            show_stats: true,
            normalize_imported_model: false,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

/// Settings specific to Solid render mode (Blender-aligned).
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(default)]
pub struct SolidModeSettings {
    /// Uniform color applied to every surface in Solid mode.
    /// Defaults to Blender's familiar 0.8 grey.
    #[serde(alias = "mesh_color")]
    pub single_color: [f32; 3],
    /// When true, cull back-faces (Blender's Backface Culling).
    /// Note: old toml field `double_side` had inverted semantics — no alias.
    pub backface_culling: bool,
    /// Enable specular highlights (matches Blender's Specular Lighting toggle).
    pub specular_highlights: bool,
    /// PBR roughness when highlights are enabled.
    pub roughness: f32,
}

impl Default for SolidModeSettings {
    fn default() -> Self {
        Self {
            // Mid-grey (0.55) rather than Blender's 0.8 because our 3-light
            // setup produces stronger PBR diffuse than Blender Workbench's
            // matcap-ish shader; 0.8 would blow out into white under the key.
            single_color: [0.55, 0.55, 0.55],
            backface_culling: false,
            specular_highlights: true,
            // Soft highlights — keeps speculars present but not glassy.
            roughness: 0.5,
        }
    }
}

/// Settings specific to Wireframe render mode.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct WireframeModeSettings {
    pub wireframe_color: [f32; 3],
    pub show_points: bool,
    pub point_color: [f32; 3],
    pub point_size: f32,
    pub use_vertex_colors: bool,
    #[serde(default = "default_false")]
    pub xray: bool,
}

impl Default for WireframeModeSettings {
    fn default() -> Self {
        Self {
            wireframe_color: [0.0, 0.0, 0.6],
            show_points: true,
            point_color: [0.855, 1.0, 0.0],
            point_size: 0.1,
            use_vertex_colors: false,
            xray: false,
        }
    }
}

/// Settings specific to Skeleton render mode.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct SkeletonModeSettings {
    /// Mesh opacity in Skeleton mode (0.0 transparent, 1.0 opaque).
    pub mesh_alpha: f32,
    /// Skeleton line color as [r, g, b].
    pub bone_color: [f32; 3],
    /// Desired bone line width hint for debug drawing.
    pub bone_line_width: f32,
    /// Draw skeleton with X-Ray style visibility (reduced occlusion).
    pub always_visible: bool,
}

impl Default for SkeletonModeSettings {
    fn default() -> Self {
        Self {
            mesh_alpha: 0.1,
            bone_color: [1.0, 0.3, 0.1],
            bone_line_width: 1.0,
            always_visible: true,
        }
    }
}

/// Settings specific to Animation render mode.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct AnimationModeSettings {
    /// Animation playback speed multiplier.
    pub playback_speed: f32,
}

impl Default for AnimationModeSettings {
    fn default() -> Self {
        Self {
            playback_speed: 1.0,
        }
    }
}

/// Persistent startup render mode (animation mode disabled for now).
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum StartupRenderMode {
    #[serde(alias = "Mesh")]
    Solid,
    Wireframe,
    Material,
    #[default]
    Component,
    Skeleton,
}

impl StartupRenderMode {
    pub fn to_render_mode(self) -> RenderMode {
        match self {
            StartupRenderMode::Solid => RenderMode::Solid,
            StartupRenderMode::Wireframe => RenderMode::Wireframe,
            StartupRenderMode::Material => RenderMode::Material,
            StartupRenderMode::Component => RenderMode::Component,
            StartupRenderMode::Skeleton => RenderMode::Skeleton,
        }
    }
}

fn default_startup_render_mode() -> StartupRenderMode {
    StartupRenderMode::default()
}

/// Persistent application settings, serialised to TOML.
#[derive(Resource, Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct AppSettings {
    /// Optional path to the Blender executable.
    pub blender_path: Option<String>,
    /// Active UI theme.
    pub theme: Theme,
    #[serde(default)]
    pub locale: Locale,
    pub common: CommonSettings,
    #[serde(default = "default_startup_render_mode")]
    pub default_startup_mode: StartupRenderMode,
    #[serde(default, alias = "mesh_mode")]
    pub solid_mode: SolidModeSettings,
    pub wireframe_mode: WireframeModeSettings,
    pub skeleton_mode: SkeletonModeSettings,
    pub animation_mode: AnimationModeSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            blender_path: None,
            theme: Theme::Dark,
            locale: Locale::default(),
            common: CommonSettings::default(),
            default_startup_mode: StartupRenderMode::default(),
            solid_mode: SolidModeSettings::default(),
            wireframe_mode: WireframeModeSettings::default(),
            skeleton_mode: SkeletonModeSettings::default(),
            animation_mode: AnimationModeSettings::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Persistence helpers
// ---------------------------------------------------------------------------

/// Return the platform-specific config directory for MeshViewer.
fn config_dir() -> PathBuf {
    ProjectDirs::from("com", "meshviewer", "MeshViewer")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Full path to the settings file.
fn settings_path() -> PathBuf {
    config_dir().join("settings.toml")
}

impl AppSettings {
    /// Load settings from disk. Falls back to `Default` on any error.
    pub fn load() -> Self {
        let path = settings_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Persist the current settings to disk (best-effort).
    pub fn save(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(content) = toml::to_string_pretty(self) {
            std::fs::write(path, content).ok();
        }
    }
}
