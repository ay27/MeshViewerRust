use bevy_egui::egui;

/// Apply the dark theme to the egui context.
pub fn apply_dark_theme(ctx: &egui::Context) {
    // --- Font setup: load a system font as fallback for CJK & Unicode symbols ---
    setup_fonts(ctx);

    let mut visuals = egui::Visuals::dark();

    // Window / panel backgrounds
    visuals.panel_fill = egui::Color32::from_rgb(30, 30, 30); // #1e1e1e
    visuals.window_fill = egui::Color32::from_rgb(35, 35, 35); // #232323
    visuals.extreme_bg_color = egui::Color32::from_rgb(20, 20, 20); // #141414

    // Widget colors
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 45, 45);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 50, 50);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(64, 64, 64); // #404040
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(37, 99, 235); // #2563eb

    // Borders
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64));
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64));

    // Text strokes
    visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(229, 231, 235)); // #e5e7eb
    visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200));
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    // Selection highlight
    visuals.selection.bg_fill = egui::Color32::from_rgb(37, 99, 235); // #2563eb
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    // Corner radius
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);

    // Window shadow
    visuals.window_shadow = egui::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: egui::Color32::from_black_alpha(100),
    };

    // Window corner radius
    visuals.window_corner_radius = egui::CornerRadius::same(6);

    ctx.set_visuals(visuals);

    // Spacing adjustments
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    ctx.set_style(style);
}

/// Load a system CJK font as fallback so Unicode symbols (geometric shapes, CJK, etc.)
/// render correctly instead of showing as blank rectangles.
fn setup_fonts(ctx: &egui::Context) {
    // Candidate system font paths — try each in order, use the first one found.
    let candidates: &[&str] = &[
        // macOS
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/PingFang.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
        // Linux — Noto Sans CJK
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        // Windows
        "C:\\Windows\\Fonts\\msyh.ttc",    // Microsoft YaHei
        "C:\\Windows\\Fonts\\simsun.ttc",   // SimSun
    ];

    let font_data = candidates.iter().find_map(|path| {
        std::fs::read(path).ok().map(|data| {
            eprintln!("[theme] Loaded system font: {path}");
            data
        })
    });

    let Some(data) = font_data else {
        eprintln!("[theme] No system CJK font found; Unicode symbols may render as □");
        return;
    };

    let mut fonts = egui::FontDefinitions::default();

    // Register the system font
    fonts.font_data.insert(
        "system_cjk".to_owned(),
        std::sync::Arc::new(egui::FontData::from_owned(data)),
    );

    // Append it as a fallback for the Proportional family (after the defaults)
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("system_cjk".to_owned());

    // Also for Monospace
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("system_cjk".to_owned());

    ctx.set_fonts(fonts);
}

/// Color constants used throughout the UI.
#[allow(dead_code)]
pub mod colors {
    use bevy_egui::egui::Color32;

    // Background layers
    pub const BG_DARKEST: Color32 = Color32::from_rgb(17, 24, 39); // #111827
    pub const BG_DARK: Color32 = Color32::from_rgb(30, 30, 30); // #1e1e1e
    pub const BG_PANEL: Color32 = Color32::from_rgb(32, 32, 32); // #202020
    pub const BG_ELEVATED: Color32 = Color32::from_rgb(45, 45, 45); // #2d2d2d
    pub const BG_INPUT: Color32 = Color32::from_rgb(48, 48, 48); // #303030

    // Borders
    pub const BORDER: Color32 = Color32::from_rgb(64, 64, 64); // #404040
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(75, 85, 99); // #4b5563

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(229, 231, 235); // #e5e7eb
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(156, 163, 175); // #9ca3af
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(107, 114, 128); // #6b7280

    // Accent
    pub const ACCENT: Color32 = Color32::from_rgb(37, 99, 235); // #2563eb
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(29, 78, 216); // #1d4ed8
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94); // #22c55e
    pub const WARNING: Color32 = Color32::from_rgb(234, 179, 8); // #eab308
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68); // #ef4444
}
