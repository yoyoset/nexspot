use crate::service::l10n::{self, L10nKey};
use crate::service::native_overlay::render::toolbar::types::{
    ButtonState, ToolGroup, ToolType, ToolbarButton,
};
use crate::service::native_overlay::state::{CaptureEngine, CaptureMode};
use windows::Win32::Foundation::RECT;

pub fn rebuild_for_mode(
    app: &tauri::AppHandle,
    mode: CaptureMode,
    engine: CaptureEngine,
    registry: &crate::service::native_overlay::state::ToolRegistry,
) -> Vec<ToolbarButton> {
    let mut buttons = Vec::new();

    match mode {
        CaptureMode::Standard => {
            // Define toolsets for each engine
            let tools = match engine {
                CaptureEngine::Gdi => vec![
                    (ToolType::Rect, "\u{F3D6}", L10nKey::ToolRect),
                    (ToolType::Ellipse, "\u{F3C1}", L10nKey::ToolEllipse),
                    (ToolType::Line, "\u{F1AF}", L10nKey::ToolLine),
                    (ToolType::Arrow, "\u{EA70}", L10nKey::ToolArrow),
                    (ToolType::Brush, "\u{EB01}", L10nKey::ToolBrush),
                    (ToolType::Number, "\u{EEBC}", L10nKey::ToolSequence),
                    (ToolType::Text, "\u{F201}", L10nKey::ToolText),
                    (ToolType::Mosaic, "\u{EDDF}", L10nKey::ToolMosaic),
                ],
                CaptureEngine::Wgc => vec![
                    (ToolType::Rect, "\u{F3D6}", L10nKey::ToolRect),
                    (ToolType::Ellipse, "\u{F3C1}", L10nKey::ToolEllipse),
                    (ToolType::Line, "\u{F1AF}", L10nKey::ToolLine),
                    (ToolType::Arrow, "\u{EA70}", L10nKey::ToolArrow),
                    (ToolType::Brush, "\u{EB01}", L10nKey::ToolBrush),
                    (ToolType::Number, "\u{EEBC}", L10nKey::ToolSequence),
                    (ToolType::Text, "\u{F201}", L10nKey::ToolText),
                    (ToolType::Mosaic, "\u{EDDF}", L10nKey::ToolMosaic),
                ],
            };

            let group = match engine {
                CaptureEngine::Gdi => ToolGroup::Standard,
                CaptureEngine::Wgc => ToolGroup::HighFidelity,
            };

            for (t, i, key) in tools {
                buttons.push(ToolbarButton {
                    tool: t,
                    group,
                    rect: RECT::default(),
                    state: ButtonState::Normal,
                    icon: i.to_string(),
                    tooltip: l10n::t(app, key),
                    has_divider: false,
                });
            }

            if let Some(last) = buttons.last_mut() {
                last.has_divider = true;
            }

            // --- Group 2: AI (Dynamic/Extensible) ---
            // 2. Dynamic Macros from Registry
            for macro_item in &registry.macros {
                buttons.push(ToolbarButton {
                    tool: ToolType::Macro(macro_item.id.clone()),
                    group: ToolGroup::AI,
                    rect: RECT::default(),
                    state: ButtonState::Normal,
                    icon: macro_item.icon.clone(),
                    tooltip: macro_item.name.clone(),
                    has_divider: false,
                });
            }

            if let Some(last) = buttons.last_mut() {
                last.has_divider = true;
            }

            // --- Group 3: Common Actions ---
            let common_actions = vec![
                (ToolType::Pin, "\u{F039}", L10nKey::ToolPin),
                (ToolType::Save, "\u{F0B3}", L10nKey::ToolSave),
                (ToolType::Copy, "\u{ECD5}", L10nKey::ToolCopy),
                (ToolType::Cancel, "\u{EB99}", L10nKey::ToolCancel),
            ];
            for (t, i, key) in common_actions {
                buttons.push(ToolbarButton {
                    tool: t,
                    group: ToolGroup::Actions,
                    rect: RECT::default(),
                    state: ButtonState::Normal,
                    icon: i.to_string(),
                    tooltip: l10n::t(app, key),
                    has_divider: false,
                });
            }
        }
    }
    buttons
}
