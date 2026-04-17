use crate::service::native_overlay::state::OverlayState;
use crate::service::native_overlay::manager::MonitorRenderContext;
use crate::service::win32;
use crate::service::win32::gdi::SafeHDC;
use windows::Win32::Foundation::RECT;

pub fn draw_magnifier(
    hdc_mem: &SafeHDC,
    mouse_x: i32,
    mouse_y: i32,
    state: &mut OverlayState,
    ctx: &mut MonitorRenderContext,
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
    if x + mag_size > state.monitor_rect.right {
        x = mouse_x - mag_size - offset_x;
    }
    if y + mag_size > state.monitor_rect.bottom {
        y = mouse_y - mag_size - offset_y;
    }

    let rect = RECT {
        left: x,
        top: y,
        right: x + mag_size,
        bottom: y + mag_size,
    };
    
    let brush_bg = ctx.cache.get_brush(0x202020)?;
    let local_rect = RECT {
        left: x - state.monitor_rect.left,
        top: y - state.monitor_rect.top,
        right: x + mag_size - state.monitor_rect.left,
        bottom: y + mag_size - state.monitor_rect.top,
    };
    win32::gdi::fill_rect(hdc_mem, &local_rect, &brush_bg);

    let brush_border = ctx.cache.get_brush(0xFFFFFF)?;
    win32::gdi::frame_rect(hdc_mem, &local_rect, &brush_border);

    if let Some(hbm_bright) = &state.gdi.hbitmap_bright {
        // OPTIMIZATION: Reuse hdc_selection_src as a generic scratch DC
        if ctx.hdc_selection_src.is_none() {
             ctx.hdc_selection_src = Some(win32::gdi::create_compatible_dc(None)?);
        }
        
        let hdc_src = ctx.hdc_selection_src.as_ref().unwrap();
        
        let prev = win32::gdi::select_object(
            hdc_src,
            windows::Win32::Graphics::Gdi::HGDIOBJ(hbm_bright.0 .0),
        )?;
        let src_w = mag_size / zoom_factor;
        let src_h = mag_size / zoom_factor;
        let src_x = mouse_x - (src_w / 2) - state.capture_x;
        let src_y = mouse_y - (src_h / 2) - state.capture_y;

        let local_x = x - state.monitor_rect.left;
        let local_y = y - state.monitor_rect.top;

        unsafe {
            let _ = windows::Win32::Graphics::Gdi::StretchBlt(
                hdc_mem.0,
                local_x + 2,
                local_y + 2,
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

        // Cleanup selection only, keep DC
        win32::gdi::select_object(hdc_src, prev)?;
    }

    let mid_x = (x + mag_size / 2) - state.monitor_rect.left;
    let mid_y = (y + mag_size / 2) - state.monitor_rect.top;
    let local_x = x - state.monitor_rect.left;
    let local_y = y - state.monitor_rect.top;

    {
        let brush_cross = ctx.cache.get_brush(0x00D7FF)?;
        let cross_rect = RECT {
            left: mid_x - (zoom_factor / 2),
            top: local_y + 2,
            right: mid_x + (zoom_factor / 2),
            bottom: local_y + mag_size - 2,
        };
        win32::gdi::fill_rect(hdc_mem, &cross_rect, &brush_cross);

        let cross_rect_h = RECT {
            left: local_x + 2,
            top: mid_y - (zoom_factor / 2),
            right: local_x + mag_size - 2,
            bottom: mid_y + (zoom_factor / 2),
        };
        win32::gdi::fill_rect(hdc_mem, &cross_rect_h, &brush_cross);
    }

    Ok(())
}
