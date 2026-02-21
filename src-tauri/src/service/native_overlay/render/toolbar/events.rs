use crate::service::native_overlay::render::toolbar::types::{
    ButtonState, PropertyChange, ToolType, ToolbarButton,
};
use crate::service::native_overlay::render::toolbar::{
    layout, property_bar, tool_type_to_drawing_tool,
};
use crate::service::native_overlay::state::DrawingTool;

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
) -> Option<PropertyChange> {
    if !property_bar_visible {
        return None;
    }

    // Check if click is inside property bar
    if x < property_bar_rect.left
        || x > property_bar_rect.right
        || y < property_bar_rect.top
        || y > property_bar_rect.bottom
    {
        return None;
    }

    let tool = current_tool
        .as_ref()
        .map(tool_type_to_drawing_tool)
        .unwrap_or(DrawingTool::None);
    let mut offset_x = property_bar_rect.left + 8;

    // Part 1: Font Size (Only for Text)
    if matches!(tool, DrawingTool::Text) {
        let sizes = [14.0, 24.0, 36.0];
        for &size in &sizes {
            let r = windows::Win32::Foundation::RECT {
                left: offset_x,
                top: property_bar_rect.top + 6,
                right: offset_x + 32,
                bottom: property_bar_rect.bottom - 6,
            };
            if x >= r.left && x <= r.right && y >= r.top && y <= r.bottom {
                return Some(PropertyChange::FontSize(size));
            }
            offset_x += 40;
        }
        offset_x += 4; // Divider offset
    }

    // Part 1.5: Thickness
    if tool != DrawingTool::Text && tool != DrawingTool::None {
        let strokes = [2.0, 4.0, 8.0];
        for &stroke in &strokes {
            let r = windows::Win32::Foundation::RECT {
                left: offset_x,
                top: property_bar_rect.top + 6,
                right: offset_x + 32,
                bottom: property_bar_rect.bottom - 6,
            };
            if x >= r.left && x <= r.right && y >= r.top && y <= r.bottom {
                return Some(PropertyChange::Stroke(stroke));
            }
            offset_x += 36;
        }

        // Part 1.6: Fill Toggle (Rect/Ellipse)
        if matches!(tool, DrawingTool::Rect | DrawingTool::Ellipse) {
            let r = windows::Win32::Foundation::RECT {
                left: offset_x,
                top: property_bar_rect.top + 6,
                right: offset_x + 32,
                bottom: property_bar_rect.bottom - 6,
            };
            if x >= r.left && x <= r.right && y >= r.top && y <= r.bottom {
                // We return Fill(true) or Fill(false) - implementation logic will toggle if needed, or we explicitly pass "NOT CURRENT".
                // Actually, the handler just receives Fill, but it needs to know the NEW value.
                // We'll pass Fill(true) and let the handler toggle it in state.
                return Some(PropertyChange::Fill(true));
            }
            offset_x += 36;
        }
        offset_x += 4; // Divider
    }

    // Part 2: Colors
    if tool != DrawingTool::Mosaic {
        let colors = property_bar::get_palette_colors();
        for color in colors {
            let r = windows::Win32::Foundation::RECT {
                left: offset_x,
                top: property_bar_rect.top + 8,
                right: offset_x + 24,
                bottom: property_bar_rect.bottom - 8,
            };
            if x >= r.left && x <= r.right && y >= r.top && y <= r.bottom {
                return Some(PropertyChange::Color(color));
            }
            offset_x += 32;
        }
    }

    // Part 3: Advanced Effects
    if enable_advanced_effects && tool != DrawingTool::Mosaic {
        offset_x += 4; // Divider

        // --- Opacity Section ---
        offset_x += 10;
        let slider_width = 60;
        let slider_rect = windows::Win32::Foundation::RECT {
            left: offset_x,
            top: property_bar_rect.top,
            right: offset_x + slider_width,
            bottom: property_bar_rect.bottom,
        };

        if x >= slider_rect.left && x <= slider_rect.right {
            let relative_x = x - slider_rect.left;
            let opacity = (relative_x as f32 / slider_width as f32).clamp(0.01, 1.0);
            return Some(PropertyChange::Opacity(opacity));
        }
        offset_x += slider_width + 6;

        // Opacity Presets
        let presets = [0.25, 0.5, 1.0];
        for &p in &presets {
            let r = windows::Win32::Foundation::RECT {
                left: offset_x,
                top: property_bar_rect.top,
                right: offset_x + 20,
                bottom: property_bar_rect.bottom,
            };
            if x >= r.left && x <= r.right {
                return Some(PropertyChange::Opacity(p));
            }
            offset_x += 22;
        }

        // --- Glow Section ---
        offset_x += 8;
        let glow_slider_rect = windows::Win32::Foundation::RECT {
            left: offset_x,
            top: property_bar_rect.top,
            right: offset_x + slider_width,
            bottom: property_bar_rect.bottom,
        };

        if x >= glow_slider_rect.left && x <= glow_slider_rect.right {
            let relative_x = x - glow_slider_rect.left;
            let glow = (relative_x as f32 / slider_width as f32).clamp(0.0, 1.0);
            return Some(PropertyChange::Glow(glow));
        }
        offset_x += slider_width + 6;

        // Glow Presets
        for &p in &presets {
            let r = windows::Win32::Foundation::RECT {
                left: offset_x,
                top: property_bar_rect.top,
                right: offset_x + 20,
                bottom: property_bar_rect.bottom,
            };
            if x >= r.left && x <= r.right {
                return Some(PropertyChange::Glow(p));
            }
            offset_x += 22;
        }
    }

    None
}

#[allow(clippy::if_let_mutex)]
pub fn handle_property_move(
    property_bar_visible: bool,
    property_bar_rect: &windows::Win32::Foundation::RECT,
    current_tool: &Option<ToolType>,
    x: i32,
    _y: i32,
    enable_advanced_effects: bool,
    is_dragging_opacity: bool,
    is_dragging_glow: bool,
) -> Option<PropertyChange> {
    if !property_bar_visible || !enable_advanced_effects {
        return None;
    }

    if !is_dragging_opacity && !is_dragging_glow {
        return None;
    }

    let tool = current_tool
        .as_ref()
        .map(tool_type_to_drawing_tool)
        .unwrap_or(DrawingTool::None);

    if tool == DrawingTool::Mosaic {
        return None;
    }

    let mut offset_x = property_bar_rect.left + 8;

    // Skip Font Size
    if matches!(tool, DrawingTool::Text) {
        offset_x += 3 * 40;
        offset_x += 4;
    }

    // Skip Thickness
    if tool != DrawingTool::Text && tool != DrawingTool::None {
        offset_x += 3 * 36;
        if matches!(tool, DrawingTool::Rect | DrawingTool::Ellipse) {
            offset_x += 36;
        }
        offset_x += 4;
    }

    // Skip Colors
    let colors_count = property_bar::get_palette_colors().len();
    offset_x += colors_count as i32 * 32;

    // Advanced Effects Divider
    offset_x += 4;

    let slider_width = 60;

    if is_dragging_opacity {
        offset_x += 10;
        let relative_x = x - offset_x;
        let opacity = (relative_x as f32 / slider_width as f32).clamp(0.01, 1.0);
        return Some(PropertyChange::Opacity(opacity));
    }

    if is_dragging_glow {
        // Skip Opacity slider and presets
        offset_x += 10 + slider_width + 6 + (3 * 22) + 8;
        let relative_x = x - offset_x;
        let glow = (relative_x as f32 / slider_width as f32).clamp(0.0, 1.0);
        return Some(PropertyChange::Glow(glow));
    }

    None
}

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
) -> PropertyHit {
    if !property_bar_visible {
        return PropertyHit::None;
    }

    // Check if click is inside property bar
    if x < property_bar_rect.left
        || x > property_bar_rect.right
        || y < property_bar_rect.top
        || y > property_bar_rect.bottom
    {
        return PropertyHit::None;
    }

    let tool = current_tool
        .as_ref()
        .map(tool_type_to_drawing_tool)
        .unwrap_or(DrawingTool::None);

    if tool == DrawingTool::Mosaic {
        return PropertyHit::Other;
    }

    if !enable_advanced_effects {
        return PropertyHit::Other;
    }

    let mut offset_x = property_bar_rect.left + 8;

    // Skip Font Size
    if matches!(tool, DrawingTool::Text) {
        offset_x += 3 * 40;
        offset_x += 4;
    }

    // Skip Thickness
    if tool != DrawingTool::Text && tool != DrawingTool::None {
        offset_x += 3 * 36;
        if matches!(tool, DrawingTool::Rect | DrawingTool::Ellipse) {
            offset_x += 36;
        }
        offset_x += 4;
    }

    // Skip Colors
    let colors_count = property_bar::get_palette_colors().len();
    offset_x += colors_count as i32 * 32;

    // Advanced Effects Divider
    offset_x += 4;

    // Opacity Section
    offset_x += 10;
    let slider_width = 60;
    if x >= offset_x && x <= offset_x + slider_width {
        return PropertyHit::OpacitySlider;
    }
    offset_x += slider_width + 6 + (3 * 22) + 8; // Presets + Padding

    // Glow Section
    if x >= offset_x && x <= offset_x + slider_width {
        return PropertyHit::GlowSlider;
    }

    PropertyHit::Other
}
