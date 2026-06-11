//! PaddleOCR-json 本地组件客户端。
//!
//! 组件来源：https://github.com/hiroi-sora/PaddleOCR-json/releases（Umi-OCR 引擎层），
//! 解压到 {app_data}/ocr/PaddleOCR-json/ 即可被检测到。
//! 协议：常驻子进程，stdin 每行一个 JSON 请求（image_base64），stdout 每行一个 JSON 响应。
//! 语言 = models/config_*.txt 配置文件，切换语言需重启子进程。

use base64::Engine as _;
use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager};

use crate::service::ocr::{OcrLine, OcrResultData, OcrWord};

struct PaddleProc {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    language: String,
}

impl Drop for PaddleProc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn proc_slot() -> &'static Mutex<Option<PaddleProc>> {
    static SLOT: OnceLock<Mutex<Option<PaddleProc>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// 组件目录：{app_data}/ocr/PaddleOCR-json
pub fn component_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default())
        .join("ocr")
        .join("PaddleOCR-json")
}

/// 在组件目录（含一层子目录，release zip 常带顶层文件夹）中找 exe
fn find_exe(dir: &Path) -> Option<PathBuf> {
    let names = ["PaddleOCR-json.exe", "PaddleOCR_json.exe"];
    for n in &names {
        let p = dir.join(n);
        if p.is_file() {
            return Some(p);
        }
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let sub = entry.path();
            if sub.is_dir() {
                for n in &names {
                    let p = sub.join(n);
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }
    }
    None
}

/// 枚举可用语言：models/config_*.txt 的后缀（chinese / en / japan / korean / chinese_cht ...）
fn list_languages_in(exe: &Path) -> Vec<String> {
    let models = exe.parent().map(|p| p.join("models")).unwrap_or_default();
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&models) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if let Some(lang) = name
                    .strip_prefix("config_")
                    .and_then(|s| s.strip_suffix(".txt"))
                {
                    out.push(lang.to_string());
                }
            }
        }
    }
    out.sort();
    out
}

#[derive(Debug, Serialize)]
pub struct PaddleStatus {
    pub installed: bool,
    pub dir: String,
    pub version: Option<String>,
    pub languages: Vec<String>,
}

fn installed_version(dir: &Path) -> Option<String> {
    std::fs::read_to_string(dir.join("version.txt"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[tauri::command]
pub fn get_paddle_status(app: AppHandle) -> PaddleStatus {
    let dir = component_dir(&app);
    let _ = std::fs::create_dir_all(&dir);
    match find_exe(&dir) {
        Some(exe) => PaddleStatus {
            installed: true,
            dir: dir.to_string_lossy().to_string(),
            version: installed_version(&dir),
            languages: list_languages_in(&exe),
        },
        None => PaddleStatus {
            installed: false,
            dir: dir.to_string_lossy().to_string(),
            version: None,
            languages: Vec::new(),
        },
    }
}

/// 关停常驻进程（更新/卸载前必须，否则 exe 被占用无法替换）
pub fn shutdown() {
    *proc_slot().lock().unwrap_or_else(|e| e.into_inner()) = None;
}

// ---------------- 组件下载 / 更新 ----------------

const GH_LATEST: &str = "https://api.github.com/repos/hiroi-sora/PaddleOCR-json/releases/latest";

#[derive(Debug, Serialize, Clone)]
pub struct PaddleUpdateInfo {
    pub current: Option<String>,
    pub latest: Option<String>,
    pub has_update: bool,
}

/// 取最新 release 的 (tag, 资产下载 url)
async fn fetch_latest() -> anyhow::Result<(String, String)> {
    let client = reqwest::Client::new();
    let v: serde_json::Value = client
        .get(GH_LATEST)
        .header("User-Agent", "NexSpot")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let tag = v
        .get("tag_name")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("release tag missing"))?
        .to_string();
    let assets = v
        .get("assets")
        .and_then(|a| a.as_array())
        .ok_or_else(|| anyhow::anyhow!("release assets missing"))?;
    // 选 Windows 包：排除 linux/docker/source，优先 .7z / .zip
    let url = assets
        .iter()
        .filter_map(|a| a.get("browser_download_url").and_then(|u| u.as_str()))
        .filter(|u| {
            let l = u.to_lowercase();
            (l.ends_with(".7z") || l.ends_with(".zip"))
                && !l.contains("linux")
                && !l.contains("docker")
                && !l.contains("source")
        })
        .max_by_key(|u| {
            let l = u.to_lowercase();
            (l.contains("win") as u8) * 2 + l.ends_with(".7z") as u8
        })
        .ok_or_else(|| anyhow::anyhow!("no suitable windows asset in latest release"))?
        .to_string();
    Ok((tag, url))
}

#[tauri::command]
pub async fn check_paddle_update(app: AppHandle) -> Result<PaddleUpdateInfo, String> {
    let current = installed_version(&component_dir(&app));
    match fetch_latest().await {
        Ok((tag, _)) => {
            let has_update = current.as_deref() != Some(tag.as_str());
            Ok(PaddleUpdateInfo { current, latest: Some(tag), has_update })
        }
        Err(e) => {
            log::warn!("[OCR] check paddle update failed: {}", e);
            Ok(PaddleUpdateInfo { current, latest: None, has_update: false })
        }
    }
}

fn emit_progress(app: &AppHandle, phase: &str, downloaded: u64, total: u64) {
    use tauri::Emitter;
    let _ = app.emit(
        "paddle://progress",
        serde_json::json!({ "phase": phase, "downloaded": downloaded, "total": total }),
    );
}

/// 一键下载并安装最新组件（也用于更新：覆盖安装）。
#[tauri::command]
pub async fn download_paddle_component(app: AppHandle) -> Result<(), String> {
    download_inner(&app).await.map_err(|e| {
        emit_progress(&app, "error", 0, 0);
        e.to_string()
    })
}

async fn download_inner(app: &AppHandle) -> anyhow::Result<()> {
    let (tag, url) = fetch_latest().await?;
    log::info!("[OCR] downloading PaddleOCR {} from {}", tag, url);

    // 1. 流式下载到临时文件
    let client = reqwest::Client::new();
    let mut resp = client
        .get(&url)
        .header("User-Agent", "NexSpot")
        .send()
        .await?
        .error_for_status()?;
    let total = resp.content_length().unwrap_or(0);
    let ext = if url.to_lowercase().ends_with(".7z") { "7z" } else { "zip" };
    let tmp = std::env::temp_dir().join(format!("nexspot_paddle_{}.{}", tag, ext));
    {
        use std::io::Write;
        let mut file = std::fs::File::create(&tmp)?;
        let mut downloaded: u64 = 0;
        let mut last_emit = 0u64;
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            // 每 ~512KB 发一次进度，避免事件风暴
            if downloaded - last_emit > 512 * 1024 {
                emit_progress(app, "download", downloaded, total);
                last_emit = downloaded;
            }
        }
        emit_progress(app, "download", downloaded, total);
    }

    // 2. 关停旧进程、清空旧版本目录
    shutdown();
    let dir = component_dir(app);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;

    // 3. 解压（阻塞操作，移到阻塞线程）
    emit_progress(app, "extract", 0, 0);
    let tmp2 = tmp.clone();
    let dir2 = dir.clone();
    let is_7z = ext == "7z";
    tauri::async_runtime::spawn_blocking(move || -> anyhow::Result<()> {
        if is_7z {
            sevenz_rust::decompress_file(&tmp2, &dir2)?;
        } else {
            let f = std::fs::File::open(&tmp2)?;
            zip::ZipArchive::new(f)?.extract(&dir2)?;
        }
        Ok(())
    })
    .await??;

    // 4. 校验 + 版本标记
    if find_exe(&dir).is_none() {
        anyhow::bail!("extracted package does not contain PaddleOCR-json.exe");
    }
    std::fs::write(dir.join("version.txt"), &tag)?;
    let _ = std::fs::remove_file(&tmp);

    emit_progress(app, "done", 0, 0);
    log::info!("[OCR] PaddleOCR {} installed", tag);
    Ok(())
}

pub fn is_installed(app: &AppHandle) -> bool {
    find_exe(&component_dir(app)).is_some()
}

/// 确保常驻进程在跑且语言匹配；语言变化时重启。
fn ensure_proc(app: &AppHandle, language: &str) -> anyhow::Result<()> {
    let mut slot = proc_slot().lock().unwrap_or_else(|e| e.into_inner());

    if let Some(p) = slot.as_mut() {
        // 进程还活着且语言一致 → 复用
        if p.language == language && p.child.try_wait().ok().flatten().is_none() {
            return Ok(());
        }
        *slot = None; // Drop 杀掉旧进程
    }

    let dir = component_dir(app);
    let exe = find_exe(&dir).ok_or_else(|| anyhow::anyhow!("PaddleOCR component not installed"))?;
    let exe_dir = exe.parent().unwrap_or(&dir).to_path_buf();
    let models = exe_dir.join("models");
    let config = models.join(format!("config_{}.txt", language));
    if !config.is_file() {
        anyhow::bail!("Paddle language config not found: {}", config.display());
    }

    let mut cmd = Command::new(&exe);
    cmd.current_dir(&exe_dir)
        .arg(format!("-models_path={}", models.display()))
        .arg(format!("-config_path={}", config.display()))
        .arg("-ensure_ascii=0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }

    let mut child = cmd.spawn()?;
    let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("no stdin"))?;
    let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout"))?;
    let mut reader = BufReader::new(stdout);

    // 等待初始化完成标记（首次加载模型可能数秒）
    let mut line = String::new();
    let mut ready = false;
    for _ in 0..200 {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break; // 进程退出
        }
        if line.contains("OCR init completed") || line.contains("OCR initialization completed") {
            ready = true;
            break;
        }
    }
    if !ready {
        let _ = child.kill();
        anyhow::bail!("PaddleOCR init failed (no ready signal)");
    }

    log::info!("[OCR] PaddleOCR ready, language={}", language);
    *slot = Some(PaddleProc {
        child,
        stdin,
        reader,
        language: language.to_string(),
    });
    Ok(())
}

/// 对 PNG 字节执行识别。坐标为输入图像像素系（与选区一致）。
pub fn run_ocr(app: &AppHandle, png_bytes: &[u8], language: &str) -> anyhow::Result<OcrResultData> {
    ensure_proc(app, language)?;

    let mut slot = proc_slot().lock().unwrap_or_else(|e| e.into_inner());
    let p = slot.as_mut().ok_or_else(|| anyhow::anyhow!("paddle proc lost"))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(png_bytes);
    let req = serde_json::json!({ "image_base64": b64 }).to_string();
    p.stdin.write_all(req.as_bytes())?;
    p.stdin.write_all(b"\n")?;
    p.stdin.flush()?;

    let mut line = String::new();
    if p.reader.read_line(&mut line)? == 0 {
        *slot = None; // 进程死了，下次重启
        anyhow::bail!("PaddleOCR process exited unexpectedly");
    }

    let v: serde_json::Value = serde_json::from_str(line.trim())?;
    let code = v.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
    match code {
        100 => {}
        101 => anyhow::bail!("No text detected in selection"),
        _ => anyhow::bail!(
            "PaddleOCR error {}: {}",
            code,
            v.get("data").map(|d| d.to_string()).unwrap_or_default()
        ),
    }

    let mut lines = Vec::new();
    let mut full_text = String::new();
    if let Some(items) = v.get("data").and_then(|d| d.as_array()) {
        for item in items {
            let text = item
                .get("text")
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_string();
            // box: [[x,y];4] 四角点 → 取外接矩形
            let (mut min_x, mut min_y, mut max_x, mut max_y) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
            if let Some(quad) = item.get("box").and_then(|b| b.as_array()) {
                for pt in quad {
                    if let Some(xy) = pt.as_array() {
                        let x = xy.first().and_then(|n| n.as_f64()).unwrap_or(0.0);
                        let y = xy.get(1).and_then(|n| n.as_f64()).unwrap_or(0.0);
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
            }
            if !text.is_empty() && min_x < max_x {
                if !full_text.is_empty() {
                    full_text.push('\n');
                }
                full_text.push_str(&text);
                lines.push(OcrLine {
                    text: text.clone(),
                    // Paddle 返回行级框，整行作为一个可选中单元
                    words: vec![OcrWord {
                        text,
                        x: min_x,
                        y: min_y,
                        width: max_x - min_x,
                        height: max_y - min_y,
                    }],
                });
            }
        }
    }

    if full_text.trim().is_empty() {
        anyhow::bail!("No text detected in selection");
    }

    Ok(OcrResultData {
        lines,
        full_text,
        engine: "PaddleOCR".to_string(),
    })
}
