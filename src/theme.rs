use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, Rounding, Stroke, Vec2, Visuals,
};

// Antigravity-inspired deep charcoal-graphite palette
pub const BG_PRIMARY: Color32 = Color32::from_rgb(24, 26, 32); // #181A20 main canvas
pub const BG_PANEL: Color32 = Color32::from_rgb(29, 31, 39); // #1D1F27 sidebars
pub const BG_HEADER: Color32 = Color32::from_rgb(29, 31, 39); // #1D1F27 top chrome
pub const BG_SURFACE: Color32 = Color32::from_rgb(35, 38, 48); // #232630 elevated cards/inputs
pub const BG_ROW_ALT: Color32 = Color32::from_rgb(26, 28, 35); // #1A1C23 subtle zebra
pub const BG_ROW_HOVER: Color32 = Color32::from_rgb(39, 42, 54); // #272A36 hover surface
pub const BG_ROW_SELECTED: Color32 = Color32::from_rgb(45, 49, 64); // #2D3140 selected pill
pub const BG_ROW_COMPARE: Color32 = Color32::from_rgb(48, 40, 66); // subtle violet compare

pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(43, 46, 58); // #2B2E3A hairline border
pub const BORDER_ELEVATED: Color32 = Color32::from_rgb(56, 60, 76); // #383C4C button/card border

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(230, 232, 238); // #E6E8EE crisp off-white
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(155, 161, 176); // #9BA1B0 calm secondary
pub const TEXT_MUTED: Color32 = Color32::from_rgb(104, 110, 126); // #686E7E subtle muted

// Refined Antigravity-style accents
pub const ACCENT_CYAN: Color32 = Color32::from_rgb(139, 164, 248); // #8BA4F8 soft periwinkle-blue
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(82, 196, 130); // #52C482 clean diff green
pub const ACCENT_AMBER: Color32 = Color32::from_rgb(229, 181, 103); // #E5B567 warm amber
pub const ACCENT_ROSE: Color32 = Color32::from_rgb(224, 108, 117); // #E06C75 soft coral red
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(180, 155, 248); // #B49BF8 lavender

pub const DIFF_ADD_BG: Color32 = Color32::from_rgba_premultiplied(24, 56, 40, 210);
pub const DIFF_ADD_FG: Color32 = Color32::from_rgb(138, 224, 172);
pub const DIFF_DEL_BG: Color32 = Color32::from_rgba_premultiplied(64, 28, 36, 210);
pub const DIFF_DEL_FG: Color32 = Color32::from_rgb(240, 154, 162);
pub const DIFF_HUNK_FG: Color32 = Color32::from_rgb(148, 174, 248);

/// Refined high-clarity lane palette for the git graph.
pub const LANE_COLORS: [Color32; 10] = [
    Color32::from_rgb(139, 164, 248), // Periwinkle
    Color32::from_rgb(82, 196, 130),  // Emerald
    Color32::from_rgb(180, 155, 248), // Lavender
    Color32::from_rgb(229, 181, 103), // Warm Amber
    Color32::from_rgb(224, 108, 117), // Coral Rose
    Color32::from_rgb(232, 148, 92),  // Soft Orange
    Color32::from_rgb(86, 198, 188),  // Teal
    Color32::from_rgb(112, 180, 240), // Sky
    Color32::from_rgb(228, 134, 188), // Soft Pink
    Color32::from_rgb(164, 206, 104), // Sage Lime
];

pub fn lane_color(idx: usize) -> Color32 {
    LANE_COLORS[idx % LANE_COLORS.len()]
}

pub const ICON_PNG_BYTES: &[u8] = include_bytes!("../assets/icon.png");

/// Decodes the embedded icon and applies an anti-aliased macOS squircle alpha mask
/// so the outer margin is fully transparent in Cmd+Tab, Dock, and UI.
fn decode_masked_icon_rgba() -> Option<(Vec<u8>, u32, u32)> {
    let src = image::load_from_memory(ICON_PNG_BYTES).ok()?.to_rgba8();
    let (w, h) = src.dimensions();
    let mut out = src.into_raw();

    // The squircle in the source image spans roughly [0.125, 0.875] with corner radius ~0.155
    let cx = (w as f32) * 0.5;
    let cy = (h as f32) * 0.5;
    let half = (w.min(h) as f32) * 0.372;
    let radius = half * 0.43;
    let inner = half - radius;

    for y in 0..h {
        for x in 0..w {
            let dx = ((x as f32 + 0.5) - cx).abs() - inner;
            let dy = ((y as f32 + 0.5) - cy).abs() - inner;
            let dist = (dx.max(0.0).hypot(dy.max(0.0))) + dx.max(dy).min(0.0) - radius;

            // Smooth 1.5px anti-aliased alpha falloff
            let alpha = (0.5 - dist / 1.5).clamp(0.0, 1.0);
            let idx = ((y * w + x) * 4) as usize;
            out[idx + 3] = (alpha * 255.0).round() as u8;
        }
    }

    Some((out, w, h))
}

pub fn load_window_icon() -> Option<egui::IconData> {
    let (rgba, width, height) = decode_masked_icon_rgba()?;
    Some(egui::IconData {
        rgba,
        width,
        height,
    })
}

pub fn load_logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let (rgba, w, h) = decode_masked_icon_rgba()?;
    let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
    Some(ctx.load_texture("gity_x_logo", color_image, egui::TextureOptions::LINEAR))
}

pub fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "plus_jakarta_regular".to_owned(),
        FontData::from_static(include_bytes!(
            "../assets/fonts/PlusJakartaSans-Regular.ttf"
        )),
    );
    fonts.font_data.insert(
        "jetbrains_mono".to_owned(),
        FontData::from_static(include_bytes!(
            "../assets/fonts/JetBrainsMono-Regular.ttf"
        )),
    );

    if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
        prop.insert(0, "plus_jakarta_regular".to_owned());
        // JetBrains Mono fallback provides crisp geometric symbols (✓, ✕, ✗, arrows) with zero extra memory
        prop.insert(1, "jetbrains_mono".to_owned());
    }
    if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
        mono.insert(0, "jetbrains_mono".to_owned());
    }

    ctx.set_fonts(fonts);
}

pub fn apply_theme(ctx: &egui::Context) {
    configure_fonts(ctx);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(7.0, 5.0);
    style.spacing.button_padding = Vec2::new(9.0, 4.5);
    style.spacing.menu_margin = egui::Margin::same(6.0);
    style.spacing.window_margin = egui::Margin::same(14.0);

    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG_PRIMARY;
    visuals.window_fill = BG_PANEL;
    visuals.extreme_bg_color = BG_SURFACE;
    visuals.faint_bg_color = BG_ROW_ALT;
    visuals.window_rounding = Rounding::same(12.0);
    visuals.menu_rounding = Rounding::same(10.0);
    visuals.window_stroke = Stroke::new(1.0_f32, BORDER_ELEVATED);
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: Vec2::new(0.0, 6.0),
        blur: 18.0,
        spread: 0.0,
        color: Color32::from_black_alpha(110),
    };

    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.widgets.noninteractive.rounding = Rounding::same(7.0);

    visuals.widgets.inactive.bg_fill = BG_SURFACE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER_ELEVATED);
    visuals.widgets.inactive.rounding = Rounding::same(7.0);

    visuals.widgets.hovered.bg_fill = BG_ROW_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT_CYAN);
    visuals.widgets.hovered.rounding = Rounding::same(7.0);

    visuals.widgets.active.bg_fill = BG_ROW_SELECTED;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT_CYAN);
    visuals.widgets.active.rounding = Rounding::same(7.0);

    visuals.widgets.open.bg_fill = BG_ROW_SELECTED;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, ACCENT_CYAN);
    visuals.widgets.open.rounding = Rounding::same(7.0);

    // High-contrast selection visuals for all selectable_label, ComboBox, and toggle pills
    visuals.selection.bg_fill = Color32::from_rgb(54, 62, 88);
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);

    style.visuals = visuals;
    ctx.set_style(style);
}
