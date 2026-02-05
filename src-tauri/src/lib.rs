pub mod service;
pub mod app_state;
pub mod commands;

pub use app_state::AppState;
use crate::service::logger::{self, LoggerState};
use crate::service::native_overlay::OverlayManager;
use crate::service::config::ConfigState;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use global_hotkey::GlobalHotKeyEvent;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};



pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .setup(|app| {
            let overlay_manager =
                OverlayManager::new(app.handle().clone()).expect("Failed to init OverlayManager");
            let logger_state = LoggerState::new(app.handle());
            let config_state = ConfigState::new(app.handle());

            let app_state = AppState {
                overlay_manager: Mutex::new(overlay_manager),
                config_state: Mutex::new(config_state),
                logger_state,
            };

            app.manage(app_state);

            // Now set the user data pointer for the Win32 window
            let state = app.state::<AppState>();
            {
                let mut overlay = state.overlay_manager.lock().unwrap();
                let ptr = &mut *overlay as *mut OverlayManager;
                overlay.set_user_data(ptr);
            }

            // --- System Tray Setup ---
            let quit_i = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let dashboard_i = MenuItem::with_id(app, "dashboard", "Dashboard", true, None::<&str>)?;
            let capture_i = MenuItem::with_id(app, "capture", "Capture", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &dashboard_i,
                    &capture_i,
                    &settings_i,
                    &tauri::menu::PredefinedMenuItem::separator(app)?,
                    &quit_i,
                ],
            )?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "dashboard" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "settings" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                                let _ = win.emit("open-settings", ()); // Optional: emit event to frontend
                            }
                        }
                        "capture" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let state = app_handle.state::<AppState>();
                                // Same logic as start_capture
                                let state_arc = {
                                    let overlay = state.overlay_manager.lock().unwrap();
                                    overlay.state.clone()
                                };

                                let res = tauri::async_runtime::spawn_blocking(move || {
                                    crate::service::native_overlay::capture::perform_capture(
                                        &state_arc,
                                    )
                                })
                                .await;

                                if let Ok(Ok((x, y, w, h))) = res {
                                    let app_handle_ui = app_handle.clone();
                                    let _ = app_handle.run_on_main_thread(move || {
                                        let state = app_handle_ui.state::<AppState>();
                                        if let Ok(mut overlay) = state.overlay_manager.lock() {
                                            let _ = overlay.show_overlay_at(x, y, w, h);
                                        };
                                    });
                                }
                            });
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 1. Show Main Window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            // 2. Check Shortcuts Conflicts
            let app_handle = app.handle();
            let state = app_handle.state::<AppState>();
            let errors = {
                let c_state = state.config_state.lock().unwrap();
                c_state.last_registration_errors.clone()
            };
            if !errors.is_empty() {
                // Emit event to frontend for translation and display
                let _ = app.emit("shortcut-startup-error", errors);
            }

            // 3. Check Mixed DPI
            if let Ok(monitors) = crate::service::win32::monitor::enumerate_monitors() {
                if monitors.len() > 1 {
                    let first_dpi_x = monitors[0].dpi_x;
                    let mixed = monitors.iter().any(|m| m.dpi_x != first_dpi_x);
                    if mixed {
                        let _ = app.emit("mixed-dpi-detected", ());
                    }
                }
            }

            // 4. Warmup GDI & Monitor APIs (Accelerate first capture)
            tauri::async_runtime::spawn_blocking(|| {
                let _ = crate::service::win32::monitor::enumerate_monitors();
                unsafe {
                     let hdc = windows::Win32::Graphics::Gdi::GetDC(None);
                     windows::Win32::Graphics::Gdi::ReleaseDC(None, hdc);
                }
            });

            // 5. Hotkey Listener
            let app_handle_hotkey = app.handle().clone();
            std::thread::spawn(move || {
                let receiver = GlobalHotKeyEvent::receiver();
                while let Ok(event) = receiver.recv() {
                    if event.state == global_hotkey::HotKeyState::Released {
                         log::info!("[Hotkey Debug] Event Received: {:?}", event);
                         let state = app_handle_hotkey.state::<AppState>();
                         let mut matched_id = None;
                         {
                             if let Ok(c_state) = state.config_state.lock() {
                                 for s in &c_state.config.shortcuts {
                                     if let Ok(hotkey) = s.shortcut.parse::<global_hotkey::hotkey::HotKey>() {
                                         if hotkey.id() == event.id {
                                             matched_id = Some(s.id.clone());
                                             log::info!("[Hotkey Debug] Matched ID: {}", s.id);
                                             break;
                                         }
                                     }
                                 }
                             } else {
                                 log::error!("[Hotkey Debug] Failed to lock shortcut_state");
                             }
                         }

                         if let Some(id) = matched_id {
                             if id == "capture" || id == "ocr" {
                                 let app = app_handle_hotkey.clone();
                                 let id_clone = id.clone();
                                 tauri::async_runtime::spawn(async move {
                                     log::info!("[Hotkey Debug] Spawned Async Task for {}", id_clone);
                                     let state = app.state::<AppState>();
                                     
                                     // 1. Check if already visible
                                     {
                                         if let Ok(overlay) = state.overlay_manager.lock() {
                                             if let Ok(os) = overlay.state.lock() {
                                                 if os.is_visible {
                                                     log::warn!("[Hotkey Debug] Overlay is visible, ignoring.");
                                                     return;
                                                 }
                                             }
                                         }
                                     }
                                     
                                     // 2. Set mode and perform capture
                                     let state_arc = {
                                         let overlay = state.overlay_manager.lock().unwrap();
                                         let mut os = overlay.state.lock().unwrap();
                                         os.capture_mode = if id_clone == "ocr" {
                                             crate::service::native_overlay::state::CaptureMode::Ocr
                                         } else {
                                             crate::service::native_overlay::state::CaptureMode::Standard
                                         };
                                         overlay.state.clone()
                                     };

                                     log::info!("[Hotkey Debug] Starting Capture Sequence ({})", id_clone);
                                     let res = tauri::async_runtime::spawn_blocking(move || {
                                         crate::service::native_overlay::capture::perform_capture(&state_arc)
                                     }).await;

                                     if let Ok(Ok((x, y, w, h))) = res {
                                         let app_ui = app.clone();
                                         let _ = app.run_on_main_thread(move || {
                                             let state = app_ui.state::<AppState>();
                                              if let Ok(mut overlay) = state.overlay_manager.lock() {
                                                  let _ = overlay.show_overlay_at(x, y, w, h);
                                              };
                                         });
                                     }
                                 });
                             }
                         }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_capture,
            commands::get_shortcuts,
            commands::update_shortcut,
            commands::get_startup_errors,
            commands::get_config,
            commands::set_save_path,
            commands::set_ocr_engine,
            commands::select_folder,
            logger::clear_logs,
            logger::reveal_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
