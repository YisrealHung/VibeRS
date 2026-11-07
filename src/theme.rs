use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Stroke, Style, Visuals};

/// Clean Modern Dark Palette (Antigravity / Cursor Minimalist Style)
pub struct Palette;

impl Palette {
    pub const BG_SIDEBAR: Color32 = Color32::from_rgb(20, 20, 23);    // #141417
    pub const BG_EDITOR: Color32 = Color32::from_rgb(26, 26, 30);     // #1A1A1E
    pub const BG_TAB_BAR: Color32 = Color32::from_rgb(17, 17, 20);    // #111114
    pub const BG_INPUT: Color32 = Color32::from_rgb(32, 32, 37);      // #202025
    pub const BG_HOVER: Color32 = Color32::from_rgb(38, 38, 44);      // #26262C
    pub const BG_SELECTED: Color32 = Color32::from_rgb(45, 46, 56);   // #2D2E38

    pub const BORDER: Color32 = Color32::from_rgb(42, 42, 48);        // #2A2A30

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(228, 228, 234); // #E4E4EA
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(142, 142, 154);   // #8E8E9A
    pub const TEXT_DIM: Color32 = Color32::from_rgb(92, 92, 104);       // #5C5C68

    pub const ACCENT: Color32 = Color32::from_rgb(110, 145, 245);       // #6E91F5
    pub const GREEN: Color32 = Color32::from_rgb(115, 195, 130);
    pub const AMBER: Color32 = Color32::from_rgb(225, 175, 105);

    // Syntax colors
    pub const SYN_KEYWORD: Color32 = Color32::from_rgb(198, 145, 235);
    pub const SYN_STRING: Color32 = Color32::from_rgb(152, 195, 121);
    pub const SYN_COMMENT: Color32 = Color32::from_rgb(100, 106, 120);
    pub const SYN_TYPE: Color32 = Color32::from_rgb(120, 185, 240);
}

pub fn configure_fonts(ctx: &egui::Context) -> String {
    let mut fonts = FontDefinitions::default();
    let mut loaded_info = Vec::new();

    let mono_candidates = [
        ("CascadiaMono", r"C:\Windows\Fonts\CascadiaMono.ttf"),
        ("Consolas", r"C:\Windows\Fonts\consola.ttf"),
    ];
    for (name, path) in mono_candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
            fonts
                .font_data
                .insert(name.to_owned(), FontData::from_static(leaked).into());
            if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
                vec.insert(0, name.to_owned());
            }
            if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
                vec.insert(0, name.to_owned());
            }
            loaded_info.push(name.to_string());
            break;
        }
    }

    let cjk_candidates = [
        ("NotoSansTC", r"C:\Windows\Fonts\NotoSansTC-VF.ttf"),
        ("MicrosoftJhengHeiLight", r"C:\Windows\Fonts\msjhl.ttc"),
        ("MicrosoftJhengHei", r"C:\Windows\Fonts\msjh.ttc"),
    ];
    for (name, path) in cjk_candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
            fonts
                .font_data
                .insert(name.to_owned(), FontData::from_static(leaked).into());
            if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
                vec.push(name.to_owned());
            }
            if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
                vec.push(name.to_owned());
            }
            loaded_info.push(name.to_string());
            break;
        }
    }

    ctx.set_fonts(fonts);
    loaded_info.join(" + ")
}

pub fn apply_vibers_theme(ctx: &egui::Context) {
    let mut style = Style::default();
    let mut visuals = Visuals::dark();

    visuals.panel_fill = Palette::BG_SIDEBAR;
    visuals.window_fill = Palette::BG_EDITOR;
    visuals.extreme_bg_color = Palette::BG_TAB_BAR;
    visuals.faint_bg_color = Palette::BG_EDITOR;
    visuals.code_bg_color = Palette::BG_EDITOR;

    visuals.override_text_color = Some(Palette::TEXT_PRIMARY);
    visuals.window_stroke = Stroke::new(1.0, Palette::BORDER);

    let radius = CornerRadius::same(4);
    visuals.widgets.noninteractive.bg_fill = Palette::BG_SIDEBAR;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Palette::BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Palette::TEXT_PRIMARY);
    visuals.widgets.noninteractive.corner_radius = radius;

    visuals.widgets.inactive.bg_fill = Palette::BG_INPUT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Palette::BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Palette::TEXT_PRIMARY);
    visuals.widgets.inactive.corner_radius = radius;

    visuals.widgets.hovered.bg_fill = Palette::BG_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Palette::BORDER);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.hovered.corner_radius = radius;

    visuals.widgets.active.bg_fill = Palette::BG_SELECTED;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Palette::ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.corner_radius = radius;

    visuals.selection.bg_fill = Palette::ACCENT.gamma_multiply(0.25);
    visuals.selection.stroke = Stroke::new(1.0, Palette::ACCENT);

    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(6.0, 3.0);
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style.spacing.scroll.bar_width = 8.0;

    ctx.set_style(style);
}
