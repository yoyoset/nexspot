use global_hotkey::GlobalHotKeyManager;
use serde::{Deserialize, Serialize};

// GlobalHotKeyManager is Send + Sync on Windows when using the default features
// and it's intended to be used across threads as long as it's not dropped.
// We keep the wrapper but document its safety rationale.
// Note: In global-hotkey 0.6.0, it doesn't implement Send/Sync by default
// due to X11 types on Linux, but on Windows it's safe.
pub struct SafeGlobalHotKeyManager(pub GlobalHotKeyManager);
unsafe impl Send for SafeGlobalHotKeyManager {}
unsafe impl Sync for SafeGlobalHotKeyManager {}

#[derive(Debug, Clone)]
pub enum HotkeyAction {
    Workflow(CaptureWorkflow),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum AestheticStyle {
    Default,
    Neon,
    PaperCut,
    Sketch,
    Glass,
}

impl Default for AestheticStyle {
    fn default() -> Self {
        AestheticStyle::Default
    }
}


// Duplicate AppConfig removed

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CaptureWorkflow {
    pub id: String,
    pub label: String,
    pub shortcut: String, // e.g. "Alt+A"
    pub action: CaptureAction,
    pub output: CaptureOutput,
    pub enabled: bool,
    #[serde(default)]
    pub is_system: bool, // If true, cannot be deleted
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CaptureOutput {
    pub save_to_file: bool,
    pub save_to_clipboard: bool,
    pub target_folder: Option<String>, // Override global save path
    pub naming_template: String,       // e.g., "capture_%Y%m%d_%H%M%S"
    #[serde(default = "default_format")]
    pub format: String, // "png", "jpg"
}

fn default_format() -> String {
    "png".to_string()
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "config")]
pub enum CaptureAction {
    Selection {
        engine: String,
    }, // "gdi" or "vello"
    Fullscreen {
        engine: String,
    },
    Window {
        engine: String,
    },
    Snapshot {
        engine: String,
        width: i32,
        height: i32,
        #[serde(default)]
        allow_resize: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    pub workflows: Vec<CaptureWorkflow>,

    pub save_path: String,
    pub language: String,
    pub font_family: String,
    #[serde(default = "default_true")]
    pub vello_enabled: bool,
    #[serde(default = "default_true")]
    pub vello_advanced_effects: bool,
    #[serde(default)]
    pub vello_aesthetic_style: AestheticStyle,

    // Legacy Snapshot Config (Migrate to Workflow)
    pub snapshot_enabled: bool,
    pub snapshot_width: i32,
    pub snapshot_height: i32,

    pub selection_engine: String, // "gdi" or "vello"
    pub snapshot_engine: String,  // "gdi" or "vello"

    /// OCR 识别语言："auto"（界面中文优先 zh-Hans，否则用户档案）或 BCP-47 标签
    #[serde(default = "default_ocr_language")]
    pub ocr_language: String,
    /// OCR 引擎："winrt"（Windows 内置）或 "paddle"（PaddleOCR-json 本地组件）
    #[serde(default = "default_ocr_engine")]
    pub ocr_engine: String,
    /// PaddleOCR 识别语言（组件 models/config_*.txt 的后缀名，如 chinese/en/japan）
    #[serde(default = "default_paddle_language")]
    pub ocr_paddle_language: String,


    // Appearance Configuration
    #[serde(default = "default_theme")]
    pub theme: String, // "light", "dark", "system"
    #[serde(default = "default_accent_color")]
    pub accent_color: String, // hex color

    // Performance & Quality
    #[serde(default = "default_jpg_quality")]
    pub jpg_quality: u8,
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,


    #[serde(default = "default_format")]
    pub default_export_format: String, // "png" or "jpg"

    #[serde(default)]
    pub quick_save: bool,

    #[serde(skip_deserializing)]
    #[serde(default)]
    pub registration_errors: Vec<String>,
}


fn default_jpg_quality() -> u8 {
    90
}

fn default_concurrency() -> usize {
    4
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_accent_color() -> String {
    "#7a6ff2".to_string() // Studio periwinkle
}

fn default_ocr_language() -> String {
    "auto".to_string()
}

fn default_ocr_engine() -> String {
    "winrt".to_string()
}

fn default_paddle_language() -> String {
    "chinese".to_string()
}

fn default_true() -> bool {
    true
}

pub fn default_workflows() -> Vec<CaptureWorkflow> {
    vec![
        CaptureWorkflow {
            id: "capture_default".to_string(),
            label: "Capture Selection".to_string(),
            shortcut: "Alt+A".to_string(),
            action: CaptureAction::Selection {
                engine: "gdi".to_string(),
            },
            output: CaptureOutput {
                save_to_file: true,
                save_to_clipboard: true,
                target_folder: None,
                naming_template: "%Y-%m-%d_%H-%M-%S".to_string(),
                format: "png".to_string(),
            },
            enabled: true,
            is_system: true,
        },
        CaptureWorkflow {
            id: "snapshot_default".to_string(),
            label: "Snapshot".to_string(),
            shortcut: "Alt+S".to_string(),
            action: CaptureAction::Snapshot {
                engine: "gdi".to_string(),
                width: 800,
                height: 600,
                allow_resize: true,
            },
            output: CaptureOutput {
                save_to_file: true,
                save_to_clipboard: true,
                target_folder: None,
                naming_template: "snapshot_%Y-%m-%d_%H-%M-%S".to_string(),
                format: "png".to_string(),
            },
            enabled: true, // Default to true so it works out of the box
            is_system: true,
        },
    ]
}
