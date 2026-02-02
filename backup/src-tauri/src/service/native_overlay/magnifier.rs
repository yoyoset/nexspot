use super::gdi::{AutoCreatedDC, AutoGdiObject, AutoSelectObject};
use super::state::OverlayState;
use windows::Win32::Foundation::{COLORREF, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateSolidBrush, FillRect, FrameRect, StretchBlt, HDC, SRCCOPY,
};

pub fn draw_magnifier(hdc_dest: HDC, x: i32, y: i32, state: &OverlayState) {
    let zoom_factor = 4;
    let box_size = 120;
    let source_size = box_size / zoom_factor;

    let margin = 20;
    let mut mag_rect = RECT {
        left: x + margin,
        top: y + margin,
        right: x + margin + box_size,
        bottom: y + margin + box_size,
    };

    if mag_rect.right > state.width {
        mag_rect.left = x - margin - box_size;
        mag_rect.right = x - margin;
    }
    if mag_rect.bottom > state.height {
        mag_rect.top = y - margin - box_size;
        mag_rect.bottom = y - margin;
    }

    unsafe {
        if let Ok(hdc_screen) = windows::Win32::Graphics::Gdi::GetDC(None) {
            if let Ok(hdc_src_raw) = CreateCompatibleDC(Some(hdc_screen)) {
                if let Some(src_dc_guard) = AutoCreatedDC::new(hdc_src_raw) {
                    let hdc_src = src_dc_guard.handle();
                    let _sel = AutoSelectObject::new(hdc_src, state.hbitmap_bright.handle());

                    if let Ok(hbrush_bg) = CreateSolidBrush(COLORREF(0x00000000)) {
                        let _ = FillRect(hdc_dest, &mag_rect, hbrush_bg);
                        let _ = windows::Win32::Graphics::Gdi::DeleteObject(hbrush_bg);
                    }

                    let _ = StretchBlt(
                        hdc_dest,
                        mag_rect.left,
                        mag_rect.top,
                        box_size,
                        box_size,
                        Some(hdc_src),
                        x - (source_size / 2),
                        y - (source_size / 2),
                        source_size,
                        source_size,
                        SRCCOPY,
                    );

                    if let Ok(hbrush_border) = CreateSolidBrush(COLORREF(0x00FFFFFF)) {
                        let _ = FrameRect(hdc_dest, &mag_rect, Some(hbrush_border));
                        let _ = windows::Win32::Graphics::Gdi::DeleteObject(hbrush_border);
                    }

                    let mid_x = mag_rect.left + box_size / 2;
                    let mid_y = mag_rect.top + box_size / 2;

                    if let Ok(hbrush_cross) = CreateSolidBrush(COLORREF(0x0000D7FF)) {
                        let r_pixel = RECT {
                            left: mid_x - (zoom_factor / 2),
                            top: mid_y - (zoom_factor / 2),
                            right: mid_x + (zoom_factor / 2),
                            bottom: mid_y + (zoom_factor / 2),
                        };
                        let _ = FrameRect(hdc_dest, &r_pixel, Some(hbrush_cross));
                        let _ = windows::Win32::Graphics::Gdi::DeleteObject(hbrush_cross);
                    }
                }
            }
            let _ = windows::Win32::Graphics::Gdi::ReleaseDC(None, hdc_screen);
        }
    }
}
