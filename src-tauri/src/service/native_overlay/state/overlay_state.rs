use super::drawing_object::DrawingObject;
use super::types::{
    CaptureEngine, CaptureMode, DrawingTool, HitZone, InteractionMode, PropertyChange,
};
use crate::service::win32::gdi::SafeHBITMAP;
use vello::peniko::ImageData;
use windows::Win32::Foundation::RECT;

#[derive(Debug, Default)]
pub struct GdiData {
    pub hbitmap_dim: Option<SafeHBITMAP>,
    pub hbitmap_bright: Option<SafeHBITMAP>,
    pub cache: crate::service::win32::gdi::GdiCache,
}

#[derive(Debug, Default)]
pub struct VelloData {
    pub background: Option<ImageData>,
    pub scene: Option<VelloSceneWrapper>,
    pub d3d_texture: Option<windows::Win32::Graphics::Direct3D11::ID3D11Texture2D>,
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
    pub pending_ai_prompt: Option<String>,
    pub active_workflow: Option<crate::service::config::types::CaptureWorkflow>,
    pub is_capturing: bool,
    pub restrict_to_monitor: Option<RECT>,

    // Decoupled Engine Data
    pub gdi: GdiData,
    pub vello: VelloData,
}

#[derive(Debug, Clone)]
pub struct AiMacro {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub prompt: String,
}

#[derive(Debug, Default, Clone)]
pub struct ToolRegistry {
    pub macros: Vec<AiMacro>,
}

#[derive(Clone)]
pub struct VelloSceneWrapper(pub vello::Scene);
impl std::fmt::Debug for VelloSceneWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VelloScene").finish()
    }
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
            current_color: 0xFFFF3B30, // Red
            current_stroke: 5.0,
            current_font_size: 24.0, // Default Medium
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
            pending_ai_prompt: None,
            active_workflow: None,
            is_capturing: false,
            restrict_to_monitor: None,
            gdi: GdiData::default(),
            vello: VelloData::default(),
        }
    }
}

impl OverlayState {
    pub fn apply_property_change(&mut self, change: PropertyChange) {
        match change {
            PropertyChange::Color(color) => {
                self.current_color = color;
                if let Some(drawing) = self.objects.iter_mut().rev().find(|o| o.is_editing_text) {
                    drawing.color = color;
                }
            }
            PropertyChange::FontSize(size) => {
                self.current_font_size = size;
                if let Some(drawing) = self.objects.iter_mut().rev().find(|o| o.is_editing_text) {
                    drawing.font_size = size;
                }
            }
            PropertyChange::Stroke(stroke) => {
                self.current_stroke = stroke;
            }
            PropertyChange::Fill(_) => {
                self.current_is_filled = !self.current_is_filled;
            }
            PropertyChange::Opacity(opacity) => {
                self.current_opacity = opacity;
                if let Some(idx) = self.selected_object_index {
                    if let Some(obj) = self.objects.get_mut(idx) {
                        obj.opacity = opacity;
                    }
                }
            }
            PropertyChange::Shadow(enabled) => {
                self.current_shadow = enabled;
                if let Some(idx) = self.selected_object_index {
                    if let Some(obj) = self.objects.get_mut(idx) {
                        obj.has_shadow = enabled;
                    }
                }
            }
            PropertyChange::Glow(glow) => {
                self.current_glow = glow;
                if let Some(idx) = self.selected_object_index {
                    if let Some(obj) = self.objects.get_mut(idx) {
                        obj.glow = glow;
                    }
                }
            }
        }
    }
}

unsafe impl Send for OverlayState {}
unsafe impl Sync for OverlayState {}
