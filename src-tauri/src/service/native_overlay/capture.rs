use crate::service::native_overlay::state::OverlayState;
use crate::service::win32;
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::RECT;

pub fn perform_capture(state: &Arc<Mutex<OverlayState>>) -> anyhow::Result<(i32, i32, i32, i32)> {
    let start = std::time::Instant::now();
    let monitors = win32::monitor::enumerate_monitors()?;
    log::info!("Monitors enumerated in {:?}", start.elapsed());

    // 1. Calculate Virtual Screen Bounds (Union of all monitors)
    let mut union_rect = RECT::default();
    if let Some(first) = monitors.first() {
        union_rect = first.rect;
    }
    for m in &monitors {
        union_rect.left = union_rect.left.min(m.rect.left);
        union_rect.top = union_rect.top.min(m.rect.top);
        union_rect.right = union_rect.right.max(m.rect.right);
        union_rect.bottom = union_rect.bottom.max(m.rect.bottom);
    }

    let width = union_rect.right - union_rect.left;
    let height = union_rect.bottom - union_rect.top;
    let x = union_rect.left;
    let y = union_rect.top;
    log::info!("Virtual Screen: {}x{} at ({}, {})", width, height, x, y);

    // 2. Capture Entire Virtual Screen (Single BitBlt)
    let t_cap = std::time::Instant::now();
    let hdc_screen = win32::gdi::get_dc(None)?;
    let hdc_mem = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
    let hbm_screen = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
    let prev_hbm_screen = win32::gdi::select_object(
        &hdc_mem,
        windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_screen.0 .0),
    )?;

    win32::gdi::bit_blt(
        &hdc_mem,
        0,
        0,
        width,
        height,
        &hdc_screen,
        x,
        y,
        windows::Win32::Graphics::Gdi::SRCCOPY,
    )?;

    // RESTORE to unlock bitmap
    win32::gdi::select_object(&hdc_mem, prev_hbm_screen)?;

    log::info!("Full Screen Capture took {:?}", t_cap.elapsed());

    // 3. Pre-render Dimmed Background (Optimization for Smoothness)
    let t_dim = std::time::Instant::now();
    let hdc_dim = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
    let hbm_dim = win32::gdi::create_compatible_bitmap(&hdc_screen, width, height)?;
    let prev_hbm_dim = win32::gdi::select_object(
        &hdc_dim,
        windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_dim.0 .0),
    )?;

    // We need hdc_mem selected again to copy from
    let _ = win32::gdi::select_object(
        &hdc_mem,
        windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_screen.0 .0),
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

    // Unselect hbm_screen again
    win32::gdi::select_object(&hdc_mem, prev_hbm_screen)?;

    // Fast Darken: Hardware Accelerated AlphaBlend (GDI)
    // As requested: Draw a semi-transparent black rectangle over the image.
    {
        let hdc_black = win32::gdi::create_compatible_dc(None)?;
        let hbm_black = win32::gdi::create_compatible_bitmap(&hdc_screen, 1, 1)?;
        let prev_hbm_black = win32::gdi::select_object(
            &hdc_black,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_black.0 .0),
        )?;

        let brush = win32::gdi::create_solid_brush(0x000000)?; // Black
        win32::gdi::fill_rect(
            &hdc_black,
            &RECT {
                left: 0,
                top: 0,
                right: 1,
                bottom: 1,
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
                1,
                1,
                blend,
            );
        }

        win32::gdi::select_object(&hdc_black, prev_hbm_black)?;
    }

    // RESTORE DIM HDC to unlock hbm_dim
    win32::gdi::select_object(&hdc_dim, prev_hbm_dim)?;

    log::info!("Dimming generation took {:?}", t_dim.elapsed());

    // 3.5 Enumerate Windows for Snapping
    let t_wc = std::time::Instant::now();
    let snap_rects = win32::window::enumerate_visible_windows();
    log::info!(
        "Enumerated {} Windows for Snapping in {:?}",
        snap_rects.len(),
        t_wc.elapsed()
    );

    // 4. Update State
    {
        let mut s = state.lock().unwrap();
        s.x = x;
        s.y = y;
        s.width = width;
        s.height = height;

        // Move ownership directly to state
        s.hbitmap_bright = Some(hbm_screen);
        s.hbitmap_dim = Some(hbm_dim);
        s.window_rects = snap_rects;

        s.is_visible = true;
        s.selection = None;
    }

    log::info!("Total perform_capture took {:?}", start.elapsed());

    // 5. Return Bounds for Main Thread UI Update
    Ok((x, y, width, height))
}
