use crate::service::monitor_info::{self};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SWP_HIDEWINDOW, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
};

pub struct OverlayManager;

impl OverlayManager {
    pub fn init_overlays(app: &AppHandle) -> Result<(), String> {
        let monitors = monitor_info::enumerate_monitors();

        crate::service::logger::log(
            "OverlayManager",
            &format!("Found {} monitors", monitors.len()),
        );

        for m in &monitors {
            crate::service::logger::log(
                "OverlayManager",
                &format!(
                    "Monitor {}: pos=({},{}) size={}x{} dpi={} primary={}",
                    m.id, m.x, m.y, m.width, m.height, m.dpi_scale, m.is_primary
                ),
            );
        }

        // Use actual DPI detection - single window for uniform DPI, per-monitor for mixed
        let uniform = monitor_info::is_dpi_uniform(&monitors);

        crate::service::logger::log("OverlayManager", &format!("DPI Uniform: {}", uniform));

        // ALWAYS use single window mode - GetSystemMetrics gives us the virtual desktop bounds
        // Per-monitor mode is unreliable with negative coordinates
        {
            // Use Win32 GetSystemMetrics for reliable virtual desktop bounds (returns PHYSICAL pixels)
            use windows::Win32::UI::WindowsAndMessaging::{
                GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
                SM_YVIRTUALSCREEN,
            };

            let (phys_x, phys_y, phys_w, phys_h) = unsafe {
                let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
                let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
                let w = GetSystemMetrics(SM_CXVIRTUALSCREEN);
                let h = GetSystemMetrics(SM_CYVIRTUALSCREEN);
                (x, y, w, h)
            };

            // Get DPI scale from any monitor (they're uniform)
            let dpi_scale = if !monitors.is_empty() {
                monitors[0].dpi_scale as f64
            } else {
                1.0
            };

            // Tauri's position() expects LOGICAL coordinates, so divide by DPI
            let log_x = phys_x as f64 / dpi_scale;
            let log_y = phys_y as f64 / dpi_scale;
            let log_w = phys_w as f64 / dpi_scale;
            let log_h = phys_h as f64 / dpi_scale;

            crate::service::logger::log(
                "OverlayManager",
                &format!(
                    "Physical: ({}, {}) {}x{}, DPI: {}, Logical: ({}, {}) {}x{}",
                    phys_x, phys_y, phys_w, phys_h, dpi_scale, log_x, log_y, log_w, log_h
                ),
            );

            // Create window with approximate logical coords first
            let win = Self::create_window(app, "overlay-main", log_x, log_y, log_w, log_h)?;

            // Then use Win32 SetWindowPos to set EXACT physical coordinates
            if let Ok(hwnd) = win.hwnd() {
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::SetWindowPos;
                    use windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER;

                    let _ = SetWindowPos(
                        HWND(hwnd.0 as _),
                        None,
                        phys_x,
                        phys_y,
                        phys_w,
                        phys_h,
                        SWP_NOZORDER,
                    );

                    crate::service::logger::log(
                        "OverlayManager",
                        &format!(
                            "SetWindowPos: ({}, {}) {}x{}",
                            phys_x, phys_y, phys_w, phys_h
                        ),
                    );
                }
            }
        }

        Ok(())
    }

    fn create_window(
        app: &AppHandle,
        label: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<WebviewWindow, String> {
        if let Some(win) = app.get_webview_window(label) {
            return Ok(win);
        }

        let win = WebviewWindowBuilder::new(app, label, WebviewUrl::App("/overlay".into()))
            .title("HyperLens Overlay")
            .position(x, y)
            .inner_size(width, height)
            .transparent(true)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .visible(false)
            .build()
            .map_err(|e| e.to_string())?;

        // Enforce TOPMOST via Win32 just in case
        /*
        let hwnd = win.hwnd().map_err(|e| e.to_string())?;
        unsafe {
            SetWindowPos(HWND(hwnd.0 as _), HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }
        */

        Ok(win)
    }

    pub fn show_all(app: &AppHandle) {
        // Get native overlay HWND to position WebView ABOVE it
        let native_hwnd = crate::service::native_overlay::get_hwnd();

        for win in app.webview_windows().values() {
            if win.label().starts_with("overlay-") {
                if let Ok(hwnd) = win.hwnd() {
                    unsafe {
                        // Position WebView window ABOVE native overlay (or TOPMOST if no native)
                        // Using HWND_TOP ensures it's at the top of the Z-order
                        let insert_after = if native_hwnd.is_some() {
                            windows::Win32::UI::WindowsAndMessaging::HWND_TOP
                        } else {
                            HWND_TOPMOST
                        };

                        let _ = SetWindowPos(
                            HWND(hwnd.0 as _),
                            insert_after,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
                        );

                        crate::service::logger::log(
                            "OverlayManager",
                            &format!("WebView shown, set to TOP Z-order"),
                        );
                    }
                } else {
                    win.show().unwrap();
                }

                win.set_focus().unwrap();
            }
        }
    }

    pub fn hide_all(app: &AppHandle) {
        for win in app.webview_windows().values() {
            if win.label().starts_with("overlay-") {
                // win.hide().unwrap();
                if let Ok(hwnd) = win.hwnd() {
                    unsafe {
                        // SWP_HIDEWINDOW
                        let _ = SetWindowPos(
                            HWND(hwnd.0 as _),
                            HWND_TOPMOST,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE | SWP_NOSIZE | SWP_HIDEWINDOW,
                        );
                    }
                } else {
                    win.hide().unwrap();
                }
            }
        }
    }
}
