use crate::service::native_overlay::render;
use crate::service::native_overlay::render::toolbar;
use crate::service::native_overlay::state::{self, OverlayState};
use crate::service::win32::window::SafeHWND;
use crate::service::win32::{self, SendHWND};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Manager};

pub struct OverlayManager {
    pub state: Arc<Mutex<OverlayState>>,
    pub windows: Vec<SendHWND>,
    pub toolbar: toolbar::Toolbar,
    pub last_render_time: Instant,
    pub vello_ctx: Option<Arc<render::vello_engine::VelloContext>>,
    pub wgc_stream: Option<win32::wgc::capture::WgcStreamManager>,
    pub app: AppHandle,
}

impl OverlayManager {
    pub fn new(app: AppHandle, font_family: String, vello_enabled: bool) -> anyhow::Result<Self> {
        let state = Arc::new(Mutex::new(OverlayState::default()));
        {
            let mut s = state.lock().unwrap();
            s.font_family = font_family;
            if vello_enabled {
                s.capture_engine = state::CaptureEngine::Wgc;
            }
        }

        // Create MAIN overlay window immediately (hidden)
        let main_hwnd =
            win32::window::create_overlay_window("HyperLensOverlay", "HyperLens Overlay")?;

        // 1. Register Custom Font from memory (Embedded)
        static FONT_DATA: &[u8] = include_bytes!("../../../resources/remixicon.ttf");
        match win32::gdi::add_font_mem(FONT_DATA) {
            Ok(_) => log::info!("Registered remixicon font from embedded memory"),
            Err(e) => {
                log::error!("Failed to register embedded remixicon font: {:?}", e);
                // Fallback to file system if embedding fails (unlikely, but safe)
                if let Ok(resource_dir) = app.path().resource_dir() {
                    let font_path = resource_dir.join("remixicon.ttf");
                    if font_path.exists() {
                        let _ = win32::gdi::register_font(&font_path);
                    }
                }
            }
        }

        let mut mgr = Self {
            state,
            windows: vec![win32::SendHWND(main_hwnd.0)],
            toolbar: toolbar::Toolbar::new(&app),
            last_render_time: Instant::now(),
            vello_ctx: None,
            wgc_stream: None,
            app,
        };

        if vello_enabled {
            // 1. Start WGC Pre-heat
            let mut stream = win32::wgc::capture::WgcStreamManager::new();
            if let Err(e) = stream.start() {
                log::error!("Failed to start pre-heat stream: {:?}", e);
            } else {
                mgr.wgc_stream = Some(stream);
            }

            // 2. Start Vello Pre-heat (Background)
            let app_handle = mgr.app.clone();
            let initial_hwnd = Some(win32::SendHWND(main_hwnd.0));
            tauri::async_runtime::spawn(async move {
                log::info!("[Advanced Mode] Pre-warming VelloContext in background...");
                match render::vello_engine::VelloContext::new(initial_hwnd).await {
                    Ok(ctx) => {
                        log::info!("[Advanced Mode] VelloContext pre-warmed successfully.");
                        if let Ok(mut mgr_lock) =
                            app_handle.state::<crate::AppState>().overlay_manager.lock()
                        {
                            mgr_lock.vello_ctx = Some(std::sync::Arc::new(ctx));
                            // Notify Frontend: Vello Engine Ready
                            use tauri::Emitter;
                            if let Err(e) =
                                app_handle.emit("vello://ready", std::time::SystemTime::now())
                            {
                                log::error!("Failed to emit vello://ready event: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[Advanced Mode] VelloContext pre-warm failed: {:?}", e);
                    }
                }
            });
        }

        Ok(mgr)
    }

    pub fn set_user_data(&self, ptr: *mut Self) {
        for send_hwnd in &self.windows {
            win32::window::set_window_handler(
                send_hwnd.0,
                ptr as *mut dyn win32::window::WindowEventHandler,
            );
        }
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

    pub fn close_and_reset(&mut self) {
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return,
        };
        state.is_visible = false;
        state.selection = None;
        state.interaction_mode = state::InteractionMode::None;
        state.hover_zone = state::HitZone::None;
        state.gdi.cache.clear();
        state.objects.clear();
        state.current_drawing = None;
        state.current_tool = state::DrawingTool::None;
        self.toolbar.current_tool = None;
        state.tool_registry.macros.clear();

        // P1 Fix: Clear all capture-related fields to prevent leakage
        state.is_snapshot_mode = false;
        state.active_workflow = None;
        state.vello.background = None;
        state.gdi.hbitmap_dim = None;
        state.gdi.hbitmap_bright = None;
        state.is_capturing = false;

        // Clear DXGI surfaces (swapchain) from VelloContext to prevent GDI state leakage,
        // but KEEP the VelloContext itself (device, adapter, renderer) alive to avoid
        // costly 10-second GPU re-initialization on the next capture.
        if let Some(ctx) = &self.vello_ctx {
            if let Ok(mut surfaces) = ctx.surfaces.lock() {
                surfaces.clear();
            }
            if let Ok(mut configs) = ctx.surface_configs.lock() {
                configs.clear();
            }
            if let Ok(mut caps) = ctx.surface_caps.lock() {
                caps.clear();
            }
            if let Ok(mut proxies) = ctx.proxy_textures.lock() {
                proxies.clear();
            }
        }

        for send_hwnd in &self.windows {
            let hwnd = win32::window::SafeHWND(send_hwnd.0);

            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(Some(hwnd.0), 1);
            }
            win32::window::hide_window(&hwnd);
        }
    }

    pub fn render_frame(&mut self) -> anyhow::Result<()> {
        let mut state = self.state.lock().unwrap();
        if let Some(send_hwnd) = self.windows.first() {
            let hwnd = SafeHWND(send_hwnd.0);
            return render::render_frame(
                &hwnd,
                &self.app,
                &mut state,
                &mut self.toolbar,
                &self.vello_ctx,
            );
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
        // 0. Check engine preference from config
        let (vello_pref, advanced_effects, ai_shortcuts) = {
            let app_state = self.app.state::<crate::AppState>();
            let cfg = match app_state.config_state.lock() {
                Ok(c) => c,
                Err(_) => return Ok(()),
            };
            (
                cfg.config.vello_enabled,
                cfg.config.vello_advanced_effects,
                cfg.config.ai_shortcuts.clone(),
            )
        };

        // Sync config to state
        {
            if let Ok(mut state) = self.state.lock() {
                state.enable_advanced_effects = advanced_effects;

                // Sync AI Macros to ToolRegistry
                state.tool_registry.macros = ai_shortcuts
                    .into_iter()
                    .map(|s| state::AiMacro {
                        id: s.id,
                        name: s.name,
                        icon: "\u{F031}".to_string(), // Default AI icon (Magic brush/Gemini-like)
                        prompt: s.prompt,
                    })
                    .collect();
            }
        }

        let (mode, engine, registry) = {
            let mut state = match self.state.lock() {
                Ok(s) => s,
                Err(_) => return Ok(()),
            };

            // CRITICAL FIX: If active_workflow is set, its action.engine should BE SACRED.
            // Do not let global vello_enabled override a specifically requested engine in a workflow.
            let mut engine = if let Some(flow) = &state.active_workflow {
                match &flow.action {
                    crate::service::config::types::CaptureAction::Selection { engine }
                    | crate::service::config::types::CaptureAction::Fullscreen { engine }
                    | crate::service::config::types::CaptureAction::Window { engine } => {
                        if engine == "vello" {
                            state::CaptureEngine::Wgc
                        } else {
                            state::CaptureEngine::Gdi
                        }
                    }
                    crate::service::config::types::CaptureAction::Snapshot { engine, .. } => {
                        if engine == "vello" {
                            state::CaptureEngine::Wgc
                        } else {
                            state::CaptureEngine::Gdi
                        }
                    }
                }
            } else {
                state.capture_engine
            };

            // Only perform auto-upgrade/downgrade if we are NOT in a specific workflow (e.g. triggered via manual commands)
            // or if we want to keep the auto-sync behavior for workflows that didn't specify?
            // Actually, CaptureAction ALWAYS specifies an engine string.

            if vello_pref
                && engine == crate::service::native_overlay::state::CaptureEngine::Gdi
                && state.active_workflow.is_none()
            // Only auto-upgrade if not in a set workflow
            {
                drop(state);
                log::info!(
                    "Auto-upgrading to Vello based on settings (No active workflow context)..."
                );
                if let Err(e) = self.upgrade_to_vello() {
                    log::error!("Auto-upgrade failed: {:?}", e);
                }
                let state_after = match self.state.lock() {
                    Ok(s) => s,
                    Err(_) => return Ok(()),
                };
                engine = state_after.capture_engine;
                (
                    state_after.capture_mode,
                    engine,
                    state_after.tool_registry.clone(),
                )
            } else {
                state.capture_engine = engine; // Ensure state matches our priority decision
                (state.capture_mode, engine, state.tool_registry.clone())
            }
        };

        self.toolbar
            .rebuild_for_mode(&self.app, mode, engine, &registry);

        if let Some(send_hwnd) = self.windows.first() {
            let hwnd = SafeHWND(send_hwnd.0);
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd.0,
                    Some(windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST),
                    x,
                    y,
                    width,
                    height,
                    windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                );
            }

            let _ = self.render_frame();

            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                    hwnd.0,
                    windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
                );
                let _ =
                    windows::Win32::UI::WindowsAndMessaging::SetTimer(Some(hwnd.0), 1, 500, None);
                let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd.0);
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(hwnd.0));
            }
        }
        Ok(())
    }

    pub fn save_selection(&mut self) -> anyhow::Result<()> {
        super::save::save_selection(&self.state, &self.app)?;
        self.close_and_reset();
        Ok(())
    }

    pub fn save_clipboard(&mut self) -> anyhow::Result<()> {
        super::save::copy_to_clipboard(&self.state, &self.app)?;
        self.close_and_reset();
        Ok(())
    }
}

impl Drop for OverlayManager {
    fn drop(&mut self) {
        log::info!("Dropping OverlayManager...");

        // 1. Stop Pre-heat (safely)
        self.stop_pre_heat();

        // 2. Destroy Windows
        for send_hwnd in &self.windows {
            let hwnd = win32::window::SafeHWND(send_hwnd.0);

            // Clean up the dispatcher pointer first
            win32::window::remove_window_handler(&hwnd);

            // Then destroy the window
            win32::window::destroy_window(&hwnd);
        }

        // 3. Clear State (Best Effort)
        if let Ok(mut state) = self.state.lock() {
            state.gdi.cache.clear();
            state.objects.clear();
        }

        log::info!("OverlayManager dropped successfully.");
    }
}
