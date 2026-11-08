use crate::state::{AiProvider, AppState, ChatRole};
use crate::theme::Palette;
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke};

pub fn render_right_panel(ctx: &egui::Context, state: &mut AppState) {
    egui::SidePanel::right("vibers_right_ai_panel")
        .resizable(true)
        .default_width(350.0)
        .width_range(240.0..=620.0)
        .frame(
            Frame::NONE
                .fill(Palette::BG_SIDEBAR)
                .inner_margin(Margin::ZERO),
        )
        .show(ctx, |ui| {
            // 1. Top Header: Only displays tabs for installed AI services (e.g. Gemini on this PC)
            egui::TopBottomPanel::top("right_ai_header")
                .exact_height(32.0)
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_SIDEBAR)
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
                            if ui
                                .selectable_label(
                                    is_active,
                                    RichText::new(provider.tab_title()).size(12.5).color(
                                        if is_active {
                                            Color32::WHITE
                                        } else {
                                            Palette::TEXT_MUTED
                                        },
                                    ),
                                )
                                .clicked()
                            {
                                state.active_provider = provider;
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .small_button(
                                    RichText::new("Clear").size(11.0).color(Palette::TEXT_DIM),
                                )
                                .clicked()
                            {
                                match state.active_provider {
                                    AiProvider::Claude => state.claude_messages.clear(),
                                    AiProvider::Gemini => state.gemini_messages.clear(),
                                }
                            }
                        });
                    });
                });

            // 2. Bottom Composer Panel (Model Picker dynamically matches active provider tab)
            egui::TopBottomPanel::bottom("right_ai_composer")
                .show_separator_line(false)
                .frame(
                    Frame::NONE
                        .fill(Palette::BG_SIDEBAR)
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
                        .fill(Palette::BG_INPUT)
                        .stroke(Stroke::new(1.0, Palette::BORDER))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::symmetric(8, 6))
                        .show(ui, |ui| {
                            let inner_w = (avail_w - 18.0).max(60.0);
                            ui.set_max_width(inner_w);

                            let resp = ui.add(
                                egui::TextEdit::multiline(&mut state.prompt_input)
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

                                ui.menu_button(
                                    RichText::new(format!("{} ▾", current_model.display_name))
                                        .size(11.5)
                                        .color(Palette::TEXT_PRIMARY),
                                    |ui| {
                                        ui.set_min_width(200.0);
                                        for &model_info in available_models {
                                            let selected = current_model.id == model_info.id;
                                            if ui
                                                .selectable_label(selected, model_info.display_name)
                                                .clicked()
                                            {
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
                                                    Palette::ACCENT
                                                } else {
                                                    Palette::TEXT_DIM
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
                                        let is_busy = state.active_agent_task.is_some();
                                        if is_busy {
                                            if ui
                                                .button(
                                                    RichText::new("Stop ⏹")
                                                        .size(11.5)
                                                        .color(Color32::from_rgb(250, 140, 140)),
                                                )
                                                .clicked()
                                            {
                                                if let Some(task) = state.active_agent_task.take() {
                                                    task.abort();
                                                }
                                            }
                                        } else {
                                            if ui
                                                .button(
                                                    RichText::new("Send ↵")
                                                        .size(11.5)
                                                        .color(Color32::WHITE),
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
                        .fill(Palette::BG_SIDEBAR)
                        .inner_margin(Margin {
                            left: 12,
                            right: 6,
                            top: 8,
                            bottom: 4,
                        }),
                )
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("ai_chat_scroll")
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
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                                let bubble_max_w = (wrap_w * 0.88).min(wrap_w - 20.0).max(60.0);
                                                Frame::NONE
                                                    .fill(Palette::BG_INPUT)
                                                    .stroke(Stroke::new(1.0, Palette::BORDER))
                                                    .corner_radius(CornerRadius::same(8))
                                                    .inner_margin(Margin::symmetric(12, 8))
                                                    .show(ui, |ui| {
                                                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                                            ui.set_max_width(bubble_max_w);
                                                            ui.add(
                                                                egui::Label::new(
                                                                    RichText::new(&msg.content)
                                                                        .size(12.5)
                                                                        .color(Palette::TEXT_PRIMARY),
                                                                )
                                                                .wrap(),
                                                            );
                                                        });
                                                    });
                                            });
                                        });
                                    }
                                    ChatRole::Assistant => {
                                        ui.add_space(2.0);
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(&msg.model_label)
                                                    .size(10.5)
                                                    .color(Palette::TEXT_DIM),
                                            );
                                            if state.active_agent_task.is_some() && msg.content.is_empty() {
                                                ui.label(
                                                    RichText::new("⚡ 正在思考並分析專案中...")
                                                        .size(10.5)
                                                        .color(Palette::ACCENT),
                                                );
                                            }
                                        });

                                        if msg.content.is_empty() && state.active_agent_task.is_some() {
                                            ui.add_space(4.0);
                                            Frame::NONE
                                                .fill(Palette::BG_INPUT.gamma_multiply(0.5))
                                                .corner_radius(CornerRadius::same(4))
                                                .inner_margin(Margin::symmetric(8, 6))
                                                .show(ui, |ui| {
                                                    ui.label(
                                                        RichText::new("正在讀取專案上下文並呼叫 Antigravity 模型，請稍候...")
                                                            .size(11.5)
                                                            .color(Palette::TEXT_MUTED),
                                                    );
                                                });
                                        } else {
                                            crate::ui::markdown::render_markdown_message(ui, &msg.content);
                                        }
                                    }
                                }
                                ui.add_space(8.0);
                            }
                        });
                });
        });
}
