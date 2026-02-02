use crate::service::shortcut_manager;
use crate::service::win32::gdi::{self, SafeHDC};
use windows::Win32::Foundation::RECT;

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
            ToolbarButton {
                command: ToolbarCommand::Save,
                rect: RECT::default(),
                label: "Save".to_string(),
                icon: "\u{E105}".to_string(),
                bg_color: 0x00aaFF,
                hover_color: 0x00D7FF,
                is_hovered: false,
            },
            ToolbarButton {
                command: ToolbarCommand::Cancel,
                rect: RECT::default(),
                label: "Cancel".to_string(),
                icon: "\u{E106}".to_string(),
                bg_color: 0x333333,
                hover_color: 0x555555,
                is_hovered: false,
            },
        ];

        let shortcuts = shortcut_manager::load_shortcuts();
        for s in shortcuts {
            if s.enabled {
                let bgr = 0x888888;
                buttons.insert(
                    0,
                    ToolbarButton {
                        command: ToolbarCommand::Plugin(s.id),
                        rect: RECT::default(),
                        label: s.label,
                        icon: s.icon,
                        bg_color: bgr,
                        hover_color: bgr | 0x222222,
                        is_hovered: false,
                    },
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

    pub fn draw(&self, hdc: &SafeHDC) -> anyhow::Result<()> {
        if !self.visible {
            return Ok(());
        }

        gdi::set_bk_mode(hdc, windows::Win32::Graphics::Gdi::TRANSPARENT);

        // RAII Font
        {
            let hfont = gdi::create_font(14, 400, "Segoe UI")?;
            let _old_font =
                gdi::select_object(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(hfont.0 .0))?;

            for btn in &self.buttons {
                let color = if btn.is_hovered {
                    btn.hover_color
                } else {
                    btn.bg_color
                };
                let brush = gdi::create_solid_brush(color)?;
                gdi::fill_rect(hdc, &btn.rect, &brush);

                gdi::set_text_color(hdc, 0xFFFFFF);
                gdi::text_out(hdc, btn.rect.left + 5, btn.rect.top + 5, &btn.label)?;
            }
        }

        Ok(())
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn update_layout(&mut self, selection: RECT, window_width: i32, window_height: i32) {
        let width = selection.right - selection.left;
        let height = selection.bottom - selection.top;
        self.visible = width > 5 && height > 5;

        if !self.visible {
            return;
        }

        let total_width = (self.buttons.len() as i32 * self.button_width)
            + ((self.buttons.len() as i32 - 1) * self.spacing);

        let mut x = selection.right - total_width;
        if x < 0 {
            x = selection.left;
        }
        if x < 0 {
            x = 0;
        }
        if x + total_width > window_width {
            x = window_width - total_width;
        }

        let mut y = selection.bottom + self.margin;
        if y + self.button_height > window_height {
            y = selection.top - self.button_height - self.margin;
            if y < 0 {
                y = selection.bottom - self.button_height - self.margin;
                if y < 0 {
                    y = 0;
                }
            }
        }

        self.rect = RECT {
            left: x,
            top: y,
            right: x + total_width,
            bottom: y + self.button_height,
        };

        let mut curr_x = x;
        for btn in &mut self.buttons {
            btn.rect = RECT {
                left: curr_x,
                top: y,
                right: curr_x + self.button_width,
                bottom: y + self.button_height,
            };
            curr_x += self.button_width + self.spacing;
        }
    }

    pub fn handle_click(&self, x: i32, y: i32) -> Option<ToolbarCommand> {
        if !self.visible {
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
}

pub fn draw_toolbar(toolbar: &Toolbar, hdc: &SafeHDC) -> anyhow::Result<()> {
    toolbar.draw(hdc)
}
