use crate::service::win32::gdi::{self, SafeHDC};
use windows::Win32::Foundation::RECT;

pub fn draw_property_bar(hdc: &SafeHDC, rect: &RECT) -> anyhow::Result<()> {
    // Draw property bar background
    let bg_brush = gdi::create_solid_brush(0x222222)?;
    let border_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 1, 0x444444)?;
    let old_p = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(border_pen.0 .0))?;
    let old_b = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(bg_brush.0 .0))?;

    let _ = gdi::round_rect(hdc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);

    // Draw some colors (Placeholder for now)
    let colors = vec![0xFF0000, 0x00FF00, 0x0000FF, 0xFFFF00, 0xFFFFFF];
    let mut cur_x = rect.left + 8;
    let start_y = rect.top + 8;
    let color_size = 24;

    for color in colors {
        let c_brush = gdi::create_solid_brush(color)?;
        let old_cb = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(c_brush.0 .0))?;
        let _ = gdi::round_rect(
            hdc,
            cur_x,
            start_y,
            cur_x + color_size,
            start_y + color_size,
            4,
            4,
        );
        gdi::select_object(hdc, old_cb)?;
        cur_x += color_size + 8;
    }

    gdi::select_object(hdc, old_p)?;
    gdi::select_object(hdc, old_b)?;
    Ok(())
}
