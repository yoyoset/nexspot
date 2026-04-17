use vello::kurbo::{Affine, Circle, Point, Rect, RoundedRect};
use vello::peniko::Color;
use vello::Scene;

pub fn draw_advanced_effects(
    scene: &mut Scene,
    offset_x: &mut f64,
    rect: &windows::Win32::Foundation::RECT,
    current_opacity: f32,
    _current_glow: f32,
) {
    let center_y = (rect.top as f64 + rect.bottom as f64) / 2.0;
    let slider_width = 60.0;

    // --- Opacity Section ---
    *offset_x += 10.0;
    // Icon (Opacity)
    let opacity_icon_circle = Circle::new(Point::new(*offset_x - 4.0, center_y), 4.0);
    scene.stroke(
        &vello::kurbo::Stroke::new(1.0),
        Affine::IDENTITY,
        Color::from_rgba8(255, 255, 255, 180),
        None,
        &opacity_icon_circle,
    );

    // Track
    let track_y = center_y;
    let track_rect = Rect::new(
        *offset_x,
        track_y - 2.0,
        *offset_x + slider_width,
        track_y + 2.0,
    );
    scene.fill(
        vello::peniko::Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(255, 255, 255, 60),
        None,
        &RoundedRect::from_rect(track_rect, 2.0),
    );

    // Filled
    let filled_width = slider_width * current_opacity as f64;
    let filled_rect = Rect::new(
        *offset_x,
        track_y - 2.0,
        *offset_x + filled_width,
        track_y + 2.0,
    );
    scene.fill(
        vello::peniko::Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgb8(0, 160, 255),
        None,
        &RoundedRect::from_rect(filled_rect, 2.0),
    );

    // Thumb
    let thumb_x = *offset_x + filled_width;
    let thumb = Circle::new(Point::new(thumb_x, track_y), 6.0);
    scene.fill(
        vello::peniko::Fill::NonZero,
        Affine::IDENTITY,
        Color::WHITE,
        None,
        &thumb,
    );

    *offset_x += slider_width + 6.0;

    *offset_x += slider_width + 6.0;
}
