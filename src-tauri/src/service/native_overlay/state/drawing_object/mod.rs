use super::types::DrawingTool;
use std::collections::{HashMap, VecDeque};

mod bounds;
mod hit_test;
mod mosaic;

#[derive(Debug, Clone)]
pub struct DrawingObject {
    pub tool: DrawingTool,
    pub points: Vec<(i32, i32)>,
    pub text: Option<String>,
    pub color: u32,
    pub stroke_width: f32,
    pub font_size: f32,
    pub is_filled: bool,
    pub is_dashed: bool,
    pub is_editing_text: bool,
    pub font_family: String,
    pub head_width: Option<f32>, // Optional custom head width for arrows
    pub opacity: f32,
    pub has_shadow: bool,
    pub glow: f32, // 0.0 to 1.0 (Simulated Outer Glow)
    pub style: crate::service::config::types::AestheticStyle,

    // Mosaic Optimization Fields
    pub mosaic_blocks: HashMap<(i32, i32), u32>, // (gx, gy) -> ARGB
    pub mosaic_pending_points: VecDeque<(i32, i32)>, // Points waiting for processing
    pub mosaic_last_pos: Option<(i32, i32)>,     // For interpolation
}
