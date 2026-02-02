use global_hotkey::hotkey::HotKey;
use global_hotkey::GlobalHotKeyManager;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// Wrapper to make GlobalHotKeyManager Send + Sync
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
}

pub struct ShortcutState {
    pub manager: SafeGlobalHotKeyManager,
    pub shortcuts: Vec<Shortcut>,
    pub config_path: PathBuf,
    pub last_registration_errors: Vec<String>,
}

impl ShortcutState {
    pub fn new(app_handle: &AppHandle) -> Self {
        let config_dir = app_handle
            .path()
            .app_config_dir()
            .unwrap_or(PathBuf::from("."));
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        let config_path = config_dir.join("shortcuts.json");

        let manager = GlobalHotKeyManager::new().expect("Failed to init GlobalHotKeyManager");

        let mut state = Self {
            manager: SafeGlobalHotKeyManager(manager),
            shortcuts: Vec::new(),
            config_path,
            last_registration_errors: Vec::new(),
        };

        state.load();
        state
    }

    pub fn load(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.config_path) {
            if let Ok(shortcuts) = serde_json::from_str::<Vec<Shortcut>>(&content) {
                self.shortcuts = shortcuts;
                self.register_all();
                return;
            }
        }
        // Fallback defaults
        self.shortcuts = get_default_shortcuts();
        self.save();
        self.register_all();
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(&self.shortcuts) {
            let _ = fs::write(&self.config_path, content);
        }
    }

    pub fn update_shortcut(&mut self, id: &str, new_keys: &str) -> Result<(), String> {
        if new_keys.is_empty() {
            return Err("Shortcut cannot be empty".to_string());
        }

        // Check conflicts
        for s in &self.shortcuts {
            if s.id != id && s.shortcut == new_keys {
                return Err(format!("Conflict with {}", s.label));
            }
        }

        // Unregister current before update
        self.unregister_all();

        if let Some(s) = self.shortcuts.iter_mut().find(|s| s.id == id) {
            s.shortcut = new_keys.to_string();
            self.save();
            // Re-register all
            self.register_all();
            Ok(())
        } else {
            // Re-register all anyway just in case
            self.register_all();
            Err("Shortcut ID not found".to_string())
        }
    }

    pub fn register_all(&mut self) {
        self.last_registration_errors.clear();
        for s in &self.shortcuts {
            if s.enabled {
                if let Ok(hotkey) = s.shortcut.parse::<HotKey>() {
                    if let Err(_) = self.manager.0.register(hotkey) {
                        self.last_registration_errors
                            .push(format!("{} ({})", s.label, s.shortcut));
                    }
                }
            }
        }
    }

    pub fn unregister_all(&self) {
        for s in &self.shortcuts {
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
            label: "Main Capture".to_string(),
            shortcut: "Alt+A".to_string(),
            icon: "\u{E11A}".to_string(),
            color: "#3b82f6".to_string(),
            enabled: true,
        },
        Shortcut {
            id: "ocr".to_string(),
            label: "OCR Selection".to_string(),
            shortcut: "Alt+S".to_string(),
            icon: "\u{E11B}".to_string(),
            color: "#10b981".to_string(),
            enabled: true,
        },
    ]
}

pub fn load_shortcuts() -> Vec<Shortcut> {
    get_default_shortcuts()
}
