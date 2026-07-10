pub mod gdi {
    use super::super::style;

    pub const BG_MAIN: u32 = style::rgba_to_gdi(style::COLOR_BG_MAIN);
    pub const BORDER_NORMAL: u32 = style::rgba_to_gdi(style::COLOR_BORDER_NORMAL);
    pub const BORDER_LIGHT: u32 = style::rgba_to_gdi(style::COLOR_BORDER_LIGHT);
    pub const BG_HOVER: u32 = style::rgba_to_gdi(style::COLOR_BG_HOVER);
    pub const BG_ACTIVE: u32 = style::rgba_to_gdi(style::COLOR_ACCENT);
    pub const ACCENT_COLOR: u32 = style::rgba_to_gdi(style::COLOR_ACCENT);
    pub const TEXT_PRIMARY: u32 = style::rgba_to_gdi(style::COLOR_TEXT_PRIMARY);
    pub const TEXT_SECONDARY: u32 = style::rgba_to_gdi(style::COLOR_TEXT_SECONDARY);

    pub const RADIUS_CONTAINER: f32 = style::RADIUS_CONTAINER;
    pub const RADIUS_WIDGET: f32 = style::RADIUS_WIDGET;
    pub const GAP_SMALL: f32 = 2.0;

    pub const ITEM_WIDTH: f32 = 36.0;
    pub const ITEM_HEIGHT: f32 = 24.0;
    pub const ITEM_STEP: f32 = 40.0;
    pub const ITEM_HPADDING: f32 = 2.0;
    pub const ITEM_VPADDING: f32 = 2.0;

    pub const COLOR_ITEM_SIZE: f32 = 24.0;
    pub const COLOR_ITEM_STEP: f32 = 28.0;
    pub const COLOR_DOT_OFFSET_X: f32 = 1.0;
    pub const COLOR_DOT_OFFSET_Y: f32 = 5.0;

    pub const GROUP_GAP: f32 = 12.0;
    pub const PB_HPADDING: f32 = 6.0;
    pub const PB_VPADDING: f32 = 4.0;
    pub const DIVIDER_WIDTH: f32 = 2.0;
    pub const SLIDER_WIDTH: f32 = 80.0;
    pub const PB_HEIGHT: i32 = 34;
}
