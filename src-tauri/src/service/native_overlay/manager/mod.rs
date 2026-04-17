use crate::service::native_overlay::render;
use crate::service::native_overlay::render::toolbar;
use crate::service::native_overlay::state::{self, OverlayState};
use crate::service::win32::window::SafeHWND;
use crate::service::win32::{self, SendHWND};
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tauri::AppHandle;

mod actions;
mod creation;
mod engine;
mod lifecycle;

pub struct MonitorRenderContext {
    pub hdc_backbuffer: Option<crate::service::win32::gdi::SafeHDC>,
    pub hbm_backbuffer: Option<crate::service::win32::gdi::SafeHBITMAP>,
    pub backbuffer_size: (i32, i32),
    pub hdc_bg_src: Option<crate::service::win32::gdi::SafeHDC>,
    pub hdc_selection_src: Option<crate::service::win32::gdi::SafeHDC>,
    pub graphics: Option<crate::service::win32::gdiplus::GraphicsWrapper>,
    pub cache: crate::service::win32::gdi::GdiCache,
}

impl Default for MonitorRenderContext {
    fn default() -> Self {
        Self {
            hdc_backbuffer: None,
            hbm_backbuffer: None,
            backbuffer_size: (0, 0),
            hdc_bg_src: None,
            hdc_selection_src: None,
            graphics: None,
            cache: crate::service::win32::gdi::GdiCache::new(),
        }
    }
}

pub struct OverlayManager {
    pub state: Arc<RwLock<OverlayState>>,
    /// Dedicated HWNDs for GDI rendering (Layered Window), keyed by HMONITOR ID.
    pub gdi_hwnds: HashMap<String, SendHWND>,
    /// Dedicated HWNDs for Vello/WGPU rendering (DXGI Swapchain), keyed by HMONITOR ID.
    pub vello_hwnds: HashMap<String, SendHWND>,
    pub toolbar: toolbar::Toolbar,
    pub last_render_time: Instant,
    pub vello_ctx: Option<Arc<render::vello_engine::VelloContext>>,
    pub vello_status: state::VelloStatus,
    pub wgc_stream: Option<win32::wgc::capture::WgcStreamManager>,
    pub scroll_stitcher: Arc<Mutex<Option<crate::service::stitch::Stitcher>>>,
    pub monitor_contexts: HashMap<String, MonitorRenderContext>,
    pub app: AppHandle,
}

impl OverlayManager {
    /// Returns the correct HWND for the active engine on the specified monitor.
    pub(crate) fn active_hwnd(&self, engine: state::CaptureEngine, monitor_id: &str) -> anyhow::Result<SafeHWND> {
        match engine {
            state::CaptureEngine::Wgc => {
                if let Some(vh) = self.vello_hwnds.get(monitor_id) {
                    Ok(SafeHWND(vh.0))
                } else {
                    Err(anyhow::anyhow!("Vello HWND for monitor {} is not available", monitor_id))
                }
            }
            state::CaptureEngine::Gdi => {
                if let Some(gh) = self.gdi_hwnds.get(monitor_id) {
                    Ok(SafeHWND(gh.0))
                } else {
                    Err(anyhow::anyhow!("GDI HWND for monitor {} is not available", monitor_id))
                }
            }
        }
    }
}
