use windows::Win32::Foundation::RECT;

/// 采集模式行为特质
pub trait CaptureModeHandler {
    /// 在采集开始前/后，针对特定模式初始化选区
    fn prepare_selection(
        &self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        mouse_x: i32,
        mouse_y: i32,
        window_rects: &[RECT],
    ) -> Option<RECT>;
}

pub mod fullscreen;
pub mod selection;
pub mod snapshot;
pub mod window;
