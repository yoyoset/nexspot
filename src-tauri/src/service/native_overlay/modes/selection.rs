use super::CaptureModeHandler;
use windows::Win32::Foundation::RECT;

pub struct SelectionHandler;

impl CaptureModeHandler for SelectionHandler {
    fn prepare_selection(
        &self,
        _x: i32,
        _y: i32,
        _w: i32,
        _h: i32,
        _mouse_x: i32,
        _mouse_y: i32,
        _window_rects: &[RECT],
    ) -> Option<RECT> {
        // Selection 模式默认不预设选区，由用户手动拖拽
        None
    }
}
