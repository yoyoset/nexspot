use crate::app_state::AppState;
use crate::service::config::ConfigError;
use tauri::State;

#[tauri::command]
pub fn suspend_hotkeys(state: State<'_, AppState>) -> Result<(), ConfigError> {
    let c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    c_state.unregister_all();
    if let Ok(mut map_lock) = state.hotkey_map.write().map_err(|e| e.into_inner()) {
        map_lock.clear();
    }
    Ok(())
}

#[tauri::command]
pub fn resume_hotkeys(state: State<'_, AppState>) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    let new_map = c_state.register_all();
    if let Ok(mut map_lock) = state.hotkey_map.write().map_err(|e| e.into_inner()) {
        *map_lock = new_map;
    }
    Ok(())
}

#[tauri::command]
pub fn refresh_hotkeys(state: State<'_, AppState>) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    let new_map = c_state.register_all();
    if let Ok(mut map_lock) = state.hotkey_map.write().map_err(|e| e.into_inner()) {
        *map_lock = new_map;
    }
    Ok(())
}
