use crate::service::native_overlay::manager::OverlayManager;
use crate::service::native_overlay::render;
use crate::service::native_overlay::state;
use std::sync::Arc;
use tauri::Manager;

impl OverlayManager {
    pub fn upgrade_to_vello(&mut self) -> anyhow::Result<()> {
        let is_already_wgc = {
            let state = match self.state.read() {
                Ok(s) => s,
                Err(_) => return Err(anyhow::anyhow!("State lock poisoned")),
            };
            state.capture_engine == state::CaptureEngine::Wgc
        };
        
        if is_already_wgc {
            return Ok(());
        }

        // Show Loading UI
        let _ = self.render_frame();

        log::info!("Upgrading Rendering Engine: GDI -> Vello (via WGC) [Non-blocking]");

        // Get target monitor info from state
        let (
            _target_index,
            target_id,
            target_friendly,
            target_rect,
            initial_hwnd,
            vello_ctx_already_exists,
        ) = {
            let s = self.state.read().unwrap_or_else(|e| e.into_inner());
            let mut id = String::new();
            let mut friendly = String::new();
            let mut m_rect = None;
            let mut target_index = 0;

            // Find current monitor by state coordinates
            if let Ok(monitors) = crate::service::win32::monitor::enumerate_monitors() {
                for (i, m) in monitors.into_iter().enumerate() {
                    // In Brd-Scheme, state.x/y are absolute
                    if s.capture_x >= m.rect.left
                        && s.capture_x < m.rect.right
                        && s.capture_y >= m.rect.top
                        && s.capture_y < m.rect.bottom
                    {
                        id = m.hmonitor.to_string();
                        friendly = m.friendly_name;
                        m_rect = Some(m.rect);
                        target_index = i;
                        break;
                    }
                }
            }

            // Provide any available vello hwnd for context probing (primary is best fallback)
            let mut first_vello_hwnd = None;
            for h in self.vello_hwnds.values() {
               first_vello_hwnd = Some(h.clone());
               break; 
            }

            (
                target_index,
                id,
                friendly,
                m_rect,
                first_vello_hwnd,
                self.vello_ctx.is_some(),
            )
        };

        let app = self.app.clone();
        tauri::async_runtime::spawn(async move {
            let mut v_ctx = None;
            if !vello_ctx_already_exists {
                log::info!("[Advanced Mode] Initializing VelloContext (B-Scheme) pool...");
                match render::vello_engine::VelloContext::new(initial_hwnd).await {
                    Ok(ctx) => {
                        log::info!("[Advanced Mode] VelloContext successfully initialized.");
                        v_ctx = Some(Arc::new(ctx));
                    }
                    Err(e) => {
                        log::error!("[Advanced Mode] Failed to initialize Vello: {:?}", e);
                        let app_inner = app.clone();
                        let _ = app.run_on_main_thread(move || {
                            let app_state = app_inner.state::<crate::AppState>();
                            let lock_res = app_state.overlay_manager.lock();
                            if let Ok(mut mgr) = lock_res {
                                mgr.toolbar.is_loading = false;
                                let _ = mgr.render_frame();
                            }
                        });
                        return;
                    }
                }
            }

            // Capture initial frame for the SPECIFIC monitor
            log::info!("[Advanced Mode] Capturing initial WGC frame for monitor {}...", target_id);
            let bg_img = match crate::service::win32::wgc::capture::capture_monitor_to_vello(
                &target_id,
                &target_friendly,
                target_rect,
            ) {
                Ok((img, _size)) => Some(img),
                Err(e) => {
                    log::warn!("[Advanced Mode] WGC Initial Capture failed: {:?}", e);
                    None
                }
            };

            // Finalize on Main Thread
            let app_inner = app.clone();
            let _ = app.run_on_main_thread(move || {
                let app_state = app_inner.state::<crate::AppState>();
                let lock_res = app_state.overlay_manager.lock();
                if let Ok(mut mgr) = lock_res {
                    if let Some(ctx) = v_ctx {
                        mgr.vello_ctx = Some(ctx);
                    }

                    {
                        if let Ok(mut s) = mgr.state.write() {
                            if let Some(img) = bg_img {
                                s.vello.background = Some(img);
                            }
                            s.capture_engine = state::CaptureEngine::Wgc;
                            s.monitor_id = target_id.clone();

                            if let Some(rect) = target_rect {
                                s.capture_x = rect.left;
                                s.capture_y = rect.top;
                                s.width = rect.right - rect.left;
                                s.height = rect.bottom - rect.top;
                                s.monitor_rect = rect;
                                s.restrict_to_monitor = Some(rect);
                            }
                        }
                    }

                    mgr.toolbar.is_loading = false;

                    // Rebuild toolbar for the new engine
                    let (mode, engine, registry) = {
                        let s = mgr.state.read().unwrap_or_else(|e| e.into_inner());
                        (s.capture_mode, s.capture_engine, s.tool_registry.clone())
                    };
                    mgr.toolbar
                        .rebuild_for_mode(&app_inner, mode, engine, &registry);

                    // Force Layout Update
                    let (sel, w, h, enable_advanced) = {
                        let s = mgr.state.read().unwrap_or_else(|e| e.into_inner());
                        (s.selection, s.width, s.height, s.enable_advanced_effects)
                    };

                    if let Some(selection) = sel {
                        let (mx, my) = {
                            let s = mgr.state.read().unwrap_or_else(|e| e.into_inner());
                            (s.monitor_rect.left, s.monitor_rect.top)
                        };
                        mgr.toolbar.update_layout(selection, mx, my, w, h, enable_advanced, engine);
                    } else {
                        mgr.toolbar.hide();
                    }

                    let _ = mgr.render_frame();
                }
            });
        });

        Ok(())
    }
}
