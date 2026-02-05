use global_hotkey::hotkey::HotKey;
use global_hotkey::GlobalHotKeyManager;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(tag = "code", content = "message")]
pub enum ConfigError {
    #[error("Shortcut conflict with \"{0}\"")]
    Conflict(String),
    #[error("Shortcut registration failed for \"{0}\"")]
    RegistrationFailed(String),
    #[error("Shortcut not found")]
    NotFound,
    #[error("Invalid shortcut format")]
    InvalidFormat,
    #[error("Empty shortcut")]
    Empty,
    #[error("IO Error: {0}")]
    Io(String),
}

// GlobalHotKeyManager is Send + Sync on Windows when using the default features
// and it's intended to be used across threads as long as it's not dropped.
// We keep the wrapper but document its safety rationale.
// Note: In global-hotkey 0.6.0, it doesn't implement Send/Sync by default
// due to X11 types on Linux, but on Windows it's safe.
pub struct SafeGlobalHotKeyManager(pub GlobalHotKeyManager);
unsafe impl Send for SafeGlobalHotKeyManager {}
unsafe impl Sync for SafeGlobalHotKeyManager {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Shortcut {
    pub id: String,
    pub label: String,
    pub shortcut: String, // e.g. "Alt+A"
    pub icon: String,
    pub color: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none", skip_deserializing)]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub shortcuts: Vec<Shortcut>,
    pub save_path: String,
    pub language: String,
    pub ocr_engine: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcuts: get_default_shortcuts(),
            save_path: "captures".to_string(),
            language: "zh".to_string(),
            ocr_engine: "default".to_string(),
        }
    }
}

pub struct ConfigState {
    pub manager: SafeGlobalHotKeyManager,
    pub config: AppConfig,
    pub config_path: PathBuf,
    pub last_registration_errors: Vec<String>,
}

impl ConfigState {
    pub fn new(app_handle: &AppHandle) -> Self {
        let config_dir = app_handle
            .path()
            .app_config_dir()
            .unwrap_or(PathBuf::from("."));

        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        let config_path = config_dir.join("config.json");

        let manager = GlobalHotKeyManager::new().expect("Failed to init GlobalHotKeyManager");

        let mut state = Self {
            manager: SafeGlobalHotKeyManager(manager),
            config: AppConfig::default(),
            config_path,
            last_registration_errors: Vec::new(),
        };

        state.load();
        state
    }

    pub fn load(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.config_path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                self.config = config;
                self.register_all();
                return;
            }
        }

        // Fallback or Initial setup
        self.config = AppConfig::default();
        self.save();
        self.register_all();
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(&self.config) {
            let _ = fs::write(&self.config_path, content);
        }
    }

    /// Differential update: Only unregisters/registers what's changed.
    pub fn update_shortcut(&mut self, id: &str, new_keys: &str) -> Result<(), ConfigError> {
        let new_keys = new_keys.trim();
        if new_keys.is_empty() {
            return Err(ConfigError::Empty);
        }

        // Validate new key format early
        let new_hotkey = new_keys
            .parse::<HotKey>()
            .map_err(|_| ConfigError::InvalidFormat)?;

        // Conflict check
        if let Some(conflict) = self
            .config
            .shortcuts
            .iter()
            .find(|s| s.id != id && s.shortcut == new_keys)
        {
            return Err(ConfigError::Conflict(conflict.label.clone()));
        }

        let s_idx = self
            .config
            .shortcuts
            .iter()
            .position(|s| s.id == id)
            .ok_or(ConfigError::NotFound)?;
        let old_keys = self.config.shortcuts[s_idx].shortcut.clone();

        if old_keys == new_keys {
            return Ok(()); // Nothing changed
        }

        // 1. Unregister OLD
        if let Ok(old_hotkey) = old_keys.parse::<HotKey>() {
            let _ = self.manager.0.unregister(old_hotkey);
        }

        // 2. Register NEW
        match self.manager.0.register(new_hotkey) {
            Ok(_) => {
                log::info!("Registered {} -> {}", id, new_keys);
                self.config.shortcuts[s_idx].shortcut = new_keys.to_string();

                // Clear from error tracking if it was there
                self.last_registration_errors
                    .retain(|e| !e.contains(&new_keys));

                self.save();
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to register {} ({}): {:?}", id, new_keys, e);
                // Attempt to re-register OLD so we don't leave it dead
                if let Ok(old_hotkey) = old_keys.parse::<HotKey>() {
                    let _ = self.manager.0.register(old_hotkey);
                }
                Err(ConfigError::RegistrationFailed(new_keys.to_string()))
            }
        }
    }

    pub fn set_save_path(&mut self, path: String) {
        self.config.save_path = path;
        self.save();
    }

    pub fn set_ocr_engine(&mut self, engine: String) {
        self.config.ocr_engine = engine;
        self.save();
    }

    /// Resolves the save path to an absolute PathBuf.
    pub fn resolve_save_path(&self, app: &AppHandle) -> PathBuf {
        let path = &self.config.save_path;
        if path.is_empty() || path == "captures" {
            app.path()
                .app_data_dir()
                .unwrap_or_default()
                .join("captures")
        } else {
            PathBuf::from(path)
        }
    }

    pub fn register_all(&mut self) {
        self.last_registration_errors.clear();
        for s in &self.config.shortcuts {
            if s.enabled {
                if let Ok(hotkey) = s.shortcut.parse::<HotKey>() {
                    if let Err(e) = self.manager.0.register(hotkey) {
                        log::error!("Init error: {} ({}): {:?}", s.label, s.shortcut, e);
                        self.last_registration_errors
                            .push(format!("{} ({})", s.label, s.shortcut));
                    }
                }
            }
        }
    }

    pub fn unregister_all(&self) {
        for s in &self.config.shortcuts {
            if let Ok(hotkey) = s.shortcut.parse::<HotKey>() {
                let _ = self.manager.0.unregister(hotkey);
            }
        }
    }
}

fn get_default_shortcuts() -> Vec<Shortcut> {
    vec![
        Shortcut {
            id: "capture".to_string(),
            label: "区域截图".to_string(),
            shortcut: "Alt+A".to_string(),
            icon: "\u{E11A}".to_string(),
            color: "#3b82f6".to_string(),
            enabled: true,
            error: None,
        },
        Shortcut {
            id: "ocr".to_string(),
            label: "OCR 识别".to_string(),
            shortcut: "Alt+S".to_string(),
            icon: "\u{E11B}".to_string(),
            color: "#10b981".to_string(),
            enabled: true,
            error: None,
        },
    ]
}
