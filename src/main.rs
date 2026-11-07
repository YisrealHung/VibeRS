#![windows_subsystem = "windows"]

mod agent;
mod dialog;
mod diff;
mod memory;
mod state;
mod theme;
mod ui;

#[cfg(windows)]
unsafe fn hide_console_window() {
    extern "system" {
        fn AllocConsole() -> i32;
        fn GetConsoleWindow() -> *mut std::ffi::c_void;
        fn ShowWindow(hwnd: *mut std::ffi::c_void, cmd: i32) -> i32;
    }
    // Allocate a hidden console so that all child processes spawned by background CLI tools
    // (such as agy.exe's language server or claude) inherit this invisible console
    // instead of Windows creating a flashing console window for each invocation.
    AllocConsole();
    let hwnd = GetConsoleWindow();
    if !hwnd.is_null() {
        ShowWindow(hwnd, 0); // 0 = SW_HIDE
    }
}

fn main() -> eframe::Result<()> {
    #[cfg(windows)]
    unsafe {
        hide_console_window();
    }

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

