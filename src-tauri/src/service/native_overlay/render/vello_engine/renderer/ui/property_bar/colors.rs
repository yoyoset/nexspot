use vello::kurbo::{Affine, Circle, Point, Rect, RoundedRect, Stroke};
use vello::peniko::Color;
use vello::Scene;

pub fn draw_color_palette(
    scene: &mut Scene,
    offset_x: &mut f64,
    rect: &windows::Win32::Foundation::RECT,
    current_color: u32,
) {
    let colors =
        crate::service::native_overlay::render::toolbar::property_bar::get_palette_colors();
    for color_u32 in colors {
        let color = Color::from_rgba8(
            ((color_u32 >> 16) & 0xFF) as u8,
            ((color_u32 >> 8) & 0xFF) as u8,
            (color_u32 & 0xFF) as u8,
            255,
        );

        let is_selected = color_u32 == current_color;
        let btn_rect = Rect::new(
            *offset_x,
            rect.top as f64 + 8.0,
            *offset_x + 24.0,
            rect.bottom as f64 - 8.0,
        );

        if is_selected {
            let ring_rect = btn_rect.inset(-2.0);
            let ring = RoundedRect::from_rect(ring_rect, 6.0);
            scene.stroke(
                &Stroke::new(1.0),
                Affine::IDENTITY,
                Color::from_rgba8(255, 255, 255, 100),
                None,
                &ring,
            );
        }

        let inner = RoundedRect::from_rect(btn_rect, 4.0);
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            color,
            None,
            &inner,
        );

        scene.stroke(
            &Stroke::new(1.0),
            Affine::IDENTITY,
            if is_selected {
                Color::from_rgb8(0, 160, 255)
            } else {
                Color::from_rgba8(255, 255, 255, 50)
            },
            None,
            &inner,
        );

        if is_selected {
            let circle = Circle::new(Point::new(btn_rect.center().x, btn_rect.center().y), 2.0);
            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                Color::WHITE,
                None,
                &circle,
            );
        }

        *offset_x += 32.0;
    }
}
