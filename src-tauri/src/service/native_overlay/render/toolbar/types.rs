use windows::Win32::Foundation::RECT;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolType {
    Rect,    // \u{EB7F}
    Arrow,   // \u{EA70}
    Ellipse, // \u{EB7D}
    Line,    // \u{E76B}
    Brush,   // \u{EB01}
    Mosaic,  // \u{EDDF}
    Text,    // \u{F201}
    Number,  // \u{F146}
    Undo,    // \u{EA1B}
    Pin,     // \u{F039}
    Save,    // \u{F0B3}
    SaveAs,  // \u{EF1B}
    Copy,    // \u{ECD5}
    Cancel,  // \u{EB99}
    More,    // \u{EF77}
    Ocr,     // \u{F327}
    Scrolling, // \u{EEA9}
}

pub use crate::service::native_overlay::state::PropertyChange;

pub use ToolType as ToolbarCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolGroup {
    Standard,
    HighFidelity,
    Actions,
}

#[derive(Clone)]
pub struct ToolbarButton {
    pub tool: ToolType,
    pub group: ToolGroup,
    pub rect: RECT,
    pub state: ButtonState,
    pub icon: String,
    pub tooltip: String,
    pub has_divider: bool,
}
