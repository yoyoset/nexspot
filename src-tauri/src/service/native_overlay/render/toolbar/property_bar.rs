use crate::service::native_overlay::render::toolbar::widgets;
use crate::service::native_overlay::state::DrawingTool;
use crate::service::win32::gdiplus::{
    draw_rounded_rectangle, fill_rounded_rectangle, BrushWrapper, GraphicsWrapper, PenWrapper,
};
use windows::Win32::Foundation::RECT;
use super::constants::gdi::*;

pub fn draw_property_bar(
    app: &tauri::AppHandle,
    graphics: &GraphicsWrapper,
    rect: &RECT,
    tool: DrawingTool,
    current_color: u32,
    current_font_size: f32,
    current_stroke: f32,
    current_is_filled: bool,
    _current_opacity: f32,
    _current_glow: f32,
) -> anyhow::Result<()> {
    // 关键修正：不再内部重建 GraphicsWrapper。复用传入的 graphics 以继承平移。

    // 1. Draw Background (Industrial Zinc Container)
    widgets::draw_rounded_container(graphics, rect, BG_MAIN, BORDER_NORMAL, RADIUS_CONTAINER)?;

    let mut offset_x = rect.left as f32 + PB_HPADDING;

    // --- Part 1: Font Size Selectors (Only for Text tool) ---
    if tool == DrawingTool::Text {
        let sizes = [14.0, 24.0, 36.0];
        let labels = [
            crate::service::l10n::t(app, "backend.property.size_small", "S"),
            crate::service::l10n::t(app, "backend.property.size_medium", "M"),
            crate::service::l10n::t(app, "backend.property.size_large", "L"),
        ];

        for (i, &size) in sizes.iter().enumerate() {
            let is_selected = (size - current_font_size).abs() < 1.0;
            let btn_rect = RECT {
                left: offset_x as i32,
                top: rect.top + ITEM_VPADDING as i32,
                right: (offset_x + ITEM_WIDTH) as i32,
                bottom: rect.bottom - ITEM_VPADDING as i32,
            };

            widgets::draw_property_selector(
                &graphics,
                &btn_rect,
                is_selected,
                Some(&labels[i]),
                TEXT_PRIMARY,
                ACCENT_COLOR,
            )?;
            offset_x += ITEM_STEP;
        }

        // Divider
        widgets::draw_separator_widget(
            &graphics,
            offset_x - (ITEM_STEP - ITEM_WIDTH) / 2.0,
            (rect.top + 10) as f32,
            offset_x - (ITEM_STEP - ITEM_WIDTH) / 2.0,
            (rect.bottom - 10) as f32,
            BORDER_NORMAL,
        )?;
        offset_x += DIVIDER_WIDTH;
    }

    // --- Part 2: Stroke Thickness (For Drawing tools) ---
    if tool != DrawingTool::Text && tool != DrawingTool::None {
        let strokes = [2.0, 4.0, 8.0];
        let dot_sizes = [3.0, 5.0, 8.0];

        for (i, &stroke) in strokes.iter().enumerate() {
            let is_selected = (stroke - current_stroke).abs() < 0.1;
            let btn_rect = RECT {
                left: offset_x as i32,
                top: rect.top + ITEM_VPADDING as i32,
                right: (offset_x + ITEM_WIDTH) as i32,
                bottom: rect.bottom - ITEM_VPADDING as i32,
            };

            // Draw selector background if selected
            let label_str: Option<String> = if tool == DrawingTool::Mosaic {
                match i {
                    0 => Some(crate::service::l10n::t(app, "backend.property.size_small", "S")),
                    1 => Some(crate::service::l10n::t(app, "backend.property.size_medium", "M")),
                    _ => Some(crate::service::l10n::t(app, "backend.property.size_large", "L")),
                }
            } else {
                None
            };
            let label = label_str.as_deref();
            
            widgets::draw_property_selector(
                &graphics,
                &btn_rect,
                is_selected,
                label,
                TEXT_PRIMARY,
                ACCENT_COLOR,
            )?;

            if tool != DrawingTool::Mosaic {
                // Draw thickness indicator dot — 选中态白点（叠在 accent 选中底上），否则次级灰
                let color = if is_selected {
                    0xFFFFFFFF
                } else {
                    TEXT_SECONDARY
                };
                let brush = BrushWrapper::new_solid(color)?;
                let cx = (btn_rect.left + btn_rect.right) as f32 / 2.0;
                let cy = (btn_rect.top + btn_rect.bottom) as f32 / 2.0;
                crate::service::win32::gdiplus::fill_ellipse(
                    &graphics,
                    &brush,
                    cx - dot_sizes[i],
                    cy - dot_sizes[i],
                    dot_sizes[i] * 2.0,
                    dot_sizes[i] * 2.0,
                )?;
            }

            offset_x += ITEM_STEP;
        }

        // --- Part 3: Fill Toggle (Rect/Ellipse) ---
        if matches!(tool, DrawingTool::Rect | DrawingTool::Ellipse) {
            let btn_rect = RECT {
                left: offset_x as i32,
                top: rect.top + ITEM_VPADDING as i32,
                right: (offset_x + ITEM_WIDTH) as i32,
                bottom: rect.bottom - ITEM_VPADDING as i32,
            };

            widgets::draw_property_selector(
                &graphics,
                &btn_rect,
                current_is_filled,
                None,
                TEXT_PRIMARY,
                ACCENT_COLOR,
            )?;

            // Draw Fill Icon — 选中(已填充)时白色叠在 accent 底上
            let color = if current_is_filled {
                0xFFFFFFFF
            } else {
                TEXT_SECONDARY
            };
            if current_is_filled {
                let brush = BrushWrapper::new_solid(color)?;
                fill_rounded_rectangle(
                    &graphics,
                    &brush,
                    (
                        (btn_rect.left + 8) as f32,
                        (btn_rect.top + 8) as f32,
                        16.0,
                        16.0,
                    ),
                    2.0,
                )?;
            } else {
                let pen = PenWrapper::new(color, 2.0)?;
                draw_rounded_rectangle(
                    &graphics,
                    &pen,
                    (
                        (btn_rect.left + 8) as f32,
                        (btn_rect.top + 8) as f32,
                        16.0,
                        16.0,
                    ),
                    2.0,
                )?;
            }

            offset_x += ITEM_STEP;
        }

        // Divider (Only if not mosaic, since mosaic has no following palette)
        if tool != DrawingTool::Mosaic {
            let div_x = offset_x - (ITEM_STEP - ITEM_WIDTH) / 2.0;
            widgets::draw_separator_widget(
                &graphics,
                div_x,
                (rect.top + 10) as f32,
                div_x,
                (rect.bottom - 10) as f32,
                BORDER_NORMAL,
            )?;
            offset_x += DIVIDER_WIDTH;
        }
    }

    // --- Part 4: Palette ---
    if tool != DrawingTool::Mosaic {
        if offset_x > (rect.left as f32 + PB_HPADDING + 1.0) {
            offset_x += GROUP_GAP;
        }

        let colors = get_palette_colors();

        for (i, &color) in colors.iter().enumerate() {
            let dot_x = offset_x + (i as f32 * COLOR_ITEM_STEP);
            let dot_rect = RECT {
                left: (dot_x + COLOR_DOT_OFFSET_X) as i32,
                top: (rect.top as f32 + COLOR_DOT_OFFSET_Y) as i32,
                right: (dot_x + COLOR_DOT_OFFSET_X + COLOR_ITEM_SIZE) as i32,
                bottom: (rect.top as f32 + COLOR_DOT_OFFSET_Y + COLOR_ITEM_SIZE) as i32,
            };

            widgets::draw_color_dot(
                &graphics,
                &dot_rect,
                color,
                current_color == color,
                ACCENT_COLOR,
            )?;
        }
    }

    Ok(())
}

pub fn get_palette_colors() -> Vec<u32> {
    vec![
        0xFFD14D4D, // Muted Red+ 
        0xFFD18A4D, // Muted Orange+
        0xFFC7B34D, // Muted Yellow+
        0xFF79A67B, // Muted Green+
        0xFF6A9DAA, // Muted Teal+
        0xFF6A86AA, // Muted Blue+
        0xFF8779A6, // Muted Purple+
        0xFFE4E4E7, // Off-white (Zinc-200)
    ]
}
