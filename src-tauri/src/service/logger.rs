use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, State};

pub struct LoggerState {
    pub enabled: Mutex<bool>,
    pub log_path: PathBuf,
}

impl LoggerState {
    pub fn new(_app_handle: &AppHandle) -> Self {
        // Resolve log path relative to the executable
        let log_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("hyper_lens.log")))
            .unwrap_or_else(|| PathBuf::from("hyper_lens.log"));

        Self {
            enabled: Mutex::new(true), // Default to true for now, until persistence is added
            log_path,
        }
    }

    pub fn init(&self) -> anyhow::Result<()> {
        let file = File::create(&self.log_path)?;

        // We use WriteLogger which writes to the file
        // Note: simplelog's init is global. Re-init calls will fail/be ignored, which is fine.
        let _ = WriteLogger::init(LevelFilter::Info, Config::default(), file);

        log::info!("Logger initialized at {:?}", self.log_path);
        Ok(())
    }

    pub fn clear_logs(&self) -> anyhow::Result<()> {
        let _ = File::create(&self.log_path)?; // This truncates
        log::info!("Logs cleared by user.");
        Ok(())
    }
}

#[tauri::command]
pub fn clear_logs(state: State<'_, LoggerState>) -> Result<(), String> {
    state.clear_logs().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reveal_logs(state: State<'_, LoggerState>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer")
            .arg("/select,")
            .arg(&state.log_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
