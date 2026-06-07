use vello::kurbo::{Affine, Point, Rect, RoundedRect, Stroke};
use vello::peniko::Color;
use vello::Scene;

pub fn draw_background(scene: &mut Scene, rect: &windows::Win32::Foundation::RECT) {
    let bg_rect = Rect::new(
        rect.left as f64,
        rect.top as f64,
        rect.right as f64,
        rect.bottom as f64,
    );
    let rounded_bg = RoundedRect::from_rect(bg_rect, 11.0);

    // Studio bg1
    scene.fill(
        vello::peniko::Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(20, 21, 25, 245),
        None,
        &rounded_bg,
    );

    // Studio border (bg3 / line2)
    scene.stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        Color::from_rgba8(35, 36, 43, 255),
        None,
        &rounded_bg,
    );
}

pub fn draw_divider(scene: &mut Scene, x: f64, top: f64, bottom: f64) {
    scene.stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        Color::from_rgba8(35, 36, 43, 255), // Studio bg3 / line2
        None,
        &vello::kurbo::Line::new(Point::new(x, top + 8.0), Point::new(x, bottom - 8.0)),
    );
}
