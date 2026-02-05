use crate::app_state::AppState;
use crate::service::config::{ConfigError, Shortcut};
use tauri::State;

#[tauri::command]
pub fn get_shortcuts(state: State<'_, AppState>) -> Vec<Shortcut> {
    let state = state.inner().config_state.lock().unwrap();
    let mut shortcuts = state.config.shortcuts.clone();

    // Populate errors
    for s in &mut shortcuts {
        if state
            .last_registration_errors
            .iter()
            .any(|e| e.contains(&s.label) && e.contains(&s.shortcut))
        {
            s.error = Some("ERR_REGISTER_FAILED".to_string());
        }
    }

    shortcuts
}

#[tauri::command]
pub fn get_startup_errors(state: State<'_, AppState>) -> Vec<String> {
    let state = state.config_state.lock().unwrap();
    state.last_registration_errors.clone()
}

#[tauri::command]
pub fn update_shortcut(
    state: State<'_, AppState>,
    id: String,
    new_keys: String,
) -> Result<(), ConfigError> {
    let mut state = state.inner().config_state.lock().unwrap();
    state.update_shortcut(&id, &new_keys)
}
