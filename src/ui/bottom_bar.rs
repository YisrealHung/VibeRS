use crate::state::AppState;
use crate::theme::Palette;
use egui::{Frame, Margin, RichText};

pub fn render_bottom_bar(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::bottom("vibers_bottom_status_bar")
        .exact_height(22.0)
        .frame(
            Frame::NONE
                .fill(Palette::BG_TAB_BAR())
                .inner_margin(Margin::symmetric(12, 0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(
                    RichText::new(&state.status_text)
                        .size(11.0)
                        .color(Palette::TEXT_MUTED()),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mem = state.memory_monitor.stats;
                    ui.label(
                        RichText::new(format!("RAM: {:.1} MB", mem.working_set_mb))
                            .monospace()
                            .size(11.0)
                            .color(Palette::GREEN()),
                    )
                    .on_hover_text(format!(
                        "Working Set: {:.1} MB | Private: {:.1} MB",
                        mem.working_set_mb, mem.private_mb
                    ));

                    ui.separator();

                    if let Some(tab) = state.active_tab() {
                        let lines = tab.content.lines().count().max(1);
                        ui.label(
                            RichText::new(format!("{lines} lines  ·  UTF-8"))
                                .size(11.0)
                                .color(Palette::TEXT_DIM()),
                        );
                    }
                });
            });
        });
}
