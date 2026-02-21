use crate::service::l10n::{self, L10nKey};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, Emitter, Manager, Runtime};

pub fn setup_tray<R: Runtime>(app: &App<R>) -> Result<(), tauri::Error> {
    let quit_i = MenuItem::with_id(
        app,
        "quit",
        l10n::t(app.handle(), L10nKey::TrayExit),
        true,
        None::<&str>,
    )?;
    let settings_i = MenuItem::with_id(
        app,
        "settings",
        l10n::t(app.handle(), L10nKey::TraySettings),
        true,
        None::<&str>,
    )?;
    let dashboard_i = MenuItem::with_id(
        app,
        "dashboard",
        l10n::t(app.handle(), L10nKey::TrayDashboard),
        true,
        None::<&str>,
    )?;
    let capture_i = MenuItem::with_id(
        app,
        "capture",
        l10n::t(app.handle(), L10nKey::TrayCapture),
        true,
        None::<&str>,
    )?;

    let menu = Menu::with_items(
        app,
        &[
            &dashboard_i,
            &capture_i,
            &settings_i,
            &PredefinedMenuItem::separator(app)?,
            &quit_i,
        ],
    )?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => app.exit(0),
                "dashboard" | "settings" => {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                        let _ = win.set_focus();
                        if event.id.as_ref() == "settings" {
                            let _ = win.emit("open-settings", ());
                        }
                    }
                }
                "capture" => {
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        // Use unified workflow execution with a default/synthetic workflow
                        // For tray, we typically want standard selection capture
                        use crate::service::config::types::{
                            CaptureAction, CaptureOutput, CaptureWorkflow,
                        };

                        let workflow = CaptureWorkflow {
                            id: "tray_manual".to_string(),
                            label: "Manual Capture".to_string(),
                            shortcut: "".to_string(),
                            enabled: true,
                            is_system: true,
                            action: CaptureAction::Selection {
                                engine: "gdi".to_string(),
                            }, // Default to GDI for compatibility
                            output: CaptureOutput {
                                save_to_file: false,
                                save_to_clipboard: true,
                                target_folder: None,
                                naming_template: "capture_%Y%m%d_%H%M%S".to_string(),
                                format: "png".to_string(),
                            },
                        };
                        crate::service::workflow::execute_capture_workflow(
                            app_handle, workflow, None,
                        )
                        .await;
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

    Ok(())
}
