use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Storage::Streams::DataWriter;
use crate::service::native_overlay::save::render::render_snapshot;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager, Emitter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrWord {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrLine {
    pub text: String,
    pub words: Vec<OcrWord>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrResultData {
    pub lines: Vec<OcrLine>,
    pub full_text: String,
}

pub async fn run_ocr_on_selection(
    app: AppHandle,
    state_arc: Arc<RwLock<crate::service::native_overlay::state::OverlayState>>,
) -> anyhow::Result<OcrResultData> {
    // 1. Render selection
    let snapshot = render_snapshot(&state_arc, &app)?;
    let rgba = crate::service::win32::bitmap::bitmap_to_rgba_image(snapshot.hbitmap.0)?;
    let (w0, h0) = rgba.dimensions();

    // 2. 仅小选区放大 2x（WinRT 对小字号在放大后识别更好）；
    //    大图放大既慢又无收益。Triangle 滤镜足够且远快于 Lanczos3。
    let upscale: u32 = if w0.min(h0) < 700 && w0.max(h0) < 1600 { 2 } else { 1 };
    let img = if upscale == 2 {
        image::imageops::resize(&rgba, w0 * 2, h0 * 2, image::imageops::FilterType::Triangle)
    } else {
        rgba
    };
    let (w, h) = img.dimensions();

    // 3. RGBA→BGRA 原始字节直构 SoftwareBitmap（跳过 PNG 编码/解码整个往返）
    let mut bgra = img.into_raw();
    for px in bgra.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    // writer/IBuffer 非 Send，限定作用域在 await 前释放
    let software_bitmap = {
        let writer = DataWriter::new()?;
        writer.WriteBytes(&bgra)?;
        let buffer = writer.DetachBuffer()?;
        SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Bgra8, w as i32, h as i32)?
    };

    // 5. Run OCR
    let engine = windows::Media::Ocr::OcrEngine::TryCreateFromUserProfileLanguages()?;
    let result = engine.RecognizeAsync(&software_bitmap)?.await?;

    let mut lines = Vec::new();
    let mut full_text = String::new();

    // BoundingRect 在送入 OCR 的（可能已放大的）图像坐标系里，
    // 必须除回放大倍数才是选区原始像素坐标 —— 否则文字层整体 2x 错位。
    let inv = 1.0 / upscale as f64;
    for line in result.Lines()? {
        let mut words = Vec::new();
        let line_text = line.Text()?.to_string();

        for word in line.Words()? {
            let rect = word.BoundingRect()?;
            words.push(OcrWord {
                text: word.Text()?.to_string(),
                x: rect.X as f64 * inv,
                y: rect.Y as f64 * inv,
                width: rect.Width as f64 * inv,
                height: rect.Height as f64 * inv,
            });
        }

        lines.push(OcrLine {
            text: line_text.clone(),
            words,
        });

        if !full_text.is_empty() {
            full_text.push('\n');
        }
        full_text.push_str(&line_text);
    }

    if full_text.trim().is_empty() {
        return Err(anyhow::anyhow!("No text detected in selection"));
    }

    Ok(OcrResultData {
        lines,
        full_text,
    })
}

#[tauri::command]
pub async fn execute_ocr(
    app: AppHandle,
) -> Result<OcrResultData, String> {
    let state_global = app.state::<crate::app_state::AppState>();
    let state_arc = {
        let manager = state_global.overlay_manager.lock().unwrap();
        manager.state.clone()
    };
    
    // 1. Get selection bounds for window positioning
    let selection_rect = {
        let state = state_arc.read().unwrap();
        state.selection.ok_or_else(|| "No selection active".to_string())?
    };

    match run_ocr_on_selection(app.clone(), state_arc).await {
        Ok(mut data) => {
            // --- Multi-Monitor DPI Awareness ---
            // 找选区所在显示器；窗口尺寸/位置与文字层坐标统一用逻辑像素。
            let monitor = app.monitor_from_point(selection_rect.left as f64, selection_rect.top as f64)
                .ok().flatten();
            let scale_factor = monitor.as_ref().map(|m| m.scale_factor()).unwrap_or(1.0);

            // 词坐标是选区物理像素，webview CSS 像素 = 逻辑像素（物理/缩放），
            // 不除以 DPI 缩放会在高分屏整体错位放大。
            if (scale_factor - 1.0).abs() > 0.001 {
                for line in &mut data.lines {
                    for w in &mut line.words {
                        w.x /= scale_factor;
                        w.y /= scale_factor;
                        w.width /= scale_factor;
                        w.height /= scale_factor;
                    }
                }
            }

            // 2. Add to Pin Collection (Silent backend update)
            let pin_state = app.state::<crate::service::pin::PinState>();
            let pin_id = uuid::Uuid::new_v4().to_string();
            pin_state.add_pin(pin_id, crate::service::pin::PinData::Text(data.full_text.clone()));

            {
                let app_state = app.state::<crate::app_state::AppState>();
                let mgr = app_state.overlay_manager.lock().unwrap();
                let mut state = mgr.state.write().unwrap();
                state.current_ocr_data = Some(data.clone());
            }

            // 3. Spawn/Position OCR Selectable Window
            let width = (selection_rect.right - selection_rect.left).abs();
            let height = (selection_rect.bottom - selection_rect.top).abs();

            let window_label = "ocr-result";
            
            let logical_w = width as f64 / scale_factor;
            let logical_h = height as f64 / scale_factor;
            let logical_x = selection_rect.left as f64 / scale_factor;
            let logical_y = selection_rect.top as f64 / scale_factor;

            // Close existing if any
            if let Some(w) = app.get_webview_window(window_label) {
                let _ = w.close();
            }

            let preview_url = tauri::WebviewUrl::App("index.html#ocr-result".into());
            if let Ok(w) = tauri::WebviewWindowBuilder::new(&app, window_label, preview_url)
                .title("OCR Result")
                .transparent(true)
                .decorations(false)
                .always_on_top(true)
                .inner_size(logical_w, logical_h)
                .position(logical_x, logical_y)
                .build() 
            {
                // Emit data directly to this window
                let _ = w.emit("ocr://data", data.clone());
            }

            // 4. Notify main UI
            let _ = app.emit("pin-collection-updated", ());
            
            // 5. Log to Activity Feed
            crate::service::activity::log_activity(
                &app, 
                "ocr", 
                None, 
                Some(data.full_text.chars().take(50).collect::<String>() + "...")
            );

            Ok(data)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_last_ocr_result(app: AppHandle) -> Result<Option<OcrResultData>, String> {
    let state = app.state::<crate::app_state::AppState>();
    let manager = state.overlay_manager.lock().unwrap();
    let state_lock = manager.state.read().unwrap();
    Ok(state_lock.current_ocr_data.clone())
}
