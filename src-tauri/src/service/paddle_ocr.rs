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
    pub languages: Vec<String>,
}

#[tauri::command]
pub fn get_paddle_status(app: AppHandle) -> PaddleStatus {
    let dir = component_dir(&app);
    let _ = std::fs::create_dir_all(&dir);
    match find_exe(&dir) {
        Some(exe) => PaddleStatus {
            installed: true,
            dir: dir.to_string_lossy().to_string(),
            languages: list_languages_in(&exe),
        },
        None => PaddleStatus {
            installed: false,
            dir: dir.to_string_lossy().to_string(),
            languages: Vec::new(),
        },
    }
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

    Ok(OcrResultData { lines, full_text })
}
