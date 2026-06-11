use crate::service::native_overlay::render::toolbar::types::{ToolType, ToolbarButton};
use vello::kurbo::{Affine, BezPath, Cap, Circle, Join, Line, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

fn lucide_stroke() -> Stroke {
    Stroke::new(1.5).with_caps(Cap::Round).with_join(Join::Round)
}

pub fn draw_action_icon(scene: &mut vello::Scene, btn: &ToolbarButton, _is_active: bool) {
    let rect = Rect::new(
        btn.rect.left as f64,
        btn.rect.top as f64,
        btn.rect.right as f64,
        btn.rect.bottom as f64,
    );
    let cx = rect.center().x;
    let cy = rect.center().y;
    let sc = 1.3;

    let brush = Brush::Solid(Color::WHITE);
    let stroke = lucide_stroke();
    let xf = Affine::translate((cx, cy)) * Affine::scale(sc);

    match btn.tool {
        // undo-2（左箭头 + 右回弯）
        ToolType::Undo => {
            let mut path = BezPath::new();
            path.move_to((-2.6, -4.6));
            path.line_to((-6.2, -1.4));
            path.line_to((-2.6, 1.8));
            path.move_to((-6.2, -1.4));
            path.line_to((2.0, -1.4));
            path.curve_to((4.6, -1.4), (6.2, 0.4), (6.2, 2.6));
            path.line_to((6.2, 4.6));
            scene.stroke(&stroke, xf, &brush, None, &path);
        }
        // download（竖杆 + 下箭头 + 托盘）
        ToolType::Save => {
            let mut path = BezPath::new();
            path.move_to((0.0, -6.4));
            path.line_to((0.0, 2.2));
            path.move_to((-3.6, -1.2));
            path.line_to((0.0, 2.4));
            path.line_to((3.6, -1.2));
            path.move_to((-6.4, 2.6));
            path.line_to((-6.4, 4.8));
            path.curve_to((-6.4, 5.8), (-5.8, 6.4), (-4.8, 6.4));
            path.line_to((4.8, 6.4));
            path.curve_to((5.8, 6.4), (6.4, 5.8), (6.4, 4.8));
            path.line_to((6.4, 2.6));
            scene.stroke(&stroke, xf, &brush, None, &path);
        }
        ToolType::SaveAs => {
            // download + 角标加号（另存为）
            let mut path = BezPath::new();
            path.move_to((-1.0, -6.0));
            path.line_to((-1.0, 1.6));
            path.move_to((-4.2, -1.4));
            path.line_to((-1.0, 1.8));
            path.line_to((2.2, -1.4));
            path.move_to((-6.4, 3.0));
            path.line_to((-6.4, 6.2));
            path.line_to((3.0, 6.2));
            scene.stroke(&stroke, xf, &brush, None, &path);
            let mut plus = BezPath::new();
            plus.move_to((4.8, 1.2));
            plus.line_to((4.8, 6.0));
            plus.move_to((2.4, 3.6));
            plus.line_to((7.2, 3.6));
            scene.stroke(&Stroke::new(1.3).with_caps(Cap::Round), xf, &brush, None, &plus);
        }
        // x（关闭 — bad 红，设计 §C）
        ToolType::Cancel => {
            let bad = Brush::Solid(Color::from_rgba8(247, 109, 109, 255)); // --bad
            let mut path = BezPath::new();
            path.move_to((-5.0, -5.0));
            path.line_to((5.0, 5.0));
            path.move_to((5.0, -5.0));
            path.line_to((-5.0, 5.0));
            scene.stroke(&stroke, xf, &bad, None, &path);
        }
        // pin（直立图钉：头帽 + 身 + 针）
        ToolType::Pin => {
            let head = RoundedRect::new(-3.4, -6.6, 3.4, -4.6, 1.0);
            scene.stroke(&stroke, xf, &brush, None, &head);
            let mut body = BezPath::new();
            body.move_to((-2.2, -4.6));
            body.line_to((-2.2, -1.4));
            body.line_to((-4.6, 1.2));
            body.line_to((4.6, 1.2));
            body.line_to((2.2, -1.4));
            body.line_to((2.2, -4.6));
            scene.stroke(&stroke, xf, &brush, None, &body);
            let needle = Line::new((0.0, 1.2), (0.0, 6.4));
            scene.stroke(&stroke, xf, &brush, None, &needle);
        }
        // copy（前矩形 + 左上后衬轮廓）
        ToolType::Copy => {
            let front = RoundedRect::new(-1.4, -1.4, 6.4, 6.4, 1.2);
            scene.stroke(&stroke, xf, &brush, None, &front);
            let mut back = BezPath::new();
            back.move_to((-4.2, 2.2));
            back.curve_to((-5.6, 2.2), (-6.4, 1.4), (-6.4, 0.0));
            back.line_to((-6.4, -4.2));
            back.curve_to((-6.4, -5.6), (-5.6, -6.4), (-4.2, -6.4));
            back.line_to((0.0, -6.4));
            back.curve_to((1.4, -6.4), (2.2, -5.6), (2.2, -4.2));
            scene.stroke(&stroke, xf, &brush, None, &back);
        }
        ToolType::More => {
            for i in -1..=1 {
                scene.fill(Fill::NonZero, xf, &brush, None, &Circle::new((i as f64 * 4.5, 0.0), 1.5));
            }
        }
        _ => {}
    }
}
