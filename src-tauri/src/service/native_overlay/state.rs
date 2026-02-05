use crate::service::win32::gdi::SafeHBITMAP;
use windows::Win32::Foundation::RECT;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitZone {
    None,
    Body,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureMode {
    Standard,
    Ocr,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionMode {
    None,
    Selecting,         // Drawing new box
    Moving,            // Dragging entire box
    Resizing(HitZone), // Dragging a handle
    Drawing,           // Drawing a shape (Rect, Arrow, etc.)
}

#[derive(Debug)]
pub struct OverlayState {
    pub is_visible: bool,
    pub x: i32,
    pub y: i32,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub selection: Option<RECT>,
    pub interaction_mode: InteractionMode,
    pub hover_zone: HitZone,

    // Dragging State
    pub start_x: i32,
    pub start_y: i32,
    pub capture_mode: CaptureMode,
    // Original selection when drag started (for move/resize)
    pub drag_start_selection: Option<RECT>,

    pub width: i32,
    pub height: i32,

    pub hbitmap_dim: Option<SafeHBITMAP>,
    pub hbitmap_bright: Option<SafeHBITMAP>,
    pub gdi_cache: crate::service::win32::gdi::GdiCache,
    // Snap targets (Window Bounds in virtual screen coords)
    pub window_rects: Vec<RECT>,

    // Drawing State
    pub current_tool: DrawingTool,
    pub current_color: u32, // ARGB
    pub current_stroke: f32,
    pub objects: Vec<DrawingObject>,
    pub current_drawing: Option<DrawingObject>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawingTool {
    None,
    Rect,
    Ellipse,
    Arrow,
    Line,
    Brush,
    Mosaic,
    Text,
    Number,
}

#[derive(Debug, Clone)]
pub struct DrawingObject {
    pub tool: DrawingTool,
    pub points: Vec<(i32, i32)>,
    pub text: Option<String>,
    pub color: u32,
    pub stroke_width: f32,
    pub is_filled: bool,
    pub is_dashed: bool,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            is_visible: false,
            x: 0,
            y: 0,
            mouse_x: 0,
            mouse_y: 0,
            selection: None,
            interaction_mode: InteractionMode::None,
            hover_zone: HitZone::None,
            start_x: 0,
            start_y: 0,
            drag_start_selection: None,
            capture_mode: CaptureMode::Standard,
            width: 0,
            height: 0,
            hbitmap_dim: None,
            hbitmap_bright: None,
            gdi_cache: crate::service::win32::gdi::GdiCache::new(),
            window_rects: Vec::new(),
            current_tool: DrawingTool::None,
            current_color: 0xFFFF0000, // Red
            current_stroke: 3.0,
            objects: Vec::new(),
            current_drawing: None,
        }
    }
}
unsafe impl Send for OverlayState {}
unsafe impl Sync for OverlayState {}

impl HitZone {
    pub fn detect(rect: &RECT, x: i32, y: i32) -> Self {
        let t = 10;
        let l = (x - rect.left).abs() <= t;
        let r = (x - rect.right).abs() <= t;
        let top = (y - rect.top).abs() <= t;
        let b = (y - rect.bottom).abs() <= t;

        if l && top {
            return HitZone::TopLeft;
        }
        if r && top {
            return HitZone::TopRight;
        }
        if l && b {
            return HitZone::BottomLeft;
        }
        if r && b {
            return HitZone::BottomRight;
        }

        if l && y >= rect.top - t && y <= rect.bottom + t {
            return HitZone::Left;
        }
        if r && y >= rect.top - t && y <= rect.bottom + t {
            return HitZone::Right;
        }
        if top && x >= rect.left - t && x <= rect.right + t {
            return HitZone::Top;
        }
        if b && x >= rect.left - t && x <= rect.right + t {
            return HitZone::Bottom;
        }

        if x > rect.left && x < rect.right && y > rect.top && y < rect.bottom {
            return HitZone::Body;
        }

        HitZone::None
    }
}
