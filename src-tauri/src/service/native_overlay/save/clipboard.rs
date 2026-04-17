use crate::service::native_overlay::state::OverlayState;
use std::sync::{Arc, RwLock};
use tauri::AppHandle;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use super::render::{render_snapshot, update_snapshot_size_config};
use super::utils::notify_copied;

pub fn copy_to_clipboard(
    state_arc: &Arc<RwLock<OverlayState>>,
    app: &AppHandle,
) -> anyhow::Result<()> {
    // 1. Update config if needed
    update_snapshot_size_config(state_arc, app);

    // 2. Render
    let rendered = render_snapshot(state_arc, app)?;

    // 3. Set to Clipboard
    unsafe {
        if OpenClipboard(None).is_ok() {
            let _ = EmptyClipboard();
            let h = rendered.hbitmap.leak();
            let _ = SetClipboardData(2, Some(HANDLE(h.0 as *mut std::ffi::c_void)));
            let _ = CloseClipboard();
        }
    }
    
    notify_copied(app);
    Ok(())
}
