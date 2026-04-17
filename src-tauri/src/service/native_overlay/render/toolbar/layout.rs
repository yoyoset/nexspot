use super::tool_type_to_drawing_tool;
use super::types::{ButtonState, ToolType, ToolbarButton};
use windows::Win32::Foundation::RECT;

pub fn update_toolbar_layout(
    main_buttons: &mut [ToolbarButton],
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
    _engine: crate::service::native_overlay::state::CaptureEngine,
) -> bool {
    let sel_w = selection.right - selection.left;
    let sel_h = selection.bottom - selection.top;

    // Don't show if selection is too small
    if sel_w < 10 || sel_h < 10 {
        return false;
    }

    // --- 1. Main Toolbar Layout (Horizontal) ---
    let btn_count = main_buttons.len() as i32;
    let divider_count = main_buttons.iter().filter(|b| b.has_divider).count() as i32;

    let total_width = (btn_count * button_size)
        + ((btn_count - 1) * spacing)
        + (divider_count * spacing)
        + (margin * 2);
    let total_height = button_size + (margin * 2);

    // Position: Below selection, aligned to right
    let mut x = selection.right - total_width;
    let mut y = selection.bottom + 10;

    // Clamp X to window
    if x < window_x { x = window_x; }
    if x + total_width > window_x + window_width { x = window_x + window_width - total_width; }

    // Check Vertical constraints
    if y + total_height > window_y + window_height {
        y = selection.top - total_height - 10;
        if y < window_y {
            y = selection.bottom - total_height - 10;
        }
    }

    *rect = RECT {
        left: x,
        top: y,
        right: x + total_width,
        bottom: y + total_height,
    };

    let mut cur_x = x + margin;
    let start_y = y + margin;
    for btn in main_buttons.iter_mut() {
        btn.rect = RECT {
            left: cur_x,
            top: start_y,
            right: cur_x + button_size,
            bottom: start_y + button_size,
        };
        cur_x += button_size + spacing;
        if btn.has_divider { cur_x += spacing; }
    }


    // --- 3. Property Bar Positioning ---
    if let Some(tool) = current_tool {
        *property_bar_visible = true;
        let tool_enum = tool_type_to_drawing_tool(tool);
        
        let (item_step, color_step, pb_hpadding, group_gap, slider_width, pb_h) = if _engine == crate::service::native_overlay::state::CaptureEngine::Gdi {
            use super::constants::gdi::*;
            (ITEM_STEP as i32, COLOR_ITEM_STEP as i32, PB_HPADDING as i32, GROUP_GAP as i32, 80 /* SLIDER_WIDTH */, PB_HEIGHT)
        } else {
            use crate::service::native_overlay::render::vello_engine::renderer::ui::property_bar::constants::*;
            ((ITEM_SIZE + ITEM_GAP) as i32, (COLOR_ITEM_SIZE + COLOR_ITEM_GAP) as i32, PB_HPADDING as i32, GROUP_GAP as i32, SLIDER_WIDTH as i32, 40)
        };

        let mut pb_w = pb_hpadding * 2;

        if tool_enum == crate::service::native_overlay::state::DrawingTool::Text {
            // Tight accumulation: (count-1)*step + width
            pb_w += (2 * item_step) + 36 /* ITEM_WIDTH */; 
            if _engine == crate::service::native_overlay::state::CaptureEngine::Gdi { pb_w += 2; } // Compact Divider
        } else if tool_enum != crate::service::native_overlay::state::DrawingTool::None {
            let mut count = 3;
            if matches!(tool_enum, crate::service::native_overlay::state::DrawingTool::Rect | crate::service::native_overlay::state::DrawingTool::Ellipse) {
                count = 4;
            }
            pb_w += (count - 1) * item_step + 36 /* ITEM_WIDTH */;
            
            if tool_enum != crate::service::native_overlay::state::DrawingTool::Mosaic {
                 if _engine == crate::service::native_overlay::state::CaptureEngine::Gdi { pb_w += 2; } 
            }
        }

        if tool_enum != crate::service::native_overlay::state::DrawingTool::Mosaic {
            if pb_w > pb_hpadding * 2 {
                pb_w += group_gap;
            }
            // Tight color accumulation: 7 * step + size
            pb_w += (7 * color_step) + 24 /* COLOR_ITEM_SIZE */;
        }

        if enable_advanced_effects && _engine != crate::service::native_overlay::state::CaptureEngine::Gdi && tool_enum != crate::service::native_overlay::state::DrawingTool::Mosaic {
            pb_w += group_gap + 10 + slider_width + 10 + 8 + slider_width + 10;
        }

        let mut pb_x = x + (total_width / 2) - (pb_w / 2);
        let pb_y = y - pb_h - 14;

        // --- Clamp to Window Boundaries ---
        if pb_x < window_x { pb_x = window_x; }
        if pb_x + pb_w > window_x + window_width { pb_x = window_x + window_width - pb_w; }

        *property_bar_rect = RECT {
            left: pb_x,
            top: pb_y,
            right: pb_x + pb_w,
            bottom: pb_y + pb_h,
        };
    } else {
        *property_bar_visible = false;
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
