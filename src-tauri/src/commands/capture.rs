use crate::app_state::AppState;
use tauri::{Manager, State};

#[tauri::command]
pub async fn start_capture(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
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
