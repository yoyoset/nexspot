use super::CaptureModeHandler;
use windows::Win32::Foundation::RECT;

pub struct WindowHandler;

impl CaptureModeHandler for WindowHandler {
    fn prepare_selection(
        &self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        mouse_x: i32,
        mouse_y: i32,
        window_rects: &[RECT],
    ) -> Option<RECT> {
        // 查找包含光标且面积最小的有效窗口
        let mut best_window = None;
        let mut best_area = i64::MAX;

        for wr in window_rects {
            if mouse_x >= wr.left && mouse_x < wr.right && mouse_y >= wr.top && mouse_y < wr.bottom
            {
                let width = wr.right - wr.left;
                let height = wr.bottom - wr.top;
                let area = width as i64 * height as i64;

                // 忽略极小窗口或背景桌面
                if area > 100 && area < best_area {
                    best_area = area;
                    best_window = Some(*wr);
                }
            }
        }

        if let Some(wr) = best_window {
            log::info!(
                "[Mode:Window] Detected window: {:?} (area: {})",
                wr,
                best_area
            );
            Some(wr)
        } else {
            log::warn!(
                "[Mode:Window] No window found under cursor at ({}, {}), falling back to screen",
                mouse_x,
                mouse_y
            );
            // 回退到全屏
            Some(RECT {
                left: x,
                top: y,
                right: x + w,
                bottom: y + h,
            })
        }
    }
}
