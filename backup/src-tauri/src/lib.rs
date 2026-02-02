use crate::capture::engine;
use crate::service::{hotkey, logger, native_overlay, overlay_manager, tray};
use std::sync::Mutex;
use tauri::{Emitter, Listener, Manager};

pub mod capture;
pub mod commands;
pub mod service;

pub struct AppState {
    pub last_capture: Mutex<Option<(Vec<u8>, u32, u32)>>,
    pub save_path: Mutex<String>,
    #[allow(dead_code)]
    pub hotkey_manager: Mutex<Option<hotkey::HotkeyManager>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
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

            service::capture_service::init_sessions();

            native_overlay::init().ok();

            // Bridge Native Events to Web
            let app_handle = app.handle().clone();
            native_overlay::set_event_callback(move |event, payload| {
                logger::log("NativeBridge", &format!("Emitting {}: {}", event, payload));

                // P9: Pure Native Logic.
                // We only log events here for debugging.
                // The Native Overlay now handles its own lifecycle until "save" or "cancel" button is clicked.

                if event == "selection-update" {
                    logger::log("NativeBridge", "Selection Updated (Native Internal)");
                } else if event == "selection-cancel" {
                    logger::log("NativeBridge", "Selection Cancelled");
                    native_overlay::hide();
                    service::overlay_manager::OverlayManager::hide_all(&app_handle);
                    service::overlay_manager::OverlayManager::set_click_through(&app_handle, true);
                    native_overlay::set_input_passthrough(false);
                    native_overlay::set_topmost(true);
                } else if event == "capture-save" {
                    // P9: Handle Native Save
                    logger::log("NativeBridge", "Processing Capture Save...");

                    // Payload: { "x": 100, "y": 100, "width": 200, "height": 200 }
                    let parsed: Option<(u32, u32, u32, u32)> = serde_json::from_str(&payload)
                        .ok()
                        .map(|v: serde_json::Value| {
                            (
                                v["x"].as_f64().unwrap_or(0.0) as u32,
                                v["y"].as_f64().unwrap_or(0.0) as u32,
                                v["width"].as_f64().unwrap_or(0.0) as u32,
                                v["height"].as_f64().unwrap_or(0.0) as u32,
                            )
                        });

                    if let Some(crop) = parsed {
                        // HIDE has already happened in window.rs.
                        // We use a thread for PNG encoding to keep UI responsive.
                        let h = app_handle.clone();

                        std::thread::spawn(move || {
                            if let Some(state) = h.try_state::<AppState>() {
                                match commands::internal_save_capture(&h, &state, None, Some(crop))
                                {
                                    Ok(path) => {
                                        logger::log("NativeBridge", &format!("Saved to: {}", path));

                                        // Notification
                                        use tauri_plugin_notification::NotificationExt;
                                        let _ = h
                                            .notification()
                                            .builder()
                                            .title("Screenshot Saved")
                                            .body(&format!("Saved to: {}", path))
                                            .show();

                                        // Logic Cleanup
                                        service::overlay_manager::OverlayManager::hide_all(&h);
                                        service::overlay_manager::OverlayManager::set_click_through(
                                            &h, true,
                                        );
                                    }
                                    Err(e) => {
                                        logger::log("NativeBridge", &format!("Save Failed: {}", e));
                                    }
                                }
                            }
                        });

                        // Immediate Cleanup of Native Overlay state
                        // These atoms are thread-safe and affect the window instantly.
                        native_overlay::set_input_passthrough(false);
                        native_overlay::set_topmost(true);
                    } else {
                        logger::log("NativeBridge", "Failed to parse save payload");
                    }
                }
            });

            tray::create_tray(app.handle())?;

            hotkey::listen_hotkeys(app.handle().clone());

            let handle = app.handle().clone();
            app.listen("hotkey-pressed", move |_| {
                logger::log("Event", "Hotkey/Trigger Pressed");

                let h = handle.clone();
                std::thread::spawn(move || {
                    logger::log("Capture", "Starting capture_all_monitors...");

                    // SAFETY RESET: Ensure state is clean before capture
                    native_overlay::set_input_passthrough(false);
                    native_overlay::set_topmost(true);

                    match engine::capture_all_monitors() {
                        Ok((bytes, w, h_px)) => {
                            logger::log(
                                "Capture",
                                &format!("SUCCESS: {}x{}, {} bytes", w, h_px, bytes.len()),
                            );

                            native_overlay::update_with_buffer(w, h_px, &bytes);
                            native_overlay::show();

                            overlay_manager::OverlayManager::set_click_through(&h, true);
                            overlay_manager::OverlayManager::show_all(&h);

                            if let Some(state) = h.try_state::<AppState>() {
                                if let Ok(mut lock) = state.last_capture.lock() {
                                    *lock = Some((bytes, w, h_px));
                                    logger::log("Capture", "Stored in AppState");
                                }
                            }

                            std::thread::sleep(std::time::Duration::from_millis(300));

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
            commands::get_capture_image,
            commands::save_capture,
            commands::hide_overlay,
            commands::open_settings,
            commands::trigger_capture,
            commands::set_log_enabled,
            commands::clean_log,
            commands::frontend_log,
            commands::is_log_enabled,
            commands::get_save_path_setting,
            commands::set_save_path_setting,
            commands::open_log_folder,
            commands::open_gallery,
            commands::check_capture_status,
            commands::get_shortcuts,
            commands::save_shortcuts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
