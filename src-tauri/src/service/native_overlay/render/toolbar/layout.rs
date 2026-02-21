use super::tool_type_to_drawing_tool;
use super::types::{ButtonState, ToolType, ToolbarButton};
use windows::Win32::Foundation::RECT;

pub fn update_toolbar_layout(
    buttons: &mut [ToolbarButton],
    rect: &mut RECT,
    current_tool: &Option<ToolType>,
    property_bar_visible: &mut bool,
    property_bar_rect: &mut RECT,
    window_x: i32,
    window_y: i32,
    window_width: i32,
    window_height: i32,
    selection: RECT,
    button_size: i32,
    margin: i32,
    spacing: i32,
    enable_advanced_effects: bool,
) -> bool {
    let sel_w = selection.right - selection.left;
    let sel_h = selection.bottom - selection.top;

    // Don't show if selection is too small
    if sel_w < 10 || sel_h < 10 {
        return false;
    }

    let btn_count = buttons.len() as i32;
    let divider_count = buttons.iter().filter(|b| b.has_divider).count() as i32;

    let total_width = (btn_count * button_size)
        + ((btn_count - 1) * spacing)
        + (divider_count * spacing)
        + (margin * 2);
    let total_height = button_size + (margin * 2);

    // Smart Positioning Logic
    let mut x = selection.right - total_width;
    let mut y = selection.bottom + 10;

    // Clamp X to screen (relative to current monitor window)
    let clamp_x = |mut val: i32| -> i32 {
        if val < window_x {
            val = window_x;
        }
        if val + total_width > window_x + window_width {
            val = window_x + window_width - total_width;
        }
        val
    };

    x = clamp_x(x);

    // Check Vertical constraints (relative to current monitor window)
    if y + total_height > window_y + window_height {
        y = selection.top - total_height - 10;
        if y < window_y {
            y = selection.bottom - total_height - 10;
        }
    }

    // Property Bar Positioning
    if let Some(tool) = current_tool {
        *property_bar_visible = true;

        // Calculate dynamic width based on tool
        let tool_enum = tool_type_to_drawing_tool(tool);
        let mut pb_w = 16; // Margins

        if tool_enum == crate::service::native_overlay::state::DrawingTool::Text {
            pb_w += 120 + 4; // Size selectors + divider
        } else if tool_enum != crate::service::native_overlay::state::DrawingTool::None {
            pb_w += 108; // 3 Thickness dots (36 * 3)
            if matches!(
                tool_enum,
                crate::service::native_overlay::state::DrawingTool::Rect
                    | crate::service::native_overlay::state::DrawingTool::Ellipse
            ) {
                pb_w += 36; // Fill toggle
            }
            if tool_enum != crate::service::native_overlay::state::DrawingTool::Mosaic {
                pb_w += 4; // Divider
            }
        }

        if tool_enum != crate::service::native_overlay::state::DrawingTool::Mosaic {
            pb_w += 256; // Color Palette (8 * 32)
        }

        if enable_advanced_effects {
            pb_w += 4 + 100 + 36; // Divider + Slider + Shadow Toggle
        }

        let pb_h = 40;

        let pb_x = x + (total_width / 2) - (pb_w / 2);
        let pb_y = y - pb_h - 10;

        *property_bar_rect = RECT {
            left: pb_x,
            top: pb_y,
            right: pb_x + pb_w,
            bottom: pb_y + pb_h,
        };
    } else {
        *property_bar_visible = false;
    }

    *rect = RECT {
        left: x,
        top: y,
        right: x + total_width,
        bottom: y + total_height,
    };

    // Layout Buttons
    let mut cur_x = x + margin;
    let start_y = y + margin;

    for btn in buttons.iter_mut() {
        btn.rect = RECT {
            left: cur_x,
            top: start_y,
            right: cur_x + button_size,
            bottom: start_y + button_size,
        };

        cur_x += button_size + spacing;
        if btn.has_divider {
            cur_x += spacing;
        }
    }

    true
}

pub fn handle_mouse_hit(buttons: &mut [ToolbarButton], x: i32, y: i32) -> bool {
    let mut needs_redraw = false;
    for btn in buttons {
        let hit =
            x >= btn.rect.left && x < btn.rect.right && y >= btn.rect.top && y < btn.rect.bottom;
        let new_state = if hit {
            if btn.state == ButtonState::Pressed {
                ButtonState::Pressed
            } else {
                ButtonState::Hover
            }
        } else {
            ButtonState::Normal
        };

        if btn.state != new_state {
            btn.state = new_state;
            needs_redraw = true;
        }
    }
    needs_redraw
}
