use crate::service::native_overlay::render::toolbar::constants::gdi::*;
use crate::service::native_overlay::render::toolbar::types::{
    ButtonState, ToolType, ToolbarButton,
};
use windows::Win32::Foundation::RECT;

// Unified GDI+ helper to ensure visibility
use crate::service::win32::gdiplus as gdi;

pub fn draw_rounded_container(
    graphics: &gdi::GraphicsWrapper,
    rect: &RECT,
    bg_color: u32,
    border_color: u32,
    radius: f32,
) -> anyhow::Result<()> {
    // 1. Fill Background
    let bg_brush = gdi::BrushWrapper::new_solid(bg_color)?;
    gdi::fill_rounded_rectangle(
        graphics,
        &bg_brush,
        (
            rect.left as f32,
            rect.top as f32,
            (rect.right - rect.left) as f32,
            (rect.bottom - rect.top) as f32,
        ),
        radius,
    )?;

    // 2. Add Industrial Inner Highlight (1px top line, 8% white)
    let highlight_color = 0x14FFFFFF; 
    let highlight_pen = gdi::PenWrapper::new(highlight_color, 1.0)?;
    gdi::draw_line(
        graphics,
        &highlight_pen,
        rect.left as f32 + radius,
        rect.top as f32 + 1.0,
        rect.right as f32 - radius,
        rect.top as f32 + 1.0,
    )?;

    // 3. Add Industrial Bottom Edge (1px dark line, 20% black)
    let shadow_color = 0x33000000;
    let shadow_pen = gdi::PenWrapper::new(shadow_color, 1.0)?;
    gdi::draw_line(
        graphics,
        &shadow_pen,
        rect.left as f32 + radius,
        rect.bottom as f32 - 1.0,
        rect.right as f32 - radius,
        rect.bottom as f32 - 1.0,
    )?;

    // 4. Draw Outer Border (Zinc-800)
    let border_pen = gdi::PenWrapper::new(border_color, 1.0)?;
    gdi::draw_rounded_rectangle(
        graphics,
        &border_pen,
        (
            rect.left as f32,
            rect.top as f32,
            (rect.right - rect.left) as f32,
            (rect.bottom - rect.top) as f32,
        ),
        radius,
    )?;

    Ok(())
}

pub fn draw_button_widget(
    graphics: &gdi::GraphicsWrapper,
    btn: &ToolbarButton,
    is_active: bool,
) -> anyhow::Result<()> {
    if btn.state != ButtonState::Normal || is_active {
        let color = if btn.state == ButtonState::Pressed {
            BG_ACTIVE
        } else if is_active {
            BG_ACTIVE
        } else {
            BG_HOVER
        };

        let brush = gdi::BrushWrapper::new_solid(color)?;
        gdi::fill_rounded_rectangle(
            graphics,
            &brush,
            (
                btn.rect.left as f32,
                btn.rect.top as f32,
                (btn.rect.right - btn.rect.left) as f32,
                (btn.rect.bottom - btn.rect.top) as f32,
            ),
            RADIUS_WIDGET,
        )?;
        
        // Active Marker (Blue Indicator bar - 2px wide, 16px high, centered)
        if is_active {
            let marker_brush = gdi::BrushWrapper::new_solid(ACCENT_COLOR)?;
            let h = (btn.rect.bottom - btn.rect.top) as f32;
            let marker_h = 16.0;
            gdi::fill_rounded_rectangle(
                graphics,
                &marker_brush,
                (
                    btn.rect.left as f32 + 2.0,
                    btn.rect.top as f32 + (h - marker_h) / 2.0,
                    2.0,
                    marker_h,
                ),
                1.0,
            )?;
        }
    }
    Ok(())
}

pub fn draw_icon_widget(
    graphics: &gdi::GraphicsWrapper,
    rect: &RECT,
    icon: &str,
    color: u32,
    size: f32,
) -> anyhow::Result<()> {
    // Designer's Tip: Use Zinc-300 instead of pure white for a softer, more premium look
    let refined_color = if color == 0xFFFFFFFF { 0xFFD4D4D8 } else { color };
    let brush = gdi::BrushWrapper::new_solid(refined_color)?;
    gdi::draw_text_centered(
        graphics,
        icon,
        (
            (rect.left + (rect.right - rect.left) / 2) as f32,
            (rect.top + (rect.bottom - rect.top) / 2) as f32 + 1.0, // Shift down to balance large bottom margin
        ),
        "remixicon",
        size,
        &brush,
        None,
    )?;
    Ok(())
}

pub fn draw_tool_icon_widget(
    graphics: &gdi::GraphicsWrapper,
    rect: &RECT,
    tool: &ToolType,
    icon: &str,
    color: u32,
    size: f32,
    current_color: u32,
) -> anyhow::Result<()> {
    match tool {
        ToolType::Number => {
            let cx = (rect.left + rect.right) as f32 / 2.0;
            let cy = (rect.top + rect.bottom) as f32 / 2.0 - 4.0; // User requested another 1px up-shift
            let r = size * 0.45;

            // 1. Draw Circle
            let pen = gdi::PenWrapper::new(color, 2.0)?;
            gdi::draw_ellipse(graphics, &pen, cx - r, cy - r, r * 2.0, r * 2.0)?;

            // 2. Draw "1" inside
            let text_color = if current_color == 0xFFFFFFFF {
                0xFF000000
            } else {
                color
            };
            let brush = gdi::BrushWrapper::new_solid(text_color)?;
            gdi::draw_text_centered(
                graphics,
                "1",
                (cx, cy),
                "Segoe UI",
                size * 0.60,
                &brush,
                None,
            )?;
        }
        _ => {
            // Standard font icon
            draw_icon_widget(graphics, rect, icon, color, size)?;
        }
    }
    Ok(())
}

pub fn draw_separator_widget(
    graphics: &gdi::GraphicsWrapper,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: u32,
) -> anyhow::Result<()> {
    let pen = gdi::PenWrapper::new(color, 1.0)?;
    gdi::draw_line(graphics, &pen, x1, y1, x2, y2)?;
    Ok(())
}

pub fn draw_property_selector(
    graphics: &gdi::GraphicsWrapper,
    rect: &RECT,
    is_selected: bool,
    label: Option<&str>,
    _content_color: u32,
    _indicator_color: u32,
) -> anyhow::Result<()> {
    if is_selected {
        let brush = gdi::BrushWrapper::new_solid(BG_ACTIVE)?;
        gdi::fill_rounded_rectangle(
            graphics,
            &brush,
            (
                rect.left as f32,
                rect.top as f32,
                (rect.right - rect.left) as f32,
                (rect.bottom - rect.top) as f32,
            ),
            RADIUS_WIDGET,
        )?;

        // Active Marker (Bottom bar for property selectors - Slimmer 2px line)
        let marker_brush = gdi::BrushWrapper::new_solid(ACCENT_COLOR)?;
        gdi::fill_rounded_rectangle(
            graphics,
            &marker_brush,
            (
                rect.left as f32 + 10.0,
                rect.bottom as f32 - 3.0,
                (rect.right - rect.left) as f32 - 20.0,
                2.0,
            ),
            1.0,
        )?;
    }

    if let Some(text) = label {
        let text_color = if is_selected {
            TEXT_PRIMARY
        } else {
            TEXT_SECONDARY
        };
        let brush = gdi::BrushWrapper::new_solid(text_color)?;
        gdi::draw_text_centered(
            graphics,
            text,
            (
                (rect.left + (rect.right - rect.left) / 2) as f32,
                (rect.top + (rect.bottom - rect.top) / 2) as f32,
            ),
            "Segoe UI",
            14.0, // Smaller professional font
            &brush,
            None,
        )?;
    }

    Ok(())
}

pub fn draw_color_dot(
    graphics: &gdi::GraphicsWrapper,
    rect: &RECT,
    color: u32,
    is_selected: bool,
    _indicator_color: u32,
) -> anyhow::Result<()> {
    if is_selected {
        // 1. Professional Inset Selection Ring (White, 1px)
        let selection_pen = gdi::PenWrapper::new(0xFFFFFFFF, 1.0)?;
        gdi::draw_rounded_rectangle(
            graphics,
            &selection_pen,
            (
                (rect.left - 2) as f32,
                (rect.top - 2) as f32,
                (rect.right - rect.left + 4) as f32,
                (rect.bottom - rect.top + 4) as f32,
            ),
            RADIUS_WIDGET + 1.0,
        )?;
    }

    let brush = gdi::BrushWrapper::new_solid(color)?;
    let border_color = if is_selected {
        0xFFFFFFFF // Selection accent
    } else {
        0xFF333333 // Muted border
    };
    let border_pen = gdi::PenWrapper::new(border_color, 1.0)?;

    gdi::fill_rounded_rectangle(
        graphics,
        &brush,
        (
            rect.left as f32,
            rect.top as f32,
            (rect.right - rect.left) as f32,
            (rect.bottom - rect.top) as f32,
        ),
        RADIUS_WIDGET,
    )?;

    gdi::draw_rounded_rectangle(
        graphics,
        &border_pen,
        (
            rect.left as f32,
            rect.top as f32,
            (rect.right - rect.left) as f32,
            (rect.bottom - rect.top) as f32,
        ),
        RADIUS_WIDGET,
    )?;

    // De-AI: No checkmarks. Pure color block is more industrial.

    Ok(())
}
