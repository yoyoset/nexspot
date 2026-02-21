use super::types::AppConfig;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub fn load(config_path: &Path) -> AppConfig {
    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
            // 2. Backwards compatibility for other fields
            if !content.contains("vello_advanced_effects") {
                config.vello_advanced_effects = true;
            }
            return config;
        }
    }
    // Fallback
    AppConfig::default()
}

pub fn save(config_path: &Path, config: &AppConfig) -> anyhow::Result<()> {
    let content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

pub fn resolve_save_path(app: &AppHandle, config: &AppConfig) -> PathBuf {
    let path = &config.save_path;
    if path.is_empty() || path == "captures" {
        app.path()
            .app_data_dir()
            .unwrap_or_default()
            .join("captures")
    } else {
        PathBuf::from(path)
    }
}

pub fn resolve_fonts_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap_or_default().join("fonts")
}
