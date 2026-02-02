use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static LOG_ENABLED: AtomicBool = AtomicBool::new(true); // ENABLED by default for debugging
static LOG_FILE: Mutex<Option<String>> = Mutex::new(None);

pub fn init(executable_dir: &str) {
    let mut path = std::path::PathBuf::from(executable_dir);
    path.push("hyper-lens.log");

    if let Ok(mut guard) = LOG_FILE.lock() {
        *guard = Some(path.to_string_lossy().to_string());
    }
}

pub fn set_enabled(enabled: bool) {
    LOG_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        log("Logger", "Logging enabled");
    }
}

pub fn is_enabled() -> bool {
    LOG_ENABLED.load(Ordering::Relaxed)
}

pub fn clear_log() -> Result<(), String> {
    if let Ok(guard) = LOG_FILE.lock() {
        if let Some(path) = guard.as_ref() {
            File::create(path).map_err(|e| e.to_string())?; // Truncate
            return Ok(());
        }
    }
    Err("Logger not initialized".to_string())
}

pub fn log(target: &str, message: &str) {
    if !LOG_ENABLED.load(Ordering::Relaxed) {
        return;
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_line = format!("[{}] [{}] {}\n", timestamp, target, message);

    print!("{}", log_line);

    if let Ok(guard) = LOG_FILE.lock() {
        if let Some(path) = guard.as_ref() {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = file.write_all(log_line.as_bytes());
            }
        }
    }
}
