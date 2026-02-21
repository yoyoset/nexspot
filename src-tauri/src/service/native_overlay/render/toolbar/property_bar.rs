use crate::service::native_overlay::state::DrawingTool;
use crate::service::win32::gdi::{self, SafeHDC};
use windows::Win32::Foundation::RECT;

pub fn draw_property_bar(
    hdc: &SafeHDC,
    rect: &RECT,
    tool: DrawingTool,
    current_color: u32,
    current_font_size: f32,
    current_stroke: f32,
    current_is_filled: bool,
    _current_opacity: f32,
    _current_glow: f32,
) -> anyhow::Result<()> {
    // Draw property bar background
    let bg_brush = gdi::create_solid_brush(0x222222)?;
    let border_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 1, 0x444444)?;
    let old_p = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(border_pen.0 .0))?;
    let old_b = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(bg_brush.0 .0))?;

    let _ = gdi::round_rect(hdc, rect.left, rect.top, rect.right, rect.bottom, 10, 10);

    // Layout configuration
    let mut offset_x = rect.left + 8;

    // --- Part 1: Font Size Selectors (Only for Text tool) ---
    if tool == DrawingTool::Text {
        let sizes = [14.0, 24.0, 36.0];
        let labels = ["S", "M", "L"];

        for (i, &size) in sizes.iter().enumerate() {
            let is_selected = (size - current_font_size).abs() < 1.0;
            let btn_rect = windows::Win32::Foundation::RECT {
                left: offset_x,
                top: rect.top + 6,
                right: offset_x + 32,
                bottom: rect.bottom - 6,
            };

            if is_selected {
                let sel_brush = gdi::create_solid_brush(0x444444)?;
                let old_sb = gdi::select_object(
                    hdc,
                    windows::Win32::Graphics::Gdi::HGDIOBJ(sel_brush.0 .0),
                )?;
                let _ = gdi::round_rect(
                    hdc,
                    btn_rect.left,
                    btn_rect.top,
                    btn_rect.right,
                    btn_rect.bottom,
                    8,
                    8,
                );
                gdi::select_object(hdc, old_sb)?;
            }

            gdi::set_text_color(hdc, if is_selected { 0x00A0FF } else { 0xFFFFFF });
            let hfont = gdi::create_font(20, if is_selected { 700 } else { 400 }, "Segoe UI")?;
            let old_f =
                gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(hfont.0 .0))?;

            let mut label_u16: Vec<u16> = labels[i].encode_utf16().collect();
            unsafe {
                let mut r = btn_rect;
                windows::Win32::Graphics::Gdi::DrawTextW(
                    hdc.0,
                    &mut label_u16,
                    &mut r,
                    windows::Win32::Graphics::Gdi::DT_CENTER
                        | windows::Win32::Graphics::Gdi::DT_VCENTER
                        | windows::Win32::Graphics::Gdi::DT_SINGLELINE,
                );
            }
            gdi::select_object(hdc, old_f)?;
            offset_x += 40;
        }

        // Divider
        let div_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 1, 0x444444)?;
        let old_dp = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(div_pen.0 .0))?;
        let _ = gdi::move_to(hdc, offset_x - 4, rect.top + 10);
        let _ = gdi::line_to(hdc, offset_x - 4, rect.bottom - 10);
        gdi::select_object(hdc, old_dp)?;
        offset_x += 4;
    }

    // --- Part 1.5: Stroke Thickness (For Draw tools except Text) ---
    if tool != DrawingTool::Text && tool != DrawingTool::None {
        let strokes = [2.0, 4.0, 8.0];
        let sizes = [4, 7, 10]; // Dot sizes for visual feedback

        for (i, &stroke) in strokes.iter().enumerate() {
            let is_selected = (stroke - current_stroke).abs() < 0.1;
            let btn_rect = windows::Win32::Foundation::RECT {
                left: offset_x,
                top: rect.top + 6,
                right: offset_x + 32,
                bottom: rect.bottom - 6,
            };

            if is_selected {
                let sel_brush = gdi::create_solid_brush(0x444444)?;
                let old_sb = gdi::select_object(
                    hdc,
                    windows::Win32::Graphics::Gdi::HGDIOBJ(sel_brush.0 .0),
                )?;
                let _ = gdi::round_rect(
                    hdc,
                    btn_rect.left,
                    btn_rect.top,
                    btn_rect.right,
                    btn_rect.bottom,
                    8,
                    8,
                );
                gdi::select_object(hdc, old_sb)?;
            }

            // Draw a circle of representative size
            let cx = (btn_rect.left + btn_rect.right) / 2;
            let cy = (btn_rect.top + btn_rect.bottom) / 2;
            let dot_size = sizes[i];

            let brush = gdi::create_solid_brush(if is_selected { 0x00A0FF } else { 0x888888 })?;
            let old_b =
                gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(brush.0 .0))?;
            let null_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_NULL, 0, 0)?;
            let old_p =
                gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(null_pen.0 .0))?;
            let _ = gdi::ellipse(
                hdc,
                cx - dot_size,
                cy - dot_size,
                cx + dot_size,
                cy + dot_size,
            );
            gdi::select_object(hdc, old_p)?;
            gdi::select_object(hdc, old_b)?;

            offset_x += 36;
        }

        // --- Part 1.6: Fill Toggle (Only for Rect and Ellipse) ---
        if matches!(tool, DrawingTool::Rect | DrawingTool::Ellipse) {
            let btn_rect = windows::Win32::Foundation::RECT {
                left: offset_x,
                top: rect.top + 6,
                right: offset_x + 32,
                bottom: rect.bottom - 6,
            };

            if current_is_filled {
                let sel_brush = gdi::create_solid_brush(0x444444)?;
                let old_sb = gdi::select_object(
                    hdc,
                    windows::Win32::Graphics::Gdi::HGDIOBJ(sel_brush.0 .0),
                )?;
                let _ = gdi::round_rect(
                    hdc,
                    btn_rect.left,
                    btn_rect.top,
                    btn_rect.right,
                    btn_rect.bottom,
                    8,
                    8,
                );
                gdi::select_object(hdc, old_sb)?;
            }

            // Draw Icon (Rect icon, filled or outline)
            let color = if current_is_filled {
                0x00A0FF
            } else {
                0x888888
            };
            let pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 2, color)?;
            let old_p = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(pen.0 .0))?;

            if current_is_filled {
                let brush = gdi::create_solid_brush(color)?;
                let old_b =
                    gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(brush.0 .0))?;
                let _ = gdi::round_rect(
                    hdc,
                    btn_rect.left + 8,
                    btn_rect.top + 8,
                    btn_rect.right - 8,
                    btn_rect.bottom - 8,
                    2,
                    2,
                );
                gdi::select_object(hdc, old_b)?;
            } else {
                let _ = gdi::round_rect(
                    hdc,
                    btn_rect.left + 8,
                    btn_rect.top + 8,
                    btn_rect.right - 8,
                    btn_rect.bottom - 8,
                    2,
                    2,
                );
            }
            gdi::select_object(hdc, old_p)?;

            offset_x += 36;
        }

        // Divider
        let div_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 1, 0x444444)?;
        let old_dp = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(div_pen.0 .0))?;
        let _ = gdi::move_to(hdc, offset_x - 4, rect.top + 10);
        let _ = gdi::line_to(hdc, offset_x - 4, rect.bottom - 10);
        gdi::select_object(hdc, old_dp)?;
        offset_x += 4;
    }

    if tool != DrawingTool::Mosaic {
        let colors = get_palette_colors();
        for color in colors {
            let is_selected = color == current_color;
            let r = windows::Win32::Foundation::RECT {
                left: offset_x,
                top: rect.top + 8,
                right: offset_x + 24,
                bottom: rect.bottom - 8,
            };

            if is_selected {
                let sel_brush = gdi::create_solid_brush(0x444444)?;
                let old_sb = gdi::select_object(
                    hdc,
                    windows::Win32::Graphics::Gdi::HGDIOBJ(sel_brush.0 .0),
                )?;
                let _ =
                    gdi::round_rect(hdc, r.left - 2, r.top - 2, r.right + 2, r.bottom + 2, 6, 6);
                gdi::select_object(hdc, old_sb)?;
            }

            let c_brush = gdi::create_solid_brush(color)?;
            let c_pen = gdi::create_pen(
                windows::Win32::Graphics::Gdi::PS_SOLID,
                1,
                if is_selected { 0x00A0FF } else { 0x888888 },
            )?;
            let old_cb =
                gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(c_brush.0 .0))?;
            let old_cp =
                gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(c_pen.0 .0))?;
            let _ = gdi::round_rect(hdc, r.left, r.top, r.right, r.bottom, 4, 4);

            if is_selected {
                let dot_brush = gdi::create_solid_brush(0xFFFFFF)?;
                let old_db = gdi::select_object(
                    hdc,
                    windows::Win32::Graphics::Gdi::HGDIOBJ(dot_brush.0 .0),
                )?;
                let cx = (r.left + r.right) / 2;
                let cy = (r.top + r.bottom) / 2;
                let _ = gdi::ellipse(hdc, cx - 2, cy - 2, cx + 2, cy + 2);
                gdi::select_object(hdc, old_db)?;
            }

            gdi::select_object(hdc, old_cp)?;
            gdi::select_object(hdc, old_cb)?;
            offset_x += 32;
        }
    }

    gdi::select_object(hdc, old_p)?;
    gdi::select_object(hdc, old_b)?;
    Ok(())
}

pub fn get_palette_colors() -> Vec<u32> {
    vec![
        0xFFFF3B30, // Red (Premium)
        0xFFFFCC00, // Yellow
        0xFF07C160, // WeChat Green
        0xFF007AFF, // iOS Blue
        0xFF5AC8FA, // Cyan
        0xFF733AE8, // Purple
        0xFFFFFFFF, // White
        0xFF333333, // Premium Black
    ]
}
