use crate::state::AppState;
use crate::theme::Palette;
use egui::{Frame, Margin, RichText};

pub fn render_top_bar(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("vibers_top_bar")
        .exact_height(30.0)
        .frame(
            Frame::NONE
                .fill(Palette::BG_TAB_BAR())
                .inner_margin(Margin::symmetric(10, 0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(
                    RichText::new("VibeRS")
                        .strong()
                        .size(13.0)
                        .color(Palette::ACCENT()),
                );

                ui.add_space(4.0);

                // Only File menu in the top-left handles Open Folder & New Window
                ui.menu_button(
                    RichText::new("File").size(12.5).color(Palette::TEXT_PRIMARY()),
                    |ui| {
                        ui.set_min_width(210.0);
                        if ui.button("開啟資料夾 (Open Folder...)        Ctrl+O").clicked() {
                            ui.close_menu();
                            state.prompt_open_folder();
                        }
                        if ui.button("開新視窗 (New Window)        Ctrl+Shift+N").clicked() {
                            ui.close_menu();
                            state.open_new_window();
                        }
                        if ui.button("在新視窗開啟資料夾...").clicked() {
                            ui.close_menu();
                            state.open_folder_in_new_window();
                        }
                        ui.separator();
                        if ui.button("儲存當前分頁 (Save)                 Ctrl+S").clicked() {
                            ui.close_menu();
                            state.save_active_tab();
                        }
                        if ui.button("儲存全部分頁 (Save All)            Ctrl+Shift+S").clicked() {
                            ui.close_menu();
                            state.save_all_tabs();
                        }
                        ui.separator();
                        ui.menu_button("頁面主題 (Theme)", |ui| {
                            let cur_theme = crate::theme::get_theme_mode();
                            if ui
                                .selectable_label(
                                    cur_theme == crate::theme::ThemeMode::Dark,
                                    "🌙 黑色主題 (Dark)",
                                )
                                .clicked()
                            {
                                crate::theme::set_theme_mode(crate::theme::ThemeMode::Dark);
                                crate::theme::apply_vibers_theme(ui.ctx());
                                state.status_text = "已切換至黑色主題 (Dark)".to_string();
                                ui.close_menu();
                            }
                            if ui
                                .selectable_label(
                                    cur_theme == crate::theme::ThemeMode::Light,
                                    "\u{2600} 白色主題 (Light)",
                                )
                                .clicked()
                            {
                                crate::theme::set_theme_mode(crate::theme::ThemeMode::Light);
                                crate::theme::apply_vibers_theme(ui.ctx());
                                state.status_text = "已切換至白色主題 (Light)".to_string();
                                ui.close_menu();
                            }
                        });
                    },
                );

                if !state.workspace_name.is_empty() {
                    ui.separator();
                    ui.label(
                        RichText::new(&state.workspace_name)
                            .size(12.5)
                            .color(Palette::TEXT_MUTED()),
                    )
                    .on_hover_text(state.workspace_root.display().to_string());
                }
            });
        });
}
