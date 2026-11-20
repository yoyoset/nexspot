// Shared commercial toolbar tokens used by both GDI and Vello renderers.
// Keep these values explicit so the two native render paths do not drift.

pub const COLOR_BG_MAIN: (u8, u8, u8, u8) = (20, 21, 25, 255);
pub const COLOR_BG_HOVER: (u8, u8, u8, u8) = (27, 28, 33, 255);
pub const COLOR_BG_PRESSED: (u8, u8, u8, u8) = (35, 36, 43, 255);
pub const COLOR_BORDER_NORMAL: (u8, u8, u8, u8) = (35, 36, 43, 255);
pub const COLOR_BORDER_LIGHT: (u8, u8, u8, u8) = (46, 47, 55, 255);
pub const COLOR_ACCENT: (u8, u8, u8, u8) = (122, 111, 242, 255);
pub const COLOR_TEXT_PRIMARY: (u8, u8, u8, u8) = (236, 236, 239, 255);
pub const COLOR_TEXT_SECONDARY: (u8, u8, u8, u8) = (154, 154, 165, 255);
pub const COLOR_ICON_MUTED: (u8, u8, u8, u8) = (212, 212, 216, 255);
pub const COLOR_BAD: (u8, u8, u8, u8) = (247, 109, 109, 255);
pub const COLOR_HIGHLIGHT: (u8, u8, u8, u8) = (255, 255, 255, 20);
pub const COLOR_SHADOW: (u8, u8, u8, u8) = (0, 0, 0, 50);

pub const RADIUS_CONTAINER: f32 = 11.0;
pub const RADIUS_WIDGET: f32 = 8.0;
pub const ICON_FONT_SIZE_GDI: f32 = 23.0;
pub const ICON_SCALE_VELLO: f64 = 1.15;
pub const ICON_STROKE_VELLO: f64 = 1.5;
pub const ICON_DETAIL_STROKE_VELLO: f64 = 1.3;

pub const BUTTON_SIZE: i32 = 44;
pub const BUTTON_SPACING: i32 = 2;
pub const TOOLBAR_MARGIN: i32 = 4;

pub const fn rgba_to_gdi(color: (u8, u8, u8, u8)) -> u32 {
    ((color.3 as u32) << 24) | ((color.0 as u32) << 16) | ((color.1 as u32) << 8) | color.2 as u32
}

pub fn rgba_to_vello(color: (u8, u8, u8, u8)) -> vello::peniko::Color {
    vello::peniko::Color::from_rgba8(color.0, color.1, color.2, color.3)
}
