use windows::Graphics::Imaging::BitmapDecoder;
use windows::Storage::Streams::{InMemoryRandomAccessStream, DataWriter};
use crate::service::native_overlay::save::render::render_snapshot;
use crate::service::win32::bitmap::bitmap_to_bytes;
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
    // 1. Render and convert to bytes
    let png_bytes = {
        let snapshot = render_snapshot(&state_arc, &app)?;
        
        // --- INDUSTRIAL IMPROVEMENT: 2x Up-scaling ---
        // WinRT OcrEngine performs significantly better on small/technical fonts when the image 
        // is up-scaled. We perform a 2x Lanczos resize before decoding.
        if let Ok(rgba) = crate::service::win32::bitmap::bitmap_to_rgba_image(snapshot.hbitmap.0) {
            let (w, h) = rgba.dimensions();
            let upscaled = image::imageops::resize(&rgba, w * 2, h * 2, image::imageops::FilterType::Lanczos3);
            let mut cursor = std::io::Cursor::new(Vec::new());
            let _ = upscaled.write_to(&mut cursor, image::ImageFormat::Png);
            cursor.into_inner()
        } else {
            bitmap_to_bytes(snapshot.hbitmap.0, image::ImageFormat::Png, 100)?
        }
    };
    
    // 3. Create WinRT Stream from bytes
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream)?;
    writer.WriteBytes(&png_bytes)?;
    writer.StoreAsync()?.await?;
    writer.FlushAsync()?.await?;
    stream.Seek(0)?;

    // 4. Decode into SoftwareBitmap
    let decoder = BitmapDecoder::CreateAsync(&stream)?.await?;
    let software_bitmap = decoder.GetSoftwareBitmapAsync()?.await?;

    // 5. Run OCR
    let engine = windows::Media::Ocr::OcrEngine::TryCreateFromUserProfileLanguages()?;
    let result = engine.RecognizeAsync(&software_bitmap)?.await?;

    let mut lines = Vec::new();
    let mut full_text = String::new();

    for line in result.Lines()? {
        let mut words = Vec::new();
        let line_text = line.Text()?.to_string();
        
        for word in line.Words()? {
            let rect = word.BoundingRect()?;
            words.push(OcrWord {
                text: word.Text()?.to_string(),
                x: rect.X as f64,
                y: rect.Y as f64,
                width: rect.Width as f64,
                height: rect.Height as f64,
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
        Ok(data) => {
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
            
            // --- INDUSTRIAL FIX: Multi-Monitor DPI Awareness ---
            // Instead of assuming primary monitor, we find the monitor where the selection started.
            let monitor = app.monitor_from_point(selection_rect.left as f64, selection_rect.top as f64)
                .ok().flatten();
            let scale_factor = monitor.as_ref().map(|m| m.scale_factor()).unwrap_or(1.0);
            
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
