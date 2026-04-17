use super::OverlayManager;
use crate::service::native_overlay::state;
use crate::service::win32;
use crate::service::win32::window::SafeHWND;

impl OverlayManager {
    pub fn close_and_reset(&mut self) {
        {
            let mut state = match self.state.write() {
                Ok(s) => s,
                Err(_) => return,
            };
            state.is_visible = false;
            state.selection = None;
            state.interaction_mode = state::InteractionMode::None;
            state.hover_zone = state::HitZone::None;
            state.objects.clear();
            state.current_drawing = None;
            state.current_tool = state::DrawingTool::None;
            self.toolbar.current_tool = None;

            // P1 Fix: Clear all capture-related fields to prevent leakage
            state.is_snapshot_mode = false;
            state.active_workflow = None;
            state.vello.background = None;
            state.gdi.hbitmap_dim = None;
            state.gdi.hbitmap_bright = None;
            state.gdi.gdiplus_bitmap_dim = None;
            state.gdi.gdiplus_bitmap_bright = None;
            state.gdi.bright_pixels = None;
            state.gdi.style_initialized = false;
            state.is_capturing = false;
            state.monitor_id = String::new(); // Clear monitor context
            state.selection_pointer = None;
        }

        // INDUSTRIAL RESET: Clear all per-monitor rendering contexts
        // This drops all SafeHDC/SafeHBITMAP handles, freeing memory immediately.
        self.monitor_contexts.clear();

        // Clear DXGI surfaces (swapchain) from VelloContext
        if let Some(ctx) = &self.vello_ctx {
            ctx.purge_surfaces();
        }

        // Hide ALL HWNDs in the pool to ensure a clean slate
        for h in self.gdi_hwnds.values() {
            let hwnd = SafeHWND(h.0);
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(Some(hwnd.0), 1);
            }
            win32::window::hide_window(&hwnd);
        }
        for h in self.vello_hwnds.values() {
            let hwnd = SafeHWND(h.0);
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(Some(hwnd.0), 1);
            }
            win32::window::hide_window(&hwnd);
        }
    }
}

impl Drop for OverlayManager {
    fn drop(&mut self) {
        log::info!("Dropping OverlayManager [B-Scheme]...");

        // 1. Stop Pre-heat (safely)
        self.stop_pre_heat();

        // 2. Destroy ALL GDI Windows
        for h in self.gdi_hwnds.values() {
            let hwnd = SafeHWND(h.0);
            win32::window::remove_window_handler(&hwnd);
            win32::window::destroy_window(&hwnd);
        }

        // 3. Destroy ALL Vello Windows
        for h in self.vello_hwnds.values() {
            let hwnd = SafeHWND(h.0);
            win32::window::remove_window_handler(&hwnd);
            win32::window::destroy_window(&hwnd);
        }

        // 4. Clear State (Best Effort)
        if let Ok(mut state) = self.state.write() {
            state.objects.clear();
        }

        log::info!("OverlayManager [B-Scheme] dropped successfully.");
    }
}
