use crate::service::native_overlay::state::{self, OverlayState};
use crate::service::win32::gdi::{self, SafeHDC, GdiCache};
use windows::Win32::Foundation::RECT;

pub fn draw_selection_overlay(
    hdc_mem: &SafeHDC,
    sel: &RECT,
    state: &mut OverlayState,
    cache: &mut GdiCache,
    offset_x: i32,
    offset_y: i32,
) -> anyhow::Result<()> {
    // RIGOR CHECK: Only draw if the selection intersects with the current monitor
    let m_rect = &state.monitor_rect;
    let intersect_w = (sel.right.min(m_rect.right) - sel.left.max(m_rect.left)).max(0);
    let intersect_h = (sel.bottom.min(m_rect.bottom) - sel.top.max(m_rect.top)).max(0);
    
    if intersect_w <= 0 || intersect_h <= 0 {
        return Ok(());
    }

    // Draw accent selection border
    let border_rect = RECT {
        left: sel.left - offset_x,
        top: sel.top - offset_y,
        right: sel.right - offset_x,
        bottom: sel.bottom - offset_y,
    };
    // Accent periwinkle #7a6ff2 (RGB)
    let border_brush = cache.get_brush(0x7A6FF2)?;
    gdi::frame_rect(hdc_mem, &border_rect, &border_brush);

    draw_handles(hdc_mem, sel, state, cache, offset_x, offset_y)
}

pub fn draw_handles(
    hdc: &SafeHDC,
    sel: &RECT,
    state: &mut OverlayState,
    cache: &mut GdiCache,
    offset_x: i32,
    offset_y: i32,
) -> anyhow::Result<()> {
    // Draw Handles (8 circular points)
    let handle_size = 10;
    let half = handle_size / 2;
    
    // Get handles and copy them to avoid holding mutable borrow on state.gdi.cache
    let tiffany_brush_handle = cache.get_brush(0x7A6FF2)?.0;
    let white_brush_handle = cache.get_brush(0xFFFFFF)?.0;
    let tiffany_pen_handle = cache.get_gdi_pen(0, 2, 0x7A6FF2)?.0;

    let prev_pen = unsafe {
        windows::Win32::Graphics::Gdi::SelectObject(
            hdc.0,
            windows::Win32::Graphics::Gdi::HGDIOBJ(tiffany_pen_handle.0),
        )
    };

    let hover_zone = state.hover_zone;
    let interaction_mode = state.interaction_mode;

    let draw_handle = |cx: i32, cy: i32, zone: state::HitZone| -> anyhow::Result<()> {
        let r = RECT {
            left: cx - half - offset_x,
            top: cy - half - offset_y,
            right: cx + half - offset_x,
            bottom: cy + half - offset_y,
        };

        let is_hover = hover_zone == zone
            || matches!(interaction_mode, state::InteractionMode::Resizing(z) if z == zone)
            || matches!(interaction_mode, state::InteractionMode::TransformingObject(z) if z == zone);

        let brush_handle = if is_hover {
            tiffany_brush_handle
        } else {
            white_brush_handle
        };

        unsafe {
            let prev_brush = windows::Win32::Graphics::Gdi::SelectObject(
                hdc.0,
                windows::Win32::Graphics::Gdi::HGDIOBJ(brush_handle.0),
            );

            // Draw Circle
            let _ = windows::Win32::Graphics::Gdi::Ellipse(hdc.0, r.left, r.top, r.right, r.bottom);

            windows::Win32::Graphics::Gdi::SelectObject(hdc.0, prev_brush);
        }
        Ok(())
    };

    // Check if we should draw special handles for Arrow
    if let Some(idx) = state.selected_object_index {
        if let Some(obj) = state.objects.get(idx).cloned() {
            if obj.tool == state::DrawingTool::Arrow && obj.points.len() == 2 {
                let p1 = obj.points[0];
                let p2 = obj.points[1];
                let dx = (p2.0 - p1.0) as f32;
                let dy = (p2.1 - p1.1) as f32;
                let len = (dx * dx + dy * dy).sqrt();

                if len > 0.1 {
                    let ux = dx / len;
                    let uy = dy / len;
                    let px = -uy;
                    let py = ux;

                    let stroke_width = obj.stroke_width.max(1.0);
                    let head_len = (stroke_width * 8.0 + 32.0).min(len * 0.9);
                    let head_width = obj.head_width.unwrap_or(head_len * 1.0);
                    let wing_dist = head_len;

                    // Tail
                    draw_handle(p1.0, p1.1, state::HitZone::Tail)?;
                    // Tip
                    draw_handle(p2.0, p2.1, state::HitZone::Tip)?;
                    // Wing Right
                    let wr_x = p2.0 as f32 - ux * wing_dist + px * head_width / 2.0;
                    let wr_y = p2.1 as f32 - uy * wing_dist + py * head_width / 2.0;
                    draw_handle(wr_x as i32, wr_y as i32, state::HitZone::WingRight)?;
                    // Wing Left
                    let wl_x = p2.0 as f32 - ux * wing_dist - px * head_width / 2.0;
                    let wl_y = p2.1 as f32 - uy * wing_dist - py * head_width / 2.0;
                    draw_handle(wl_x as i32, wl_y as i32, state::HitZone::WingLeft)?;

                    // Restore & Exit
                    unsafe {
                        windows::Win32::Graphics::Gdi::SelectObject(hdc.0, prev_pen);
                    }
                    return Ok(());
                }
            }
        }
    }

    // Default 8 Points
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

    // Clean up
    unsafe {
        windows::Win32::Graphics::Gdi::SelectObject(hdc.0, prev_pen);
    }
    Ok(())
}
