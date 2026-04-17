use crate::app_state::AppState;
use crate::service::config::ConfigError;
use tauri::Manager;

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
        let config = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
        config.config.save_path.clone()
    };

    app.opener()
        .open_path(path_to_open, None::<&str>)
        .map_err(|e| e.to_string())
}
