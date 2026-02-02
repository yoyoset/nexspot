use crate::service::native_overlay::state::{self, OverlayState};
use crate::service::win32::gdi::{self, SafeHDC};
use windows::Win32::Foundation::RECT;

pub fn draw_selection_overlay(
    hdc_mem: &SafeHDC,
    sel: &RECT,
    state: &OverlayState,
) -> anyhow::Result<()> {
    // Draw Cyan Border
    let border_rect = RECT {
        left: sel.left,
        top: sel.top,
        right: sel.right,
        bottom: sel.bottom,
    };
    let border_brush = gdi::create_solid_brush(0xFFFF00)?;
    gdi::frame_rect(hdc_mem, &border_rect, &border_brush);

    // Draw Handles (8 circular points)
    let handle_size = 8;
    let half = handle_size / 2;
    let cyan_brush = gdi::create_solid_brush(0xFFFF00)?;
    let white_brush = gdi::create_solid_brush(0xFFFFFF)?;

    // Use CreatePen for Ellipse outline
    let cyan_pen = unsafe {
        windows::Win32::Graphics::Gdi::CreatePen(
            windows::Win32::Graphics::Gdi::PS_SOLID,
            1,
            windows::Win32::Foundation::COLORREF(0xFFFF00),
        )
    };
    let prev_pen = unsafe {
        windows::Win32::Graphics::Gdi::SelectObject(
            hdc_mem.0,
            windows::Win32::Graphics::Gdi::HGDIOBJ(cyan_pen.0),
        )
    };

    let draw_handle = |cx: i32, cy: i32, zone: state::HitZone| -> anyhow::Result<()> {
        let r = RECT {
            left: cx - half,
            top: cy - half,
            right: cx + half,
            bottom: cy + half,
        };

        let is_hover = state.hover_zone == zone
            || matches!(state.interaction_mode, state::InteractionMode::Resizing(z) if z == zone);

        let brush = if is_hover { &cyan_brush } else { &white_brush };

        unsafe {
            let prev_brush = windows::Win32::Graphics::Gdi::SelectObject(
                hdc_mem.0,
                windows::Win32::Graphics::Gdi::HGDIOBJ(brush.0 .0),
            );

            // Draw Circle
            let _ =
                windows::Win32::Graphics::Gdi::Ellipse(hdc_mem.0, r.left, r.top, r.right, r.bottom);

            windows::Win32::Graphics::Gdi::SelectObject(hdc_mem.0, prev_brush);
        }
        Ok(())
    };

    // 8 Points
    let mid_x = sel.left + (sel.right - sel.left) / 2;
    let mid_y = sel.top + (sel.bottom - sel.top) / 2;

    draw_handle(sel.left, sel.top, state::HitZone::TopLeft)?;
    draw_handle(mid_x, sel.top, state::HitZone::Top)?;
    draw_handle(sel.right, sel.top, state::HitZone::TopRight)?;
    draw_handle(sel.right, mid_y, state::HitZone::Right)?;
    draw_handle(sel.right, sel.bottom, state::HitZone::BottomRight)?;
    draw_handle(mid_x, sel.bottom, state::HitZone::Bottom)?;
    draw_handle(sel.left, sel.bottom, state::HitZone::BottomLeft)?;
    draw_handle(sel.left, mid_y, state::HitZone::Left)?;

    // Clean up Pen
    unsafe {
        windows::Win32::Graphics::Gdi::SelectObject(hdc_mem.0, prev_pen);
        let _ = windows::Win32::Graphics::Gdi::DeleteObject(
            windows::Win32::Graphics::Gdi::HGDIOBJ(cyan_pen.0),
        );
    }
    Ok(())
}
