use crate::service::pin::{PinData, PinItem, PinState};
use tauri::{AppHandle, Emitter, State, Manager};
use tauri_plugin_notification::NotificationExt;
use base64::prelude::*;

#[tauri::command]
pub async fn create_text_pin(
    app: AppHandle,
    state: State<'_, PinState>,
    id: String,
    content: String,
) -> Result<(), String> {
    // 1. Store Content
    state.add_pin(id.clone(), PinData::Text(content));

    // 2. Notify frontend if it exists
    let _ = app.emit("pin-collection-updated", ());

    // 3. Open or focus Collection
    crate::service::pin::open_pin_collection_window(&app)
        .map_err(|e| format!("Failed to open collection: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_all_pins(state: State<'_, PinState>) -> Result<Vec<PinItem>, String> {
    Ok(state.get_all_pins())
}

#[tauri::command]
pub fn remove_pin(state: State<'_, PinState>, id: String) -> Result<(), String> {
    state.remove_pin(&id);
    Ok(())
}

#[tauri::command]
pub fn clear_all_pins(state: State<'_, PinState>) -> Result<(), String> {
    state.clear_all();
    Ok(())
}

#[tauri::command]
pub async fn toggle_pin_always_on_top(window: tauri::Window) -> Result<bool, String> {
    let current = window.is_always_on_top().map_err(|e| e.to_string())?;
    let next = !current;
    window.set_always_on_top(next).map_err(|e| e.to_string())?;
    Ok(next)
}

#[tauri::command]
pub async fn is_pin_always_on_top(window: tauri::Window) -> Result<bool, String> {
    window.is_always_on_top().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_pin_window_size(window: tauri::Window, width: f64, height: f64) -> Result<(), String> {
    window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height })).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_pin_min_size(window: tauri::Window, width: f64, height: f64) -> Result<(), String> {
    window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize { width, height }))).map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn save_pin_as(
    app: AppHandle,
    state: State<'_, PinState>,
    id: String,
    path: String,
) -> Result<(), String> {
    let pin = state.get_pin(&id).ok_or("Pin not found")?;

    match pin.data {
        PinData::ImageBase64(base64_data) => {
            // 1. Parse base64
            let data = base64_data
                .strip_prefix("data:image/png;base64,")
                .or_else(|| base64_data.strip_prefix("data:image/jpeg;base64,"))
                .unwrap_or(&base64_data);

            let decoded = BASE64_STANDARD.decode(data).map_err(|e| e.to_string())?;
            let img = image::load_from_memory(&decoded).map_err(|e| e.to_string())?;

            // 2. Save
            let path_buf = std::path::PathBuf::from(path);
            let extension = path_buf
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png")
                .to_lowercase();

            let quality = {
                let app_state = app.state::<crate::AppState>();
                let config = app_state.config_state.lock().unwrap_or_else(|e| e.into_inner());
                config.config.jpg_quality
            };

            if extension == "jpg" || extension == "jpeg" {
                let mut file = std::fs::File::create(&path_buf).map_err(|e| e.to_string())?;
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
                encoder.encode_image(&img.to_rgb8()).map_err(|e| e.to_string())?;
            } else {
                img.save(&path_buf).map_err(|e| e.to_string())?;
            }

            // 3. Notify Success
            use crate::service::l10n;
            let _ = app
                .notification()
                .builder()
                .title(l10n::t(&app, "notification.saved_title", "Saved"))
                .body(format!(
                    "{}: {}",
                    l10n::t(&app, "notification.saved_body", "Image saved locally"),
                    path_buf.display()
                ))
                .show();
        }
        _ => return Err("Pin is not an image".to_string()),
    }

    Ok(())
}
