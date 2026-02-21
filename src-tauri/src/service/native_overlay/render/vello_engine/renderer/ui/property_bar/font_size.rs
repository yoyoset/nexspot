use vello::kurbo::{Affine, Circle, Point, Rect, RoundedRect};
use vello::peniko::Color;
use vello::Scene;

pub fn draw_font_size_selectors(
    scene: &mut Scene,
    offset_x: &mut f64,
    rect: &windows::Win32::Foundation::RECT,
    current_font_size: f32,
) {
    let sizes = [14.0, 24.0, 36.0];

    for (i, &size) in sizes.iter().enumerate() {
        let is_selected = (size - current_font_size).abs() < 1.0;
        let btn_rect = Rect::new(
            *offset_x,
            rect.top as f64 + 6.0,
            *offset_x + 32.0,
            rect.bottom as f64 - 6.0,
        );
        let rounded_btn = RoundedRect::from_rect(btn_rect, 4.0);

        if is_selected {
            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                Color::from_rgba8(255, 255, 255, 40),
                None,
                &rounded_btn,
            );
        }

        let circle = Circle::new(
            Point::new(btn_rect.center().x, btn_rect.center().y),
            2.0 + (i as f64 * 1.5),
        );
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            if is_selected {
                Color::from_rgb8(0, 160, 255)
            } else {
                Color::from_rgba8(255, 255, 255, 200)
            },
            None,
            &circle,
        );

        *offset_x += 40.0;
    }
}
