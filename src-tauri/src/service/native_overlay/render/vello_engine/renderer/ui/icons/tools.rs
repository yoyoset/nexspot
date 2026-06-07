use crate::service::native_overlay::render::toolbar::types::{ToolType, ToolbarButton};
use vello::kurbo::{Affine, BezPath, Circle, Line, Rect};
use vello::peniko::{Brush, Color, Fill};

pub fn draw_tool_icon(scene: &mut vello::Scene, btn: &ToolbarButton, is_active: bool) {
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
        Color::WHITE // on-accent，叠在 periwinkle 选中底上
    } else {
        Color::WHITE
    };
    let brush = Brush::Solid(color);
    let stroke = vello::kurbo::Stroke::new(1.5);
    let xf = Affine::translate((cx, cy)) * Affine::scale(sc);

    match btn.tool {
        ToolType::Rect => {
            let icon_rect = Rect::from_center_size((0.0, 0.0), (14.0, 11.0));
            scene.stroke(&stroke, xf, &brush, None, &icon_rect);
        }
        ToolType::Ellipse => {
            let circle = Circle::new((0.0, 0.0), 7.0);
            scene.stroke(&stroke, xf, &brush, None, &circle);
        }
        ToolType::Line => {
            let line = Line::new((-7.0, 7.0), (7.0, -7.0));
            scene.stroke(&stroke, xf, &brush, None, &line);
        }
        ToolType::Arrow => {
            let mut path = BezPath::new();
            path.move_to((-7.0, 7.0));
            path.line_to((7.0, -7.0));
            path.move_to((1.0, -7.0));
            path.line_to((7.0, -7.0));
            path.line_to((7.0, -1.0));
            scene.stroke(&stroke, xf, &brush, None, &path);
        }
        ToolType::Brush => {
            let mut path = BezPath::new();
            path.move_to((-6.0, 6.0));
            path.line_to((-3.0, 1.0));
            path.line_to((0.0, -2.0));
            path.line_to((2.0, 0.0));
            path.line_to((-1.0, 3.0));
            path.close_path();

            let mut barrel = BezPath::new();
            barrel.move_to((0.0, -2.0));
            barrel.line_to((4.0, -6.0));
            barrel.line_to((6.0, -4.0));
            barrel.line_to((2.0, 0.0));

            let slit = Line::new((-6.0, 6.0), (-2.0, 2.0));
            let hole = Circle::new((-2.0, 2.0), 0.8);

            scene.stroke(&vello::kurbo::Stroke::new(1.2), xf, &brush, None, &path);
            scene.stroke(&stroke, xf, &brush, None, &barrel);
            scene.stroke(&vello::kurbo::Stroke::new(1.0), xf, &brush, None, &slit);
            scene.stroke(&vello::kurbo::Stroke::new(1.0), xf, &brush, None, &hole);
        }
        ToolType::Text => {
            if let Ok(t_path) = BezPath::from_svg("M -5,-6 L 5,-6 M 0,-6 L 0,6 M -2,6 L 2,6") {
                scene.stroke(&stroke, xf, &brush, None, &t_path);
            }
        }
        ToolType::Mosaic => {
            let s = 2.5;
            for i in -2..=1 {
                for j in -2..=1 {
                    if (i + j) % 2 == 0 {
                        scene.fill(
                            Fill::NonZero,
                            xf,
                            &brush,
                            None,
                            &Rect::new(i as f64 * s, j as f64 * s, (i + 1) as f64 * s, (j + 1) as f64 * s),
                        );
                    }
                }
            }
        }
        ToolType::Number => {
            let circle = Circle::new((0.0, 0.0), 6.5);
            scene.stroke(&stroke, xf, &brush, None, &circle);
            let mut one = BezPath::new();
            one.move_to((-1.5, -1.0));
            one.line_to((0.0, -3.0));
            one.line_to((0.0, 3.0));
            scene.stroke(&vello::kurbo::Stroke::new(1.2), xf, &brush, None, &one);
        }
        ToolType::Ocr => {
            // Document/Scan icon
            let icon_rect = Rect::from_center_size((0.0, 0.0), (12.0, 14.0));
            scene.stroke(&stroke, xf, &brush, None, &icon_rect);
            let mut lines = BezPath::new();
            lines.move_to((-3.0, -3.0)); lines.line_to((3.0, -3.0));
            lines.move_to((-3.0, 0.0));  lines.line_to((3.0, 0.0));
            lines.move_to((-3.0, 3.0));  lines.line_to((1.0, 3.0));
            scene.stroke(&vello::kurbo::Stroke::new(1.0), xf, &brush, None, &lines);
        }
        ToolType::Scrolling => {
            // Stacked squares/layers icon
            let r1 = Rect::from_center_size((0.0, -3.0), (12.0, 8.0));
            let r2 = Rect::from_center_size((0.0, 3.0), (12.0, 8.0));
            scene.stroke(&stroke, xf, &brush, None, &r1);
            scene.stroke(&stroke, xf, &brush, None, &r2);
        }
        _ => {}
    }
}
