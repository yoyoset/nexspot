use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};

fn main() {
    // Initialize Logging
    if let Ok(exe_path) = std::env::current_exe() {
        let log_path = exe_path.parent().unwrap().join("hyper_lens_debug.log");
        let _ = simplelog::WriteLogger::init(
            simplelog::LevelFilter::Debug,
            simplelog::Config::default(),
            std::fs::File::create(log_path).expect("Failed to create log file"),
        );
    }

    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    hyper_lens_lib::run();
}
