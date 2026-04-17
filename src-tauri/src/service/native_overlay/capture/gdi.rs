use crate::service::native_overlay::state::OverlayState;
use crate::service::win32;
use std::sync::{Arc, RwLock};
use windows::Win32::Foundation::{POINT, RECT};

pub fn capture_gdi(
    state: &Arc<RwLock<OverlayState>>,
    _monitors: &[win32::monitor::MonitorInfo],
    target_monitor_rect: &RECT,
    has_mixed_dpi: bool,
    union_rect: &RECT,
    cursor: &POINT,
) -> anyhow::Result<(i32, i32, i32, i32)> {
    log::info!("Starting GDI Mode (Virtual Screen Capture - Mixed DPI: {})", has_mixed_dpi);

    // [PROTOCOL RESTORED] §3.1: Full Virtual Desktop Scope
    let (x, y, width, height) = (
        union_rect.left,
        union_rect.top,
        union_rect.right - union_rect.left,
        union_rect.bottom - union_rect.top,
    );

    log::info!("GDI Mode: Virtual Area=({},{}) {}x{}", x, y, width, height);

    let t_cap = std::time::Instant::now();
    let hdc_screen = win32::gdi::get_dc(None)?;
    let hdc_mem = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
    let hbm_screen = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
    let prev_hbm_screen = win32::gdi::select_object(
        &hdc_mem,
        windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_screen.0 .0),
    )?;

    // 2. Perform Capture (Direct or Stitched)
    // Always use SRCCOPY | CAPTUREBLT to include layered windows
    const SRCCOPY_CAPTURE: windows::Win32::Graphics::Gdi::ROP_CODE = windows::Win32::Graphics::Gdi::ROP_CODE(windows::Win32::Graphics::Gdi::SRCCOPY.0 | 0x40000000);

    if let Err(e) = win32::gdi::bit_blt(
        &hdc_mem,
        0,
        0,
        width,
        height,
        &hdc_screen,
        x,
        y,
        SRCCOPY_CAPTURE,
    ) {
        log::error!("Failed to capture virtual screen: {:?}", e);
        return Err(e);
    }

    log::info!("GDI BitBlt took {:?}", t_cap.elapsed());

    // 3. Pre-render Dimmed Background
    let t_dim = std::time::Instant::now();
    let hdc_dim = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
    let hbm_dim = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
    let prev_hbm_dim = win32::gdi::select_object(
        &hdc_dim,
        windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_dim.0 .0),
    )?;

    // Copy Screen to Dim
    win32::gdi::bit_blt(
        &hdc_dim,
        0,
        0,
        width,
        height,
        &hdc_mem,
        0,
        0,
        windows::Win32::Graphics::Gdi::SRCCOPY,
    )?;

    // Fast Darken: Hardware Accelerated AlphaBlend (GDI)
    // [PROTOCOL RESTORED] §6.4: Use full-size black bitmap for AlphaBlend safety
    {
        let hdc_black = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let hbm_black = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
        let prev_hbm_black = win32::gdi::select_object(
            &hdc_black,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_black.0 .0),
        )?;

        let brush = win32::gdi::create_solid_brush(0x000000)?;
        win32::gdi::fill_rect(
            &hdc_black,
            &RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            },
            &brush,
        );

        unsafe {
            use windows::Win32::Graphics::Gdi::{GdiAlphaBlend, AC_SRC_OVER, BLENDFUNCTION};
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 120, // ~47% Darken
                AlphaFormat: 0,
            };

            let _ = GdiAlphaBlend(
                hdc_dim.0,
                0,
                0,
                width,
                height,
                hdc_black.0,
                0,
                0,
                width,
                height,
                blend,
            );
        }
        win32::gdi::select_object(&hdc_black, prev_hbm_black)?;
    }

    // 4. Pre-initialize GDI+ Bitmaps for Performance
    let gdiplus_bright = crate::service::win32::gdiplus::BitmapWrapper::from_hbitmap(hbm_screen.0)?;
    let gdiplus_dim = crate::service::win32::gdiplus::BitmapWrapper::from_hbitmap(hbm_dim.0)?;

    // 5. Extract Pixels for RAM Sampling (Mosaic, etc.)
    // Optimized: get_bitmap_pixels_u32 already returns Arc-ready Vec<u32> with Alpha fixed.
    let bright_pixels =
        match win32::gdi::get_bitmap_pixels_u32(&hdc_mem, &hbm_screen, width, height) {
            Ok(v) => Some(std::sync::Arc::new(v)),
            Err(e) => {
                log::error!("Failed to extract pixels for sampling: {:?}", e);
                None
            }
        };

    // Restore & Cleanup
    win32::gdi::select_object(&hdc_mem, prev_hbm_screen)?;
    win32::gdi::select_object(&hdc_dim, prev_hbm_dim)?;
    win32::gdi::release_dc(None, hdc_screen);

    log::info!("GDI Dimming took {:?}", t_dim.elapsed());

    // Enumerate Windows for Snapping
    let t_wc = std::time::Instant::now();
    let snap_rects = win32::window::enumerate_visible_windows();
    log::info!(
        "Enumerated {} Windows in {:?}",
        snap_rects.len(),
        t_wc.elapsed()
    );

    // 6. Update State
    if let Ok(mut s) = state.write() {
        s.capture_x = x;
        s.capture_y = y;
        s.width = width;
        s.height = height;
        s.mouse_x = cursor.x;
        s.mouse_y = cursor.y;
        s.gdi.hbitmap_bright = Some(hbm_screen);
        s.gdi.hbitmap_dim = Some(hbm_dim);
        s.gdi.gdiplus_bitmap_bright = Some(gdiplus_bright);
        s.gdi.gdiplus_bitmap_dim = Some(gdiplus_dim);
        s.gdi.bright_pixels = bright_pixels;
        s.window_rects = snap_rects;
        s.selection = None;
        s.is_capturing = false;
        s.vello.background = None;
        s.restrict_to_monitor = if has_mixed_dpi { Some(*target_monitor_rect) } else { None };
    }

    Ok((x, y, width, height))
}
