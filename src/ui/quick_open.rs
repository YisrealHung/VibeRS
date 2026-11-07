use crate::state::AppState;
use crate::theme::Palette;
use egui::{Color32, CornerRadius, Frame, Key, Margin, RichText, Stroke};

pub fn render_quick_open(ctx: &egui::Context, state: &mut AppState) {
    if !state.quick_open.is_open {
        return;
    }

    let screen_rect = ctx.screen_rect();
    let window_width = 540.0f32.min(screen_rect.width() - 40.0);
    let pos = egui::pos2(
        screen_rect.center().x - window_width / 2.0,
        screen_rect.top() + 60.0,
    );

    let mut close = false;
    let mut selected_target = None;

    // Check Escape
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        close = true;
    }

    egui::Window::new("quick_open_dialog")
        .title_bar(false)
        .resizable(false)
        .fixed_pos(pos)
        .fixed_size([window_width, 320.0])
        .frame(
            Frame::NONE
                .fill(Palette::BG_SIDEBAR)
                .stroke(Stroke::new(1.0, Palette::BORDER))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(10, 8)),
        )
        .show(ctx, |ui| {
            ui.set_width(window_width - 20.0);

            // Filter entries
            let query = state.quick_open.query.to_lowercase();
            let matches: Vec<(std::path::PathBuf, String)> = state
                .file_list()
                .into_iter()
                .filter(|(_, rel)| {
                    if query.is_empty() {
                        true
                    } else {
                        rel.to_lowercase().contains(&query)
                    }
                })
                .collect();

            let matches_len = matches.len();

            // Arrow keys for selection
            if ctx.input(|i| i.key_pressed(Key::ArrowDown)) && matches_len > 0 {
                state.quick_open.selected_index = (state.quick_open.selected_index + 1) % matches_len;
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowUp)) && matches_len > 0 {
                if state.quick_open.selected_index == 0 {
                    state.quick_open.selected_index = matches_len.saturating_sub(1);
                } else {
                    state.quick_open.selected_index -= 1;
                }
            }
            if state.quick_open.selected_index >= matches_len && matches_len > 0 {
                state.quick_open.selected_index = 0;
            }

            // Input box
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍").size(13.0).color(Palette::TEXT_MUTED));
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut state.quick_open.query)
                        .hint_text("搜尋檔案名稱... (Enter 開啟, Esc 取消)")
                        .desired_width(ui.available_width() - 8.0)
                        .font(egui::FontId::proportional(13.5)),
                );

                if !state.quick_open.focused {
                    resp.request_focus();
                    state.quick_open.focused = true;
                }

                if resp.changed() {
                    state.quick_open.selected_index = 0;
                }

                if resp.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)) {
                    if let Some((path, rel)) = matches.get(state.quick_open.selected_index) {
                        selected_target = Some((path.clone(), rel.clone()));
                    }
                }
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            // Results list
            egui::ScrollArea::vertical()
                .id_salt("quick_open_results_scroll")
                .max_height(240.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if matches.is_empty() {
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("找不到符合的檔案")
                                .size(12.5)
                                .color(Palette::TEXT_DIM),
                        );
                        return;
                    }

                    for (idx, (path, rel)) in matches.iter().enumerate() {
                        let is_sel = idx == state.quick_open.selected_index;
                        let bg = if is_sel {
                            Palette::BG_SELECTED
                        } else {
                            Color32::TRANSPARENT
                        };

                        let item_resp = Frame::NONE
                            .fill(bg)
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(8, 5))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let filename = path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or(rel);
                                    let dir = path
                                        .parent()
                                        .and_then(|p| p.strip_prefix(&state.workspace_root).ok())
                                        .map(|p| p.to_string_lossy().replace('\\', "/"))
                                        .unwrap_or_default();

                                    ui.label(
                                        RichText::new(filename)
                                            .size(12.5)
                                            .strong()
                                            .color(if is_sel {
                                                Color32::WHITE
                                            } else {
                                                Palette::TEXT_PRIMARY
                                            }),
                                    );

                                    if !dir.is_empty() {
                                        ui.label(
                                            RichText::new(&dir)
                                                .size(11.0)
                                                .color(Palette::TEXT_DIM),
                                        );
                                    }
                                });
                            })
                            .response;

                        if item_resp.clicked() {
                            selected_target = Some((path.clone(), rel.clone()));
                        }
                    }
                });
        });

    if let Some((path, rel)) = selected_target {
        state.open_file_tab(&path, &rel);
        close = true;
    }

    if close {
        state.quick_open.is_open = false;
        state.quick_open.focused = false;
        state.quick_open.query.clear();
        state.quick_open.selected_index = 0;
    }
}
