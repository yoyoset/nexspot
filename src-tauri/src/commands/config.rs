use crate::app_state::AppState;
use crate::service::config::{AppConfig, ConfigError};
use tauri::State;

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
pub fn set_ocr_engine(state: State<'_, AppState>, engine: String) -> Result<(), ConfigError> {
    let mut state = state.config_state.lock().unwrap();
    state.set_ocr_engine(engine);
    Ok(())
}

#[tauri::command]
pub async fn select_folder(app: tauri::AppHandle) -> Result<Option<String>, ConfigError> {
    use tauri_plugin_dialog::DialogExt;

    // In Tauri v2, the dialog plugin provides this on AppHandle
    let folder = app.dialog().file().blocking_pick_folder();

    Ok(folder.map(|f| f.to_string()))
}
