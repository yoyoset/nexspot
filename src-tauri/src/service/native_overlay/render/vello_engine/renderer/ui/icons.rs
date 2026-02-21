use crate::service::native_overlay::render::toolbar::types::{ToolType, ToolbarButton};
use vello::kurbo::{Affine, BezPath, Circle, Line, Rect};
use vello::peniko::{Brush, Color, Fill};

pub fn draw_button_icon(scene: &mut vello::Scene, btn: &ToolbarButton, is_active: bool) {
    let rect = Rect::new(
        btn.rect.left as f64,
        btn.rect.top as f64,
        btn.rect.right as f64,
        btn.rect.bottom as f64,
    );
    let cx = rect.center().x;
    let cy = rect.center().y;
    let size = 16.0;

    let color = if is_active {
        Color::from_rgba8(0, 160, 255, 255)
    } else {
        Color::WHITE
    };
    let brush = Brush::Solid(color);
    let stroke = vello::kurbo::Stroke::new(1.5);

    match btn.tool {
        ToolType::Rect => {
            let icon_rect = Rect::from_center_size((cx, cy), (size, size * 0.8));
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &icon_rect);
        }
        ToolType::Ellipse => {
            let circle = Circle::new((cx, cy), size / 2.0);
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &circle);
        }
        ToolType::Line => {
            let line = Line::new(
                (cx - size / 2.0, cy + size / 2.0),
                (cx + size / 2.0, cy - size / 2.0),
            );
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &line);
        }
        ToolType::Arrow => {
            let mut path = BezPath::new();
            path.move_to((cx - 8.0, cy + 8.0));
            path.line_to((cx + 8.0, cy - 8.0));
            path.move_to((cx, cy - 8.0));
            path.line_to((cx + 8.0, cy - 8.0));
            path.line_to((cx + 8.0, cy));
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &path);
        }
        ToolType::Brush => {
            let mut path = BezPath::new();
            path.move_to((cx - 6.0, cy + 4.0));
            path.quad_to((cx, cy - 8.0), (cx + 6.0, cy + 4.0));
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &path);
        }
        ToolType::Text => {
            if let Ok(t_path) = BezPath::from_svg("M -6,-6 L 6,-6 M 0,-6 L 0,6") {
                scene.stroke(&stroke, Affine::translate((cx, cy)), &brush, None, &t_path);
            }
        }
        ToolType::Mosaic => {
            for i in -1..=1 {
                scene.stroke(
                    &stroke,
                    Affine::IDENTITY,
                    &brush,
                    None,
                    &Line::new(
                        (cx - 6.0, cy + i as f64 * 4.0),
                        (cx + 6.0, cy + i as f64 * 4.0),
                    ),
                );
                scene.stroke(
                    &stroke,
                    Affine::IDENTITY,
                    &brush,
                    None,
                    &Line::new(
                        (cx + i as f64 * 4.0, cy - 6.0),
                        (cx + i as f64 * 4.0, cy + 6.0),
                    ),
                );
            }
        }
        ToolType::Number => {
            let circle = Circle::new((cx, cy), 7.0);
            scene.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &circle);
        }
        ToolType::Save => {
            let icon_rect = Rect::from_center_size((cx, cy), (size, size));
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &icon_rect);
            scene.stroke(
                &stroke,
                Affine::IDENTITY,
                &brush,
                None,
                &Line::new((cx - 4.0, cy + 4.0), (cx + 4.0, cy + 4.0)),
            );
        }
        ToolType::Cancel => {
            let line1 = Line::new((cx - 6.0, cy - 6.0), (cx + 6.0, cy + 6.0));
            let line2 = Line::new((cx + 6.0, cy - 6.0), (cx - 6.0, cy + 6.0));
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &line1);
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &line2);
        }
        ToolType::Pin => {
            let mut path = BezPath::new();
            path.move_to((cx - 4.0, cy + 2.0));
            path.line_to((cx + 4.0, cy + 2.0));
            path.line_to((cx, cy + 8.0));
            path.close_path();
            path.move_to((cx, cy + 2.0));
            path.line_to((cx, cy - 6.0));
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &path);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &brush,
                None,
                &Circle::new((cx, cy - 6.0), 2.5),
            );
        }
        ToolType::Copy => {
            let r1 = Rect::from_center_size((cx - 2.0, cy + 2.0), (10.0, 12.0));
            let r2 = Rect::from_center_size((cx + 2.0, cy - 2.0), (10.0, 12.0));
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &r1);
            // Fill background of front rect to cover back rect lines
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(Color::from_rgba8(20, 20, 20, 255)),
                None,
                &r2,
            );
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &r2);
        }
        ToolType::More => {
            for i in -1..=1 {
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &brush,
                    None,
                    &Circle::new((cx + i as f64 * 5.0, cy), 1.5),
                );
            }
        }
        ToolType::Macro(_) => {
            // Draw a "Puzzle Piece" or generic Plugin icon
            let r = Rect::from_center_size((cx, cy), (12.0, 12.0));
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &r);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &brush,
                None,
                &Circle::new((cx + 2.0, cy - 2.0), 2.0),
            );
            let mut path = BezPath::new();
            path.move_to((cx - 2.0, cy + 2.0));
            path.line_to((cx + 2.0, cy - 2.0));
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &path);
        }
        ToolType::AiExecute(_) => {
            // Draw a "Sparkles" or logic similar to Macro
            let r = Rect::from_center_size((cx, cy), (12.0, 12.0));
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &r);
            // Draw an 'A' inside?
            let mut path = BezPath::new();
            path.move_to((cx - 3.0, cy + 3.0));
            path.line_to((cx, cy - 4.0));
            path.line_to((cx + 3.0, cy + 3.0));
            scene.stroke(&stroke, Affine::IDENTITY, &brush, None, &path);
        }
    }
}
