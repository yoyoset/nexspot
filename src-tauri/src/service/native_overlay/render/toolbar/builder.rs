use crate::service::l10n;
use crate::service::native_overlay::render::toolbar::types::{
    ButtonState, ToolGroup, ToolType, ToolbarButton,
};
use crate::service::native_overlay::state::{CaptureEngine, CaptureMode};
use windows::Win32::Foundation::RECT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub fn rebuild_for_mode(
    app: &tauri::AppHandle,
    mode: CaptureMode,
    engine: CaptureEngine,
    _registry: &crate::service::native_overlay::state::ToolRegistry,
) -> Vec<ToolbarButton> {
    let mut main_buttons = Vec::new();

    match mode {
        CaptureMode::Standard | CaptureMode::Snapshot { .. } | CaptureMode::FixedWindow => {
            // Define toolsets for each engine
            // remixicon 字形按设计稿 lucide 语义就近选取（码点经 cmap 校验）
            let tools = vec![
                (ToolType::Rect, "\u{F3DC}", "backend.tool.rect", "Rectangle"), // checkbox-blank-line ≈ square
                (ToolType::Ellipse, "\u{F3C1}", "backend.tool.ellipse", "Ellipse"), // ≈ circle
                (ToolType::Line, "\u{F1AF}", "backend.tool.line", "Line"), // subtract-line = minus
                (ToolType::Arrow, "\u{EA70}", "backend.tool.arrow", "Arrow"), // arrow-right-up-line = move-up-right
                (ToolType::Brush, "\u{EC86}", "backend.tool.brush", "Brush"), // edit-line ≈ pen-line
                (ToolType::Number, "\u{EDFC}", "backend.tool.sequence", "Sequence"), // hashtag = hash
                (ToolType::Text, "\u{F201}", "backend.tool.text", "Text"), // text = type
                (ToolType::Mosaic, "\u{EDDF}", "backend.tool.mosaic", "Mosaic"), // grid-line ≈ grid-3x3
                (ToolType::Ocr, "\u{F0BD}", "backend.tool.ocr", "OCR"), // scan-line
                (ToolType::Scrolling, "\u{F4AF}", "backend.tool.scrolling", "Scrolling"), // scroll-to-bottom-line
            ];

            let group = match engine {
                CaptureEngine::Gdi => ToolGroup::Standard,
                CaptureEngine::Wgc => ToolGroup::HighFidelity,
            };

            for (t, i, key, fallback) in tools {
                main_buttons.push(ToolbarButton {
                    tool: t,
                    group,
                    rect: RECT::default(),
                    state: ButtonState::Normal,
                    icon: i.to_string(),
                    tooltip: l10n::t(app, key, fallback),
                    has_divider: false,
                });
            }

            if let Some(last) = main_buttons.last_mut() {
                last.has_divider = true;
            }

            // --- Group 2: Common Actions ---
            let common_actions = vec![
                (ToolType::Undo, "\u{EA58}", "backend.tool.undo", "Undo"), // arrow-go-back-line = undo-2
                (ToolType::Pin, "\u{F039}", "backend.tool.pin", "Pin"), // pushpin-line = pin
                (ToolType::Save, "\u{EC5A}", "backend.tool.save", "Save"), // download-line = download（设计 §C）
                (ToolType::Copy, "\u{ECD5}", "backend.tool.copy", "Copy"), // file-copy-line = copy
                (ToolType::Cancel, "\u{EB99}", "backend.tool.cancel", "Cancel"), // close-fill = x（红，见 render.rs）
            ];
            for (t, i, key, fallback) in common_actions {
                main_buttons.push(ToolbarButton {
                    tool: t,
                    group: ToolGroup::Actions,
                    rect: RECT::default(),
                    state: ButtonState::Normal,
                    icon: i.to_string(),
                    tooltip: l10n::t(app, key, fallback),
                    has_divider: false,
                });
            }
        }
    }
    main_buttons
}
