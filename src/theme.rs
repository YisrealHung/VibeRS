use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Stroke, Style, Visuals};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ThemeMode {
    Dark,
    #[default]
    Light,
}

static CURRENT_THEME: AtomicU8 = AtomicU8::new(1); // 0 = Dark, 1 = Light

pub fn get_theme_mode() -> ThemeMode {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        0 => ThemeMode::Dark,
        _ => ThemeMode::Light,
    }
}

pub fn set_theme_mode(mode: ThemeMode) {
    let val = match mode {
        ThemeMode::Dark => 0,
        ThemeMode::Light => 1,
    };
    CURRENT_THEME.store(val, Ordering::Relaxed);
}

/// Dynamic Theme Palette supporting both Dark and Light modes (Antigravity Minimalist Style)
pub struct Palette;

impl Palette {
    pub fn bg_sidebar() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(243, 244, 246) // #F3F4F6
        } else {
            Color32::from_rgb(20, 20, 23) // #141417
        }
    }

    pub fn bg_editor() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(255, 255, 255) // #FFFFFF
        } else {
            Color32::from_rgb(26, 26, 30) // #1A1A1E
        }
    }

    pub fn bg_tab_bar() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(235, 236, 240) // #EBECF0
        } else {
            Color32::from_rgb(17, 17, 20) // #111114
        }
    }

    pub fn bg_input() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(240, 241, 245)
        } else {
            Color32::from_rgb(32, 32, 37) // #202025
        }
    }

    pub fn bg_hover() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(228, 230, 236)
        } else {
            Color32::from_rgb(38, 38, 44) // #26262C
        }
    }

    pub fn bg_selected() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(218, 222, 235)
        } else {
            Color32::from_rgb(45, 46, 56) // #2D2E38
        }
    }

    pub fn border() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(216, 218, 226) // #D8DAE2
        } else {
            Color32::from_rgb(42, 42, 48) // #2A2A30
        }
    }

    pub fn text_primary() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(25, 26, 32)
        } else {
            Color32::from_rgb(228, 228, 234) // #E4E4EA
        }
    }

    pub fn text_muted() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(105, 108, 120)
        } else {
            Color32::from_rgb(142, 142, 154) // #8E8E9A
        }
    }

    pub fn text_dim() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(148, 150, 162)
        } else {
            Color32::from_rgb(92, 92, 104) // #5C5C68
        }
    }

    pub fn accent() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(30, 95, 230) // #1E5FE6
        } else {
            Color32::from_rgb(110, 145, 245) // #6E91F5
        }
    }

    pub fn green() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(35, 140, 60)
        } else {
            Color32::from_rgb(115, 195, 130)
        }
    }

    pub fn amber() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(195, 110, 20)
        } else {
            Color32::from_rgb(225, 175, 105)
        }
    }

    // Syntax colors
    pub fn syn_keyword() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(140, 60, 20) // 咖啡色 (Coffee Brown Keyword)
        } else {
            Color32::from_rgb(198, 145, 235)
        }
    }

    pub fn syn_string() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(35, 125, 55)
        } else {
            Color32::from_rgb(152, 195, 121)
        }
    }

    pub fn syn_comment() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(125, 130, 142)
        } else {
            Color32::from_rgb(100, 106, 120)
        }
    }

    pub fn syn_type() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(165, 85, 35) // 淺咖啡色 / 摩卡咖啡 (Mocha Type)
        } else {
            Color32::from_rgb(120, 185, 240)
        }
    }

    /// Highlight text color for emphasis (bold, headers, key terms)
    /// In Dark mode: clean white. In Light mode: rich coffee brown
    pub fn highlight_text() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(138, 65, 25) // 咖啡色 (Coffee Brown Highlight)
        } else {
            Color32::WHITE
        }
    }

    /// Inline code text color
    /// In Dark mode: accent cyan/blue. In Light mode: rich coffee brown
    pub fn inline_code_text() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(148, 72, 28) // 咖啡色 (Coffee Brown Inline Code)
        } else {
            Self::accent()
        }
    }

    /// Inline code background
    pub fn inline_code_bg() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(245, 238, 230) // 柔和拿鐵米白底 (Latte Cream BG)
        } else {
            Self::bg_input()
        }
    }

    pub fn user_bubble_fill() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(230, 233, 242)
        } else {
            Color32::from_rgb(44, 45, 52)
        }
    }

    pub fn user_bubble_border() -> Color32 {
        if get_theme_mode() == ThemeMode::Light {
            Color32::from_rgb(205, 210, 225)
        } else {
            Color32::from_rgb(64, 66, 76)
        }
    }

    // UPPERCASE helper functions for backwards-compatible call sites
    #[allow(non_snake_case, dead_code)]
    pub fn BG_SIDEBAR() -> Color32 { Self::bg_sidebar() }
    #[allow(non_snake_case, dead_code)]
    pub fn BG_EDITOR() -> Color32 { Self::bg_editor() }
    #[allow(non_snake_case, dead_code)]
    pub fn BG_TAB_BAR() -> Color32 { Self::bg_tab_bar() }
    #[allow(non_snake_case, dead_code)]
    pub fn BG_INPUT() -> Color32 { Self::bg_input() }
    #[allow(non_snake_case, dead_code)]
    pub fn BG_HOVER() -> Color32 { Self::bg_hover() }
    #[allow(non_snake_case, dead_code)]
    pub fn BG_SELECTED() -> Color32 { Self::bg_selected() }
    #[allow(non_snake_case, dead_code)]
    pub fn BORDER() -> Color32 { Self::border() }
    #[allow(non_snake_case, dead_code)]
    pub fn TEXT_PRIMARY() -> Color32 { Self::text_primary() }
    #[allow(non_snake_case, dead_code)]
    pub fn TEXT_MUTED() -> Color32 { Self::text_muted() }
    #[allow(non_snake_case, dead_code)]
    pub fn TEXT_DIM() -> Color32 { Self::text_dim() }
    #[allow(non_snake_case, dead_code)]
    pub fn ACCENT() -> Color32 { Self::accent() }
    #[allow(non_snake_case, dead_code)]
    pub fn GREEN() -> Color32 { Self::green() }
    #[allow(non_snake_case, dead_code)]
    pub fn AMBER() -> Color32 { Self::amber() }
    #[allow(non_snake_case, dead_code)]
    pub fn SYN_KEYWORD() -> Color32 { Self::syn_keyword() }
    #[allow(non_snake_case, dead_code)]
    pub fn SYN_STRING() -> Color32 { Self::syn_string() }
    #[allow(non_snake_case, dead_code)]
    pub fn SYN_COMMENT() -> Color32 { Self::syn_comment() }
    #[allow(non_snake_case, dead_code)]
    pub fn SYN_TYPE() -> Color32 { Self::syn_type() }
}

pub fn configure_fonts(ctx: &egui::Context) -> String {
    let mut fonts = FontDefinitions::default();
    let mut loaded_info = Vec::new();

    // Dynamically locate Windows Fonts directory across different drives/machines
    let win_dir = std::env::var("SystemRoot")
        .or_else(|_| std::env::var("windir"))
        .unwrap_or_else(|_| "C:\\Windows".to_string());
    let sys_fonts_dir = std::path::Path::new(&win_dir).join("Fonts");
    let user_fonts_dir = std::env::var("LOCALAPPDATA")
        .ok()
        .map(|lad| std::path::Path::new(&lad).join("Microsoft").join("Windows").join("Fonts"));

    let try_read_font = |filename: &str| -> Option<Vec<u8>> {
        if let Some(user_dir) = &user_fonts_dir {
            let p = user_dir.join(filename);
            if let Ok(bytes) = std::fs::read(&p) {
                return Some(bytes);
            }
        }
        let p = sys_fonts_dir.join(filename);
        std::fs::read(&p).ok()
    };

    let mono_candidates = [
        ("CascadiaMono", "CascadiaMono.ttf"),
        ("Consolas", "consola.ttf"),
    ];
    for (name, filename) in mono_candidates {
        if let Some(bytes) = try_read_font(filename) {
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

    // Load both Traditional and Simplified Chinese fonts and Emoji to ensure 100% glyph coverage
    let cjk_fonts = [
        ("MicrosoftJhengHei", "msjh.ttc"),
        ("MicrosoftYaHei", "msyh.ttc"),
        ("NotoSansTC", "NotoSansTC-VF.ttf"),
        ("SegoeUIEmoji", "seguiemj.ttf"),
    ];
    for (name, filename) in cjk_fonts {
        if let Some(bytes) = try_read_font(filename) {
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
        }
    }

    ctx.set_fonts(fonts);
    loaded_info.join(" + ")
}

pub fn apply_vibers_theme(ctx: &egui::Context) {
    let mode = get_theme_mode();
    let mut style = Style::default();
    let mut visuals = match mode {
        ThemeMode::Dark => Visuals::dark(),
        ThemeMode::Light => Visuals::light(),
    };

    visuals.panel_fill = Palette::bg_sidebar();
    visuals.window_fill = Palette::bg_editor();
    visuals.extreme_bg_color = Palette::bg_tab_bar();
    visuals.faint_bg_color = Palette::bg_editor();
    visuals.code_bg_color = Palette::bg_editor();

    visuals.override_text_color = Some(Palette::text_primary());
    visuals.window_stroke = Stroke::new(1.0, Palette::border());

    let radius = CornerRadius::same(4);
    visuals.widgets.noninteractive.bg_fill = Palette::bg_sidebar();
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Palette::border());
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Palette::text_primary());
    visuals.widgets.noninteractive.corner_radius = radius;

    visuals.widgets.inactive.bg_fill = Palette::bg_input();
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Palette::border());
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Palette::text_primary());
    visuals.widgets.inactive.corner_radius = radius;

    visuals.widgets.hovered.bg_fill = Palette::bg_hover();
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Palette::border());
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Palette::text_primary());
    visuals.widgets.hovered.corner_radius = radius;

    visuals.widgets.active.bg_fill = Palette::bg_selected();
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Palette::accent());
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Palette::text_primary());
    visuals.widgets.active.corner_radius = radius;

    visuals.selection.bg_fill = Palette::accent().gamma_multiply(0.25);
    visuals.selection.stroke = Stroke::new(1.0, Palette::accent());

    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(6.0, 3.0);
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style.spacing.scroll.bar_width = 8.0;

    ctx.set_style(style);
}
