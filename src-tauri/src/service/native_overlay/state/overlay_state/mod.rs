use super::drawing_object::DrawingObject;
use super::types::{
    CaptureEngine, CaptureMode, DrawingTool, HitZone, InteractionMode,
};
use windows::Win32::Foundation::RECT;

mod data;
mod logic;

pub use data::*;

#[derive(Debug)]
pub struct OverlayState {
    pub is_visible: bool,
    pub capture_x: i32,
    pub capture_y: i32,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub selection: Option<RECT>,
    pub interaction_mode: InteractionMode,
    pub hover_zone: HitZone,
    pub is_snapshot_mode: bool,
    pub start_x: i32,
    pub start_y: i32,
    pub capture_mode: CaptureMode,
    pub capture_engine: CaptureEngine,
    pub drag_start_selection: Option<RECT>,
    pub width: i32,
    pub height: i32,
    pub window_rects: Vec<RECT>,
    pub current_tool: DrawingTool,
    pub current_color: u32, // ARGB
    pub current_stroke: f32,
    pub current_font_size: f32,
    pub current_is_filled: bool,
    pub objects: Vec<DrawingObject>,
    pub current_drawing: Option<DrawingObject>,
    pub tool_registry: ToolRegistry,
    pub is_advanced_switching: bool,
    pub selected_object_index: Option<usize>,
    pub is_selection_active: bool,
    pub font_family: String,
    pub current_opacity: f32,
    pub current_glow: f32, // 0.0 to 1.0
    pub current_shadow: bool,
    pub enable_advanced_effects: bool,
    pub current_style: crate::service::config::types::AestheticStyle,
    pub active_workflow: Option<crate::service::config::types::CaptureWorkflow>,
    pub is_capturing: bool,
    pub monitor_id: String,
    pub monitor_rect: RECT,
    pub restrict_to_monitor: Option<RECT>,
    pub selection_pointer: Option<(i32, i32)>, 

    // Scrolling Capture
    pub is_scrolling: bool,
    pub scroll_stitched_path: Option<String>,

    // OCR Cache
    pub current_ocr_data: Option<crate::service::ocr::OcrResultData>,

    // Decoupled Engine Data
    pub gdi: GdiData,
    pub vello: VelloData,

    // Performance/Optimization Flags
    pub monitors: Vec<crate::service::win32::monitor::MonitorInfo>,
    pub snapping_dirty: bool,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            is_visible: false,
            capture_x: 0,
            capture_y: 0,
            mouse_x: 0,
            mouse_y: 0,
            selection: None,
            interaction_mode: InteractionMode::None,
            hover_zone: HitZone::None,
            is_snapshot_mode: false,
            start_x: 0,
            start_y: 0,
            drag_start_selection: None,
            capture_mode: CaptureMode::Standard,
            capture_engine: CaptureEngine::Gdi,
            width: 0,
            height: 0,
            window_rects: Vec::new(),
            current_tool: DrawingTool::None,
            current_color: 0xFFD14D4D, // Muted Red+
            current_stroke: 4.0,       // Default Medium (M)
            current_font_size: 24.0,   // Default Medium
            current_is_filled: false,
            objects: Vec::new(),
            current_drawing: None,
            tool_registry: ToolRegistry::default(),
            is_advanced_switching: false,
            selected_object_index: None,
            is_selection_active: false,
            font_family: "Segoe UI".to_string(),
            current_opacity: 1.0,
            current_glow: 0.0,
            current_shadow: false,
            enable_advanced_effects: true,
            current_style: crate::service::config::types::AestheticStyle::Default,
            active_workflow: None,
            is_capturing: false,
            monitor_id: String::new(),
            monitor_rect: RECT::default(),
            restrict_to_monitor: None,
            selection_pointer: None,
            gdi: GdiData::default(),
            vello: VelloData::default(),
            is_scrolling: false,
            scroll_stitched_path: None,
            current_ocr_data: None,
            monitors: Vec::new(),
            snapping_dirty: true,
        }
    }
}

unsafe impl Send for OverlayState {}
unsafe impl Sync for OverlayState {}
