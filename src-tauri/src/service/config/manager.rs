use super::error::ConfigError;
use super::{fonts, hotkey, io, types};
use global_hotkey::GlobalHotKeyManager;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use types::{AIShortcut, AppConfig, HotkeyAction, SafeGlobalHotKeyManager};

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
            let _ = std::fs::create_dir_all(&config_dir);
        }
        let config_path = config_dir.join("config.json");

        let manager = GlobalHotKeyManager::new().expect("Failed to init GlobalHotKeyManager");

        let mut state = Self {
            manager: SafeGlobalHotKeyManager(manager),
            config: AppConfig::default(),
            config_path: config_path.clone(),
            last_registration_errors: Vec::new(),
        };

        // Load config via IO module
        state.config = io::load(&config_path);

        // Fonts registration
        let fonts_dir = io::resolve_fonts_path(app_handle);
        if !fonts_dir.exists() {
            let _ = std::fs::create_dir_all(&fonts_dir);
        }
        let _ = fonts::register_custom_fonts(&fonts_dir);

        state
    }

    pub fn load(&mut self) {
        self.config = io::load(&self.config_path);
        // Differential update logic is handled by io::load returning full config
        // Re-register all to be safe? Or differential?
        // Original logic was: load -> register_all (which effectively re-registers)
        let _ = hotkey::register_all(
            &self.manager.0,
            &self.config,
            &mut self.last_registration_errors,
        );
    }

    pub fn save(&self) {
        let _ = io::save(&self.config_path, &self.config);
    }

    pub fn update_shortcut(&mut self, id: &str, new_keys: &str) -> Result<(), ConfigError> {
        let changed = hotkey::update_shortcut(
            &self.manager.0,
            &mut self.config,
            &mut self.last_registration_errors,
            id,
            new_keys,
        )?;
        if changed {
            self.save();
        }
        // Sync registration errors to config object for frontend
        self.config.registration_errors = self.last_registration_errors.clone();
        Ok(())
    }

    pub fn add_ai_shortcut(&mut self, shortcut: AIShortcut) -> Result<(), ConfigError> {
        let changed = hotkey::add_ai_shortcut(&self.manager.0, &mut self.config, shortcut)?;
        if changed {
            self.save();
        }
        Ok(())
    }

    pub fn remove_ai_shortcut(&mut self, id: &str) -> Result<(), ConfigError> {
        let changed = hotkey::remove_ai_shortcut(&self.manager.0, &mut self.config, id)?;
        if changed {
            self.save();
        }
        Ok(())
    }

    pub fn update_ai_shortcut(
        &mut self,
        id: &str,
        new_shortcut: AIShortcut,
    ) -> Result<(), ConfigError> {
        let changed =
            hotkey::update_ai_shortcut(&self.manager.0, &mut self.config, id, new_shortcut)?;
        if changed {
            self.save();
        }
        Ok(())
    }

    pub fn add_workflow(&mut self, workflow: types::CaptureWorkflow) -> Result<(), ConfigError> {
        let changed = hotkey::add_workflow(&self.manager.0, &mut self.config, workflow)?;
        if changed {
            self.save();
        }
        Ok(())
    }

    pub fn remove_workflow(&mut self, id: &str) -> Result<(), ConfigError> {
        let changed = hotkey::remove_workflow(&self.manager.0, &mut self.config, id)?;
        if changed {
            self.save();
        }
        Ok(())
    }

    pub fn update_workflow(
        &mut self,
        id: &str,
        workflow: types::CaptureWorkflow,
    ) -> Result<(), ConfigError> {
        // ... (existing implementation)
        let idx = self
            .config
            .workflows
            .iter()
            .position(|w| w.id == id)
            .ok_or(ConfigError::NotFound)?;
        let old_shortcut = self.config.workflows[idx].shortcut.clone();
        let new_shortcut = workflow.shortcut.clone();

        if old_shortcut != new_shortcut {
            self.update_shortcut(id, &new_shortcut)?;
        }

        // Re-fetch index because update_shortcut might have modified the vector
        let idx = self
            .config
            .workflows
            .iter()
            .position(|w| w.id == id)
            .ok_or(ConfigError::NotFound)?;
        self.config.workflows[idx].label = workflow.label;
        self.config.workflows[idx].action = workflow.action;
        self.config.workflows[idx].output = workflow.output;
        self.config.workflows[idx].enabled = true; // Force enabled

        self.save();
        Ok(())
    }

    pub fn set_theme(&mut self, theme: String) {
        self.config.theme = theme;
        self.save();
    }

    pub fn set_accent_color(&mut self, color: String) {
        self.config.accent_color = color;
        self.save();
    }

    // Pass-through setters
    pub fn set_save_path(&mut self, path: String) {
        self.config.save_path = path;
        self.save();
    }

    pub fn set_font_family(&mut self, font: String) {
        self.config.font_family = font;
        self.save();
    }

    pub fn set_vello_enabled(&mut self, enabled: bool) {
        self.config.vello_enabled = enabled;
        self.save();
    }

    pub fn set_vello_advanced_effects(&mut self, enabled: bool) {
        self.config.vello_advanced_effects = enabled;
        self.save();
    }

    pub fn set_jpg_quality(&mut self, quality: u8) {
        self.config.jpg_quality = quality;
        self.save();
    }

    pub fn set_concurrency(&mut self, concurrency: usize) {
        self.config.concurrency = concurrency;
        self.save();
    }

    pub fn set_snapshot_enabled(&mut self, enabled: bool) {
        self.config.snapshot_enabled = enabled;
        // Update Default Workflow
        if let Some(w) = self
            .config
            .workflows
            .iter_mut()
            .find(|w| w.id == "snapshot_default")
        {
            w.enabled = true; // Always true if referenced
        }
        self.save();
    }

    pub fn set_snapshot_size(&mut self, width: i32, height: i32) {
        self.config.snapshot_width = width;
        self.config.snapshot_height = height;
        // Update Default Workflow
        if let Some(w) = self
            .config
            .workflows
            .iter_mut()
            .find(|w| w.id == "snapshot_default")
        {
            if let types::CaptureAction::Snapshot {
                width: w_val,
                height: h_val,
                ..
            } = &mut w.action
            {
                *w_val = width;
                *h_val = height;
            }
        }
        self.save();
    }

    pub fn resolve_save_path(&self, app: &AppHandle) -> PathBuf {
        io::resolve_save_path(app, &self.config)
    }

    pub fn resolve_fonts_path(&self, app: &AppHandle) -> PathBuf {
        io::resolve_fonts_path(app)
    }

    pub fn register_all(&mut self) -> HashMap<u32, HotkeyAction> {
        let map = hotkey::register_all(
            &self.manager.0,
            &self.config,
            &mut self.last_registration_errors,
        );
        // Sync to config object for UI
        self.config.registration_errors = self.last_registration_errors.clone();
        map
    }

    pub fn unregister_all(&self) {
        hotkey::unregister_all(&self.manager.0, &self.config);
    }
}
