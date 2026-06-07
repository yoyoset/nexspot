use crate::service::native_overlay::render::toolbar::types::{
    ButtonState, PropertyChange, ToolType, ToolbarButton,
};
use crate::service::native_overlay::render::toolbar::{
    layout, property_bar, tool_type_to_drawing_tool,
};
use crate::service::native_overlay::state::{CaptureEngine, DrawingTool};

pub fn handle_mouse_move(buttons: &mut [ToolbarButton], x: i32, y: i32) -> bool {
    layout::handle_mouse_hit(buttons, x, y)
}

pub fn handle_mouse_down(buttons: &mut [ToolbarButton], x: i32, y: i32) -> bool {
    let mut handled = false;
    for btn in buttons {
        if x >= btn.rect.left && x < btn.rect.right && y >= btn.rect.top && y < btn.rect.bottom {
            btn.state = ButtonState::Pressed;
            handled = true;
        }
    }
    handled
}

pub fn handle_mouse_up(buttons: &mut [ToolbarButton], x: i32, y: i32) -> Option<ToolType> {
    let mut triggered = None;
    for btn in buttons {
        let hit =
            x >= btn.rect.left && x < btn.rect.right && y >= btn.rect.top && y < btn.rect.bottom;
        if hit && btn.state == ButtonState::Pressed {
            triggered = Some(btn.tool.clone());
        }
        btn.state = if hit {
            ButtonState::Hover
        } else {
            ButtonState::Normal
        };
    }
    triggered
}

pub fn handle_click(buttons: &[ToolbarButton], x: i32, y: i32) -> Option<ToolType> {
    for btn in buttons {
        if x >= btn.rect.left && x < btn.rect.right && y >= btn.rect.top && y < btn.rect.bottom {
            return Some(btn.tool.clone());
        }
    }
    None
}

pub fn handle_property_click(
    property_bar_visible: bool,
    property_bar_rect: &windows::Win32::Foundation::RECT,
    current_tool: &Option<ToolType>,
    x: i32,
    y: i32,
    enable_advanced_effects: bool,
    current_is_filled: bool,
    engine: CaptureEngine,
) -> Option<PropertyChange> {
    if !property_bar_visible {
        return None;
    }

    if x < property_bar_rect.left || x > property_bar_rect.right || y < property_bar_rect.top || y > property_bar_rect.bottom {
        return None;
    }

    // Constants branching based on Engine
    let (item_step, color_step, pb_hpadding, group_gap, slider_width) = if engine == CaptureEngine::Gdi {
        use super::constants::gdi::*;
        (ITEM_STEP, COLOR_ITEM_STEP, PB_HPADDING, GROUP_GAP, 80.0 /* SLIDER_WIDTH */)
    } else {
        use crate::service::native_overlay::render::vello_engine::renderer::ui::property_bar::constants::*;
        (ITEM_SIZE as f32 + ITEM_GAP as f32, COLOR_ITEM_SIZE as f32 + COLOR_ITEM_GAP as f32, PB_HPADDING as f32, GROUP_GAP as f32, SLIDER_WIDTH as f32)
    };
    
    let tool = current_tool.as_ref().map(tool_type_to_drawing_tool).unwrap_or(DrawingTool::None);
    let mut offset_x = property_bar_rect.left + pb_hpadding as i32;

    // Group 1: Base Tool Params
    let g1_start = offset_x;
    if matches!(tool, DrawingTool::Text) {
        let sizes = [14.0, 24.0, 36.0];
        for &size in &sizes {
            if x >= offset_x && x <= offset_x + (if engine == CaptureEngine::Gdi { 40 } else { 28 }) && y >= property_bar_rect.top + 6 && y <= property_bar_rect.bottom - 6 {
                return Some(PropertyChange::FontSize(size));
            }
            offset_x += item_step as i32;
        }
    } else if tool != DrawingTool::None {
        let strokes = [2.0, 4.0, 8.0];
        for &stroke in &strokes {
            if x >= offset_x && x <= offset_x + (if engine == CaptureEngine::Gdi { 40 } else { 28 }) && y >= property_bar_rect.top + 6 && y <= property_bar_rect.bottom - 6 {
                return Some(PropertyChange::Stroke(stroke));
            }
            offset_x += item_step as i32;
        }
        if matches!(tool, DrawingTool::Rect | DrawingTool::Ellipse) {
            if x >= offset_x && x <= offset_x + (if engine == CaptureEngine::Gdi { 40 } else { 28 }) && y >= property_bar_rect.top + 6 && y <= property_bar_rect.bottom - 6 {
                return Some(PropertyChange::Fill(!current_is_filled));
            }
            offset_x += item_step as i32;
        }
    }

    // Group 2: Colors
    if tool != DrawingTool::Mosaic {
        if offset_x > g1_start {
            offset_x += group_gap as i32;
        }
        let colors = property_bar::get_palette_colors();
        // 命中盒覆盖整步宽 + 整条高度，避免色块间隙/垂直错位导致点不中
        for color in colors {
            let hit_l = offset_x;
            let hit_r = offset_x + color_step as i32;
            if x >= hit_l && x < hit_r && y >= property_bar_rect.top && y <= property_bar_rect.bottom {
                return Some(PropertyChange::Color(color));
            }
            offset_x += color_step as i32;
        }
    }

    // Group 3: Advanced Effects
    if enable_advanced_effects && tool != DrawingTool::Mosaic {
        offset_x += group_gap as i32;
        let sw = slider_width as i32;
        
        // Opacity Slider
        offset_x += 10;
        if x >= offset_x && x <= offset_x + sw {
            let opacity = ((x - offset_x) as f32 / sw as f32).clamp(0.01, 1.0);
            return Some(PropertyChange::Opacity(opacity));
        }
    }

    None
}

pub fn handle_property_move(
    property_bar_visible: bool,
    property_bar_rect: &windows::Win32::Foundation::RECT,
    current_tool: &Option<ToolType>,
    x: i32,
    _y: i32,
    enable_advanced_effects: bool,
    is_dragging_opacity: bool,
    engine: CaptureEngine,
) -> Option<PropertyChange> {
    if !property_bar_visible || !enable_advanced_effects {
        return None;
    }
    if !is_dragging_opacity {
        return None;
    }

    let (item_step, color_step, pb_hpadding, group_gap, slider_width) = if engine == CaptureEngine::Gdi {
        use super::constants::gdi::*;
        (ITEM_STEP, COLOR_ITEM_STEP, PB_HPADDING, GROUP_GAP, SLIDER_WIDTH)
    } else {
        use crate::service::native_overlay::render::vello_engine::renderer::ui::property_bar::constants::*;
        (ITEM_SIZE as f32 + ITEM_GAP as f32, COLOR_ITEM_SIZE as f32 + COLOR_ITEM_GAP as f32, PB_HPADDING as f32, GROUP_GAP as f32, SLIDER_WIDTH as f32)
    };

    let tool = current_tool.as_ref().map(tool_type_to_drawing_tool).unwrap_or(DrawingTool::None);
    if tool == DrawingTool::Mosaic { return None; }

    let mut offset_x = property_bar_rect.left + pb_hpadding as i32;
    let g1_start = offset_x;

    // Skip Group 1
    if matches!(tool, DrawingTool::Text) {
        offset_x += (3.0 * item_step) as i32;
    } else if tool != DrawingTool::None {
        offset_x += (3.0 * item_step) as i32;
        if matches!(tool, DrawingTool::Rect | DrawingTool::Ellipse) {
            offset_x += item_step as i32;
        }
    }

    // Skip Group 2
    if tool != DrawingTool::Mosaic {
        if offset_x > g1_start {
            offset_x += group_gap as i32;
        }
        offset_x += (8.0 * color_step) as i32;
    }

    // Group 3
    offset_x += group_gap as i32;
    let sw = slider_width as i32;
    if is_dragging_opacity {
        offset_x += 10;
        let opacity = ((x - offset_x) as f32 / sw as f32).clamp(0.01, 1.0);
        return Some(PropertyChange::Opacity(opacity));
    }

    None
}

#[derive(Debug, PartialEq)]
pub enum PropertyHit {
    None,
    OpacitySlider,
    GlowSlider,
    Other,
}

pub fn hit_test_property_bar(
    property_bar_visible: bool,
    property_bar_rect: &windows::Win32::Foundation::RECT,
    current_tool: &Option<ToolType>,
    x: i32,
    y: i32,
    enable_advanced_effects: bool,
    engine: CaptureEngine,
) -> PropertyHit {
    if !property_bar_visible || x < property_bar_rect.left || x > property_bar_rect.right || y < property_bar_rect.top || y > property_bar_rect.bottom {
        return PropertyHit::None;
    }

    let (item_step, color_step, pb_hpadding, group_gap, slider_width) = if engine == CaptureEngine::Gdi {
        use super::constants::gdi::*;
        (ITEM_STEP, COLOR_ITEM_STEP, PB_HPADDING, GROUP_GAP, SLIDER_WIDTH)
    } else {
        use crate::service::native_overlay::render::vello_engine::renderer::ui::property_bar::constants::*;
        (ITEM_SIZE as f32 + ITEM_GAP as f32, COLOR_ITEM_SIZE as f32 + COLOR_ITEM_GAP as f32, PB_HPADDING as f32, GROUP_GAP as f32, SLIDER_WIDTH as f32)
    };

    let tool = current_tool.as_ref().map(tool_type_to_drawing_tool).unwrap_or(DrawingTool::None);
    if tool == DrawingTool::Mosaic || !enable_advanced_effects { return PropertyHit::Other; }

    let mut offset_x = property_bar_rect.left + pb_hpadding as i32;
    let g1_start = offset_x;

    // Skip Group 1
    if matches!(tool, DrawingTool::Text) {
        offset_x += (3.0 * item_step) as i32;
    } else if tool != DrawingTool::None {
        offset_x += (3.0 * item_step) as i32;
        if matches!(tool, DrawingTool::Rect | DrawingTool::Ellipse) { 
            offset_x += item_step as i32; 
        }
    }

    // Skip Group 2
    if tool != DrawingTool::Mosaic {
        if offset_x > g1_start {
            offset_x += group_gap as i32;
        }
        offset_x += (8.0 * color_step) as i32;
    }

    // Group 3
    offset_x += group_gap as i32;
    let sw = slider_width as i32;
    
    // Opacity
    offset_x += 10;
    if x >= offset_x && x <= offset_x + sw {
        return PropertyHit::OpacitySlider;
    }

    PropertyHit::Other
}
