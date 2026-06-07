use crate::service::native_overlay::render::toolbar::types::{ToolType, ToolbarButton};
use vello::kurbo::{Affine, BezPath, Circle, Line, Rect};
use vello::peniko::{Brush, Color, Fill};

pub fn draw_action_icon(scene: &mut vello::Scene, btn: &ToolbarButton, is_active: bool) {
    let rect = Rect::new(
        btn.rect.left as f64,
        btn.rect.top as f64,
        btn.rect.right as f64,
        btn.rect.bottom as f64,
    );
    let cx = rect.center().x;
    let cy = rect.center().y;
    let sc = 1.3;

    let color = if is_active {
        Color::WHITE // on-accent
    } else {
        Color::WHITE
    };
    let brush = Brush::Solid(color);
    let stroke = vello::kurbo::Stroke::new(1.5);
    let xf = Affine::translate((cx, cy)) * Affine::scale(sc);

    match btn.tool {
        ToolType::Undo => {
            let mut path = BezPath::new();
            path.move_to((5.0, 2.0));
            path.quad_to((5.0, -4.0), (0.0, -4.0));
            path.quad_to((-5.0, -4.0), (-5.0, 0.0));
            path.move_to((-8.0, -2.0));
            path.line_to((-5.0, 1.0));
            path.line_to((-2.0, -2.0));
            scene.stroke(&stroke, xf, &brush, None, &path);
        }
        ToolType::Save => {
            let icon_rect = Rect::from_center_size((0.0, 0.0), (12.0, 12.0));
            scene.stroke(&stroke, xf, &brush, None, &icon_rect);
            let line = Line::new((-3.0, 3.0), (3.0, 3.0));
            scene.stroke(&stroke, xf, &brush, None, &line);
            let header = Rect::new(-2.0, -6.0, 2.0, -2.0);
            scene.stroke(&vello::kurbo::Stroke::new(1.0), xf, &brush, None, &header);
        }
        ToolType::SaveAs => {
            // "Stack" effect for SaveAs
            let r1 = Rect::from_center_size((-1.5, -1.5), (10.0, 10.0));
            let r2 = Rect::from_center_size((1.5, 1.5), (10.0, 10.0));
            scene.stroke(&vello::kurbo::Stroke::new(1.0), xf, &brush, None, &r1);
            scene.fill(Fill::NonZero, xf, &Brush::Solid(Color::from_rgba8(20, 20, 20, 255)), None, &r2);
            scene.stroke(&stroke, xf, &brush, None, &r2);
        }
        ToolType::Cancel => {
            let line1 = Line::new((-5.0, -5.0), (5.0, 5.0));
            let line2 = Line::new((5.0, -5.0), (-5.0, 5.0));
            scene.stroke(&stroke, xf, &brush, None, &line1);
            scene.stroke(&stroke, xf, &brush, None, &line2);
        }
        ToolType::Pin => {
            let xf_pin = xf * Affine::rotate(-std::f64::consts::FRAC_PI_4);
            let mut needle = BezPath::new();
            needle.move_to((0.0, 6.0));
            needle.line_to((0.0, 2.0));
            scene.stroke(&stroke, xf_pin, &brush, None, &needle);

            let mut body = BezPath::new();
            body.move_to((-3.5, 2.0));
            body.line_to((3.5, 2.0));
            body.line_to((1.5, -2.0));
            body.line_to((-1.5, -2.0));
            body.close_path();
            scene.stroke(&stroke, xf_pin, &brush, None, &body);

            let head = Rect::from_center_size((0.0, -3.25), (5.0, 2.5));
            scene.fill(Fill::NonZero, xf_pin, &brush, None, &head);
        }
        ToolType::Copy => {
            let r1 = Rect::from_center_size((-2.0, 2.0), (8.0, 10.0));
            let r2 = Rect::from_center_size((2.0, -2.0), (8.0, 10.0));
            scene.stroke(&vello::kurbo::Stroke::new(1.2), xf, &brush, None, &r1);
            scene.fill(Fill::NonZero, xf, &Brush::Solid(Color::from_rgba8(20, 20, 20, 255)), None, &r2);
            scene.stroke(&vello::kurbo::Stroke::new(1.2), xf, &brush, None, &r2);
        }
        ToolType::More => {
            for i in -1..=1 {
                scene.fill(Fill::NonZero, xf, &brush, None, &Circle::new((i as f64 * 4.5, 0.0), 1.5));
            }
        }
        _ => {}
    }
}
