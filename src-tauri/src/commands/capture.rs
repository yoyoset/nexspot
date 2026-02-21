use crate::app_state::AppState;
use crate::service::config::types::{CaptureAction, CaptureOutput, CaptureWorkflow};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn start_capture(app: AppHandle) -> Result<(), String> {
    log::debug!("Native Capture Command Received");

    // Synthesize a generic "manual" workflow for the legacy command
    let workflow = CaptureWorkflow {
        id: "manual_capture".to_string(),
        label: "Manual Capture".to_string(),
        shortcut: "".to_string(),
        enabled: true,
        is_system: true,
        action: CaptureAction::Selection {
            engine: "gdi".to_string(),
        },
        output: CaptureOutput {
            save_to_file: false,
            save_to_clipboard: true,
            target_folder: None,
            naming_template: "capture_%Y%m%d_%H%M%S".to_string(),
            format: "png".to_string(),
        },
    };

    crate::service::workflow::execute_capture_workflow(app, workflow, None).await;
    Ok(())
}

#[tauri::command]
pub async fn trigger_capture(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    action: String,
) -> Result<(), String> {
    log::debug!("Trigger Capture Command Received: {}", action);

    // 1. Find target workflow by ID
    let workflow = {
        let config_guard = state.config_state.lock().map_err(|e| e.to_string())?;

        // Try to find matching ID
        if let Some(w) = config_guard
            .config
            .workflows
            .iter()
            .find(|w| w.id == action)
        {
            w.clone()
        } else {
            // Fallback for legacy hardcoded names
            let (target_id, workflow_action) = match action.as_str() {
                "selection" => (
                    "capture_default",
                    CaptureAction::Selection {
                        engine: config_guard.config.selection_engine.clone(),
                    },
                ),
                "snapshot" => (
                    "snapshot_default",
                    CaptureAction::Snapshot {
                        engine: config_guard.config.snapshot_engine.clone(),
                        width: config_guard.config.snapshot_width,
                        height: config_guard.config.snapshot_height,
                        allow_resize: true,
                    },
                ),
                _ => return Err(format!("Workflow not found: {}", action)),
            };

            CaptureWorkflow {
                id: format!("legacy_{}", target_id),
                label: format!("Legacy {} Capture", target_id),
                shortcut: "".to_string(),
                enabled: true,
                is_system: true,
                action: workflow_action,
                output: CaptureOutput {
                    save_to_file: false,
                    save_to_clipboard: true,
                    target_folder: None,
                    naming_template: "capture_%Y%m%d_%H%M%S".to_string(),
                    format: "png".to_string(),
                },
            }
        }
    };

    // 2. Execute
    crate::service::workflow::execute_capture_workflow(app, workflow, None).await;
    Ok(())
}

#[tauri::command]
pub fn is_vello_ready(state: State<'_, AppState>) -> bool {
    if let Ok(mgr) = state.overlay_manager.lock() {
        mgr.vello_ctx.is_some()
    } else {
        false
    }
}
