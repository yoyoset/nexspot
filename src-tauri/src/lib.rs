use crate::capture::engine;
use crate::service::{hotkey, logger, native_overlay, overlay_manager, tray};
use chrono::Local;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Listener, Manager, State};

pub mod capture;
pub mod service;

struct AppState {
    last_capture: Mutex<Option<(Vec<u8>, u32, u32)>>,
    save_path: Mutex<String>,
    #[allow(dead_code)]
    hotkey_manager: Mutex<Option<hotkey::HotkeyManager>>,
}

#[tauri::command]
fn get_capture_image(_state: State<AppState>) -> Result<Vec<u8>, String> {
    // Disable fetching large image data via IPC
    logger::log("Command", "get_capture_image disabled for native rendering");
    Err("Direct image fetch disabled for performance. Use Native Overlay.".to_string())
}

#[tauri::command]
fn save_capture(
    app: AppHandle,
    state: State<AppState>,
    path: Option<String>,
    crop: Option<(u32, u32, u32, u32)>,
) -> Result<String, String> {
    logger::log("Command", "save_capture called");
    let lock = state.last_capture.lock().map_err(|_| "Lock fail")?;

    // Deconstruct tuple (bytes, width, height)
    let (data, width, height) = match &*lock {
        Some(tuple) => tuple,
        None => return Err("No capture data".to_string()),
    };

    // Default path logic - Modified for Portability (Feature 2)
    let mut save_path_str = path.unwrap_or_else(|| {
        let mut p = state.save_path.lock().unwrap().clone();
        if p == "." {
            // Default to generic "Screenshots" folder next to executable
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let mut target = exe_dir.to_path_buf();
                    target.push("Screenshots");
                    std::fs::create_dir_all(&target).ok();
                    p = target.to_string_lossy().to_string();
                }
            }
            if p == "." {
                // Fallback if exe path fails
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

    // Generate filename if directory
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

    // Create Image from Raw Pixels (Raw RGBA -> DynamicImage)
    let img_buffer = image::RgbaImage::from_raw(*width, *height, data.clone())
        .ok_or("Failed to create image from raw pixels")?;
    let img = image::DynamicImage::ImageRgba8(img_buffer);

    let final_img = if let Some((x, y, w, h)) = crop {
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
async fn hide_overlay(app: AppHandle) -> Result<(), String> {
    // Hide both native and WebView overlays
    native_overlay::hide();
    overlay_manager::OverlayManager::hide_all(&app);
    Ok(())
}

#[tauri::command]
async fn open_settings(app: AppHandle) -> Result<(), String> {
    use tauri::webview::WebviewWindowBuilder;
    use tauri::WebviewUrl;

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
async fn trigger_capture(app: AppHandle) -> Result<(), String> {
    logger::log("Command", "Manual capture triggered");
    app.emit("hotkey-pressed", ()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_log_enabled(enabled: bool) {
    logger::set_enabled(enabled);
}

#[tauri::command]
fn clean_log() -> Result<(), String> {
    logger::clear_log()
}

#[tauri::command]
fn get_save_path_setting(state: State<AppState>) -> String {
    state.save_path.lock().unwrap().clone()
}

#[tauri::command]
fn set_save_path_setting(state: State<AppState>, path: String) {
    if let Ok(mut guard) = state.save_path.lock() {
        *guard = path;
    }
}

#[tauri::command]
fn open_log_folder(_app: AppHandle) -> Result<(), String> {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let log_path = exe_dir.join("hyper-lens.log");
            // Use opener or shell to reveal in folder
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
fn is_log_enabled() -> bool {
    logger::is_enabled()
}

#[tauri::command]
fn check_capture_status(state: State<AppState>) -> bool {
    if let Ok(lock) = state.last_capture.lock() {
        lock.is_some()
    } else {
        false
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Init Logger (Feature 1)
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    logger::init(&exe_dir.to_string_lossy());
                }
            }
            logger::log("System", "HyperLens Startup");

            let hk_manager = hotkey::HotkeyManager::new();

            app.manage(AppState {
                last_capture: Mutex::new(None),
                save_path: Mutex::new(".".to_string()),
                hotkey_manager: Mutex::new(Some(hk_manager)),
            });

            overlay_manager::OverlayManager::init_overlays(app.handle())?;

            // Initialize hot-standby capture sessions
            service::capture_service::init_sessions();

            // Initialize native overlay for instant dimming
            native_overlay::init().ok();

            tray::create_tray(app.handle())?;

            hotkey::listen_hotkeys(app.handle().clone());

            let handle = app.handle().clone();
            app.listen("hotkey-pressed", move |_| {
                logger::log("Event", "Hotkey/Trigger Pressed");

                let h = handle.clone();
                // 1. Start capture FIRST (Instant Freeze)
                // Running in thread to avoid blocking event loop, but must be fast
                std::thread::spawn(move || {
                    logger::log("Capture", "Starting capture_all_monitors...");

                    match engine::capture_all_monitors() {
                        Ok((bytes, w, h_px)) => {
                            logger::log(
                                "Capture",
                                &format!("SUCCESS: {}x{}, {} bytes", w, h_px, bytes.len()),
                            );

                            // 2. IMMEDIATE NATIVE RENDER (Zero Latency)
                            native_overlay::update_with_buffer(w, h_px, &bytes);
                            native_overlay::show();

                            // 3. Show WebView overlay (UI only)
                            overlay_manager::OverlayManager::show_all(&h);

                            // 4. Update State
                            if let Some(state) = h.try_state::<AppState>() {
                                if let Ok(mut lock) = state.last_capture.lock() {
                                    *lock = Some((bytes, w, h_px));
                                    logger::log("Capture", "Stored in AppState");
                                }
                            }

                            // SHORT DELAY to allow frontend listeners to mount/hydrate (Race Condition Fix)
                            std::thread::sleep(std::time::Duration::from_millis(300));

                            // 5. Notify frontend (Ready for selection)
                            // We pass "native-mode" to tell frontend not to look for image
                            if let Err(e) = h.emit("capture-ready", "native-mode") {
                                logger::log("Capture", &format!("Emit failed: {}", e));
                            } else {
                                logger::log("Capture", "Event 'capture-ready' emitted");
                            }
                        }
                        Err(e) => {
                            logger::log("Capture", &format!("FAILED: {}", e));
                        }
                    }
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_capture_image,
            save_capture,
            hide_overlay,
            open_settings,
            trigger_capture, // Feature 3
            set_log_enabled, // Feature 1
            clean_log,       // Feature 1
            is_log_enabled,
            get_save_path_setting,
            set_save_path_setting,
            open_log_folder,
            check_capture_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
