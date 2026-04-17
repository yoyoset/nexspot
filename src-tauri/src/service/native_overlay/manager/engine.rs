use super::OverlayManager;
use crate::service::native_overlay::render;
use crate::service::native_overlay::state;
use crate::service::win32;
use tauri::{AppHandle, Manager};

impl OverlayManager {
    pub fn init_vello_async(app_handle: AppHandle) {
        let state = app_handle.state::<crate::AppState>();
        let initial_hwnd = {
            if let Ok(mut mgr) = state.overlay_manager.lock().map_err(|e| e.into_inner()) {
                mgr.vello_status = state::VelloStatus::Pending;
                mgr.vello_hwnds.values().next().cloned()
            } else {
                return;
            }
        };

        tauri::async_runtime::spawn(async move {
            log::info!("[Advanced Mode] Starting VelloContext pre-warm task...");

            let init_future = render::vello_engine::VelloContext::new(initial_hwnd);
            let timeout_duration = std::time::Duration::from_secs(15);

            match tokio::time::timeout(timeout_duration, init_future).await {
                Ok(Ok(ctx)) => {
                    log::info!("[Advanced Mode] VelloContext pre-warmed successfully.");
                    if let Ok(mut mgr_lock) =
                        app_handle.state::<crate::AppState>().overlay_manager.lock()
                    {
                        mgr_lock.vello_ctx = Some(std::sync::Arc::new(ctx));
                        mgr_lock.vello_status = state::VelloStatus::Ready;

                        // INDUSTRIAL COMPOSITOR: Inject context into capture streams for zero-copy
                        if let Some(stream) = &mgr_lock.wgc_stream {
                            if let Ok(states) = stream.states.lock() {
                                for state_arc in states.values() {
                                    if let Ok(mut s) = state_arc.lock() {
                                        s.vello_ctx = mgr_lock.vello_ctx.clone();
                                    }
                                }
                            }
                        }

                        // Notify Frontend
                        use tauri::Emitter;
                        let _ = app_handle.emit("vello://ready", ());
                    }
                }
                Ok(Err(e)) => {
                    let technical_err = e.to_string();
                    use crate::service::l10n;
                    let friendly_err = if technical_err.contains("Adapter")
                        || technical_err.contains("Device")
                        || technical_err.contains("Queue")
                    {
                        l10n::t(&app_handle, "engine.hardware_init_failed", "Hardware acceleration init failed: {reason}").replace("{reason}", &technical_err)
                    } else {
                        l10n::t(&app_handle, "engine.start_failed", "Vello engine failed to start: {reason}").replace("{reason}", &technical_err)
                    };

                    log::error!("[Advanced Mode] Vello Error: {}", technical_err);
                    if let Ok(mut mgr_lock) =
                        app_handle.state::<crate::AppState>().overlay_manager.lock().map_err(|e| e.into_inner())
                    {
                        mgr_lock.vello_status = state::VelloStatus::Failed(friendly_err.clone());
                        use tauri::Emitter;
                        let _ = app_handle.emit("vello://error", friendly_err);
                    }
                }
                Err(_) => {
                    use crate::service::l10n;
                    let err_msg = l10n::t(&app_handle, "engine.start_timeout", "Vello engine start timeout (15s).");
                    log::error!("[Advanced Mode] {}", err_msg);
                    if let Ok(mut mgr_lock) =
                        app_handle.state::<crate::AppState>().overlay_manager.lock().map_err(|e| e.into_inner())
                    {
                        mgr_lock.vello_status = state::VelloStatus::Failed(err_msg.clone());
                        use tauri::Emitter;
                        let _ = app_handle.emit("vello://error", err_msg);
                    }
                }
            }
        });
    }

    pub fn start_pre_heat(&mut self) {
        if self.wgc_stream.is_none() {
            log::info!("Starting WGC Pre-heat Stream...");
            let mut stream = win32::wgc::capture::WgcStreamManager::new();
            if let Err(e) = stream.start() {
                log::error!("Failed to start pre-heat stream: {:?}", e);
            } else {
                self.wgc_stream = Some(stream);
            }
        }
    }

    pub fn stop_pre_heat(&mut self) {
        if let Some(stream) = self.wgc_stream.take() {
            log::info!("Stopping WGC Pre-heat Streams (All Monitors)...");
            if let Ok(states) = stream.states.lock() {
                for state_arc in states.values() {
                    if let Ok(mut s) = state_arc.lock() {
                        s.stop = true;
                    }
                }
            }
        }
    }
}
