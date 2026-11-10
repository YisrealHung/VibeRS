use crate::state::{AiProvider, AppState, ChatRole};
use crate::theme::Palette;
use egui::text::LayoutJob;
use egui::{Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke};

pub fn render_right_panel(ctx: &egui::Context, state: &mut AppState) {
    egui::SidePanel::right("vibers_right_ai_panel")
        .resizable(true)
        .default_width(350.0)
        .width_range(240.0..=620.0)
        .frame(
            Frame::NONE
                .fill(Palette::BG_SIDEBAR())
                .inner_margin(Margin::ZERO),
        )
        .show(ctx, |ui| {
            // 1. Top Header: Only displays tabs for installed AI services (e.g. Gemini on this PC)
            egui::TopBottomPanel::top("right_ai_header")
                .exact_height(32.0)
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_SIDEBAR())
                        .inner_margin(Margin {
                            left: 12,
                            right: 10,
                            top: 0,
                            bottom: 0,
                        }),
                )
                .show_inside(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        let providers = state.installed_providers.clone();
                        for provider in providers {
                            let is_active = state.active_provider == provider;
                            let is_running = state.is_task_running(provider);
                            let title_string = if is_running {
                                format!("{} ⚡", provider.tab_title())
                            } else {
                                provider.tab_title().to_string()
                            };
                            let mut title_text = RichText::new(title_string).size(12.5).color(
                                if is_active {
                                    Palette::TEXT_PRIMARY()
                                } else if is_running {
                                    Palette::AMBER()
                                } else {
                                    Palette::TEXT_MUTED()
                                },
                            );
                            if is_active {
                                title_text = title_text.strong();
                            }
                            if ui.selectable_label(is_active, title_text).clicked() {
                                state.active_provider = provider;
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .small_button(
                                    RichText::new("Clear").size(11.0).color(Palette::TEXT_DIM()),
                                )
                                .clicked()
                            {
                                state.clear_active_session();
                            }

                            let count = state.active_prompts_count();
                            if count > 0 {
                                ui.label(
                                    RichText::new(format!("會話: {} 次", count))
                                        .size(11.0)
                                        .color(Palette::TEXT_DIM()),
                                );
                            }
                        });
                    });
                });

            // 2. Bottom Composer Panel (Model Picker dynamically matches active provider tab)
            egui::TopBottomPanel::bottom("right_ai_composer")
                .show_separator_line(false)
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_SIDEBAR())
                        .inner_margin(Margin {
                            left: 12,
                            right: 10,
                            top: 4,
                            bottom: 8,
                        }),
                )
                .show_inside(ui, |ui| {
                    let avail_w = ui.available_width();
                    Frame::NONE
                        .fill(Palette::BG_INPUT())
                        .stroke(Stroke::new(1.0, Palette::BORDER()))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::symmetric(8, 6))
                        .show(ui, |ui| {
                            let inner_w = (avail_w - 18.0).max(60.0);
                            ui.set_max_width(inner_w);

                            let prompt_ref = match state.active_provider {
                                AiProvider::Claude => &mut state.claude_prompt_input,
                                AiProvider::Gemini => &mut state.gemini_prompt_input,
                            };
                            let resp = ui.add(
                                egui::TextEdit::multiline(prompt_ref)
                                    .frame(false)
                                    .hint_text("輸入指令或問題 (Ctrl+Enter 發送)...")
                                    .desired_rows(2)
                                    .desired_width(inner_w),
                            );

                            let send_shortcut = resp.has_focus()
                                && ui.input(|i| {
                                    (i.modifiers.ctrl || i.modifiers.command)
                                        && i.key_pressed(egui::Key::Enter)
                                });

                            ui.add_space(4.0);

                            // Bottom toolbar inside input box: Provider-specific Model Selector + @File + Send
                            ui.horizontal(|ui| {
                                let current_model = state.current_model();
                                let available_models = state.active_provider.models();
                                let btn_label = format!(
                                    "{} [{}] ▾",
                                    current_model.display_name,
                                    current_model.badge()
                                );

                                ui.menu_button(
                                    RichText::new(btn_label)
                                        .size(11.5)
                                        .color(Palette::TEXT_PRIMARY()),
                                    |ui| {
                                        ui.set_min_width(320.0);
                                        for &model_info in available_models {
                                            let selected = current_model.id == model_info.id;
                                            let used_count = state.get_model_session_count(model_info.id);
                                            let tier_desc = if used_count > 0 {
                                                format!("{} (本期: {})", model_info.desc(), used_count)
                                            } else {
                                                model_info.desc().to_string()
                                            };

                                            let mut clicked = false;
                                            ui.horizontal(|ui| {
                                                if ui.selectable_label(selected, model_info.display_name).clicked() {
                                                    clicked = true;
                                                }
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    let resp = ui.add(
                                                        egui::Label::new(
                                                            RichText::new(&tier_desc)
                                                                .size(11.0)
                                                                .color(if selected {
                                                                    Palette::ACCENT()
                                                                } else {
                                                                    Palette::TEXT_MUTED()
                                                                }),
                                                        )
                                                        .sense(egui::Sense::click()),
                                                    );
                                                    if resp.clicked() {
                                                        clicked = true;
                                                    }
                                                });
                                            });

                                            if clicked {
                                                state.set_current_model(model_info);
                                                ui.close_menu();
                                            }
                                        }
                                    },
                                );

                                if let Some(tab) = state.active_tab() {
                                    let label = format!("@{}", tab.name);
                                    let active = state.attach_active_file;
                                    if ui
                                        .selectable_label(
                                            active,
                                            RichText::new(label).monospace().size(11.0).color(
                                                if active {
                                                    Palette::ACCENT()
                                                } else {
                                                    Palette::TEXT_DIM()
                                                },
                                            ),
                                        )
                                        .on_hover_text("點擊切換是否附帶當前分頁檔案")
                                        .clicked()
                                    {
                                        state.attach_active_file = !state.attach_active_file;
                                    }
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let is_busy = state.active_task_is_running();
                                        if is_busy {
                                            if ui
                                                .button(
                                                    RichText::new("Stop ⏹")
                                                        .size(11.5)
                                                        .color(Color32::from_rgb(250, 140, 140)),
                                                )
                                                .clicked()
                                            {
                                                state.abort_active_task();
                                            }
                                        } else {
                                            if ui
                                                .button(
                                                    RichText::new("Send ↵")
                                                        .size(11.5)
                                                        .color(Palette::TEXT_PRIMARY()),
                                                )
                                                .clicked()
                                                || send_shortcut
                                            {
                                                state.send_prompt();
                                            }
                                        }
                                    },
                                );
                            });
                        });
                });

            // 3. Middle Chat ScrollArea
            egui::CentralPanel::default()
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_SIDEBAR())
                        .inner_margin(Margin {
                            left: 12,
                            right: 6,
                            top: 8,
                            bottom: 4,
                        }),
                )
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt(match state.active_provider {
                            AiProvider::Claude => "ai_chat_scroll_claude",
                            AiProvider::Gemini => "ai_chat_scroll_gemini",
                        })
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            let wrap_w = (ui.available_width() - 14.0).max(60.0);
                            ui.set_max_width(wrap_w);

                            let messages = match state.active_provider {
                                AiProvider::Claude => &state.claude_messages,
                                AiProvider::Gemini => &state.gemini_messages,
                            };

                            for msg in messages {
                                match msg.role {
                                    ChatRole::User => {
                                        ui.add_space(14.0);
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                                ui.add_space(6.0); // right padding from scrollbar edge

                                                let max_bubble_w = (wrap_w * 0.78).max(120.0);
                                                let font_id = FontId::proportional(13.0);
                                                let mono_font = FontId::monospace(12.0);

                                                let mut job = LayoutJob::default();
                                                if msg.content.starts_with('@') && msg.content.contains('\n') {
                                                    let newline_idx = msg.content.find('\n').unwrap();
                                                    let file_tag = &msg.content[..newline_idx];
                                                    let rest = &msg.content[newline_idx + 1..];

                                                    job.append(
                                                        file_tag,
                                                        0.0,
                                                        egui::TextFormat {
                                                            font_id: mono_font,
                                                            color: Palette::ACCENT(),
                                                            ..Default::default()
                                                        },
                                                    );
                                                    job.append(
                                                        "\n",
                                                        0.0,
                                                        egui::TextFormat {
                                                            font_id: font_id.clone(),
                                                            color: Palette::TEXT_PRIMARY(),
                                                            ..Default::default()
                                                        },
                                                    );
                                                    job.append(
                                                        rest,
                                                        0.0,
                                                        egui::TextFormat {
                                                            font_id: font_id.clone(),
                                                            color: Palette::TEXT_PRIMARY(),
                                                            ..Default::default()
                                                        },
                                                    );
                                                } else {
                                                    job.append(
                                                        &msg.content,
                                                        0.0,
                                                        egui::TextFormat {
                                                            font_id: font_id.clone(),
                                                            color: Palette::TEXT_PRIMARY(),
                                                            ..Default::default()
                                                        },
                                                    );
                                                }
                                                job.wrap.max_width = max_bubble_w - 32.0;

                                                let galley = ui.fonts(|f| f.layout_job(job));
                                                let text_size = galley.size();
                                                let bubble_size = egui::vec2(
                                                    (text_size.x + 32.0).min(max_bubble_w).max(50.0),
                                                    text_size.y + 20.0,
                                                );

                                                let (rect, _response) = ui.allocate_exact_size(bubble_size, egui::Sense::hover());

                                                // Antigravity style pill bubble
                                                ui.painter().rect_filled(
                                                    rect,
                                                    CornerRadius::same(14),
                                                    Palette::user_bubble_fill(),
                                                );
                                                ui.painter().rect_stroke(
                                                    rect,
                                                    CornerRadius::same(14),
                                                    Stroke::new(1.0, Palette::user_bubble_border()),
                                                    egui::StrokeKind::Inside,
                                                );

                                                let text_pos = rect.min + egui::vec2(16.0, 10.0);
                                                ui.painter().galley(text_pos, galley, Palette::TEXT_PRIMARY());
                                            });
                                        });
                                        ui.add_space(8.0);
                                    }
                                    ChatRole::Assistant => {
                                        ui.add_space(8.0);
                                        let is_provider_busy = state.is_task_running(state.active_provider);
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(&msg.model_label)
                                                    .size(11.0)
                                                    .strong()
                                                    .color(Palette::ACCENT()),
                                            );
                                            if is_provider_busy && msg.content.is_empty() {
                                                ui.label(
                                                    RichText::new("⚡ 正在思考並分析專案中...")
                                                        .size(11.0)
                                                        .color(Palette::AMBER()),
                                                );
                                            }
                                        });

                                        ui.add_space(6.0);

                                        if msg.content.is_empty() && is_provider_busy {
                                            Frame::NONE
                                                .fill(Palette::BG_INPUT().gamma_multiply(0.5))
                                                .corner_radius(CornerRadius::same(6))
                                                .inner_margin(Margin::symmetric(10, 8))
                                                .show(ui, |ui| {
                                                    let wait_text = match state.active_provider {
                                                        AiProvider::Claude => "正在讀取專案上下文並呼叫 Claude Code，請稍候...",
                                                        AiProvider::Gemini => "正在讀取專案上下文並呼叫 Antigravity 模型，請稍候...",
                                                    };
                                                    ui.label(
                                                        RichText::new(wait_text)
                                                            .size(11.5)
                                                            .color(Palette::TEXT_MUTED()),
                                                    );
                                                });
                                        } else {
                                            crate::ui::markdown::render_markdown_message(ui, &msg.content);
                                        }
                                        ui.add_space(14.0);
                                    }
                                }
                            }
                        });
                });
        });
}
