#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};

fn main() {
    // Initialize Logging
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let log_path = parent.join("nexspot_debug.log");
            let log_file = std::fs::File::create(log_path).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to create log file: {}. Falling back to null device.", e);
                #[cfg(target_os = "windows")]
                { std::fs::File::create("NUL").expect("Failed to open NUL") }
                #[cfg(not(target_os = "windows"))]
                { std::fs::File::create("/dev/null").expect("Failed to open /dev/null") }
            });

            let _ = simplelog::WriteLogger::init(
                simplelog::LevelFilter::Debug,
                simplelog::Config::default(),
                log_file,
            );
        }
    }

    // Set panic hook to log panics
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            *s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Box<Any>"
        };
        log::error!("PANIC at {}: {}", location, message);
    }));

    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    nexspot_lib::run();
}
