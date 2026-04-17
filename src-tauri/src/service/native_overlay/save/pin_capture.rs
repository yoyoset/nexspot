use crate::service::native_overlay::state::OverlayState;
use std::sync::{Arc, RwLock};
use tauri::AppHandle;
use super::render::render_snapshot;
use base64::Engine;

pub fn capture_to_base64(
    state_arc: &Arc<RwLock<OverlayState>>,
    app: &AppHandle,
) -> anyhow::Result<String> {
    // 1. Render to HBITMAP
    let rendered = render_snapshot(state_arc, app)?;

    // 2. Convert HBITMAP to PNG Bytes
    let bytes = crate::service::win32::bitmap::bitmap_to_bytes(
        rendered.hbitmap.0,
        image::ImageFormat::Png,
        100,
    )?;

    // 3. Encode to Base64
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:image/png;base64,{}", b64))
}
