pub mod gdi {
    // 工业级核心配色 (Zinc Palette)
    pub const BG_MAIN: u32 = 0xFF18181B;       // Zinc-950
    pub const BORDER_NORMAL: u32 = 0xFF27272A; // Zinc-800
    pub const BORDER_LIGHT: u32 = 0xFF3F3F46;  // Zinc-700
    pub const BG_HOVER: u32 = 0xFF27272A;
    pub const BG_ACTIVE: u32 = 0xFF3F3F46;     // Zinc-700
    pub const ACCENT_COLOR: u32 = 0xFF3B82F6;  // Blue-500
    pub const TEXT_PRIMARY: u32 = 0xFFE4E4E7;  // Zinc-200
    pub const TEXT_SECONDARY: u32 = 0xFFA1A1AA; // Zinc-400

    // 工业级几何参数 (Rationalist Geometry)
    pub const RADIUS_CONTAINER: f32 = 1.0;
    pub const RADIUS_WIDGET: f32 = 1.0;
    pub const GAP_SMALL: f32 = 2.0;

    // 属性栏尺寸
    pub const ITEM_WIDTH: f32 = 36.0;   // 压缩宽度
    pub const ITEM_HEIGHT: f32 = 24.0;
    pub const ITEM_STEP: f32 = 40.0;    // 压缩步进
    pub const ITEM_HPADDING: f32 = 2.0;
    pub const ITEM_VPADDING: f32 = 2.0;

    pub const COLOR_ITEM_SIZE: f32 = 24.0; // 压缩色块
    pub const COLOR_ITEM_STEP: f32 = 28.0; // 极高密度
    pub const COLOR_DOT_OFFSET_X: f32 = 1.0;
    pub const COLOR_DOT_OFFSET_Y: f32 = 5.0;

    pub const GROUP_GAP: f32 = 12.0;    // 压缩间距
    pub const PB_HPADDING: f32 = 6.0;
    pub const PB_VPADDING: f32 = 4.0;
    pub const DIVIDER_WIDTH: f32 = 2.0;
    pub const SLIDER_WIDTH: f32 = 80.0;
    pub const PB_HEIGHT: i32 = 34;     // Professional vertical compression
}
