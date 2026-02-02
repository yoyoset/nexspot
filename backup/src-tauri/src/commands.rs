use crate::service::{logger, native_overlay, overlay_manager};
use crate::AppState;
use chrono::Local;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl};

#[tauri::command]
pub fn get_capture_image(_state: State<AppState>) -> Result<Vec<u8>, String> {
    logger::log("Command", "get_capture_image disabled for native rendering");
    Err("Direct image fetch disabled for performance. Use Native Overlay.".to_string())
}

#[tauri::command]
pub fn save_capture(
    app: AppHandle,
    state: State<AppState>,
    path: Option<String>,
    crop: Option<(u32, u32, u32, u32)>,
) -> Result<String, String> {
    internal_save_capture(&app, &state, path, crop)
}

pub fn internal_save_capture(
    app: &AppHandle,
    state: &AppState,
    path: Option<String>,
    crop: Option<(u32, u32, u32, u32)>,
) -> Result<String, String> {
    logger::log("Command", "internal_save_capture called");
    let lock = state.last_capture.lock().map_err(|_| "Lock fail")?;

    let (data, width, height) = match &*lock {
        Some(tuple) => tuple,
        None => return Err("No capture data".to_string()),
    };

    let mut save_path_str = path.unwrap_or_else(|| {
        let mut p = state.save_path.lock().unwrap().clone();
        if p == "." {
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let mut target = exe_dir.to_path_buf();
                    target.push("Screenshots");
                    std::fs::create_dir_all(&target).ok();
                    p = target.to_string_lossy().to_string();
                }
            }
            if p == "." {
                let path_resolver = app.path();
                if let Ok(mut docs) = path_resolver.picture_dir() {
                    docs.push("HyperLens");
                    std::fs::create_dir_all(&docs).ok();
                    p = docs.to_string_lossy().to_string();
                }
            }
        }
        p
    });

    logger::log("Save", &format!("Saving to: {}", save_path_str));

    let metadata = std::fs::metadata(&save_path_str);
    if let Ok(meta) = metadata {
        if meta.is_dir() {
            let now = Local::now();
            let name = format!("Screenshot_{}.png", now.format("%Y%m%d_%H%M%S"));
            let mut p_buf = std::path::PathBuf::from(save_path_str);
            p_buf.push(name);
            save_path_str = p_buf.to_string_lossy().to_string();
        }
    }

    let img_buffer = image::RgbaImage::from_raw(*width, *height, data.clone())
        .ok_or("Failed to create image from raw pixels")?;
    let img = image::DynamicImage::ImageRgba8(img_buffer);

    let final_img = if let Some((x, y, w, h)) = crop {
        // Validation for Crop
        let x = std::cmp::max(0, x);
        let y = std::cmp::max(0, y);
        let w = std::cmp::min(w, *width - x);
        let h = std::cmp::min(h, *height - y);
        img.crop_imm(x, y, w, h)
    } else {
        img
    };

    if let Err(e) = final_img.save(&save_path_str) {
        logger::log("Save", &format!("Error saving file: {}", e));
        return Err(e.to_string());
    }

    Ok(save_path_str)
}

#[tauri::command]
pub async fn hide_overlay(app: AppHandle) -> Result<(), String> {
    native_overlay::hide();
    overlay_manager::OverlayManager::hide_all(&app);
    Ok(())
}

#[tauri::command]
pub async fn open_settings(app: AppHandle) -> Result<(), String> {
    use tauri::webview::WebviewWindowBuilder;

    if let Some(win) = app.get_webview_window("settings") {
        win.show().ok();
        win.set_focus().ok();
        return Ok(());
    }

    let _ = WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("/settings".into()))
        .title("HyperLens Settings")
        .inner_size(600.0, 400.0)
        .center()
        .build();
    Ok(())
}

#[tauri::command]
pub async fn trigger_capture(app: AppHandle) -> Result<(), String> {
    logger::log("Command", "Manual capture triggered");
    app.emit("hotkey-pressed", ()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_log_enabled(enabled: bool) {
    logger::set_enabled(enabled);
}

#[tauri::command]
pub fn clean_log() -> Result<(), String> {
    logger::clear_log()
}

#[tauri::command]
pub fn frontend_log(msg: String) {
    logger::log("Frontend", &msg);
}

#[tauri::command]
pub fn get_save_path_setting(state: State<AppState>) -> String {
    state.save_path.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_save_path_setting(state: State<AppState>, path: String) {
    if let Ok(mut guard) = state.save_path.lock() {
        *guard = path;
    }
}

#[tauri::command]
pub fn open_log_folder(_app: AppHandle) -> Result<(), String> {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let log_path = exe_dir.join("hyper-lens.log");
            #[cfg(target_os = "windows")]
            {
                use std::process::Command;
                Command::new("explorer")
                    .arg("/select,")
                    .arg(log_path)
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn open_gallery(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let save_path = state.save_path.lock().unwrap().clone();
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // If path is relative ".", resolve it first (logic duplicated from save, maybe clean up later)
        let mut target_path = save_path.clone();
        if target_path == "." {
            // Fallback logic
            if let Ok(mut docs) = app.path().picture_dir() {
                docs.push("HyperLens");
                std::fs::create_dir_all(&docs).ok();
                target_path = docs.to_string_lossy().to_string();
            }
        }

        Command::new("explorer")
            .arg(target_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn is_log_enabled() -> bool {
    logger::is_enabled()
}

#[tauri::command]
pub fn check_capture_status(state: State<AppState>) -> bool {
    if let Ok(lock) = state.last_capture.lock() {
        lock.is_some()
    } else {
        false
    }
}

#[tauri::command]
pub fn get_shortcuts() -> Vec<crate::service::shortcut_manager::Shortcut> {
    crate::service::shortcut_manager::load_shortcuts()
}

#[tauri::command]
pub fn save_shortcuts(
    shortcuts: Vec<crate::service::shortcut_manager::Shortcut>,
) -> Result<(), String> {
    crate::service::shortcut_manager::save_shortcuts(shortcuts)
}
