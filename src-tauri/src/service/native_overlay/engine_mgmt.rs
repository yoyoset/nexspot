use crate::service::native_overlay::manager::OverlayManager;
use crate::service::native_overlay::render;
use crate::service::native_overlay::state;
use std::sync::Arc;
use tauri::Manager;

impl OverlayManager {
    pub fn upgrade_to_vello(&mut self) -> anyhow::Result<()> {
        let state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return Err(anyhow::anyhow!("State mutex poisoned")),
        };
        if state.capture_engine == state::CaptureEngine::Wgc {
            return Ok(());
        }

        // Show Loading UI
        // self.toolbar.is_loading = true; // Removed to prevent white bar artifact
        drop(state);
        let _ = self.render_frame();

        log::info!("Upgrading Rendering Engine: GDI -> Vello (via WGC) [Non-blocking]");

        // Get target monitor info from state
        let (
            target_index,
            target_name,
            target_friendly,
            target_rect,
            initial_hwnd,
            vello_ctx_already_exists,
        ) = {
            let s = self.state.lock().unwrap();
            let mut name = String::new();
            let mut friendly = String::new();
            let mut m_rect = None;
            let mut target_index = 0;

            // Find current monitor by coordinates
            if let Ok(monitors) = crate::service::win32::monitor::enumerate_monitors() {
                for (i, m) in monitors.into_iter().enumerate() {
                    if s.x >= m.rect.left
                        && s.x < m.rect.right
                        && s.y >= m.rect.top
                        && s.y < m.rect.bottom
                    {
                        name = m.name;
                        friendly = m.friendly_name;
                        m_rect = Some(m.rect);
                        target_index = i;
                        break;
                    }
                }
            }
            (
                target_index,
                name,
                friendly,
                m_rect,
                self.windows.first().cloned(),
                self.vello_ctx.is_some(),
            )
        };

        let app = self.app.clone();
        tauri::async_runtime::spawn(async move {
            let mut v_ctx = None;
            if !vello_ctx_already_exists {
                log::info!("[Advanced Mode] Initializing VelloContext in background...");
                match render::vello_engine::VelloContext::new(initial_hwnd).await {
                    Ok(ctx) => {
                        log::info!("[Advanced Mode] VelloContext successfully initialized.");
                        v_ctx = Some(Arc::new(ctx));
                    }
                    Err(e) => {
                        log::error!("[Advanced Mode] Failed to initialize Vello: {:?}", e);
                        let app_inner = app.clone();
                        let _ = app.run_on_main_thread(move || {
                            let app_state = app_inner.state::<crate::app_state::AppState>();
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

            // Capture initial frame
            log::info!("[Advanced Mode] Capturing initial WGC frame...");
            let bg_img = match crate::service::win32::wgc::capture::capture_monitor_to_vello(
                target_index,
                &target_name,
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
                let app_state = app_inner.state::<crate::app_state::AppState>();
                let lock_res = app_state.overlay_manager.lock();
                if let Ok(mut mgr) = lock_res {
                    if let Some(ctx) = v_ctx {
                        mgr.vello_ctx = Some(ctx);
                    }

                    {
                        if let Ok(mut s) = mgr.state.lock() {
                            if let Some(img) = bg_img {
                                s.vello.background = Some(img);
                            }
                            s.capture_engine = state::CaptureEngine::Wgc;

                            if let Some(rect) = target_rect {
                                s.x = rect.left;
                                s.y = rect.top;
                                s.width = rect.right - rect.left;
                                s.height = rect.bottom - rect.top;
                                s.restrict_to_monitor = Some(rect);
                            }
                        }
                    }

                    mgr.toolbar.is_loading = false;

                    // Rebuild toolbar for the new engine
                    let (mode, engine, registry) = match mgr.state.lock() {
                        Ok(s) => (s.capture_mode, s.capture_engine, s.tool_registry.clone()),
                        Err(_) => return,
                    };
                    mgr.toolbar
                        .rebuild_for_mode(&app_inner, mode, engine, &registry);

                    // Force Layout Update
                    // We need to re-layout the new buttons based on current selection
                    let (sel, w, h, enable_advanced) = if let Ok(s) = mgr.state.lock() {
                        (s.selection, s.width, s.height, s.enable_advanced_effects)
                    } else {
                        (None, 0, 0, false)
                    };

                    if let Some(selection) = sel {
                        let (wx, wy) = if let Ok(s) = mgr.state.lock() {
                            (s.x, s.y)
                        } else {
                            (0, 0)
                        };

                        mgr.toolbar.update_layout(selection, wx, wy, w, h, enable_advanced);
                        log::info!("[Advanced Mode Debug] Toolbar Layout Updated. Rect: {:?}, Buttons: {}, Loading: {}", 
                            mgr.toolbar.rect, mgr.toolbar.buttons.len(), mgr.toolbar.is_loading);
                        for (i, btn) in mgr.toolbar.buttons.iter().enumerate() {
                             log::info!("[Advanced Mode Debug] Button {}: {:?} Rect: {:?}", i, btn.tool, btn.rect);
                        }
                    } else {
                        // If no selection (unlikely in this context), hide toolbar
                        mgr.toolbar.hide();
                    }

                    if let (Some(rect), Some(initial_hwnd)) = (target_rect, initial_hwnd) {
                        let width = rect.right - rect.left;
                        let height = rect.bottom - rect.top;
                        unsafe {
                            let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                                initial_hwnd.0,
                                Some(windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST),
                                rect.left,
                                rect.top,
                                width,
                                height,
                                windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                            );
                        }
                    }

                    log::info!("[Advanced Mode] Promotion Complete. Triggering render.");
                    let _ = mgr.render_frame();
                }
            });
        });

        Ok(())
    }
}
