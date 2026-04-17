use crate::app_state::AppState;
use crate::service::config::types::{CaptureAction, CaptureWorkflow};
use crate::service::native_overlay::state::{CaptureEngine, CaptureMode};
use tauri::Manager;

pub async fn execute_capture_workflow<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    workflow: CaptureWorkflow,
) {
    let workflow_id = workflow.id.clone();

    // 0. Global Check: Prevent overlapping triggers
    {
        let app_state = app.state::<AppState>();
        let manager = app_state.overlay_manager.lock().unwrap_or_else(|e| e.into_inner());

        let mut os = manager.state.write().unwrap_or_else(|e| e.into_inner());
        if os.is_visible || os.is_capturing {
            log::info!(
                "[Workflow] Overlay active or capturing, ignoring workflow {}.",
                workflow_id
            );
            return;
        }
        os.is_capturing = true; // Mark as capturing
    };

    log::info!("[Workflow] Executing: {}", workflow_id);

    // 1. Check if already visible & HIDE before capture to avoid double-dimming
    {
        if let Ok(mut overlay) = app.state::<AppState>().overlay_manager.lock() {
            overlay.close_and_reset();
        }
    }
    // Small sleep to ensure OS window state sync AFTER releasing lock
    std::thread::sleep(std::time::Duration::from_millis(10));

    // 2. Decode Capture Action & Engine
    let (target_mode, target_engine, snapshot_dim) = match &workflow.action {
        CaptureAction::Selection { engine } => (CaptureMode::Standard, engine.as_str(), None),
        CaptureAction::Snapshot {
            engine,
            width,
            height,
            allow_resize,
            ..
        } => (
            CaptureMode::Snapshot {
                allow_resize: *allow_resize,
            },
            engine.as_str(),
            Some((*width, *height)),
        ),
        CaptureAction::Fullscreen { engine } => (CaptureMode::Standard, engine.as_str(), None),
        CaptureAction::Window { engine } => (CaptureMode::FixedWindow, engine.as_str(), None),
    };

    let use_snapshot_mode = snapshot_dim.is_some();
    let engine_enum = if target_engine == "vello" {
        let (v_ready, v_err) = if let Ok(mgr) = app.state::<AppState>().overlay_manager.lock().map_err(|e| e.into_inner()) {
            match &mgr.vello_status {
                crate::service::native_overlay::state::VelloStatus::Ready => (true, None),
                crate::service::native_overlay::state::VelloStatus::Failed(e) => {
                    (false, Some(e.clone()))
                }
                crate::service::native_overlay::state::VelloStatus::Pending => {
                    (false, Some("Vello engine is initializing...".to_string()))
                }
            }
        } else {
            (false, Some("System lock failed".to_string()))
        };

        if !v_ready {
            let err_msg = v_err.unwrap_or_else(|| "Vello engine not ready".to_string());
            log::warn!("[Workflow] Blocked Vello capture: {}", err_msg);

            // ERROR: Reset capturing flag before returning
            {
                if let Ok(manager) = app.state::<AppState>().overlay_manager.lock() {
                    if let Ok(mut os) = manager.state.write() {
                        os.is_capturing = false;
                    }
                }
            }

            use tauri::Emitter;
            let _ = app.emit("vello://error", err_msg);
            return;
        }
        CaptureEngine::Wgc
    } else {
        CaptureEngine::Gdi
    };

    // 3. Prepare Overlay State
    let (state_arc_for_capture, wgc_state) = {
        let state = app.state::<AppState>();
        let overlay = state.overlay_manager.lock().unwrap_or_else(|e| e.into_inner());
        let mut os = overlay.state.write().unwrap_or_else(|e| e.into_inner());

        os.active_workflow = Some(workflow.clone());
        os.capture_mode = target_mode;
        os.capture_engine = engine_enum;
        os.is_snapshot_mode = use_snapshot_mode;

        (
            overlay.state.clone(),
            overlay.wgc_stream.as_ref().map(|s| s.get_states()),
        )
    };

    // 4. Run Capture (Blocking, CPU heavy) on thread pool
    let res = tauri::async_runtime::spawn_blocking(move || {
        crate::service::native_overlay::capture::perform_capture(&state_arc_for_capture, wgc_state)
    })
    .await;

    // 5. Handle Result
    if let Ok(Ok((x, y, w, h))) = res {
        let app_ui = app.clone();
        let _ = app.run_on_main_thread(move || {
            let app_state = app_ui.state::<AppState>();
            if let Ok(mut overlay) = app_state.overlay_manager.lock().map_err(|e| e.into_inner()) {
                // Get necessary state via read lock
                let (mouse_x, mouse_y, window_rects) = {
                    let s = overlay.state.read().unwrap_or_else(|e| e.into_inner());
                    (s.mouse_x, s.mouse_y, s.window_rects.clone())
                };

                let config_action = workflow.action.clone();
                let handler: Box<dyn crate::service::native_overlay::modes::CaptureModeHandler> =
                    match config_action {
                        CaptureAction::Selection { .. } => Box::new(
                            crate::service::native_overlay::modes::selection::SelectionHandler,
                        ),
                        CaptureAction::Snapshot { width, height, .. } => Box::new(
                            crate::service::native_overlay::modes::snapshot::SnapshotHandler {
                                width,
                                height,
                            },
                        ),
                        CaptureAction::Fullscreen { .. } => Box::new(
                            crate::service::native_overlay::modes::fullscreen::FullscreenHandler,
                        ),
                        CaptureAction::Window { .. } => {
                            Box::new(crate::service::native_overlay::modes::window::WindowHandler)
                        }
                    };

                let selection =
                    handler.prepare_selection(x, y, w, h, mouse_x, mouse_y, &window_rects);

                {
                    let mut s = overlay.state.write().unwrap_or_else(|e| e.into_inner());
                    s.selection = selection;
                }

                if selection.is_some() {
                    log::info!(
                        "[Workflow] Selection applied via mode handler for {:?}",
                        workflow.action
                    );
                }
                if let Err(e) = overlay.show_overlay_at(x, y, w, h) {
                    log::error!("[Workflow] UI display failed: {:?}", e);
                    overlay.close_and_reset(); // Reset state on failure
                }

                // FINISHED: Clear capturing flag ALWAYS
                if let Ok(mut os) = overlay.state.write() {
                    os.is_capturing = false;
                }
            };
        });
    } else {
        // ERROR or CANCELLED: Clear capturing flag
        if let Some(state) = app.try_state::<AppState>() {
            if let Ok(manager) = state.overlay_manager.lock().map_err(|e| e.into_inner()) {
                if let Ok(mut os) = manager.state.write().map_err(|e| e.into_inner()) {
                    os.is_capturing = false;
                }
            }
        }
        if let Err(e) = res {
            log::error!("[Workflow] Capture task failed: {:?}", e);
        } else if let Ok(Err(e)) = res {
            log::error!("[Workflow] Capture execution failed: {:?}", e);
        }
    }
}
