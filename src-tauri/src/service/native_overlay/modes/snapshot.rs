use super::CaptureModeHandler;
use windows::Win32::Foundation::RECT;

pub struct SnapshotHandler {
    pub width: i32,
    pub height: i32,
}

impl CaptureModeHandler for SnapshotHandler {
    fn prepare_selection(
        &self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        mouse_x: i32,
        mouse_y: i32,
        _window_rects: &[RECT],
    ) -> Option<RECT> {
        let sw = self.width;
        let sh = self.height;

        // 居中逻辑：优先使用鼠标位置，否则使用屏幕中心
        let cx = x + w / 2;
        let cy = y + h / 2;

        let center_x = if mouse_x >= x && mouse_x <= x + w {
            mouse_x
        } else {
            cx
        };
        let center_y = if mouse_y >= y && mouse_y <= y + h {
            mouse_y
        } else {
            cy
        };

        let left = (center_x - sw / 2).max(x).min(x + w - sw);
        let top = (center_y - sh / 2).max(y).min(y + h - sh);
        let right = left + sw;
        let bottom = top + sh;

        Some(RECT {
            left,
            top,
            right,
            bottom,
        })
    }
}
