use crate::AppState;
use tauri::Manager;

pub enum L10nKey {
    // Tray
    TrayDashboard,
    TrayCapture,
    TraySettings,
    TrayExit,

    // Toolbar Tooltips
    ToolRect,
    ToolEllipse,
    ToolLine,
    ToolArrow,
    ToolBrush,
    ToolText,
    ToolMosaic,
    ToolMore,
    ToolSequence,
    ToolPin,
    ToolSave,
    ToolCopy,
    ToolCancel,

    // Engine Switching
    SwitchingToAdvanced,
    AdvancedModeConfirmTitle,
    AdvancedModeConfirmBody,

    // Notifications
    NotificationCopiedTitle,
    NotificationCopiedBody,
    NotificationSavedTitle,
    NotificationSavedBody,
}

pub fn t<R: tauri::Runtime>(app: &tauri::AppHandle<R>, key: L10nKey) -> String {
    let lang = app
        .try_state::<AppState>()
        .and_then(|s| {
            s.config_state
                .lock()
                .ok()
                .map(|c| c.config.language.clone())
        })
        .unwrap_or_else(|| "zh".to_string());
    let is_zh = lang == "zh" || lang == "zh-CN";

    match key {
        L10nKey::TrayDashboard => if is_zh { "仪表盘" } else { "Dashboard" }.to_string(),
        L10nKey::TrayCapture => if is_zh { "立即截图" } else { "Capture" }.to_string(),
        L10nKey::TraySettings => if is_zh { "设置" } else { "Settings" }.to_string(),
        L10nKey::TrayExit => if is_zh { "退出" } else { "Exit" }.to_string(),

        L10nKey::ToolRect => if is_zh { "矩形" } else { "Rectangle" }.to_string(),
        L10nKey::ToolEllipse => if is_zh { "椭圆" } else { "Circle" }.to_string(),
        L10nKey::ToolLine => if is_zh { "直线" } else { "Line" }.to_string(),
        L10nKey::ToolArrow => if is_zh { "箭头" } else { "Arrow" }.to_string(),
        L10nKey::ToolBrush => if is_zh { "画笔" } else { "Brush" }.to_string(),
        L10nKey::ToolText => if is_zh { "文字" } else { "Text" }.to_string(),
        L10nKey::ToolMosaic => if is_zh { "马赛克" } else { "Mosaic" }.to_string(),
        L10nKey::ToolMore => if is_zh {
            "高级模式 (Vello)"
        } else {
            "Advanced Mode (Vello)"
        }
        .to_string(),
        L10nKey::ToolSequence => if is_zh { "序号" } else { "Sequence" }.to_string(),
        L10nKey::ToolPin => if is_zh { "置顶" } else { "Pin" }.to_string(),
        L10nKey::ToolSave => if is_zh { "保存" } else { "Save" }.to_string(),
        L10nKey::ToolCopy => if is_zh { "复制" } else { "Copy" }.to_string(),
        L10nKey::ToolCancel => if is_zh { "退出截图" } else { "Cancel" }.to_string(),

        L10nKey::SwitchingToAdvanced => if is_zh {
            "正在切换至高级引擎..."
        } else {
            "Switching to Advanced Engine..."
        }
        .to_string(),

        L10nKey::AdvancedModeConfirmTitle => if is_zh {
            "切换到高级模式？"
        } else {
            "Switch to Advanced Mode?"
        }
        .to_string(),
        L10nKey::AdvancedModeConfirmBody => if is_zh {
            "高级模式提供高性能渲染 (Vello) 和 WGC 捕捉，但初始化可能需要一点时间。是否继续？"
        } else {
            "Advanced Mode provides high-performance rendering (Vello) and WGC capture, but initialization may take a moment. Carry on?"
        }
        .to_string(),

        L10nKey::NotificationCopiedTitle => if is_zh { "已复制" } else { "Copied" }.to_string(),
        L10nKey::NotificationCopiedBody => if is_zh {
            "图片已复制到剪贴板"
        } else {
            "Image copied to clipboard"
        }
        .to_string(),
        L10nKey::NotificationSavedTitle => if is_zh { "已保存" } else { "Saved" }.to_string(),
        L10nKey::NotificationSavedBody => if is_zh {
            "图片已保存到本地"
        } else {
            "Image saved to captures"
        }
        .to_string(),
    }
}
