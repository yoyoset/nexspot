use crate::app_state::AppState;
use crate::service::config::{AIShortcut, AppConfig, ConfigError};
use tauri::{Manager, State};

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> AppConfig {
    let state = state.config_state.lock().unwrap();
    state.config.clone()
}

#[tauri::command]
pub fn set_save_path(state: State<'_, AppState>, path: String) -> Result<(), ConfigError> {
    let mut state = state.config_state.lock().unwrap();
    state.set_save_path(path);
    Ok(())
}

#[tauri::command]
pub fn set_font_family(state: State<'_, AppState>, font: String) -> Result<(), ConfigError> {
    {
        let mut config = state.config_state.lock().unwrap();
        config.set_font_family(font.clone());
    }
    // Update active overlay state immediately
    let overlay = state.overlay_manager.lock().unwrap();
    let mut overlay_state = overlay.state.lock().unwrap();
    overlay_state.font_family = font;
    Ok(())
}

#[tauri::command]
pub fn set_vello_enabled(state: State<'_, AppState>, enabled: bool) -> Result<(), ConfigError> {
    {
        let mut config_state = state.config_state.lock().unwrap();
        config_state.set_vello_enabled(enabled);
    }

    // Update OverlayManager's pre-heat stream lifecycle
    let mut overlay_mgr = state.overlay_manager.lock().unwrap();
    if enabled {
        overlay_mgr.start_pre_heat();
    } else {
        overlay_mgr.stop_pre_heat();
    }

    Ok(())
}

#[tauri::command]
pub fn set_vello_advanced_effects(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), ConfigError> {
    {
        let mut config_state = state.config_state.lock().unwrap();
        config_state.set_vello_advanced_effects(enabled);
    }

    // Update active overlay state immediately
    if let Ok(overlay) = state.overlay_manager.lock() {
        if let Ok(mut overlay_state) = overlay.state.lock() {
            overlay_state.enable_advanced_effects = enabled;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn set_snapshot_enabled(state: State<'_, AppState>, enabled: bool) -> Result<(), ConfigError> {
    let mut state = state.config_state.lock().unwrap();
    state.set_snapshot_enabled(enabled);
    Ok(())
}

#[tauri::command]
pub fn set_snapshot_size(
    state: State<'_, AppState>,
    width: i32,
    height: i32,
) -> Result<(), ConfigError> {
    let mut state = state.config_state.lock().unwrap();
    state.set_snapshot_size(width, height);
    Ok(())
}

#[tauri::command]
pub fn set_selection_engine(state: State<'_, AppState>, engine: String) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap();
    c_state.config.selection_engine = engine;
    c_state.save();
    Ok(())
}

#[tauri::command]
pub fn set_theme(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    theme: String,
) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap();
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
    let mut c_state = state.config_state.lock().unwrap();
    c_state.set_accent_color(color);
    Ok(())
}

#[tauri::command]
pub fn set_jpg_quality(state: State<'_, AppState>, quality: u8) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap();
    c_state.set_jpg_quality(quality);
    Ok(())
}

#[tauri::command]
pub fn set_concurrency(state: State<'_, AppState>, concurrency: usize) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap();
    c_state.set_concurrency(concurrency);
    Ok(())
}

#[tauri::command]
pub fn set_snapshot_engine(state: State<'_, AppState>, engine: String) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap();
    c_state.config.snapshot_engine = engine;
    c_state.save();
    Ok(())
}

#[tauri::command]
pub fn add_ai_shortcut(
    state: State<'_, AppState>,
    shortcut: AIShortcut,
) -> Result<(), ConfigError> {
    {
        let mut c_state = state.config_state.lock().unwrap();
        c_state.add_ai_shortcut(shortcut)?;
        let new_map = c_state.register_all();
        if let Ok(mut map_lock) = state.hotkey_map.write() {
            *map_lock = new_map;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn remove_ai_shortcut(state: State<'_, AppState>, id: String) -> Result<(), ConfigError> {
    {
        let mut c_state = state.config_state.lock().unwrap();
        c_state.remove_ai_shortcut(&id)?;
        let new_map = c_state.register_all();
        if let Ok(mut map_lock) = state.hotkey_map.write() {
            *map_lock = new_map;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn update_ai_shortcut(
    state: State<'_, AppState>,
    id: String,
    shortcut: AIShortcut,
) -> Result<(), ConfigError> {
    {
        let mut c_state = state.config_state.lock().unwrap();
        c_state.update_ai_shortcut(&id, shortcut)?;
        let new_map = c_state.register_all();
        if let Ok(mut map_lock) = state.hotkey_map.write() {
            *map_lock = new_map;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn add_workflow(
    state: State<'_, AppState>,
    workflow: crate::service::config::types::CaptureWorkflow,
) -> Result<(), ConfigError> {
    {
        let mut c_state = state.config_state.lock().unwrap();
        c_state.add_workflow(workflow)?;
        let new_map = c_state.register_all();
        if let Ok(mut map_lock) = state.hotkey_map.write() {
            *map_lock = new_map;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn remove_workflow(state: State<'_, AppState>, id: String) -> Result<(), ConfigError> {
    {
        let mut c_state = state.config_state.lock().unwrap();
        c_state.remove_workflow(&id)?;
        let new_map = c_state.register_all();
        if let Ok(mut map_lock) = state.hotkey_map.write() {
            *map_lock = new_map;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn update_workflow(
    state: State<'_, AppState>,
    id: String,
    workflow: crate::service::config::types::CaptureWorkflow,
) -> Result<(), ConfigError> {
    {
        let mut c_state = state.config_state.lock().unwrap();
        c_state.update_workflow(&id, workflow)?;
        // Refresh hotkey map immediately
        let new_map = c_state.register_all();
        if let Ok(mut map_lock) = state.hotkey_map.write() {
            *map_lock = new_map;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn suspend_hotkeys(state: State<'_, AppState>) -> Result<(), ConfigError> {
    let c_state = state.config_state.lock().unwrap();
    c_state.unregister_all();
    if let Ok(mut map_lock) = state.hotkey_map.write() {
        map_lock.clear();
    }
    Ok(())
}

#[tauri::command]
pub fn resume_hotkeys(state: State<'_, AppState>) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap();
    let new_map = c_state.register_all();
    if let Ok(mut map_lock) = state.hotkey_map.write() {
        *map_lock = new_map;
    }
    Ok(())
}

#[tauri::command]
pub fn refresh_hotkeys(state: State<'_, AppState>) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap();
    let new_map = c_state.register_all();
    if let Ok(mut map_lock) = state.hotkey_map.write() {
        *map_lock = new_map;
    }
    Ok(())
}

#[tauri::command]
pub async fn select_folder(app: tauri::AppHandle) -> Result<Option<String>, ConfigError> {
    use tauri_plugin_dialog::DialogExt;

    // In Tauri v2, the dialog plugin provides this on AppHandle
    let folder = app.dialog().file().blocking_pick_folder();

    Ok(folder.map(|f| f.to_string()))
}

#[tauri::command]
pub async fn open_folder(app: tauri::AppHandle, path: Option<String>) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    let path_to_open = if let Some(p) = path {
        p
    } else {
        let state = app.try_state::<AppState>().ok_or("State not found")?;
        let config = state.config_state.lock().unwrap();
        config.config.save_path.clone()
    };

    app.opener()
        .open_path(path_to_open, None::<&str>)
        .map_err(|e| e.to_string())
}
