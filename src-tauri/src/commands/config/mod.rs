pub mod hotkey;
pub mod io;
pub mod snapshot;
pub mod vello;
pub mod workflow;

pub use hotkey::*;
pub use io::*;
pub use snapshot::*;
pub use vello::*;
pub use workflow::*;

use crate::app_state::AppState;
use crate::service::config::{AppConfig, ConfigError};
use tauri::{Manager, State};

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> AppConfig {
    let state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    state.config.clone()
}

/// Fetch-on-mount catch-up for hotkey registration errors.
/// The backend also emits `shortcut-startup-error` during setup, but that event
/// may fire before the webview registers its listener; this command lets the
/// frontend reconcile the cached errors once it is ready.
#[tauri::command]
pub fn get_startup_errors(state: State<'_, AppState>) -> Vec<String> {
    let state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    state.last_registration_errors.clone()
}

#[tauri::command]
pub fn set_save_path(state: State<'_, AppState>, path: String) -> Result<(), ConfigError> {
    let mut state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    state.set_save_path(path);
    Ok(())
}

#[tauri::command]
pub fn set_font_family(state: State<'_, AppState>, font: String) -> Result<(), ConfigError> {
    {
        let mut config = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
        config.set_font_family(font.clone());
    }
    // Update active overlay state immediately
    if let Ok(overlay) = state.overlay_manager.lock() {
        if let Ok(mut overlay_state) = overlay.state.write() {
            overlay_state.font_family = font;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_theme(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    theme: String,
) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    c_state.set_theme(theme.clone());

    // Apply to window dynamically
    if let Some(window) = app.get_webview_window("main") {
        #[cfg(target_os = "windows")]
        {
            if let Ok(hwnd) = window.hwnd() {
                let is_dark = if theme == "system" {
                    window
                        .theme()
                        .map(|t| t == tauri::Theme::Dark)
                        .unwrap_or(true)
                } else {
                    theme == "dark"
                };

                let _ = crate::service::win32::window::apply_theme(
                    windows::Win32::Foundation::HWND(hwnd.0 as *mut _),
                    is_dark,
                );
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_accent_color(state: State<'_, AppState>, color: String) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    c_state.set_accent_color(color);
    Ok(())
}

#[tauri::command]
pub fn set_ocr_language(state: State<'_, AppState>, lang: String) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    c_state.config.ocr_language = lang;
    c_state.save();
    Ok(())
}


#[tauri::command]
pub fn set_jpg_quality(state: State<'_, AppState>, quality: u8) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    c_state.set_jpg_quality(quality);
    Ok(())
}

#[tauri::command]
pub fn set_concurrency(state: State<'_, AppState>, concurrency: usize) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    c_state.set_concurrency(concurrency);
    Ok(())
}

#[tauri::command]
pub fn set_default_export_format(state: State<'_, AppState>, format: String) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    c_state.set_default_export_format(format);
    Ok(())
}

#[tauri::command]
pub fn set_quick_save(state: State<'_, AppState>, enabled: bool) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    c_state.set_quick_save(enabled);
    Ok(())
}

#[tauri::command]
pub fn set_selection_engine(state: State<'_, AppState>, engine: String) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    c_state.config.selection_engine = engine;
    c_state.save();
    Ok(())
}

#[tauri::command]
pub fn set_language(state: State<'_, AppState>, lang: String) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    c_state.set_language(lang);
    Ok(())
}
