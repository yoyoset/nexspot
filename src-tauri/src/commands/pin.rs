use crate::service::pin::PinState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn create_text_pin(
    app: AppHandle,
    state: State<'_, PinState>,
    id: String,
    content: String,
) -> Result<(), String> {
    // 1. Store Content
    state.add_pin(id.clone(), content);

    // 2. Create Window
    crate::service::pin::create_text_pin_window(&app, &id)
        .map_err(|e| format!("Failed to create window: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_pin_content(state: State<'_, PinState>, id: String) -> Result<String, String> {
    state
        .get_content(&id)
        .ok_or_else(|| "Pin not found".to_string())
}
