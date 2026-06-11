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
    #[serde(default)]
    pub engine: String,
}

#[derive(Debug, Serialize)]
pub struct OcrLanguageInfo {
    pub tag: String,
    pub display_name: String,
}

/// 枚举系统已安装的 OCR 识别语言包（设置页下拉只列真实可用项）
#[tauri::command]
pub fn get_ocr_languages() -> Result<Vec<OcrLanguageInfo>, String> {
    use windows::Media::Ocr::OcrEngine;
    let langs = OcrEngine::AvailableRecognizerLanguages().map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for lang in langs {
        out.push(OcrLanguageInfo {
            tag: lang.LanguageTag().map_err(|e| e.to_string())?.to_string(),
            display_name: lang.DisplayName().map_err(|e| e.to_string())?.to_string(),
        });
    }
    Ok(out)
}

fn try_engine_for_tag(tag: &str) -> Option<windows::Media::Ocr::OcrEngine> {
    use windows::Globalization::Language;
    use windows::Media::Ocr::OcrEngine;
    let lang = Language::CreateLanguage(&windows::core::HSTRING::from(tag)).ok()?;
    if !OcrEngine::IsLanguageSupported(&lang).unwrap_or(false) {
        return None;
    }
    OcrEngine::TryCreateFromLanguage(&lang).ok()
}

/// 选择 OCR 引擎语言：
/// 1) 设置里指定了具体语言 → 用它；
/// 2) auto + 中文界面 → 优先 zh-Hans（需系统装有中文 OCR 语言包）；
/// 3) 否则回退 Windows 用户档案语言。
fn create_ocr_engine(app: &AppHandle) -> anyhow::Result<windows::Media::Ocr::OcrEngine> {
    use windows::Media::Ocr::OcrEngine;

    let (app_lang, ocr_pref) = {
        let state = app.state::<crate::app_state::AppState>();
        let cfg = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
        (cfg.config.language.clone(), cfg.config.ocr_language.clone())
    };

    if ocr_pref != "auto" && !ocr_pref.is_empty() {
        if let Some(engine) = try_engine_for_tag(&ocr_pref) {
            log::info!("[OCR] engine language (user setting): {}", ocr_pref);
            return Ok(engine);
        }
        log::warn!("[OCR] 设置的识别语言 {} 不可用，回退自动选择", ocr_pref);
    }

    if app_lang.starts_with("zh") {
        for tag in ["zh-Hans-CN", "zh-Hans", "zh-CN"] {
            if let Some(engine) = try_engine_for_tag(tag) {
                log::info!("[OCR] engine language (auto): {}", tag);
                return Ok(engine);
            }
        }
        log::warn!("[OCR] 中文 OCR 语言包不可用，回退用户档案语言（设置→时间和语言→语言→中文→选项→添加 OCR 功能）");
    }

    Ok(OcrEngine::TryCreateFromUserProfileLanguages()?)
}

/// 渲染当前选区为 RGBA 图（需在关闭覆盖层之前调用）
pub fn render_selection_image(
    app: &AppHandle,
    state_arc: &Arc<RwLock<crate::service::native_overlay::state::OverlayState>>,
) -> anyhow::Result<image::RgbaImage> {
    let snapshot = render_snapshot(state_arc, app)?;
    Ok(crate::service::win32::bitmap::bitmap_to_rgba_image(snapshot.hbitmap.0)?)
}

pub async fn run_ocr_on_image(
    app: AppHandle,
    rgba: image::RgbaImage,
) -> anyhow::Result<OcrResultData> {
    let (w0, h0) = rgba.dimensions();

    // --- 引擎分发 ---
    let (engine_kind, paddle_lang) = {
        let state = app.state::<crate::app_state::AppState>();
        let cfg = state.config_state.lock().unwrap_or_else(|e| e.into_inner());
        (cfg.config.ocr_engine.clone(), cfg.config.ocr_paddle_language.clone())
    };
    if engine_kind == "paddle" {
        if crate::service::paddle_ocr::is_installed(&app) {
            // Paddle 自带多尺度检测，不做放大；坐标即选区像素系。
            // BMP 编码（无压缩）远快于 PNG，本地管道不在乎体积。
            let mut buf = std::io::Cursor::new(Vec::new());
            image::DynamicImage::ImageRgba8(rgba)
                .to_rgb8()
                .write_to(&mut buf, image::ImageFormat::Bmp)?;
            let app2 = app.clone();
            let bytes = buf.into_inner();
            // 子进程 IO 为阻塞调用，移到阻塞线程，避免占住 tokio worker
            return tauri::async_runtime::spawn_blocking(move || {
                crate::service::paddle_ocr::run_ocr(&app2, &bytes, &paddle_lang)
            })
            .await?;
        }
        log::warn!("[OCR] 引擎设为 PaddleOCR 但组件未安装，回退 Windows 内置");
    }

    // 2. 仅小选区放大 2x（WinRT 对小字号在放大后识别更好）；
    //    大图放大既慢又无收益。Triangle 滤镜足够且远快于 Lanczos3。
    let upscale: u32 = if w0.min(h0) < 700 && w0.max(h0) < 1600 { 2 } else { 1 };
    let img = if upscale == 2 {
        image::imageops::resize(&rgba, w0 * 2, h0 * 2, image::imageops::FilterType::Triangle)
    } else {
        rgba
    };
    let (w, h) = img.dimensions();

    // 3. RGBA→BGRA 原始字节直构 SoftwareBitmap（跳过 PNG 编码/解码整个往返）。
    //    GDI 位图经 GetDIBits 得到的 alpha 恒为 0，必须强制 255，
    //    否则按预乘 alpha 解释即全黑图，识别归零。
    let mut bgra = img.into_raw();
    for px in bgra.chunks_exact_mut(4) {
        px.swap(0, 2);
        px[3] = 255;
    }
    // writer/IBuffer 非 Send，限定作用域在 await 前释放
    let software_bitmap = {
        let writer = DataWriter::new()?;
        writer.WriteBytes(&bgra)?;
        let buffer = writer.DetachBuffer()?;
        SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Bgra8, w as i32, h as i32)?
    };

    // 5. Run OCR — 引擎语言按应用界面语言优先选择。
    //    TryCreateFromUserProfileLanguages 跟随 Windows 用户语言档案，
    //    档案为英文时中文全盲（只识别出拉丁片段）。
    let engine = create_ocr_engine(&app)?;
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
        engine: "Windows OCR".to_string(),
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

    // 2. 渲染选区图像（必须在关闭覆盖层之前 —— 快照依赖 overlay 状态）
    let rgba = render_selection_image(&app, &state_arc).map_err(|e| e.to_string())?;

    // 3. 几何/DPI：窗口与文字层坐标统一逻辑像素
    let monitor = app
        .monitor_from_point(selection_rect.left as f64, selection_rect.top as f64)
        .ok()
        .flatten();
    let scale_factor = monitor.as_ref().map(|m| m.scale_factor()).unwrap_or(1.0);
    let width = (selection_rect.right - selection_rect.left).abs();
    let height = (selection_rect.bottom - selection_rect.top).abs();
    let logical_w = width as f64 / scale_factor;
    let logical_h = height as f64 / scale_factor;
    let logical_x = selection_rect.left as f64 / scale_factor;
    let logical_y = selection_rect.top as f64 / scale_factor;

    // 4. 立即创建结果窗（data 未到 → 前端显示"识别中"动画，给即时反馈）。
    //    先清上一轮结果，否则窗口挂载时 get_last_ocr_result 会闪现旧数据。
    {
        let mut state = state_arc.write().unwrap_or_else(|e| e.into_inner());
        state.current_ocr_data = None;
    }
    let window_label = "ocr-result";
    if let Some(w) = app.get_webview_window(window_label) {
        let _ = w.close();
    }
    let preview_url = tauri::WebviewUrl::App("index.html#ocr-result".into());
    let win = tauri::WebviewWindowBuilder::new(&app, window_label, preview_url)
        .title("OCR Result")
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .inner_size(logical_w, logical_h)
        .position(logical_x, logical_y)
        .build()
        .ok();
    if let Some(w) = &win {
        let _ = w.set_focus();
    }

    // 5. 关闭截图覆盖层：Vello 的 DXGI 覆盖窗同为 topmost 且会压住结果窗，
    //    识别开始后覆盖层已无用处，直接关闭，z-order 不再竞争（GDI 同样统一此行为）。
    {
        let mut manager = state_global.overlay_manager.lock().unwrap_or_else(|e| e.into_inner());
        manager.close_and_reset();
    }

    // 6. 识别
    match run_ocr_on_image(app.clone(), rgba).await {
        Ok(mut data) => {
            // 词坐标是选区物理像素 → 除以 DPI 缩放归一到逻辑像素
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

            // Pin Collection（静默）
            let pin_state = app.state::<crate::service::pin::PinState>();
            let pin_id = uuid::Uuid::new_v4().to_string();
            pin_state.add_pin(pin_id, crate::service::pin::PinData::Text(data.full_text.clone()));

            // 存档供 get_last_ocr_result 兜底
            {
                let mut state = state_arc.write().unwrap_or_else(|e| e.into_inner());
                state.current_ocr_data = Some(data.clone());
            }

            if let Some(w) = &win {
                let _ = w.emit("ocr://data", data.clone());
            }

            let _ = app.emit("pin-collection-updated", ());
            crate::service::activity::log_activity(
                &app,
                "ocr",
                None,
                Some(data.full_text.chars().take(50).collect::<String>() + "..."),
            );

            Ok(data)
        }
        Err(e) => {
            // 失败：关掉"识别中"窗口，错误交由调用方通知
            if let Some(w) = &win {
                let _ = w.close();
            }
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_last_ocr_result(app: AppHandle) -> Result<Option<OcrResultData>, String> {
    let state = app.state::<crate::app_state::AppState>();
    let manager = state.overlay_manager.lock().unwrap();
    let state_lock = manager.state.read().unwrap();
    Ok(state_lock.current_ocr_data.clone())
}
