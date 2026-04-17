use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

pub struct LoggerState {
    pub enabled: Mutex<bool>,
    pub log_path: PathBuf,
}

impl LoggerState {
    pub fn new(app_handle: &AppHandle) -> Self {
        // Resolve log path using Tauri's standard app_log_dir
        let log_dir = app_handle.path().app_log_dir().unwrap_or_else(|_| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        });

        if !log_dir.exists() {
            let _ = fs::create_dir_all(&log_dir);
        }

        let log_path = log_dir.join("nexspot_debug.log");

        Self {
            enabled: Mutex::new(true),
            log_path,
        }
    }

    pub fn init(&self) -> anyhow::Result<()> {
        let file = File::create(&self.log_path)?;
        
        // Use system local time for logs instead of UTC
        let mut config_builder = simplelog::ConfigBuilder::new();
        let _ = config_builder.set_time_offset_to_local();
        let config = config_builder.build();

        let _ = CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Info,
                config.clone(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(LevelFilter::Info, config, file),
        ]);

        log::info!(
            "Logger initialized. Logs are being written to stdout and {:?}",
            self.log_path
        );
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
