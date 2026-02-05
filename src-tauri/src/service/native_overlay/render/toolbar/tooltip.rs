use super::types::ToolbarButton;
use crate::service::win32::gdi::{self, SafeHDC};
use windows::Win32::Foundation::RECT;

pub fn draw_tooltip(hdc: &SafeHDC, btn: &ToolbarButton) -> anyhow::Result<()> {
    let text = &btn.tooltip;
    if text.is_empty() {
        return Ok(());
    }

    // Select a small UI font
    let hfont = gdi::create_font(14, 400, "Microsoft YaHei")?;
    let old_font = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(hfont.0 .0))?;

    // Measure text
    let mut rect = RECT::default();
    let mut u16_text: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        windows::Win32::Graphics::Gdi::DrawTextW(
            hdc.0,
            &mut u16_text,
            &mut rect,
            windows::Win32::Graphics::Gdi::DT_CALCRECT,
        );
    }

    let padding_h = 10;
    let padding_v = 6;
    let tw = rect.right - rect.left + padding_h * 2;
    let th = rect.bottom - rect.top + padding_v * 2;

    let tx = btn.rect.left + (btn.rect.right - btn.rect.left) / 2 - tw / 2;
    let mut ty = btn.rect.top - th - 12;

    // If tooltip is going OOB at top, show below?
    if ty < 0 {
        ty = btn.rect.bottom + 12;
    }

    let tooltip_rect = RECT {
        left: tx,
        top: ty,
        right: tx + tw,
        bottom: ty + th,
    };

    // Draw Background
    let bg_brush = gdi::create_solid_brush(0x1a1a1a)?;
    let border_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 1, 0x444444)?;
    let old_p = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(border_pen.0 .0))?;
    let old_b = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(bg_brush.0 .0))?;

    let _ = gdi::round_rect(
        hdc,
        tooltip_rect.left,
        tooltip_rect.top,
        tooltip_rect.right,
        tooltip_rect.bottom,
        6,
        6,
    );

    gdi::select_object(hdc, old_p)?;
    gdi::select_object(hdc, old_b)?;

    // Draw Text
    gdi::set_text_color(hdc, 0xFFFFFF);
    unsafe {
        let text_rect = tooltip_rect;
        windows::Win32::Graphics::Gdi::DrawTextW(
            hdc.0,
            &mut u16_text,
            &mut std::mem::transmute(text_rect),
            windows::Win32::Graphics::Gdi::DT_CENTER
                | windows::Win32::Graphics::Gdi::DT_VCENTER
                | windows::Win32::Graphics::Gdi::DT_SINGLELINE,
        );
    }

    gdi::select_object(hdc, old_font)?;
    Ok(())
}
