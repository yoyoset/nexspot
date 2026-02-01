use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter,
};

pub fn create_tray(app: &AppHandle) -> Result<(), String> {
    let capture_i = MenuItem::with_id(app, "capture", "Capture Now", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit_i =
        MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).map_err(|e| e.to_string())?;

    let menu =
        Menu::with_items(app, &[&capture_i, &settings_i, &quit_i]).map_err(|e| e.to_string())?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "capture" => {
                use crate::service::logger;
                logger::log("Tray", "Capture Now clicked");
                app.emit("hotkey-pressed", ()).unwrap();
            }
            "quit" => app.exit(0),
            "settings" => {
                use tauri::Manager;
                app.emit("open-settings", ()).ok();
                use tauri::webview::WebviewWindowBuilder;
                use tauri::WebviewUrl;
                if let Some(win) = app.get_webview_window("settings") {
                    win.show().ok();
                    win.set_focus().ok();
                } else {
                    let _ = WebviewWindowBuilder::new(
                        app,
                        "settings",
                        WebviewUrl::App("/settings".into()),
                    )
                    .title("HyperLens Settings")
                    .inner_size(600.0, 400.0)
                    .center()
                    .build();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                tray.app_handle().emit("hotkey-pressed", ()).unwrap();
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    Ok(())
}
