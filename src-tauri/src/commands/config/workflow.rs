use crate::app_state::AppState;
use crate::service::config::ConfigError;
use tauri::State;

#[tauri::command]
pub fn add_workflow(
    state: State<'_, AppState>,
    workflow: crate::service::config::types::CaptureWorkflow,
) -> Result<(), ConfigError> {
    {
        let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
        c_state.add_workflow(workflow)?;
        let new_map = c_state.register_all();
        if let Ok(mut map_lock) = state.hotkey_map.write().map_err(|e| e.into_inner()) {
            *map_lock = new_map;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn remove_workflow(state: State<'_, AppState>, id: String) -> Result<(), ConfigError> {
    {
        let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
        c_state.remove_workflow(&id)?;
        let new_map = c_state.register_all();
        if let Ok(mut map_lock) = state.hotkey_map.write().map_err(|e| e.into_inner()) {
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
        let mut c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
        c_state.update_workflow(&id, workflow)?;
        // Refresh hotkey map immediately
        let new_map = c_state.register_all();
        if let Ok(mut map_lock) = state.hotkey_map.write().map_err(|e| e.into_inner()) {
            *map_lock = new_map;
        }
    }
    Ok(())
}
