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
        let cx = w / 2;
        let cy = h / 2;

        let mx = mouse_x - x;
        let my = mouse_y - y;

        let center_x = if mx >= 0 && mx <= w { mx } else { cx };
        let center_y = if my >= 0 && my <= h { my } else { cy };

        let left = (center_x - sw / 2).max(0).min(w - sw);
        let top = (center_y - sh / 2).max(0).min(h - sh);
        let right = left + sw;
        let bottom = top + sh;

        Some(RECT {
            left: left + x,
            top: top + y,
            right: right + x,
            bottom: bottom + y,
        })
    }
}
