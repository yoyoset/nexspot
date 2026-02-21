use crate::service::native_overlay::render::toolbar::types::{
    ButtonState, ToolType, ToolbarButton,
};
use crate::service::native_overlay::render::toolbar::{
    property_bar, tool_type_to_drawing_tool, tooltip,
};
use crate::service::win32::gdi::{self, SafeHDC};
use windows::Win32::Foundation::RECT;

pub fn draw_toolbar(
    buttons: &[ToolbarButton],
    rect: &RECT,
    visible: bool,
    is_loading: bool,
    current_tool: &Option<ToolType>,
    property_bar_visible: bool,
    property_bar_rect: &RECT,
    spacing: i32,
    hdc: &SafeHDC,
    app: &tauri::AppHandle,
    current_color: u32,
    current_font_size: f32,
    current_stroke: f32,
    current_is_filled: bool,
    current_opacity: f32,
    current_glow: f32,
) -> anyhow::Result<()> {
    if !visible {
        return Ok(());
    }

    if is_loading {
        return draw_loading_message(rect, hdc, app);
    }

    gdi::set_bk_mode(hdc, windows::Win32::Graphics::Gdi::TRANSPARENT);

    // 1. Draw Toolbar Background
    let bg_brush = gdi::create_solid_brush(0x222222)?;
    let border_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 1, 0x444444)?;
    let old_p = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(border_pen.0 .0))?;
    let old_b = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(bg_brush.0 .0))?;
    let _ = gdi::round_rect(hdc, rect.left, rect.top, rect.right, rect.bottom, 12, 12);
    gdi::select_object(hdc, old_b)?;
    gdi::select_object(hdc, old_p)?;

    // 2. Select Font for Icons
    let hfont = gdi::create_font(22, 400, "remixicon")?;
    let old_font = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(hfont.0 .0))?;

    for btn in buttons {
        let is_active = current_tool.as_ref() == Some(&btn.tool);

        // Draw Button Highlights
        if btn.state != ButtonState::Normal || is_active {
            let color = if btn.state == ButtonState::Pressed {
                0x555555
            } else if is_active {
                0x444444
            } else {
                0x3a3a3a
            };
            let brush = gdi::create_solid_brush(color)?;
            let old_b =
                gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(brush.0 .0))?;
            let null_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_NULL, 0, 0)?;
            let old_p =
                gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(null_pen.0 .0))?;
            let _ = gdi::round_rect(
                hdc,
                btn.rect.left,
                btn.rect.top,
                btn.rect.right,
                btn.rect.bottom,
                8,
                8,
            );
            gdi::select_object(hdc, old_p)?;
            gdi::select_object(hdc, old_b)?;
        }

        // Draw Icon
        let icon_color = if is_active { 0x00A0FF } else { 0xFFFFFF };
        gdi::set_text_color(hdc, icon_color);

        let mut icon_u16: Vec<u16> = btn.icon.encode_utf16().collect();
        unsafe {
            let text_rect = btn.rect;
            windows::Win32::Graphics::Gdi::DrawTextW(
                hdc.0,
                &mut icon_u16,
                &mut std::mem::transmute(text_rect),
                windows::Win32::Graphics::Gdi::DT_CENTER
                    | windows::Win32::Graphics::Gdi::DT_VCENTER
                    | windows::Win32::Graphics::Gdi::DT_SINGLELINE,
            );
        }

        // Draw Divider
        if btn.has_divider {
            let div_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 1, 0x444444)?;
            let old_p =
                gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(div_pen.0 .0))?;
            let div_x = btn.rect.right + (spacing / 2);
            let _ = gdi::move_to(hdc, div_x, btn.rect.top + 8);
            let _ = gdi::line_to(hdc, div_x, btn.rect.bottom - 8);
            gdi::select_object(hdc, old_p)?;
        }
    }

    // 3. Draw Hover Tooltip
    for btn in buttons {
        if btn.state == ButtonState::Hover {
            tooltip::draw_tooltip(hdc, btn)?;
            break;
        }
    }

    // 4. Draw Property Bar
    if property_bar_visible {
        if let Some(tool_type) = current_tool {
            let drawing_tool = tool_type_to_drawing_tool(tool_type);
            property_bar::draw_property_bar(
                hdc,
                property_bar_rect,
                drawing_tool,
                current_color,
                current_font_size,
                current_stroke,
                current_is_filled,
                current_opacity,
                current_glow,
            )?;
        }
    }

    gdi::select_object(hdc, old_font)?;
    Ok(())
}

fn draw_loading_message(rect: &RECT, hdc: &SafeHDC, app: &tauri::AppHandle) -> anyhow::Result<()> {
    use crate::service::l10n::{self, L10nKey};
    // Draw centered loading message
    let bg_brush = gdi::create_solid_brush(0x222222)?;
    let border_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 1, 0x00A0FF)?;
    let old_p = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(border_pen.0 .0))?;
    let old_b = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(bg_brush.0 .0))?;

    let _ = gdi::round_rect(hdc, rect.left, rect.top, rect.right, rect.bottom, 12, 12);

    gdi::set_text_color(hdc, 0xFFFFFF);
    let hfont = gdi::create_font(18, 400, "Segoe UI")?;
    let old_font = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(hfont.0 .0))?;

    let text = l10n::t(app, L10nKey::SwitchingToAdvanced);
    let mut text_u16: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        let mut r = *rect;
        windows::Win32::Graphics::Gdi::DrawTextW(
            hdc.0,
            &mut text_u16,
            &mut r,
            windows::Win32::Graphics::Gdi::DT_CENTER
                | windows::Win32::Graphics::Gdi::DT_VCENTER
                | windows::Win32::Graphics::Gdi::DT_SINGLELINE,
        );
    }

    gdi::select_object(hdc, old_font)?;
    gdi::select_object(hdc, old_b)?;
    gdi::select_object(hdc, old_p)?;
    Ok(())
}
