use crate::diff::{accept_hunk_in_snapshot, compute_hunks, revert_hunk_in_text, DiffOpKind};
use crate::state::AppState;
use crate::theme::Palette;
use egui::{
    Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke,
};

pub fn render_center_panel(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default()
        .frame(
            Frame::NONE
                .fill(Palette::BG_EDITOR())
                .inner_margin(Margin::ZERO),
        )
        .show(ctx, |ui| {
            // 1. Multi-File Tab Bar
            egui::TopBottomPanel::top("center_tabs_bar")
                .exact_height(32.0)
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_TAB_BAR())
                        .inner_margin(Margin::symmetric(6, 0)),
                )
                .show_inside(ui, |ui| {
                    render_tab_bar(ui, state);
                });

            if state.open_tabs.is_empty() {
                egui::CentralPanel::default()
                    .frame(Frame::NONE.fill(Palette::BG_EDITOR()))
                    .show_inside(ui, |ui| {
                        ui.centered_and_justified(|ui| {
                            let msg = if state.workspace_name.is_empty() {
                                "尚未開啟資料夾 (點選左上角 File -> 開啟資料夾 或按 Ctrl+O)"
                            } else {
                                "從左側點擊檔案開啟分頁，或按 Ctrl+P 快速搜尋檔案"
                            };
                            ui.label(
                                RichText::new(msg)
                                    .size(13.5)
                                    .color(Palette::TEXT_DIM()),
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
                        .fill(Palette::BG_EDITOR())
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
                                    .color(Palette::TEXT_DIM()),
                            );
                            if is_dirty {
                                ui.label(
                                    RichText::new("● 未儲存 (Ctrl+S)")
                                        .size(11.0)
                                        .color(Palette::AMBER()),
                                );
                            }

                            // Right-aligned Diff Toggle & Markdown Mode Toggles
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if tab.is_markdown() {
                                    let is_prev = tab.markdown_mode == crate::state::MarkdownViewMode::Preview;
                                    if ui
                                        .selectable_label(
                                            is_prev,
                                            RichText::new("👁 預覽 (Preview)")
                                                .size(11.5)
                                                .color(if is_prev { Palette::ACCENT() } else { Palette::TEXT_MUTED() }),
                                        )
                                        .on_hover_text("完整 Markdown 預覽模式")
                                        .clicked()
                                    {
                                        tab.markdown_mode = if is_prev {
                                            crate::state::MarkdownViewMode::Editor
                                        } else {
                                            crate::state::MarkdownViewMode::Preview
                                        };
                                    }

                                    let is_split = tab.markdown_mode == crate::state::MarkdownViewMode::Split;
                                    if ui
                                        .selectable_label(
                                            is_split,
                                            RichText::new("◫ 分割 (Split)")
                                                .size(11.5)
                                                .color(if is_split { Palette::ACCENT() } else { Palette::TEXT_MUTED() }),
                                        )
                                        .on_hover_text("左右分割預覽 (即時雙向同步)")
                                        .clicked()
                                    {
                                        tab.markdown_mode = if is_split {
                                            crate::state::MarkdownViewMode::Editor
                                        } else {
                                            crate::state::MarkdownViewMode::Split
                                        };
                                    }

                                    let is_edit = tab.markdown_mode == crate::state::MarkdownViewMode::Editor;
                                    if ui
                                        .selectable_label(
                                            is_edit,
                                            RichText::new("✏ 編輯 (Edit)")
                                                .size(11.5)
                                                .color(if is_edit { Palette::ACCENT() } else { Palette::TEXT_MUTED() }),
                                        )
                                        .on_hover_text("原始碼編輯模式")
                                        .clicked()
                                    {
                                        tab.markdown_mode = crate::state::MarkdownViewMode::Editor;
                                    }

                                    ui.separator();
                                }

                                if has_diff {
                                    let diff_label = if tab.show_diff { "Code 模式" } else { "⚡ Diff 模式" };
                                    if ui
                                        .selectable_label(
                                            tab.show_diff,
                                            RichText::new(diff_label)
                                                .size(11.5)
                                                .color(if tab.show_diff { Palette::ACCENT() } else { Palette::AMBER() }),
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

            // 3. Code Editor Area, Markdown Preview, or Inline Diff Area
            let is_showing_diff = state.active_tab().map(|t| t.show_diff).unwrap_or(false);
            let md_mode = state
                .active_tab()
                .map(|t| (t.is_markdown(), t.markdown_mode))
                .unwrap_or((false, crate::state::MarkdownViewMode::Editor));

            egui::CentralPanel::default()
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_EDITOR())
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
                    } else if let Some(tab) = state.active_tab() {
                        if tab.is_binary {
                            render_binary_file_view(ui, tab);
                        } else if md_mode.0 && md_mode.1 == crate::state::MarkdownViewMode::Preview {
                            render_markdown_preview_view(ui, &tab.content);
                        } else if md_mode.0 && md_mode.1 == crate::state::MarkdownViewMode::Split {
                            render_markdown_split_view(ui, state);
                        } else {
                            render_code_editor_view(ui, state);
                        }
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
                                .color(Palette::TEXT_DIM()),
                        )
                        .selectable(false),
                    );

                    ui.add_space(8.0);

                    let lang = crate::ui::syntax::Language::from_file_name(&tab.name);
                    let mut layouter =
                        |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                            let job = crate::ui::syntax::highlight_code(string, lang);
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

fn render_markdown_preview_view(ui: &mut egui::Ui, content: &str) {
    egui::ScrollArea::vertical()
        .id_salt("center_markdown_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Frame::NONE
                .fill(Palette::BG_EDITOR())
                .inner_margin(Margin::symmetric(24, 16))
                .show(ui, |ui| {
                    ui.set_max_width(900.0);
                    crate::ui::markdown::render_markdown_message(ui, content);
                });
        });
}

fn render_markdown_split_view(ui: &mut egui::Ui, state: &mut AppState) {
    let avail_width = ui.available_width();
    let col_w = ((avail_width - 12.0) * 0.5).max(100.0);

    ui.horizontal_top(|ui| {
        // Left Column: Code Editor
        ui.allocate_ui_with_layout(
            egui::vec2(col_w, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                render_code_editor_view(ui, state);
            },
        );

        // Vertical divider
        ui.separator();

        // Right Column: Live Rendered Preview
        ui.allocate_ui_with_layout(
            egui::vec2(col_w, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                if let Some(tab) = state.active_tab() {
                    egui::ScrollArea::vertical()
                        .id_salt("center_markdown_split_preview_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            Frame::NONE
                                .fill(Palette::BG_EDITOR())
                                .inner_margin(Margin::symmetric(16, 12))
                                .show(ui, |ui| {
                                    crate::ui::markdown::render_markdown_message(ui, &tab.content);
                                });
                        });
                }
            },
        );
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
                    .color(Palette::GREEN()),
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
                    .fill(Palette::BG_SIDEBAR())
                    .stroke(Stroke::new(1.0, Palette::BORDER()))
                    .corner_radius(CornerRadius::same(5))
                    .inner_margin(Margin::ZERO)
                    .show(ui, |ui| {
                        // Hunk header bar
                        Frame::NONE
                            .fill(Palette::BG_INPUT())
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
                                        .color(Palette::TEXT_MUTED()),
                                    );

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui
                                            .button(RichText::new("✗ Revert").size(11.5).color(Color32::from_rgb(240, 100, 100)))
                                            .clicked()
                                        {
                                            *tab_action_revert = Some(hunk.id);
                                        }

                                        if ui
                                            .button(RichText::new("✓ Accept").size(11.5).color(Palette::GREEN()))
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
                                        DiffOpKind::Equal => (Color32::TRANSPARENT, " ", Palette::TEXT_PRIMARY()),
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
                                                        .color(Palette::TEXT_DIM()),
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
                        Palette::BG_EDITOR()
                    } else {
                        Palette::BG_TAB_BAR()
                    };
                    let border_top = if is_active {
                        Palette::ACCENT()
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
                                Palette::BORDER()
                            } else {
                                Color32::TRANSPARENT
                            },
                        ));

                    let inner_resp = tab_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let text_col = if is_active {
                                Color32::WHITE
                            } else {
                                Palette::TEXT_MUTED()
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
                                ui.label(RichText::new("●").size(9.0).color(Palette::AMBER()));
                            }

                            let close_resp = ui.add(
                                egui::Button::new(
                                    RichText::new("×").size(12.0).color(Palette::TEXT_DIM()),
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

fn render_binary_file_view(ui: &mut egui::Ui, tab: &crate::state::OpenTab) {
    ui.centered_and_justified(|ui| {
        Frame::NONE
            .fill(Palette::BG_INPUT().gamma_multiply(0.4))
            .stroke(Stroke::new(1.0, Palette::BORDER()))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(36, 28))
            .show(ui, |ui| {
                ui.set_max_width(500.0);
                ui.vertical_centered(|ui| {
                    let ext = tab
                        .full_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let is_img = tab.is_image();

                    let icon = if is_img { "🖼" } else { "📦" };
                    let title = if is_img {
                        "圖片檔案 (Image File)"
                    } else {
                        "二進位檔案 (Binary File)"
                    };

                    ui.label(RichText::new(icon).size(36.0));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(title)
                            .size(16.0)
                            .strong()
                            .color(Palette::highlight_text()),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(&tab.name)
                            .font(FontId::monospace(13.5))
                            .color(Palette::ACCENT()),
                    );

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // Details grid
                    egui::Grid::new("binary_file_info_grid")
                        .spacing([18.0, 8.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("檔案格式:").size(12.0).color(Palette::TEXT_DIM()));
                            ui.label(
                                RichText::new(get_file_type_desc(&ext))
                                    .size(12.0)
                                    .color(Palette::TEXT_PRIMARY()),
                            );
                            ui.end_row();

                            ui.label(RichText::new("檔案大小:").size(12.0).color(Palette::TEXT_DIM()));
                            ui.label(
                                RichText::new(format_file_size(tab.file_size))
                                    .size(12.0)
                                    .color(Palette::TEXT_PRIMARY()),
                            );
                            ui.end_row();

                            ui.label(RichText::new("專案路徑:").size(12.0).color(Palette::TEXT_DIM()));
                            ui.label(
                                RichText::new(&tab.rel_path)
                                    .size(12.0)
                                    .color(Palette::TEXT_PRIMARY()),
                            );
                            ui.end_row();
                        });

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // Explanation
                    let reason_text = if is_img {
                        "💡 為確保 IDE 極致效能並維持超低記憶體佔用（< 10 MB），\n本軟體預設不於純文字編輯區中解碼載入高解析度點陣像素圖。"
                    } else {
                        "💡 此檔案為二進位檔案，無法直接在文字編輯器中檢視或修改。"
                    };

                    ui.label(
                        RichText::new(reason_text)
                            .size(11.5)
                            .color(Palette::TEXT_MUTED()),
                    );

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        let btn_text = if is_img {
                            "🚀 以系統相片檢視器開啟"
                        } else {
                            "🚀 以系統關聯程式開啟"
                        };
                        if ui
                            .button(RichText::new(btn_text).size(12.0).color(Palette::ACCENT()))
                            .clicked()
                        {
                            let _ = std::process::Command::new("cmd")
                                .args(["/C", "start", "", &tab.full_path.to_string_lossy()])
                                .spawn();
                        }

                        if ui
                            .button(
                                RichText::new("📂 在檔案總管中顯示")
                                    .size(12.0)
                                    .color(Palette::TEXT_PRIMARY()),
                            )
                            .clicked()
                        {
                            let _ = std::process::Command::new("explorer")
                                .args(["/select,", &tab.full_path.to_string_lossy()])
                                .spawn();
                        }
                    });
                });
            });
    });
}

fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB ({bytes} bytes)", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB ({bytes} bytes)", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn get_file_type_desc(ext: &str) -> &'static str {
    match ext {
        "png" => "PNG 點陣圖影像 (image/png)",
        "jpg" | "jpeg" => "JPEG 壓縮影像 (image/jpeg)",
        "gif" => "GIF 動態/靜態影像 (image/gif)",
        "bmp" => "BMP 點陣圖 (image/bmp)",
        "webp" => "WebP 高效影像 (image/webp)",
        "ico" => "ICO 圖示檔案 (image/x-icon)",
        "svg" => "SVG 向量影像 (image/svg+xml)",
        "mp4" | "mkv" | "avi" => "視訊媒體檔案 (Video)",
        "mp3" | "wav" | "flac" => "音訊媒體檔案 (Audio)",
        "pdf" => "PDF 文件檔案 (application/pdf)",
        "zip" | "tar" | "gz" | "7z" => "壓縮封裝檔案 (Archive)",
        "exe" | "dll" | "pdb" => "Windows 二進位執行檔或組件",
        _ => "二進位檔案 (Binary File)",
    }
}
