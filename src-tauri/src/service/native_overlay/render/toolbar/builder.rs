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
            let tools = vec![
                (ToolType::Rect, "\u{F3DC}", "backend.tool.rect", "Rectangle"),
                (ToolType::Ellipse, "\u{F3C1}", "backend.tool.ellipse", "Ellipse"),
                (ToolType::Line, "\u{F1AF}", "backend.tool.line", "Line"),
                (ToolType::Arrow, "\u{F5DD}", "backend.tool.arrow", "Arrow"),
                (ToolType::Brush, "\u{EFE0}", "backend.tool.brush", "Brush"),
                (ToolType::Number, "\u{EEBC}", "backend.tool.sequence", "Sequence"),
                (ToolType::Text, "\u{EE60}", "backend.tool.text", "Text"),
                (ToolType::Mosaic, "\u{EDDF}", "backend.tool.mosaic", "Mosaic"),
                (ToolType::Ocr, "\u{F327}", "backend.tool.ocr", "OCR"),
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
                (ToolType::Undo, "\u{EA58}", "backend.tool.undo", "Undo"),
                (ToolType::Pin, "\u{F039}", "backend.tool.pin", "Pin"),
                (ToolType::Save, "\u{F0B3}", "backend.tool.save", "Save"),
                (ToolType::Copy, "\u{ECD5}", "backend.tool.copy", "Copy"),
                (ToolType::Cancel, "\u{EB99}", "backend.tool.cancel", "Cancel"),
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
