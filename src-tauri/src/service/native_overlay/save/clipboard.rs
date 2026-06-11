use crate::service::native_overlay::state::OverlayState;
use crate::service::win32;
use std::sync::{Arc, RwLock};
use tauri::AppHandle;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Gdi::{HGDIOBJ, SRCCOPY};
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

    // 3. 统一转 DDB 再上剪贴板。
    //    Wgc/Vello 路径产出的是 DIB section（CreateDIBSection），直接作为
    //    CF_BITMAP 放剪贴板时多数粘贴方无法解码（"Failed to process image"）；
    //    GDI 路径本就是 DDB。此处对两引擎统一 BitBlt 进兼容位图。
    let ddb = {
        let hdc_screen = win32::gdi::get_dc(None)?;
        let hdc_src = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let prev_src = win32::gdi::select_object(&hdc_src, HGDIOBJ(rendered.hbitmap.0 .0))?;

        let hdc_dst = win32::gdi::create_compatible_dc(Some(&hdc_screen))?;
        let ddb = win32::gdi::create_compatible_bitmap(&hdc_screen, rendered.width, rendered.height)?;
        let prev_dst = win32::gdi::select_object(&hdc_dst, HGDIOBJ(ddb.0 .0))?;

        win32::gdi::bit_blt(
            &hdc_dst,
            0,
            0,
            rendered.width,
            rendered.height,
            &hdc_src,
            0,
            0,
            SRCCOPY,
        )?;

        win32::gdi::select_object(&hdc_src, prev_src)?;
        win32::gdi::select_object(&hdc_dst, prev_dst)?;
        win32::gdi::release_dc(None, hdc_screen);
        ddb
    };

    // 4. Set to Clipboard（所有权移交剪贴板，leak 防止 Drop 释放）
    unsafe {
        if OpenClipboard(None).is_ok() {
            let _ = EmptyClipboard();
            let h = ddb.leak();
            let _ = SetClipboardData(2, Some(HANDLE(h.0 as *mut std::ffi::c_void)));
            let _ = CloseClipboard();
        }
    }

    notify_copied(app);
    Ok(())
}
