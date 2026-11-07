#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dialog;
mod diff;
mod memory;
mod state;
mod theme;
mod ui;

fn main() -> eframe::Result<()> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("VibeRS — 次世代超輕量 Rust 原生 AI 審閱型 IDE")
        .with_inner_size([1440.0, 860.0])
        .with_min_inner_size([960.0, 600.0]);

    let native_options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "VibeRS",
        native_options,
        Box::new(|cc| Ok(Box::new(ui::VibeApp::new(cc)))),
    )
}

