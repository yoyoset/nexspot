use super::OverlayManager;
use crate::service::native_overlay::render;
use crate::service::native_overlay::state;
use crate::service::win32::window::SafeHWND;
use tauri::Manager;

impl OverlayManager {
    pub fn render_frame(&mut self) -> anyhow::Result<()> {
        let (engine, monitor_id, is_visible) = {
            let s = self.state.read().unwrap_or_else(|e| e.into_inner());
            (s.capture_engine, s.monitor_id.clone(), s.is_visible)
        };

        if !is_visible {
            return Ok(());
        }

        // --- VELLO PERFORMANCE: Pre-build scene once per logical frame ---
        // Industrial Standard: Lock the scene ONCE to prevent per-monitor deadlock
        let vello_scene_guard = if engine == state::CaptureEngine::Wgc {
            if let Some(ctx) = &self.vello_ctx {
                if let Ok(mut scene) = ctx.scene.lock() {
                    let state = self.state.read().unwrap_or_else(|e| e.into_inner());
                    render::vello_engine::renderer::render_state_to_scene(&state, Some(&self.toolbar), ctx, &mut scene);
                    Some(scene)
                } else { None }
            } else { None }
        } else { None };

        // SOVEREIGN DISPATCH: Mixed-Monitor Logic
        // OPTIMIZATION: Use cached monitors from state to avoid high-frequency enumerate_monitors()
        let monitors = {
            let state = self.state.read().unwrap_or_else(|e| e.into_inner());
            if !state.monitors.is_empty() {
                state.monitors.clone()
            } else {
                drop(state);
                let monitors = crate::service::win32::monitor::enumerate_monitors()?;
                if let Ok(mut state) = self.state.write() {
                    state.monitors = monitors.clone();
                }
                monitors
            }
        };

        let gdi_hwnds = &self.gdi_hwnds;
        let vello_hwnds = &self.vello_hwnds;
        let vello_ctx_opt = &self.vello_ctx;

        let restrict_rect = {
            let s = self.state.read().unwrap_or_else(|e| e.into_inner());
            s.restrict_to_monitor.clone()
        };

        // Since we need to mutate MonitorRenderContext, we iterate over them.
        for monitor in &monitors {
            let m_id = monitor.hmonitor.to_string();
            let ctx = self.monitor_contexts.entry(m_id.clone()).or_insert_with(super::MonitorRenderContext::default);
            
            // INDUSTRIAL ISOLATION:
            // 1. WGC natively uses single-monitor isolation.
            // 2. GDI uses it if `restrict_to_monitor` is active (Mixed DPI fallback).
            let is_isolated = engine == state::CaptureEngine::Wgc || restrict_rect.is_some();
            if is_isolated && m_id != monitor_id {
                continue;
            }

            let hwnd_opt = if engine == state::CaptureEngine::Gdi {
                gdi_hwnds.get(&m_id).map(|send| SafeHWND(send.0))
            } else {
                vello_hwnds.get(&m_id).map(|send| SafeHWND(send.0))
            };

            if let Some(hwnd) = hwnd_opt {
                // SEQUENTIAL SAFETY: Pass mutable state directly
                let mut state = self.state.write().unwrap_or_else(|e| e.into_inner());
                
                // Pre-sync toolbar layout for the global frame (only needed once, but safe here)
                if engine == state::CaptureEngine::Gdi {
                    let sel = state.selection.unwrap_or(state.monitor_rect);
                    self.toolbar.update_layout(
                        sel,
                        state.monitor_rect.left,
                        state.monitor_rect.top,
                        state.width,
                        state.height,
                        state.enable_advanced_effects,
                        state.capture_engine,
                    );
                }

                let _ = render::render_frame(
                    &hwnd,
                    &self.app,
                    &mut state,
                    ctx,
                    &mut self.toolbar,
                    vello_ctx_opt,
                    &m_id,
                    monitor.rect,
                    vello_scene_guard.as_deref(),
                );
            }
        }

        Ok(())
    }

    pub fn show_overlay_at(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> anyhow::Result<()> {
        // 0. Find the monitor this area belongs to (B-Scheme Routing)
        let monitors = crate::service::win32::monitor::enumerate_monitors()?;
        let target_monitor = monitors.iter().find(|m| {
            // Precise hit: the top-left (x, y) should be within the monitor bounds
            x >= m.rect.left && x < m.rect.right && y >= m.rect.top && y < m.rect.bottom
        }).or_else(|| monitors.first()) // Fallback to primary if out of bounds
          .ok_or_else(|| anyhow::anyhow!("No monitors detected"))?;
        
        let monitor_id = target_monitor.hmonitor.to_string();

        let advanced_effects = self.app.state::<crate::AppState>()
            .config_state.lock().unwrap_or_else(|e| e.into_inner())
            .config.vello_advanced_effects;

        // Sync config and monitor context to state
        {
            if let Ok(mut state) = self.state.write() {
                state.enable_advanced_effects = advanced_effects;
                state.monitor_id = monitor_id.clone();
                state.monitor_rect = target_monitor.rect;
            }
        }

        let (mode, engine, registry) = {
            let mut state = match self.state.write() {
                Ok(s) => s,
                Err(_) => return Ok(()),
            };

            let engine = state.capture_engine;

            // ENGINE ISOLATION: Check health before proceeding
            if engine == state::CaptureEngine::Wgc && self.vello_status != state::VelloStatus::Ready
            {
                let err_msg = match &self.vello_status {
                    state::VelloStatus::Failed(e) => format!("Vello Error: {}", e),
                    _ => crate::service::l10n::t(&self.app, "backend.engine.switching_to_advanced", "Vello engine is initializing..."),
                };
                log::warn!("Blocking Vello overlay attempt: {}", err_msg);
                use tauri::Emitter;
                let _ = self.app.emit("vello://error", err_msg);
                return Err(anyhow::anyhow!("Vello engine is not ready"));
            }

            // MANDATORY CLEANUP: Prevent backgrounds from mixing
            match engine {
                state::CaptureEngine::Gdi => {
                    state.vello.background = None;
                }
                state::CaptureEngine::Wgc => {
                    state.gdi.hbitmap_bright = None;
                    state.gdi.hbitmap_dim = None;
                }
            }

            (state.capture_mode, engine, state.tool_registry.clone())
        };

        self.toolbar
            .rebuild_for_mode(&self.app, mode, engine, &registry);

        {
            if let Ok(mut state) = self.state.write() {
                state.is_visible = true;
                // --- CRITICAL: Update state geometry for local normalization ---
                state.capture_x = x;
                state.capture_y = y;
                state.width = width;
                state.height = height;

                let sel = state.selection.unwrap_or(state.monitor_rect);
                self.toolbar.update_layout(
                    sel,
                    state.monitor_rect.left,
                    state.monitor_rect.top,
                    width,
                    height,
                    state.enable_advanced_effects,
                    state.capture_engine,
                );
            }
        }

        let is_isolated = {
            let s = self.state.read().unwrap_or_else(|e| e.into_inner());
            engine == state::CaptureEngine::Wgc || s.restrict_to_monitor.is_some()
        };

        // 2. Activate Target Windows (Sovereign Engine Model)
        // Global GDI covers all monitors, but isolated instances are strictly constrained to the target.
        if !is_isolated {
            self.render_frame()?;
            
            for hwnd_send in self.gdi_hwnds.values() {
                let h = SafeHWND(hwnd_send.0);
                unsafe {
                    let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(h.0, windows::Win32::UI::WindowsAndMessaging::SW_SHOWNOACTIVATE);
                    let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                        h.0, Some(windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST),
                        0, 0, 0, 0,
                        windows::Win32::UI::WindowsAndMessaging::SWP_NOMOVE | windows::Win32::UI::WindowsAndMessaging::SWP_NOSIZE | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                    );
                }
            }
        } else {
            // ISOLATED MODE (WGC or Restricted GDI)
            let h = self.active_hwnd(engine, &monitor_id)?;
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(h.0, windows::Win32::UI::WindowsAndMessaging::SW_SHOWNOACTIVATE);
                let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    h.0, Some(windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST),
                    target_monitor.rect.left, target_monitor.rect.top, 
                    target_monitor.rect.right - target_monitor.rect.left,
                    target_monitor.rect.bottom - target_monitor.rect.top,
                    windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                );
            }
        }

        // 4. Force a secondary render to ensure toolbar and monitors are synced correctly
        self.render_frame()?;

        let active_hwnd = self.active_hwnd(engine, &monitor_id)?;
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                active_hwnd.0,
                windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
            );
            let _ = windows::Win32::UI::WindowsAndMessaging::SetTimer(Some(active_hwnd.0), 1, 500, None);
            let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(active_hwnd.0);
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(active_hwnd.0));
        }
        Ok(())
    }

    pub fn save_selection(&mut self) -> anyhow::Result<()> {
        crate::service::native_overlay::save::save_selection(&self.state, &self.app)?;
        self.close_and_reset();
        Ok(())
    }

    pub fn save_clipboard(&mut self) -> anyhow::Result<()> {
        crate::service::native_overlay::save::copy_to_clipboard(&self.state, &self.app)?;
        self.close_and_reset();
        Ok(())
    }

    pub fn save_as(&mut self) -> anyhow::Result<()> {
        let app = self.app.clone();
        let state_arc = self.state.clone();

        let default_format = app.state::<crate::AppState>()
            .config_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .config
            .default_export_format
            .clone();

        use crate::service::native_overlay::save::save_selection_to_path;
        use tauri_plugin_dialog::DialogExt;
        app.dialog()
            .file()
            .add_filter("PNG Image", &["png"])
            .add_filter("JPEG Image", &["jpg", "jpeg"])
            .set_file_name(format!("capture.{}", default_format))
            .save_file(move |path| {
                if let Some(p) = path {
                    let path_str = p.to_string();
                    let _ = save_selection_to_path(
                        state_arc.clone(), &app, path_str,
                    );
                }
            });

        // We don't automatically close here to allow user cancellation
        // However, if they successfuly saved, the notify callback will run.
        // Usually, 'Save As' should also close the session on success.
        // But tauri's save_file is async-callback based.
        // I'll add a close inside the callback if needed, but for now let's keep it simple.

        Ok(())
    }
}
