use crate::app_state::AppState;
use tauri::Manager;

pub fn spawn_hotkey_listener<R: tauri::Runtime>(app: tauri::AppHandle<R>) {
    std::thread::spawn(move || {
        let receiver = global_hotkey::GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            // Trigger on Pressed for immediate feedback
            if event.state == global_hotkey::HotKeyState::Pressed {
                let state = app.state::<AppState>();

                // Fast Lookup without locking the heavy ConfigState
                let action = if let Ok(map) = state.hotkey_map.read() {
                    map.get(&event.id).cloned()
                } else {
                    None
                };

                if let Some(action) = action {
                    log::info!("[Hotkey] Matched Action: {:?}", action);
                    let app_handle = app.clone();

                    match action {
                        crate::service::config::types::HotkeyAction::Workflow(workflow) => {
                            tauri::async_runtime::spawn(async move {
                                crate::service::workflow::execute_capture_workflow(
                                    app_handle, workflow, None,
                                )
                                .await;
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    });
}
