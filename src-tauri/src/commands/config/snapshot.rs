use crate::app_state::AppState;
use crate::service::config::ConfigError;
use tauri::State;

#[tauri::command]
pub fn set_snapshot_enabled(state: State<'_, AppState>, enabled: bool) -> Result<(), ConfigError> {
    let mut state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    state.set_snapshot_enabled(enabled);
    Ok(())
}

#[tauri::command]
pub fn set_snapshot_size(
    state: State<'_, AppState>,
    width: i32,
    height: i32,
) -> Result<(), ConfigError> {
    let mut state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    state.set_snapshot_size(width, height);
    Ok(())
}

#[tauri::command]
pub fn set_snapshot_engine(state: State<'_, AppState>, engine: String) -> Result<(), ConfigError> {
    let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
    c_state.config.snapshot_engine = engine;
    c_state.save();
    Ok(())
}
