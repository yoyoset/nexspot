use crate::service::native_overlay::render::toolbar::types::{ToolType, ToolbarButton};
use vello::kurbo::{Affine, BezPath, Cap, Circle, Join, Line, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

/// lucide 风格描边：1.5px + 圆端帽/圆转角（设计基准 docs/design §C）
fn lucide_stroke() -> Stroke {
    Stroke::new(1.5).with_caps(Cap::Round).with_join(Join::Round)
}

pub fn draw_tool_icon(scene: &mut vello::Scene, btn: &ToolbarButton, _is_active: bool) {
    let rect = Rect::new(
        btn.rect.left as f64,
        btn.rect.top as f64,
        btn.rect.right as f64,
        btn.rect.bottom as f64,
    );
    let cx = rect.center().x;
    let cy = rect.center().y;
    let sc = 1.3;

    // 选中态按钮底为 accent 填充，图标恒为白（on-accent）
    let brush = Brush::Solid(Color::WHITE);
    let stroke = lucide_stroke();
    let xf = Affine::translate((cx, cy)) * Affine::scale(sc);

    match btn.tool {
        // square
        ToolType::Rect => {
            let r = RoundedRect::new(-6.0, -6.0, 6.0, 6.0, 1.5);
            scene.stroke(&stroke, xf, &brush, None, &r);
        }
        // circle
        ToolType::Ellipse => {
            let circle = Circle::new((0.0, 0.0), 6.5);
            scene.stroke(&stroke, xf, &brush, None, &circle);
        }
        // minus（水平短线）
        ToolType::Line => {
            let line = Line::new((-6.0, 0.0), (6.0, 0.0));
            scene.stroke(&stroke, xf, &brush, None, &line);
        }
        // move-up-right（对角线 + 右上箭头头部）
        ToolType::Arrow => {
            let mut path = BezPath::new();
            path.move_to((-5.5, 5.5));
            path.line_to((5.5, -5.5));
            path.move_to((-0.5, -5.5));
            path.line_to((5.5, -5.5));
            path.line_to((5.5, 0.5));
            scene.stroke(&stroke, xf, &brush, None, &path);
        }
        // pen-line（斜置钢笔 + 底部短横线）
        ToolType::Brush => {
            // 笔身
            let body = Line::new((-6.0, 5.4), (2.6, -3.2));
            scene.stroke(&stroke, xf, &brush, None, &body);
            // 笔头（旋转小方块）
            let mut nib = BezPath::new();
            nib.move_to((2.6, -3.2));
            nib.line_to((4.7, -5.3));
            nib.line_to((6.1, -3.9));
            nib.line_to((4.0, -1.8));
            nib.close_path();
            scene.stroke(&stroke, xf, &brush, None, &nib);
            // 底部 line
            let base = Line::new((0.5, 6.5), (6.5, 6.5));
            scene.stroke(&stroke, xf, &brush, None, &base);
        }
        // type（衬线 T）
        ToolType::Text => {
            let mut path = BezPath::new();
            path.move_to((-6.0, -3.2));
            path.line_to((-6.0, -5.8));
            path.line_to((6.0, -5.8));
            path.line_to((6.0, -3.2));
            path.move_to((0.0, -5.8));
            path.line_to((0.0, 5.8));
            path.move_to((-2.6, 5.8));
            path.line_to((2.6, 5.8));
            scene.stroke(&stroke, xf, &brush, None, &path);
        }
        // grid-3x3
        ToolType::Mosaic => {
            let r = RoundedRect::new(-6.0, -6.0, 6.0, 6.0, 1.5);
            scene.stroke(&stroke, xf, &brush, None, &r);
            let mut lines = BezPath::new();
            lines.move_to((-2.0, -6.0));
            lines.line_to((-2.0, 6.0));
            lines.move_to((2.0, -6.0));
            lines.line_to((2.0, 6.0));
            lines.move_to((-6.0, -2.0));
            lines.line_to((6.0, -2.0));
            lines.move_to((-6.0, 2.0));
            lines.line_to((6.0, 2.0));
            scene.stroke(&Stroke::new(1.2).with_caps(Cap::Round), xf, &brush, None, &lines);
        }
        // hash（#）
        ToolType::Number => {
            let mut path = BezPath::new();
            path.move_to((-6.0, -2.3));
            path.line_to((6.0, -2.3));
            path.move_to((-6.0, 2.3));
            path.line_to((6.0, 2.3));
            path.move_to((-1.3, -6.2));
            path.line_to((-2.6, 6.2));
            path.move_to((2.6, -6.2));
            path.line_to((1.3, 6.2));
            scene.stroke(&stroke, xf, &brush, None, &path);
        }
        // scan-text（四角括号 + 文本行）
        ToolType::Ocr => {
            let mut corners = BezPath::new();
            corners.move_to((-6.5, -3.5));
            corners.line_to((-6.5, -6.5));
            corners.line_to((-3.5, -6.5));
            corners.move_to((3.5, -6.5));
            corners.line_to((6.5, -6.5));
            corners.line_to((6.5, -3.5));
            corners.move_to((-6.5, 3.5));
            corners.line_to((-6.5, 6.5));
            corners.line_to((-3.5, 6.5));
            corners.move_to((6.5, 3.5));
            corners.line_to((6.5, 6.5));
            corners.line_to((3.5, 6.5));
            scene.stroke(&stroke, xf, &brush, None, &corners);
            let mut lines = BezPath::new();
            lines.move_to((-3.0, -1.8));
            lines.line_to((3.0, -1.8));
            lines.move_to((-3.0, 1.8));
            lines.line_to((1.5, 1.8));
            scene.stroke(&Stroke::new(1.3).with_caps(Cap::Round), xf, &brush, None, &lines);
        }
        // 滚动长截图（顶部面板 + 向下箭头）
        ToolType::Scrolling => {
            let panel = RoundedRect::new(-6.0, -6.5, 6.0, -1.5, 1.2);
            scene.stroke(&stroke, xf, &brush, None, &panel);
            let mut arrow = BezPath::new();
            arrow.move_to((0.0, 0.8));
            arrow.line_to((0.0, 5.8));
            arrow.move_to((-2.8, 3.2));
            arrow.line_to((0.0, 6.0));
            arrow.line_to((2.8, 3.2));
            scene.stroke(&stroke, xf, &brush, None, &arrow);
        }
        _ => {
            // 兜底：小圆点占位（不应到达）
            scene.fill(Fill::NonZero, xf, &brush, None, &Circle::new((0.0, 0.0), 2.0));
        }
    }
}
