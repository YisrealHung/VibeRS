use crate::state::AppState;
use crate::theme::Palette;
use egui::{
    text::LayoutJob, Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke, TextFormat,
};

pub fn render_center_panel(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default()
        .frame(
            Frame::NONE
                .fill(Palette::BG_EDITOR)
                .inner_margin(Margin::ZERO),
        )
        .show(ctx, |ui| {
            // 1. Multi-File Tab Bar
            egui::TopBottomPanel::top("center_tabs_bar")
                .exact_height(32.0)
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_TAB_BAR)
                        .inner_margin(Margin::symmetric(6, 0)),
                )
                .show_inside(ui, |ui| {
                    render_tab_bar(ui, state);
                });

            if state.open_tabs.is_empty() {
                egui::CentralPanel::default()
                    .frame(Frame::NONE.fill(Palette::BG_EDITOR))
                    .show_inside(ui, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                RichText::new("從左側點擊檔案開啟分頁，或按 Ctrl+O 選擇其他資料夾")
                                    .size(13.5)
                                    .color(Palette::TEXT_DIM),
                            );
                        });
                    });
                return;
            }

            // 2. Subtle Breadcrumb Bar
            egui::TopBottomPanel::top("center_breadcrumb_bar")
                .exact_height(24.0)
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_EDITOR)
                        .inner_margin(Margin::symmetric(14, 0)),
                )
                .show_inside(ui, |ui| {
                    if let Some(tab) = state.active_tab() {
                        let breadcrumb = tab.rel_path.replace('/', "  ›  ");
                        let is_dirty = tab.is_dirty;
                        ui.horizontal_centered(|ui| {
                            ui.label(
                                RichText::new(breadcrumb)
                                    .size(11.5)
                                    .color(Palette::TEXT_DIM),
                            );
                            if is_dirty {
                                ui.label(
                                    RichText::new("● 未儲存 (Ctrl+S)")
                                        .size(11.0)
                                        .color(Palette::AMBER),
                                );
                            }
                        });
                    }
                });

            // 3. Code Editor Area (6px right margin keeps scrollbar clear of right panel's resize handle)
            egui::CentralPanel::default()
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_EDITOR)
                        .inner_margin(Margin {
                            left: 0,
                            right: 6,
                            top: 4,
                            bottom: 0,
                        }),
                )
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("center_code_editor_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let code_width = (ui.available_width() - 68.0).max(120.0);

                            if let Some(tab) = state.active_tab_mut() {
                                let line_count = tab.content.lines().count().max(1);
                                let mut line_nums = String::with_capacity(line_count * 5);
                                for i in 1..=line_count {
                                    if i > 1 {
                                        line_nums.push('\n');
                                    }
                                    line_nums.push_str(&format!("{i:>4}"));
                                }

                                ui.horizontal_top(|ui| {
                                    ui.add_space(6.0);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(line_nums)
                                                .font(FontId::monospace(13.0))
                                                .color(Palette::TEXT_DIM),
                                        )
                                        .selectable(false),
                                    );

                                    ui.add_space(8.0);

                                    let is_rust = tab.name.ends_with(".rs");
                                    let mut layouter =
                                        |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                                            let job = highlight_code(string, is_rust);
                                            ui.fonts(|f| f.layout_job(job))
                                        };

                                    let resp = ui.add(
                                        egui::TextEdit::multiline(&mut tab.content)
                                            .font(FontId::monospace(13.0))
                                            .frame(false)
                                            .code_editor()
                                            .desired_width(code_width)
                                            .desired_rows(32)
                                            .layouter(&mut layouter),
                                    );

                                    if resp.changed() {
                                        tab.is_dirty = true;
                                    }
                                });
                            }
                        });
                });
        });
}

fn render_tab_bar(ui: &mut egui::Ui, state: &mut AppState) {
    egui::ScrollArea::horizontal()
        .id_salt("editor_tabs_scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;

                let mut tab_to_activate = None;
                let mut tab_to_close = None;

                for (idx, tab) in state.open_tabs.iter().enumerate() {
                    let is_active = idx == state.active_tab_idx;
                    let bg = if is_active {
                        Palette::BG_EDITOR
                    } else {
                        Palette::BG_TAB_BAR
                    };
                    let border_top = if is_active {
                        Palette::ACCENT
                    } else {
                        Color32::TRANSPARENT
                    };

                    let tab_frame = Frame::NONE
                        .fill(bg)
                        .inner_margin(Margin::symmetric(10, 5))
                        .corner_radius(CornerRadius {
                            nw: 4,
                            ne: 4,
                            sw: 0,
                            se: 0,
                        })
                        .stroke(Stroke::new(
                            1.0,
                            if is_active {
                                Palette::BORDER
                            } else {
                                Color32::TRANSPARENT
                            },
                        ));

                    let inner_resp = tab_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let text_col = if is_active {
                                Color32::WHITE
                            } else {
                                Palette::TEXT_MUTED
                            };

                            let label_resp = ui.add(
                                egui::Label::new(
                                    RichText::new(&tab.name).size(12.0).color(text_col),
                                )
                                .sense(egui::Sense::click()),
                            );
                            if label_resp.clicked() {
                                tab_to_activate = Some(idx);
                            }
                            if label_resp.middle_clicked() {
                                tab_to_close = Some(idx);
                            }

                            if tab.is_dirty {
                                ui.label(RichText::new("●").size(9.0).color(Palette::AMBER));
                            }

                            let close_resp = ui.add(
                                egui::Button::new(
                                    RichText::new("×").size(12.0).color(Palette::TEXT_DIM),
                                )
                                .frame(false),
                            );
                            if close_resp.clicked() {
                                tab_to_close = Some(idx);
                            }
                        });
                    });

                    if is_active {
                        let rect = inner_resp.response.rect;
                        ui.painter().line_segment(
                            [rect.left_top(), rect.right_top()],
                            Stroke::new(2.0, border_top),
                        );
                    }
                }

                if let Some(idx) = tab_to_activate {
                    state.active_tab_idx = idx;
                }
                if let Some(idx) = tab_to_close {
                    state.close_tab(idx);
                }
            });
        });
}

fn highlight_code(text: &str, is_rust: bool) -> LayoutJob {
    let mut job = LayoutJob::default();
    let font_id = FontId::monospace(13.0);

    if !is_rust {
        job.append(
            text,
            0.0,
            TextFormat {
                font_id,
                color: Palette::TEXT_PRIMARY,
                ..Default::default()
            },
        );
        return job;
    }

    const KEYWORDS: &[&str] = &[
        "fn", "pub", "mod", "use", "let", "mut", "struct", "enum", "impl", "for", "in", "if",
        "else", "match", "return", "self", "Self", "super", "crate", "const", "static", "where",
        "trait", "type", "async", "await", "move", "ref", "true", "false", "unsafe", "extern",
    ];

    for (line_idx, line) in text.split('\n').enumerate() {
        if line_idx > 0 {
            job.append(
                "\n",
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: Palette::TEXT_PRIMARY,
                    ..Default::default()
                },
            );
        }

        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            job.append(
                line,
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: Palette::SYN_COMMENT,
                    ..Default::default()
                },
            );
            continue;
        }

        let mut chars = line.char_indices().peekable();
        while let Some(&(start, ch)) = chars.peek() {
            if ch == '"' {
                chars.next();
                let mut end = line.len();
                let mut escaped = false;
                for (i, c) in chars.by_ref() {
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        end = i + 1;
                        break;
                    }
                }
                job.append(
                    &line[start..end],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Palette::SYN_STRING,
                        ..Default::default()
                    },
                );
            } else if ch.is_alphabetic() || ch == '_' {
                let mut end = line.len();
                while let Some(&(i, c)) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        chars.next();
                    } else {
                        end = i;
                        break;
                    }
                }
                let word = &line[start..end];
                let color = if KEYWORDS.contains(&word) {
                    Palette::SYN_KEYWORD
                } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                    Palette::SYN_TYPE
                } else {
                    Palette::TEXT_PRIMARY
                };
                job.append(
                    word,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );
            } else {
                chars.next();
                let next_pos = chars.peek().map(|&(i, _)| i).unwrap_or(line.len());
                job.append(
                    &line[start..next_pos],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: Palette::TEXT_PRIMARY,
                        ..Default::default()
                    },
                );
            }
        }
    }

    job
}
