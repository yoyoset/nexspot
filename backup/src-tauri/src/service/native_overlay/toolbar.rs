use super::gdi::{AutoGdiObject, AutoSelectObject};
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    CreateSolidBrush, DeleteObject, FillRect, SetBkMode, SetTextColor, TextOutW, BKMODE, HDC,
    HGDIOBJ,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ToolbarCommand {
    Save,
    Cancel,
    Plugin(String),
}

pub struct ToolbarButton {
    pub command: ToolbarCommand,
    pub rect: RECT,
    pub label: String,
    pub icon: String,
    pub bg_color: u32,
    pub hover_color: u32,
    pub is_hovered: bool,
}

impl ToolbarButton {
    pub fn new(command: ToolbarCommand, label: &str, icon: &str, bg: u32, hover: u32) -> Self {
        Self {
            command,
            rect: RECT::default(),
            label: label.to_string(),
            icon: icon.to_string(),
            bg_color: bg,
            hover_color: hover,
            is_hovered: false,
        }
    }
}

pub struct Toolbar {
    buttons: Vec<ToolbarButton>,
    pub rect: RECT,
    visible: bool,
    margin: i32,
    button_width: i32,
    button_height: i32,
    spacing: i32,
}

impl Toolbar {
    pub fn new() -> Self {
        let mut buttons = vec![
            ToolbarButton::new(
                ToolbarCommand::Save,
                "Save",
                "\u{E105}",
                0x00FFaa00,
                0x00FFD700,
            ),
            ToolbarButton::new(
                ToolbarCommand::Cancel,
                "Cancel",
                "\u{E106}",
                0x00333333,
                0x00555555,
            ),
        ];

        let shortcuts = crate::service::shortcut_manager::load_shortcuts();
        for s in shortcuts {
            if s.enabled {
                let r = u32::from_str_radix(&s.color[1..3], 16).unwrap_or(0);
                let g = u32::from_str_radix(&s.color[3..5], 16).unwrap_or(0);
                let b = u32::from_str_radix(&s.color[5..7], 16).unwrap_or(0);
                let bgr = (b << 16) | (g << 8) | r;

                buttons.insert(
                    0,
                    ToolbarButton::new(
                        ToolbarCommand::Plugin(s.id),
                        &s.label,
                        &s.icon,
                        bgr,
                        bgr | 0x222222,
                    ),
                );
            }
        }

        Self {
            buttons,
            rect: RECT::default(),
            visible: false,
            margin: 5,
            button_width: 80,
            button_height: 30,
            spacing: 5,
        }
    }

    pub fn update_layout(&mut self, selection: RECT, window_width: i32, window_height: i32) {
        if selection.right == 0 && selection.bottom == 0 {
            self.visible = false;
            return;
        }
        self.visible = true;

        let total_width = (self.buttons.len() as i32 * self.button_width)
            + ((self.buttons.len() as i32 - 1) * self.spacing);
        let total_height = self.button_height;

        let mut x = selection.right - total_width;
        let mut y = selection.bottom + self.margin;

        if y + total_height > window_height {
            y = selection.top - total_height - self.margin;
        }
        if y < 0 {
            y = selection.bottom - total_height - self.margin;
        }

        if x < 0 {
            x = 0;
        }
        if x + total_width > window_width {
            x = window_width - total_width;
        }

        self.rect = RECT {
            left: x,
            top: y,
            right: x + total_width,
            bottom: y + total_height,
        };

        let mut current_x = x;
        for btn in &mut self.buttons {
            btn.rect = RECT {
                left: current_x,
                top: y,
                right: current_x + self.button_width,
                bottom: y + self.button_height,
            };
            current_x += self.button_width + self.spacing;
        }
    }

    pub fn draw(&self, hdc: HDC) {
        if !self.visible {
            return;
        }

        unsafe {
            // SetBkMode takes HDC and BKMODE
            let _ = SetBkMode(hdc, windows::Win32::Graphics::Gdi::TRANSPARENT);

            if let Ok(hfont_symbol) = windows::Win32::Graphics::Gdi::CreateFontW(
                20,
                0,
                0,
                0,
                400,
                0,
                0,
                0,
                windows::Win32::Graphics::Gdi::DEFAULT_CHARSET.0 as u32,
                0,
                0,
                0,
                0,
                windows::core::w!("Segoe Fluent Icons"),
            ) {
                if let Ok(hfont_text) = windows::Win32::Graphics::Gdi::CreateFontW(
                    14,
                    0,
                    0,
                    0,
                    400,
                    0,
                    0,
                    0,
                    windows::Win32::Graphics::Gdi::DEFAULT_CHARSET.0 as u32,
                    0,
                    0,
                    0,
                    0,
                    windows::core::w!("Segoe UI"),
                ) {
                    let symbol_guard = AutoGdiObject::new(hfont_symbol);
                    let text_guard = AutoGdiObject::new(hfont_text);

                    for btn in &self.buttons {
                        let color = if btn.is_hovered {
                            btn.hover_color
                        } else {
                            btn.bg_color
                        };

                        if let Ok(hbrush_raw) =
                            CreateSolidBrush(windows::Win32::Foundation::COLORREF(color))
                        {
                            let _ = FillRect(hdc, &btn.rect, hbrush_raw);
                            let _ = DeleteObject(hbrush_raw);
                        }

                        if let Some(ref font) = symbol_guard {
                            let _select = AutoSelectObject::new(hdc, font.handle());
                            let _ =
                                SetTextColor(hdc, windows::Win32::Foundation::COLORREF(0x00FFFFFF));
                            let icon_w: Vec<u16> =
                                btn.icon.encode_utf16().chain(std::iter::once(0)).collect();
                            let _ = TextOutW(hdc, btn.rect.left + 10, btn.rect.top + 5, &icon_w);
                        }

                        if let Some(ref font) = text_guard {
                            let _select = AutoSelectObject::new(hdc, font.handle());
                            let _ =
                                SetTextColor(hdc, windows::Win32::Foundation::COLORREF(0x00FFFFFF));
                            let label_w: Vec<u16> =
                                btn.label.encode_utf16().chain(std::iter::once(0)).collect();
                            let _ = TextOutW(hdc, btn.rect.left + 35, btn.rect.top + 8, &label_w);
                        }
                    }
                }
            }
        }
    }

    pub fn hit_test(&self, x: i32, y: i32) -> Option<ToolbarCommand> {
        if !self.visible {
            return None;
        }
        if x < self.rect.left || x > self.rect.right || y < self.rect.top || y > self.rect.bottom {
            return None;
        }
        for btn in &self.buttons {
            if x >= btn.rect.left
                && x <= btn.rect.right
                && y >= btn.rect.top
                && y <= btn.rect.bottom
            {
                return Some(btn.command.clone());
            }
        }
        None
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32) -> bool {
        if !self.visible {
            return false;
        }
        let mut changed = false;
        for btn in &mut self.buttons {
            let hover = x >= btn.rect.left
                && x <= btn.rect.right
                && y >= btn.rect.top
                && y <= btn.rect.bottom;
            if btn.is_hovered != hover {
                btn.is_hovered = hover;
                changed = true;
            }
        }
        changed
    }
}
