pub mod app_state;
pub mod commands;
pub mod service;

use crate::service::config::ConfigState;
use crate::service::logger::{self, LoggerState};
use crate::service::native_overlay::OverlayManager;
use crate::service::pin::PinState;
pub use app_state::AppState;
use std::sync::{Mutex, RwLock, Arc};
#[cfg(target_os = "windows")]
use tauri::{Emitter, Manager};

pub fn run() {
    // Capture panic location to a file even when the panic is non-unwinding (renderer
    // callbacks abort before stderr is readable). Written to %TEMP%/nexspot_panic.log.
    {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let msg = format!(
                "[{:?}] PANIC: {}\n",
                std::time::SystemTime::now(),
                info
            );
            let path = std::env::temp_dir().join("nexspot_panic.log");
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                let _ = f.write_all(msg.as_bytes());
            }
            eprintln!("{}", msg);
            default_hook(info);
        }));
    }

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
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::ThemeChanged(theme) = event {
                let state = window.state::<AppState>();
                let config = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
                if config.config.theme == "system" {
                    #[cfg(target_os = "windows")]
                    {
                        if let Ok(hwnd) = window.hwnd() {
                            let is_dark = *theme == tauri::Theme::Dark;
                            let _ = crate::service::win32::window::apply_theme(
                                windows::Win32::Foundation::HWND(hwnd.0 as *mut _),
                                is_dark,
                            );
                        }
                    }
                }
            }
        })
        .setup(|app| {
            let logger_state = LoggerState::new(app.handle());
            let mut config_state = ConfigState::new(app.handle());
            
            // Initialize Activity Feed Persistence
            let app_data_dir = app.path().app_data_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
            let activity_state = crate::service::activity::ActivityState::new(app_data_dir);

            let font_family = config_state.config.font_family.clone();
            let vello_enabled = config_state.config.vello_enabled;
            let overlay_manager = OverlayManager::new(
                app.handle().clone(),
                font_family,
                vello_enabled,
            )
            .expect("Failed to init OverlayManager");
            let hotkey_map = config_state.register_all();

            let _ = logger_state.init();
            log::info!("==== Nexspot Startup (v0.2.1) ====");
            log::info!("GPU: RTX 3060 detected? Check Vello status...");

            let app_state = AppState {
                overlay_manager: Arc::new(Mutex::new(overlay_manager)),
                config_state: Mutex::new(config_state),
                activity_state,
                logger_state,
                hotkey_map: RwLock::new(hotkey_map),
            };

            app.manage(app_state);
            app.manage(PinState::new());

            // Now set the user data pointer for the Win32 window
            let state = app.state::<AppState>();
            {
                let mut overlay = state.overlay_manager.lock().unwrap_or_else(|e| e.into_inner());
                let ptr = &mut *overlay as *mut OverlayManager;
                overlay.set_user_data(ptr);
            }

            // --- System Tray Setup ---
            crate::service::tray::setup_tray(app)?;

            // 1. Show Main Window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();

                // Force Adaptive Title Bar on Windows
                #[cfg(target_os = "windows")]
                {
                    if let Ok(hwnd) = window.hwnd() {
                        let config = app.state::<crate::AppState>();
                        let config_guard = config.config_state.lock().unwrap_or_else(|e| e.into_inner());
                        let theme = config_guard.config.theme.clone();

                        let is_dark = if theme == "system" {
                            window
                                .theme()
                                .map(|t| t == tauri::Theme::Dark)
                                .unwrap_or(true)
                        } else {
                            theme == "dark"
                        };

                        let _ = crate::service::win32::window::apply_theme(
                            windows::Win32::Foundation::HWND(hwnd.0 as *mut _),
                            is_dark,
                        );
                    }
                }
            }

            // 2. Check Shortcuts Conflicts
            let app_handle = app.handle();
            let state = app_handle.state::<crate::AppState>();
            let errors = {
                let c_state = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
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
            crate::service::hotkey::spawn_hotkey_listener(app.handle().clone());

            // 6. Start Vello Pre-heat (if enabled)
            if vello_enabled {
                OverlayManager::init_vello_async(app.handle().clone());
            }

            // 7. PaddleOCR 预热（引擎为 paddle 且组件已装时后台拉起，点击即识别）
            crate::service::paddle_ocr::warmup(app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_capture,
            commands::trigger_capture,
            crate::service::ocr::execute_ocr,
            crate::service::ocr::get_ocr_languages,
            crate::service::paddle_ocr::get_paddle_status,
            crate::service::paddle_ocr::check_paddle_update,
            crate::service::paddle_ocr::download_paddle_component,
            crate::service::ocr::get_last_ocr_result,
            crate::service::native_overlay::scrolling::start_scrolling,
            crate::service::native_overlay::scrolling::stop_scrolling,
            crate::service::native_overlay::scrolling::get_last_scrolled_image,
            crate::service::native_overlay::scrolling::save_scrolled_image_to,
            crate::service::native_overlay::scrolling::copy_image_to_clipboard,
            commands::get_config,
            commands::get_startup_errors,
            commands::set_save_path,
            commands::set_font_family,
            commands::set_vello_enabled,
            commands::set_vello_advanced_effects,
            commands::config::set_vello_aesthetic_style,
            commands::set_theme,
            commands::set_accent_color,
            commands::set_ocr_language,
            commands::set_ocr_engine,
            commands::set_paddle_language,
            commands::set_jpg_quality,
            commands::set_concurrency,
            commands::set_default_export_format,
            commands::set_language,
            commands::set_quick_save,
            commands::set_snapshot_enabled,
            commands::set_snapshot_size,
            commands::set_selection_engine,
            commands::set_snapshot_engine,
            commands::config::emergency_reset_to_gdi,
            commands::add_workflow,
            commands::remove_workflow,
            commands::update_workflow,
            commands::get_vello_status,
            commands::config::suspend_hotkeys,
            commands::config::resume_hotkeys,
            commands::config::refresh_hotkeys,
            commands::select_folder,
            commands::open_folder,
            commands::pin::create_text_pin,
            commands::pin::get_all_pins,
            commands::pin::remove_pin,
            commands::pin::clear_all_pins,
            commands::pin::toggle_pin_always_on_top,
            commands::pin::is_pin_always_on_top,
            commands::pin::set_pin_window_size,
            commands::pin::set_pin_min_size,
            commands::pin::save_pin_as,
            crate::service::activity::get_activity,
            logger::clear_logs,
            logger::reveal_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
