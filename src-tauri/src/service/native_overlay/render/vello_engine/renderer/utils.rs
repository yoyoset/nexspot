use vello::kurbo::Rect;
use vello::peniko::Color;

pub fn argb_to_vello(argb: u32) -> Color {
    let a = ((argb >> 24) & 0xFF) as u8;
    let r = ((argb >> 16) & 0xFF) as u8;
    let g = ((argb >> 8) & 0xFF) as u8;
    let b = (argb & 0xFF) as u8;
    Color::from_rgba8(r, g, b, a)
}

pub fn points_to_rect(p1: (i32, i32), p2: (i32, i32)) -> Rect {
    Rect::new(
        p1.0.min(p2.0) as f64,
        p1.1.min(p2.1) as f64,
        p1.0.max(p2.0) as f64,
        p1.1.max(p2.1) as f64,
    )
}
