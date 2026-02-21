use windows::Win32::Foundation::RECT;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitZone {
    None,
    Body,
    Stroke,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    // Special handles for Arrow/Line
    Tail,
    Tip,
    WingLeft,
    WingRight,
}

impl HitZone {
    pub fn detect(rect: &RECT, x: i32, y: i32) -> Self {
        let t = 10;

        // 1. Check Handles (Priority)
        // Corners - Use i64 to prevent overflows in .abs()
        let x64 = x as i64;
        let y64 = y as i64;
        let left64 = rect.left as i64;
        let right64 = rect.right as i64;
        let top64 = rect.top as i64;
        let bottom64 = rect.bottom as i64;

        if (x64 - left64).abs() <= t && (y64 - top64).abs() <= t {
            return HitZone::TopLeft;
        }
        if (x64 - right64).abs() <= t && (y64 - top64).abs() <= t {
            return HitZone::TopRight;
        }
        if (x64 - left64).abs() <= t && (y64 - bottom64).abs() <= t {
            return HitZone::BottomLeft;
        }
        if (x64 - right64).abs() <= t && (y64 - bottom64).abs() <= t {
            return HitZone::BottomRight;
        }

        // Mids
        let mid_x = (left64 + right64) / 2;
        let mid_y = (top64 + bottom64) / 2;

        if (x64 - mid_x).abs() <= t && (y64 - top64).abs() <= t {
            return HitZone::Top;
        }
        if (x64 - mid_x).abs() <= t && (y64 - bottom64).abs() <= t {
            return HitZone::Bottom;
        }
        if (x64 - left64).abs() <= t && (y64 - mid_y).abs() <= t {
            return HitZone::Left;
        }
        if (x64 - right64).abs() <= t && (y64 - mid_y).abs() <= t {
            return HitZone::Right;
        }

        // 2. Check Edges (Stroke -> Move)
        let on_left = (x64 - left64).abs() <= t && y64 >= top64 - t && y64 <= bottom64 + t;
        let on_right = (x64 - right64).abs() <= t && y64 >= top64 - t && y64 <= bottom64 + t;
        let on_top = (y64 - top64).abs() <= t && x64 >= left64 - t && x64 <= right64 + t;
        let on_bottom = (y64 - bottom64).abs() <= t && x64 >= left64 - t && x64 <= right64 + t;

        if on_left || on_right || on_top || on_bottom {
            return HitZone::Stroke;
        }

        // 3. Check Body
        if x > rect.left && x < rect.right && y > rect.top && y < rect.bottom {
            return HitZone::Body;
        }

        HitZone::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureEngine {
    Gdi,
    Wgc,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureMode {
    Standard,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionMode {
    None,
    Selecting,                   // Drawing new box
    Moving,                      // Dragging entire box
    Resizing(HitZone),           // Dragging a handle
    Drawing,                     // Drawing a shape (Rect, Arrow, etc.)
    TransformingObject(HitZone), // Transforming an existing object (move/resize)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawingTool {
    None,
    Rect,
    Ellipse,
    Arrow,
    Line,
    Brush,
    Mosaic,
    Text,
    Number,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropertyChange {
    Color(u32),
    FontSize(f32),
    Stroke(f32),
    Fill(bool),
    Opacity(f32),
    Shadow(bool),
    Glow(f32),
}
