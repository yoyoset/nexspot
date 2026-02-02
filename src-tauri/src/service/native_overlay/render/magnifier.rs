use crate::service::native_overlay::state::OverlayState;
use crate::service::win32;
use crate::service::win32::gdi::SafeHDC;
use windows::Win32::Foundation::RECT;

pub fn draw_magnifier(
    hdc_mem: &SafeHDC,
    mouse_x: i32,
    mouse_y: i32,
    state: &OverlayState,
) -> anyhow::Result<()> {
    if !state.is_visible {
        return Ok(());
    }

    let mag_size = 120;
    let zoom_factor = 2;
    let offset_x = 20;
    let offset_y = 20;

    let mut x = mouse_x + offset_x;
    let mut y = mouse_y + offset_y;
    if x + mag_size > state.width {
        x = mouse_x - mag_size - offset_x;
    }
    if y + mag_size > state.height {
        y = mouse_y - mag_size - offset_y;
    }

    let rect = RECT {
        left: x,
        top: y,
        right: x + mag_size,
        bottom: y + mag_size,
    };
    let brush_bg = win32::gdi::create_solid_brush(0x202020)?;
    win32::gdi::fill_rect(hdc_mem, &rect, &brush_bg);

    let brush_border = win32::gdi::create_solid_brush(0xFFFFFF)?;
    win32::gdi::frame_rect(hdc_mem, &rect, &brush_border);

    if let Some(hbm_bright) = &state.hbitmap_bright {
        let hdc_src = win32::gdi::create_compatible_dc(None)?;
        win32::gdi::select_object(
            &hdc_src,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_bright.0 .0),
        )?;

        let src_w = mag_size / zoom_factor;
        let src_h = mag_size / zoom_factor;
        let src_x = mouse_x - (src_w / 2);
        let src_y = mouse_y - (src_h / 2);

        unsafe {
            let _ = windows::Win32::Graphics::Gdi::StretchBlt(
                hdc_mem.0,
                x + 2,
                y + 2,
                mag_size - 4,
                mag_size - 4,
                Some(hdc_src.0),
                src_x,
                src_y,
                src_w,
                src_h,
                windows::Win32::Graphics::Gdi::SRCCOPY,
            );
        }
    }

    let mid_x = x + mag_size / 2;
    let mid_y = y + mag_size / 2;
    {
        let brush_cross = win32::gdi::create_solid_brush(0x00D7FF)?;
        let cross_rect = RECT {
            left: mid_x - (zoom_factor / 2),
            top: y + 2,
            right: mid_x + (zoom_factor / 2),
            bottom: y + mag_size - 2,
        };
        win32::gdi::fill_rect(hdc_mem, &cross_rect, &brush_cross);

        let cross_rect_h = RECT {
            left: x + 2,
            top: mid_y - (zoom_factor / 2),
            right: x + mag_size - 2,
            bottom: mid_y + (zoom_factor / 2),
        };
        win32::gdi::fill_rect(hdc_mem, &cross_rect_h, &brush_cross);
    }

    Ok(())
}
