use super::CaptureModeHandler;
use windows::Win32::Foundation::RECT;

pub struct FullscreenHandler;

impl CaptureModeHandler for FullscreenHandler {
    fn prepare_selection(
        &self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        _mouse_x: i32,
        _mouse_y: i32,
        _window_rects: &[RECT],
    ) -> Option<RECT> {
        // 直接选中整个区域
        Some(RECT {
            left: x,
            top: y,
            right: x + w,
            bottom: y + h,
        })
    }
}
