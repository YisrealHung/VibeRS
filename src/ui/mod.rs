pub mod bottom_bar;
pub mod center_panel;
pub mod left_panel;
pub mod right_panel;
pub mod top_bar;

use crate::memory::trim_working_set;
use crate::state::AppState;
use crate::theme::{apply_vibers_theme, configure_fonts};

pub struct VibeApp {
    pub state: AppState,
    frame_count: u32,
}

impl VibeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _font_summary = configure_fonts(&cc.egui_ctx);
        apply_vibers_theme(&cc.egui_ctx);

        let mut state = AppState::default();
        state.memory_monitor.force_refresh();

        Self {
            state,
            frame_count: 0,
        }
    }
}

impl eframe::App for VibeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.frame_count < 4 {
            self.frame_count += 1;
            if self.frame_count == 2 {
                ctx.request_repaint();
            } else if self.frame_count == 3 {
                trim_working_set();
                self.state.memory_monitor.force_refresh();
            }
        } else {
            self.state.memory_monitor.update_if_needed();
        }

        // Ctrl+O to open folder dialog
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::O)) {
            self.state.prompt_open_folder();
        }

        // Ctrl+Shift+N to open a new window
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.shift
                && i.key_pressed(egui::Key::N)
        }) {
            self.state.open_new_window();
        }

        // Ctrl+S to save current tab
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::S)) {
            self.state.save_active_tab();
        }

        // Ctrl+W to close current tab
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::W)) {
            let active = self.state.active_tab_idx;
            self.state.close_tab(active);
        }

        top_bar::render_top_bar(ctx, &mut self.state);
        bottom_bar::render_bottom_bar(ctx, &mut self.state);
        left_panel::render_left_panel(ctx, &mut self.state);
        right_panel::render_right_panel(ctx, &mut self.state);
        center_panel::render_center_panel(ctx, &mut self.state);
    }
}
