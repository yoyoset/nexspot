pub mod layout;
pub mod property_bar;
pub mod tooltip;
pub mod types;

pub use types::{ButtonState, ToolType, ToolbarButton};

use crate::service::win32::gdi::{self, SafeHDC};
use windows::Win32::Foundation::RECT;

pub struct Toolbar {
    pub buttons: Vec<ToolbarButton>,
    pub rect: RECT,
    pub current_tool: Option<ToolType>,
    pub visible: bool,
    pub margin: i32,
    pub button_size: i32,
    pub spacing: i32,
    pub property_bar_visible: bool,
    pub property_bar_rect: RECT,
}

impl Toolbar {
    pub fn new() -> Self {
        let mut slf = Self {
            buttons: Vec::new(),
            rect: RECT::default(),
            current_tool: None,
            visible: false,
            margin: 8,
            button_size: 40,
            spacing: 4,
            property_bar_visible: false,
            property_bar_rect: RECT::default(),
        };
        slf.rebuild_for_mode(crate::service::native_overlay::state::CaptureMode::Standard);
        slf
    }

    pub fn rebuild_for_mode(&mut self, mode: crate::service::native_overlay::state::CaptureMode) {
        use crate::service::native_overlay::state::CaptureMode;
        let tools = match mode {
            CaptureMode::Standard => vec![
                (ToolType::Rect, "\u{EB7F}", "Rectangle", false),
                (ToolType::Ellipse, "\u{EB7D}", "Circle", false),
                (ToolType::Arrow, "\u{EA70}", "Arrow", false),
                (ToolType::Brush, "\u{EB01}", "Brush", false),
                (ToolType::Text, "\u{F201}", "Text", false),
                (ToolType::Mosaic, "\u{EDDF}", "Mosaic", false),
                (ToolType::More, "\u{EF77}", "More Tools", true),
                (ToolType::Pin, "\u{F039}", "Pin to Screen", false),
                (ToolType::Save, "\u{F0B3}", "Save to File", false),
                (ToolType::Copy, "\u{ECD5}", "Copy to Clipboard", false),
                (ToolType::Cancel, "\u{EB99}", "Cancel", false),
            ],
            CaptureMode::Ocr => vec![
                (ToolType::Ocr, "\u{E11B}", "Recognize Text", true),
                (ToolType::Cancel, "\u{EB99}", "Cancel", false),
            ],
        };

        let mut buttons = Vec::new();
        for (tool, icon, tip, divider) in tools {
            buttons.push(ToolbarButton {
                tool,
                rect: RECT::default(),
                state: ButtonState::Normal,
                icon: icon.to_string(),
                tooltip: tip.to_string(),
                has_divider: divider,
            });
        }
        self.buttons = buttons;
    }

    pub fn draw(&self, hdc: &SafeHDC) -> anyhow::Result<()> {
        if !self.visible {
            return Ok(());
        }

        gdi::set_bk_mode(hdc, windows::Win32::Graphics::Gdi::TRANSPARENT);

        // 1. Draw Toolbar Background
        let bg_brush = gdi::create_solid_brush(0x222222)?;
        let border_pen = gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 1, 0x444444)?;
        let old_p =
            gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(border_pen.0 .0))?;
        let old_b = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(bg_brush.0 .0))?;
        let _ = gdi::round_rect(
            hdc,
            self.rect.left,
            self.rect.top,
            self.rect.right,
            self.rect.bottom,
            12,
            12,
        );
        gdi::select_object(hdc, old_b)?;
        gdi::select_object(hdc, old_p)?;

        // 2. Select Font for Icons
        let hfont = gdi::create_font(22, 400, "remixicon")?;
        let old_font = gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(hfont.0 .0))?;

        for btn in &self.buttons {
            let is_active = self.current_tool == Some(btn.tool);

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
                let div_pen =
                    gdi::create_pen(windows::Win32::Graphics::Gdi::PS_SOLID, 1, 0x444444)?;
                let old_p =
                    gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(div_pen.0 .0))?;
                let div_x = btn.rect.right + (self.spacing / 2);
                let _ = gdi::move_to(hdc, div_x, btn.rect.top + 8);
                let _ = gdi::line_to(hdc, div_x, btn.rect.bottom - 8);
                gdi::select_object(hdc, old_p)?;
            }
        }

        // 3. Draw Hover Tooltip
        for btn in &self.buttons {
            if btn.state == ButtonState::Hover {
                tooltip::draw_tooltip(hdc, btn)?;
                break;
            }
        }

        // 4. Draw Property Bar
        if self.property_bar_visible {
            property_bar::draw_property_bar(hdc, &self.property_bar_rect)?;
        }

        gdi::select_object(hdc, old_font)?;
        Ok(())
    }

    pub fn hide(&mut self) {
        self.visible = false;
        for btn in &mut self.buttons {
            btn.state = ButtonState::Normal;
        }
    }

    pub fn update_layout(&mut self, selection: RECT, window_width: i32, window_height: i32) {
        layout::update_toolbar_layout(
            &mut self.buttons,
            &mut self.rect,
            self.current_tool,
            &mut self.property_bar_visible,
            &mut self.property_bar_rect,
            window_width,
            window_height,
            selection,
            self.button_size,
            self.margin,
            self.spacing,
        );
        self.visible = self.rect.right - self.rect.left > 0;
    }

    pub fn handle_mouse_move(&mut self, x: i32, y: i32) -> bool {
        if !self.visible {
            return false;
        }
        layout::handle_mouse_hit(&mut self.buttons, x, y)
    }

    pub fn handle_mouse_down(&mut self, x: i32, y: i32) -> bool {
        if !self.visible {
            return false;
        }
        let mut handled = false;
        for btn in &mut self.buttons {
            if x >= btn.rect.left && x < btn.rect.right && y >= btn.rect.top && y < btn.rect.bottom
            {
                btn.state = ButtonState::Pressed;
                handled = true;
            }
        }
        handled
    }

    pub fn handle_mouse_up(&mut self, x: i32, y: i32) -> Option<ToolType> {
        if !self.visible {
            return None;
        }
        let mut triggered = None;
        for btn in &mut self.buttons {
            let hit = x >= btn.rect.left
                && x < btn.rect.right
                && y >= btn.rect.top
                && y < btn.rect.bottom;
            if hit && btn.state == ButtonState::Pressed {
                triggered = Some(btn.tool);
            }
            btn.state = if hit {
                ButtonState::Hover
            } else {
                ButtonState::Normal
            };
        }
        triggered
    }

    pub fn handle_click(&mut self, x: i32, y: i32) -> Option<ToolType> {
        for btn in &self.buttons {
            if x >= btn.rect.left && x < btn.rect.right && y >= btn.rect.top && y < btn.rect.bottom
            {
                return Some(btn.tool);
            }
        }
        None
    }
}
