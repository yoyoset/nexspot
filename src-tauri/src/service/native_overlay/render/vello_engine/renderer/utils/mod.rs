pub mod text;
pub mod styles;

use crate::service::native_overlay::state::DrawingObject;
use vello::peniko::Color;

pub fn argb_to_vello(argb: u32) -> Color {
    let a = ((argb >> 24) & 0xff) as u8;
    let r = ((argb >> 16) & 0xff) as u8;
    let g = ((argb >> 8) & 0xff) as u8;
    let b = (argb & 0xff) as u8;
    Color::from_rgba8(r, g, b, a)
}

pub fn argb_to_vello_alpha(argb: u32, alpha_factor: f32) -> Color {
    let a = ((argb >> 24) & 0xff) as u8;
    let r = ((argb >> 16) & 0xff) as u8;
    let g = ((argb >> 8) & 0xff) as u8;
    let b = (argb & 0xff) as u8;
    let new_a = ((a as f32) * alpha_factor).clamp(0.0, 255.0) as u8;
    Color::from_rgba8(r, g, b, new_a)
}

pub fn points_to_rect(obj: &DrawingObject) -> vello::kurbo::Rect {
    if obj.points.len() < 2 {
        return vello::kurbo::Rect::ZERO;
    }
    let p1 = obj.points[0];
    let p2 = obj.points[1];
    vello::kurbo::Rect::new(p1.0 as f64, p1.1 as f64, p2.0 as f64, p2.1 as f64)
}
