pub mod service;

use crate::service::native_overlay::OverlayManager;
use crate::service::shortcut_manager::{Shortcut, ShortcutState};
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};

use global_hotkey::GlobalHotKeyEvent;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri_plugin_notification::NotificationExt;

pub struct AppState {
    pub overlay_manager: Mutex<OverlayManager>,
    pub shortcut_state: Mutex<ShortcutState>,
}

#[tauri::command]
async fn start_capture(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    println!("Native Capture Triggered");

    // Prevent Recursive Capture
    {
        if let Ok(overlay) = state.overlay_manager.lock() {
            if let Ok(os) = overlay.state.lock() {
                if os.is_visible {
                    println!("Overlay already active, ignoring capture.");
                    return Ok(());
                }
            }
        }
    }

    // 1. Get State Arc (lock briefly)
    let state_arc = {
        let overlay = state.overlay_manager.lock().map_err(|e| e.to_string())?;
        overlay.state.clone()
    };

    // 2. Run Capture (Blocking, CPU heavy) on thread pool
    let app_handle = app.clone();

    // We can't easily spawn_blocking inside async fn unless we wrap it?
    // tauri command is already async (tokio).
    // So we can just call it? NO, `perform_capture` is synchronous/blocking.
    // It will block the async runtime thread. That is okay for short tasks, but better to spawn_blocking if it takes time.
    // However, Tauri 2.0 async commands run on a thread pool, allowing blocking?
    // Ideally use spawn_blocking to be safe.

    // Use std thread or tokio spawn_blocking
    let result = tauri::async_runtime::spawn_blocking(move || {
        crate::service::native_overlay::capture::perform_capture(&state_arc)
    })
    .await
    .map_err(|e| e.to_string())?;

    match result {
        Ok((x, y, w, h)) => {
            // 3. UI Update on Main Thread
            let app_handle_ui = app_handle.clone();
            app_handle
                .run_on_main_thread(move || {
                    let state = app_handle_ui.state::<AppState>();
                    if let Ok(mut overlay) = state.overlay_manager.lock() {
                        if let Err(e) = overlay.show_overlay_at(x, y, w, h) {
                            eprintln!("Failed to show overlay: {:?}", e);
                        }
                    };
                })
                .map_err(|_| "Failed to run on main thread".to_string())?;

            Ok(())
        }
        Err(e) => Err(format!("Capture failed: {:?}", e)),
    }
}

#[tauri::command]
fn get_shortcuts(state: State<'_, AppState>) -> Vec<Shortcut> {
    let state = state.inner().shortcut_state.lock().unwrap();
    state.shortcuts.clone()
}

#[tauri::command]
fn update_shortcut(state: State<'_, AppState>, id: String, new_keys: String) -> Result<(), String> {
    let mut state = state.inner().shortcut_state.lock().unwrap();
    state.update_shortcut(&id, &new_keys)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .setup(|app| {
            let overlay_manager =
                OverlayManager::new(app.handle().clone()).expect("Failed to init OverlayManager");
            let shortcut_state = ShortcutState::new(app.handle());

            let app_state = AppState {
                overlay_manager: Mutex::new(overlay_manager),
                shortcut_state: Mutex::new(shortcut_state),
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
                let s_state = state.shortcut_state.lock().unwrap();
                s_state.last_registration_errors.clone()
            };
            if !errors.is_empty() {
                let _ = app.notification().builder()
                    .title("Shortcut Conflict")
                    .body(&format!("Failed to register:\n{}", errors.join("\n")))
                    .show();
            }

            // 3. Check Mixed DPI
            if let Ok(monitors) = crate::service::win32::monitor::enumerate_monitors() {
                if monitors.len() > 1 {
                    let first_dpi_x = monitors[0].dpi_x;
                    let mixed = monitors.iter().any(|m| m.dpi_x != first_dpi_x);
                    if mixed {
                        let _ = app.notification().builder()
                            .title("Mixed DPI Detected")
                            .body("Multi-monitor setup detection.\nNexSpot will rely on Windows scaling.")
                            .show();
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
                             if let Ok(s_state) = state.shortcut_state.lock() {
                                 for s in &s_state.shortcuts {
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
                             if id == "capture" {
                                 let app = app_handle_hotkey.clone();
                                 tauri::async_runtime::spawn(async move {
                                     log::info!("[Hotkey Debug] Spawned Async Task");
                                     let state = app.state::<AppState>();
                                     
                                     // Prevent Recursive
                                     {
                                         log::info!("[Hotkey Debug] Acquiring Lock Check...");
                                         if let Ok(overlay) = state.overlay_manager.lock() {
                                             log::info!("[Hotkey Debug] Lock Acquired. Checking visible...");
                                             if let Ok(os) = overlay.state.lock() {
                                                 if os.is_visible {
                                                     log::warn!("[Hotkey Debug] Overlay is visible, ignoring.");
                                                     return;
                                                 }
                                             }
                                         }
                                     }
                                     
                                     log::info!("[Hotkey Debug] Starting Capture Sequence");

                                     let state_arc = {
                                         let overlay = state.overlay_manager.lock().unwrap();
                                         overlay.state.clone()
                                     };
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
            start_capture,
            get_shortcuts,
            update_shortcut
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
