use crate::service::native_overlay::state::DrawingTool;
use vello::kurbo::{Affine, Circle, Point, Rect, RoundedRect, Stroke};
use vello::peniko::Color;
use vello::Scene;

pub fn draw_stroke_selectors(
    scene: &mut Scene,
    offset_x: &mut f64,
    rect: &windows::Win32::Foundation::RECT,
    tool: DrawingTool,
    current_stroke: f32,
    current_is_filled: bool,
) {
    if tool == DrawingTool::Text || tool == DrawingTool::None {
        return;
    }

    let strokes = [2.0, 4.0, 8.0];
    let sizes = [2.0, 3.5, 5.0];

    for (i, &stroke) in strokes.iter().enumerate() {
        let is_selected = (stroke - current_stroke).abs() < 0.1;
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
            sizes[i],
        );
        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            if is_selected {
                Color::from_rgb8(0, 160, 255)
            } else {
                Color::from_rgba8(255, 255, 255, 150)
            },
            None,
            &circle,
        );
        *offset_x += 36.0;
    }

    // --- Fill Toggle ---
    if matches!(tool, DrawingTool::Rect | DrawingTool::Ellipse) {
        let btn_rect = Rect::new(
            *offset_x,
            rect.top as f64 + 6.0,
            *offset_x + 32.0,
            rect.bottom as f64 - 6.0,
        );

        // Icon
        let icon_rect = btn_rect.inset(-8.0);
        let rounded_icon = RoundedRect::from_rect(icon_rect, 2.0);

        if current_is_filled {
            let bg_active = RoundedRect::from_rect(btn_rect, 4.0);
            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                Color::from_rgba8(255, 255, 255, 40),
                None,
                &bg_active,
            );

            scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                Color::from_rgb8(0, 160, 255),
                None,
                &rounded_icon,
            );
        } else {
            scene.stroke(
                &Stroke::new(2.0),
                Affine::IDENTITY,
                Color::from_rgba8(255, 255, 255, 150),
                None,
                &rounded_icon,
            );
        }
        *offset_x += 36.0;
    }
}
