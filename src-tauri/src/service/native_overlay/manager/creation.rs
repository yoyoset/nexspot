use super::OverlayManager;
use crate::service::native_overlay::render::toolbar;
use crate::service::native_overlay::state::{self, OverlayState};
use crate::service::win32;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Manager};

impl OverlayManager {
    pub fn new(
        app: AppHandle,
        font_family: String,
        vello_enabled: bool,
    ) -> anyhow::Result<Self> {
        let state = Arc::new(RwLock::new(OverlayState::default()));
        {
            let mut s = state.write().unwrap_or_else(|e| e.into_inner());
            s.font_family = font_family;
            if vello_enabled {
                s.capture_engine = state::CaptureEngine::Wgc;
            }
        }

        let mut gdi_hwnds = HashMap::new();
        let mut vello_hwnds = HashMap::new();

        // Enumerate monitors to create dedicated windows for each (Monitor-Specific Pipeline)
        let monitors = win32::monitor::enumerate_monitors()?;
        for monitor in monitors {
            let monitor_id = monitor.hmonitor.to_string();
            
            // 1. Create DEDICATED GDI overlay window for this monitor
            let gdi_hwnd = win32::window::create_overlay_window(
                &format!("HyperLensOverlayGDI_{}", monitor_id),
                &format!("HyperLens GDI - {}", monitor.friendly_name)
            )?;
            
            // Fix position to monitor top-left immediately
            let _ = win32::window::set_window_pos(
                &gdi_hwnd,
                monitor.rect.left,
                monitor.rect.top,
                monitor.rect.right - monitor.rect.left,
                monitor.rect.bottom - monitor.rect.top,
                windows::Win32::UI::WindowsAndMessaging::SWP_HIDEWINDOW | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE
            );
            
            log::info!("[Advanced] Created GDI HWND for monitor {}: {:?}", monitor_id, gdi_hwnd.0);
            gdi_hwnds.insert(monitor_id.clone(), win32::SendHWND(gdi_hwnd.0));

            // 2. Create DEDICATED Vello overlay window for this monitor
            if vello_enabled {
                match win32::window::create_overlay_window(
                    &format!("HyperLensOverlayVello_{}", monitor_id),
                    &format!("HyperLens Vello - {}", monitor.friendly_name)
                ) {
                    Ok(h) => {
                        // Fix position to monitor top-left immediately
                        let _ = win32::window::set_window_pos(
                            &h,
                            monitor.rect.left,
                            monitor.rect.top,
                            monitor.rect.right - monitor.rect.left,
                            monitor.rect.bottom - monitor.rect.top,
                            windows::Win32::UI::WindowsAndMessaging::SWP_HIDEWINDOW | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE
                        );
                        
                        log::info!("[Advanced] Created Vello HWND for monitor {}: {:?}", monitor_id, h.0);
                        vello_hwnds.insert(monitor_id, win32::SendHWND(h.0));
                    }
                    Err(e) => {
                        log::error!("Failed to create Vello HWND for monitor {}: {:?}", monitor_id, e);
                    }
                }
            }
        }

        // 1. Register Custom Font from memory (Embedded)
        static FONT_DATA: &[u8] = include_bytes!("../../../../resources/remixicon.ttf");
        match win32::gdi::add_font_mem(FONT_DATA) {
            Ok(_) => {
                log::info!("Registered remixicon font from embedded memory");
                win32::gdiplus::register_font_for_gdiplus(FONT_DATA);
            }
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
            gdi_hwnds,
            vello_hwnds,
            toolbar: toolbar::Toolbar::new(&app),
            last_render_time: Instant::now(),
            vello_ctx: None,
            vello_status: if vello_enabled {
                state::VelloStatus::Pending
            } else {
                state::VelloStatus::Failed("Vello disabled".to_string())
            },
            wgc_stream: None,
            scroll_stitcher: Arc::new(Mutex::new(None)),
            monitor_contexts: HashMap::new(),
            app,
        };

        if vello_enabled {
            mgr.start_pre_heat();
            // Note: init_vello_async will be called from lib.rs after AppState is managed
        }

        Ok(mgr)
    }

    pub fn set_user_data(&self, ptr: *mut Self) {
        // Register handler on ALL GDI HWNDs
        for hwnd in self.gdi_hwnds.values() {
            win32::window::set_window_handler(
                hwnd.0,
                ptr as *mut dyn win32::window::WindowEventHandler,
            );
        }
        // Register handler on ALL Vello HWNDs
        for hwnd in self.vello_hwnds.values() {
            win32::window::set_window_handler(
                hwnd.0,
                ptr as *mut dyn win32::window::WindowEventHandler,
            );
        }
    }
}
