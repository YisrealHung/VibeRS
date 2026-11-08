pub mod bottom_bar;
pub mod center_panel;
pub mod left_panel;
pub mod markdown;
pub mod quick_open;
pub mod right_panel;
pub mod syntax;
pub mod top_bar;

use crate::memory::trim_working_set;
use crate::state::AppState;
use crate::theme::{apply_vibers_theme, configure_fonts};

pub struct VibeApp {
    pub state: AppState,
    frame_count: u32,
    ime_composing: bool,
    last_ime_commit: Option<std::time::Instant>,
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
            ime_composing: false,
            last_ime_commit: None,
        }
    }
}

impl eframe::App for VibeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // IME Enter filtering:
        // When typing Chinese / Japanese with an IME, pressing Enter confirms the candidate word.
        // Windows/winit sends both ImeEvent::Commit and Key::Enter. If unhandled, egui TextEdit
        // inserts an extra newline on the very first Enter during candidate confirmation.
        // We filter out plain Key::Enter whenever IME is active or just committed.
        ctx.input_mut(|i| {
            let mut has_ime_commit = false;
            let mut has_ime_preedit = false;

            for event in &i.events {
                match event {
                    egui::Event::Ime(egui::ImeEvent::Commit(_)) => {
                        has_ime_commit = true;
                    }
                    egui::Event::Ime(egui::ImeEvent::Preedit(s)) if !s.is_empty() => {
                        has_ime_preedit = true;
                    }
                    _ => {}
                }
            }

            let ime_just_committed = self
                .last_ime_commit
                .map(|t| t.elapsed() < std::time::Duration::from_millis(150))
                .unwrap_or(false);

            if has_ime_commit || has_ime_preedit || self.ime_composing || ime_just_committed {
                let mut removed_enter = false;
                i.events.retain(|e| {
                    if let egui::Event::Key {
                        key: egui::Key::Enter,
                        pressed: true,
                        modifiers,
                        ..
                    } = e
                    {
                        if modifiers.is_none() {
                            removed_enter = true;
                            return false;
                        }
                    }
                    true
                });

                if removed_enter {
                    self.last_ime_commit = None;
                }
            }

            for event in &i.events {
                match event {
                    egui::Event::Ime(egui::ImeEvent::Enabled) => {
                        self.ime_composing = true;
                    }
                    egui::Event::Ime(egui::ImeEvent::Preedit(s)) => {
                        self.ime_composing = !s.is_empty();
                    }
                    egui::Event::Ime(egui::ImeEvent::Commit(_)) => {
                        self.ime_composing = false;
                        self.last_ime_commit = Some(std::time::Instant::now());
                    }
                    egui::Event::Ime(egui::ImeEvent::Disabled) => {
                        self.ime_composing = false;
                    }
                    _ => {}
                }
            }
        });

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
            self.state.check_external_disk_changes();
            // Regularly update RAM and system telemetry once per second even when idle
            ctx.request_repaint_after(std::time::Duration::from_millis(1000));
        }

        // Drain active agent task events
        if let Some(task) = &self.state.active_agent_task {
            let mut finished = false;
            let mut chunks = Vec::new();
            let mut error = None;

            while let Ok(event) = task.rx.try_recv() {
                match event {
                    crate::agent::AgentEvent::Chunk(text) => chunks.push(text),
                    crate::agent::AgentEvent::Finished => finished = true,
                    crate::agent::AgentEvent::Error(err) => error = Some(err),
                }
            }

            let list = match self.state.active_provider {
                crate::state::AiProvider::Claude => &mut self.state.claude_messages,
                crate::state::AiProvider::Gemini => &mut self.state.gemini_messages,
            };

            if let Some(last) = list.last_mut() {
                for c in chunks {
                    last.content.push_str(&c);
                }
                if let Some(err) = error {
                    last.content.push_str(&format!("\n[錯誤: {err}]"));
                    finished = true;
                }
            }

            if finished {
                self.state.active_agent_task = None;
            } else {
                ctx.request_repaint();
            }
        }

        // Ctrl+P for Quick Open
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && !i.modifiers.shift && i.key_pressed(egui::Key::P)) {
            self.state.quick_open.is_open = true;
            self.state.quick_open.focused = false;
            self.state.quick_open.query.clear();
            self.state.quick_open.selected_index = 0;
        }

        // Ctrl+O to open folder dialog
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && !i.modifiers.shift && i.key_pressed(egui::Key::O)) {
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

        // Ctrl+Shift+S to save all tabs
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.shift
                && i.key_pressed(egui::Key::S)
        }) {
            self.state.save_all_tabs();
        }

        // Ctrl+Shift+V to cycle Markdown preview mode (Editor -> Split -> Preview)
        if ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.command)
                && i.modifiers.shift
                && i.key_pressed(egui::Key::V)
        }) {
            if let Some(tab) = self.state.active_tab_mut() {
                if tab.is_markdown() {
                    tab.markdown_mode = match tab.markdown_mode {
                        crate::state::MarkdownViewMode::Editor => crate::state::MarkdownViewMode::Split,
                        crate::state::MarkdownViewMode::Split => crate::state::MarkdownViewMode::Preview,
                        crate::state::MarkdownViewMode::Preview => crate::state::MarkdownViewMode::Editor,
                    };
                }
            }
        }

        // Ctrl+S to save current tab
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && !i.modifiers.shift && i.key_pressed(egui::Key::S)) {
            self.state.save_active_tab();
        }

        // Ctrl+W to close current tab
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::W)) {
            let active = self.state.active_tab_idx;
            self.state.request_close_tab(active);
        }

        top_bar::render_top_bar(ctx, &mut self.state);
        bottom_bar::render_bottom_bar(ctx, &mut self.state);
        left_panel::render_left_panel(ctx, &mut self.state);
        right_panel::render_right_panel(ctx, &mut self.state);
        center_panel::render_center_panel(ctx, &mut self.state);
        quick_open::render_quick_open(ctx, &mut self.state);
    }
}
