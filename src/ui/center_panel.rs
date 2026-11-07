use crate::diff::{accept_hunk_in_snapshot, compute_hunks, revert_hunk_in_text, DiffOpKind};
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
                                RichText::new("從左側點擊檔案開啟分頁，或按 Ctrl+P 快速搜尋檔案")
                                    .size(13.5)
                                    .color(Palette::TEXT_DIM),
                            );
                        });
                    });
                return;
            }

            let mut tab_action_accept_hunk = None;
            let mut tab_action_revert_hunk = None;

            // 2. Subtle Breadcrumb Bar & Diff Mode Toggle
            egui::TopBottomPanel::top("center_breadcrumb_bar")
                .exact_height(28.0)
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_EDITOR)
                        .inner_margin(Margin::symmetric(14, 0)),
                )
                .show_inside(ui, |ui| {
                    if let Some(tab) = state.active_tab_mut() {
                        let breadcrumb = tab.rel_path.replace('/', "  ›  ");
                        let is_dirty = tab.is_dirty;
                        let has_diff = tab.content != tab.disk_snapshot;

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

                            // Right-aligned Diff Toggle
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if has_diff {
                                    let diff_label = if tab.show_diff { "Code 模式" } else { "⚡ Diff 模式" };
                                    if ui
                                        .selectable_label(
                                            tab.show_diff,
                                            RichText::new(diff_label)
                                                .size(11.5)
                                                .color(if tab.show_diff { Palette::ACCENT } else { Palette::AMBER }),
                                        )
                                        .clicked()
                                    {
                                        tab.show_diff = !tab.show_diff;
                                    }
                                }
                            });
                        });
                    }
                });

            // 3. Code Editor Area or Inline Diff Area
            let is_showing_diff = state.active_tab().map(|t| t.show_diff).unwrap_or(false);

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
                    if is_showing_diff {
                        render_inline_diff_view(ui, state, &mut tab_action_accept_hunk, &mut tab_action_revert_hunk);
                    } else {
                        render_code_editor_view(ui, state);
                    }
                });

            // Handle hunk actions
            if let Some(hunk_id) = tab_action_accept_hunk {
                if let Some(tab) = state.active_tab_mut() {
                    tab.disk_snapshot = accept_hunk_in_snapshot(&tab.disk_snapshot, &tab.content, hunk_id);
                    let _ = std::fs::write(&tab.full_path, &tab.content);
                    if tab.disk_snapshot == tab.content {
                        tab.is_dirty = false;
                        tab.show_diff = false;
                    }
                    state.status_text = format!("✓ 已接受 Hunk #{hunk_id} 並存檔");
                }
            }
            if let Some(hunk_id) = tab_action_revert_hunk {
                if let Some(tab) = state.active_tab_mut() {
                    tab.content = revert_hunk_in_text(&tab.disk_snapshot, &tab.content, hunk_id);
                    let _ = std::fs::write(&tab.full_path, &tab.content);
                    if tab.disk_snapshot == tab.content {
                        tab.is_dirty = false;
                        tab.show_diff = false;
                    }
                    state.status_text = format!("✗ 已還原 Hunk #{hunk_id} 至快照");
                }
            }
        });
}

fn render_code_editor_view(ui: &mut egui::Ui, state: &mut AppState) {
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
}

fn render_inline_diff_view(
    ui: &mut egui::Ui,
    state: &AppState,
    tab_action_accept: &mut Option<usize>,
    tab_action_revert: &mut Option<usize>,
) {
    let Some(tab) = state.active_tab() else {
        return;
    };

    let hunks = compute_hunks(&tab.disk_snapshot, &tab.content);

    if hunks.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("✨ 目前與磁碟快照完全相同 (無差異)")
                    .size(13.0)
                    .color(Palette::GREEN),
            );
        });
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("center_inline_diff_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for hunk in &hunks {
                Frame::NONE
                    .fill(Palette::BG_SIDEBAR)
                    .stroke(Stroke::new(1.0, Palette::BORDER))
                    .corner_radius(CornerRadius::same(5))
                    .inner_margin(Margin::ZERO)
                    .show(ui, |ui| {
                        // Hunk header bar
                        Frame::NONE
                            .fill(Palette::BG_INPUT)
                            .inner_margin(Margin::symmetric(10, 5))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "Hunk #{}  @@ -{},{} +{},{} @@",
                                            hunk.id, hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
                                        ))
                                        .monospace()
                                        .size(11.5)
                                        .color(Palette::TEXT_MUTED),
                                    );

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui
                                            .button(RichText::new("✗ Revert").size(11.5).color(Color32::from_rgb(240, 100, 100)))
                                            .clicked()
                                        {
                                            *tab_action_revert = Some(hunk.id);
                                        }

                                        if ui
                                            .button(RichText::new("✓ Accept").size(11.5).color(Palette::GREEN))
                                            .clicked()
                                        {
                                            *tab_action_accept = Some(hunk.id);
                                        }
                                    });
                                });
                            });

                        // Lines table
                        Frame::NONE
                            .inner_margin(Margin::symmetric(6, 4))
                            .show(ui, |ui| {
                                for line in &hunk.lines {
                                    let (bg, prefix, text_color) = match line.kind {
                                        DiffOpKind::Equal => (Color32::TRANSPARENT, " ", Palette::TEXT_PRIMARY),
                                        DiffOpKind::Delete => (
                                            Color32::from_rgba_premultiplied(70, 25, 30, 160),
                                            "-",
                                            Color32::from_rgb(250, 150, 160),
                                        ),
                                        DiffOpKind::Insert => (
                                            Color32::from_rgba_premultiplied(25, 65, 35, 160),
                                            "+",
                                            Color32::from_rgb(150, 235, 160),
                                        ),
                                    };

                                    Frame::NONE
                                        .fill(bg)
                                        .inner_margin(Margin::symmetric(6, 1))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                let old_no = line.old_line.map(|n| format!("{n:>3}")).unwrap_or_else(|| "   ".into());
                                                let new_no = line.new_line.map(|n| format!("{n:>3}")).unwrap_or_else(|| "   ".into());

                                                ui.label(
                                                    RichText::new(format!("{old_no} {new_no} {prefix}"))
                                                        .monospace()
                                                        .size(12.0)
                                                        .color(Palette::TEXT_DIM),
                                                );

                                                ui.label(
                                                    RichText::new(&line.text)
                                                        .monospace()
                                                        .size(12.5)
                                                        .color(text_color),
                                                );
                                            });
                                        });
                                }
                            });
                    });
                ui.add_space(8.0);
            }
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

                            if tab.is_dirty || tab.content != tab.disk_snapshot {
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
                    state.request_close_tab(idx);
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
