use crate::service::native_overlay::render::toolbar::types::{
    ButtonState, ToolType, ToolbarButton,
};
use crate::service::native_overlay::render::toolbar::{
    property_bar, tool_type_to_drawing_tool, tooltip, widgets,
};
use crate::service::win32::gdi::SafeHDC;
use crate::service::win32::gdiplus::GraphicsWrapper;
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
    graphics: &GraphicsWrapper,
    _hdc: &SafeHDC,
    app: &tauri::AppHandle,
    current_color: u32,
    current_font_size: f32,
    current_stroke: f32,
    current_is_filled: bool,
    current_opacity: f32,
    current_glow: f32,
    orientation: crate::service::native_overlay::render::toolbar::builder::Orientation,
) -> anyhow::Result<()> {
    if !visible {
        return Ok(());
    }

    if is_loading {
        return draw_loading_message(rect, &graphics, app);
    }

    // 1. Draw Toolbar Background (Industrial Zinc)
    use super::constants::gdi::*;
    widgets::draw_rounded_container(&graphics, rect, BG_MAIN, BORDER_NORMAL, RADIUS_CONTAINER)?;

    // 2. Draw Buttons
    for btn in buttons {
        let is_active = current_tool.as_ref() == Some(&btn.tool);

        // Draw Button Background (Normal/Hover/Pressed/Active)
        widgets::draw_button_widget(&graphics, btn, is_active)?;

        // Draw Icon — 选中态白色（叠在 accent 填充上）；关闭键 bad 红（设计 §C）；否则常规前景色
        let icon_color = if is_active {
            0xFFFFFFFF
        } else if btn.tool == ToolType::Cancel {
            0xFFF76D6D // --bad
        } else {
            TEXT_PRIMARY
        };
        widgets::draw_tool_icon_widget(
            &graphics,
            &btn.rect,
            &btn.tool,
            &btn.icon,
            icon_color,
            if let ToolType::Number = btn.tool {
                28.0
            } else {
                30.0
            },
            current_color,
        )?;

        // Draw Divider
        if btn.has_divider {
            if orientation
                == crate::service::native_overlay::render::toolbar::builder::Orientation::Horizontal
            {
                let div_x = (btn.rect.right + (spacing / 2)) as f32;
                widgets::draw_separator_widget(
                    &graphics,
                    div_x,
                    (btn.rect.top + 8) as f32,
                    div_x,
                    (btn.rect.bottom - 8) as f32,
                    BORDER_NORMAL,
                )?;
            } else {
                let div_y = (btn.rect.bottom + (spacing / 2)) as f32;
                widgets::draw_separator_widget(
                    &graphics,
                    (btn.rect.left + 8) as f32,
                    div_y,
                    (btn.rect.right - 8) as f32,
                    div_y,
                    BORDER_NORMAL,
                )?;
            }
        }
    }

    // 3. Draw Hover Tooltip on Top (HDC based for now as it uses DrawTextW internally in some versions,
    // but the GDI+ refactor should eventually cover this too)
    // 3. Draw Hover Tooltip on Top
    for btn in buttons {
        if btn.state == ButtonState::Hover {
            tooltip::draw_tooltip(&graphics, btn, orientation)?;
            break;
        }
    }

    // 4. Draw Property Bar
    if property_bar_visible {
        if let Some(tool_type) = current_tool {
            let drawing_tool = tool_type_to_drawing_tool(tool_type);
            property_bar::draw_property_bar(
                app,
                graphics,
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

    Ok(())
}

fn draw_loading_message(
    rect: &RECT,
    graphics: &GraphicsWrapper,
    app: &tauri::AppHandle,
) -> anyhow::Result<()> {
    use crate::service::l10n;
    use crate::service::win32::gdiplus::BrushWrapper;

    // 1. Draw rounded background with indicator border
    use super::constants::gdi::*;
    widgets::draw_rounded_container(graphics, rect, BG_MAIN, ACCENT_COLOR, RADIUS_CONTAINER)?;

    // 2. Draw text
    let text = l10n::t(app, "engine.switching_to_advanced", "Switching to Advanced Engine...");
    let brush = BrushWrapper::new_solid(TEXT_PRIMARY)?;
    crate::service::win32::gdiplus::draw_text_centered(
        graphics,
        &text,
        (
            (rect.left + (rect.right - rect.left) / 2) as f32,
            (rect.top + (rect.bottom - rect.top) / 2) as f32,
        ),
        "Segoe UI",
        18.0,
        &brush,
        None,
    )?;

    Ok(())
}
