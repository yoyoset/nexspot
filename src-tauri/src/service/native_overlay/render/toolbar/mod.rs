pub mod builder;
pub mod events;
pub mod layout;
pub mod property_bar;
pub mod render;
pub mod tooltip;
pub mod types;

pub use types::{ButtonState, PropertyChange, ToolType, ToolbarButton};

use crate::service::win32::gdi::SafeHDC;
use windows::Win32::Foundation::RECT;

pub struct Toolbar {
    pub buttons: Vec<ToolbarButton>,
    pub rect: RECT,
    pub current_tool: Option<ToolType>,
    pub visible: bool,
    pub margin: i32,
    pub button_size: i32,
    pub spacing: i32,
    pub property_bar_visible: bool,
    pub property_bar_rect: RECT,
    pub is_loading: bool,
    pub is_dragging_opacity: bool,
    pub is_dragging_glow: bool,
}

impl Toolbar {
    pub fn rebuild_for_mode(
        &mut self,
        app: &tauri::AppHandle,
        mode: crate::service::native_overlay::state::CaptureMode,
        engine: crate::service::native_overlay::state::CaptureEngine,
        registry: &crate::service::native_overlay::state::ToolRegistry,
    ) {
        self.buttons = builder::rebuild_for_mode(app, mode, engine, registry);
    }

    pub fn draw(
        &self,
        hdc: &SafeHDC,
        app: &tauri::AppHandle,
        current_color: u32,
        current_font_size: f32,
        current_stroke: f32,
        current_is_filled: bool,
        current_opacity: f32,
        current_glow: f32,
    ) -> anyhow::Result<()> {
        render::draw_toolbar(
            &self.buttons,
            &self.rect,
            self.visible,
            self.is_loading,
            &self.current_tool,
            self.property_bar_visible,
            &self.property_bar_rect,
            self.spacing,
            hdc,
            app,
            current_color,
            current_font_size,
            current_stroke,
            current_is_filled,
            current_opacity,
            current_glow,
        )
    }

    pub fn hide(&mut self) {
        self.visible = false;
        for btn in &mut self.buttons {
            btn.state = ButtonState::Normal;
        }
    }

    pub fn update_layout(
        &mut self,
        selection: RECT,
        window_x: i32,
        window_y: i32,
        window_width: i32,
        window_height: i32,
        enable_advanced_effects: bool,
    ) {
        layout::update_toolbar_layout(
            &mut self.buttons,
            &mut self.rect,
            &self.current_tool,
            &mut self.property_bar_visible,
            &mut self.property_bar_rect,
            window_x,
            window_y,
            window_width,
            window_height,
            selection,
            self.button_size,
            self.margin,
            self.spacing,
            enable_advanced_effects,
        );
        self.visible = self.rect.right - self.rect.left > 0;
    }

    pub fn handle_mouse_move(&mut self, x: i32, y: i32) -> bool {
        if !self.visible {
            return false;
        }
        events::handle_mouse_move(&mut self.buttons, x, y)
    }

    pub fn handle_mouse_down(&mut self, x: i32, y: i32) -> bool {
        if !self.visible {
            return false;
        }
        events::handle_mouse_down(&mut self.buttons, x, y)
    }

    pub fn handle_mouse_up(&mut self, x: i32, y: i32) -> Option<ToolType> {
        if !self.visible {
            return None;
        }
        events::handle_mouse_up(&mut self.buttons, x, y)
    }

    pub fn handle_click(&mut self, x: i32, y: i32) -> Option<ToolType> {
        events::handle_click(&self.buttons, x, y)
    }

    pub fn handle_property_click(
        &self,
        x: i32,
        y: i32,
        enable_advanced_effects: bool,
    ) -> Option<PropertyChange> {
        events::handle_property_click(
            self.property_bar_visible,
            &self.property_bar_rect,
            &self.current_tool,
            x,
            y,
            enable_advanced_effects,
        )
    }

    pub fn handle_property_down(&mut self, x: i32, y: i32, enable_advanced_effects: bool) -> bool {
        let hit = events::hit_test_property_bar(
            self.property_bar_visible,
            &self.property_bar_rect,
            &self.current_tool,
            x,
            y,
            enable_advanced_effects,
        );

        match hit {
            events::PropertyHit::OpacitySlider => {
                self.is_dragging_opacity = true;
                true
            }
            events::PropertyHit::GlowSlider => {
                self.is_dragging_glow = true;
                true
            }
            _ => false,
        }
    }

    pub fn handle_property_move(
        &mut self,
        x: i32,
        y: i32,
        enable_advanced_effects: bool,
    ) -> Option<PropertyChange> {
        events::handle_property_move(
            self.property_bar_visible,
            &self.property_bar_rect,
            &self.current_tool,
            x,
            y,
            enable_advanced_effects,
            self.is_dragging_opacity,
            self.is_dragging_glow,
        )
    }

    pub fn new(app: &tauri::AppHandle) -> Self {
        let mut slf = Self {
            buttons: Vec::new(),
            rect: RECT::default(),
            current_tool: None,
            visible: false,
            margin: 10,
            button_size: 48,
            spacing: 6,
            property_bar_visible: false,
            property_bar_rect: RECT::default(),
            is_loading: false,
            is_dragging_opacity: false,
            is_dragging_glow: false,
        };
        slf.rebuild_for_mode(
            app,
            crate::service::native_overlay::state::CaptureMode::Standard,
            crate::service::native_overlay::state::CaptureEngine::Gdi,
            &crate::service::native_overlay::state::ToolRegistry::default(),
        );
        slf
    }
}

pub fn tool_type_to_drawing_tool(
    tool: &ToolType,
) -> crate::service::native_overlay::state::DrawingTool {
    use crate::service::native_overlay::state::DrawingTool as DT;
    match tool {
        ToolType::Rect => DT::Rect,
        ToolType::Ellipse => DT::Ellipse,
        ToolType::Arrow => DT::Arrow,
        ToolType::Line => DT::Line,
        ToolType::Brush => DT::Brush,
        ToolType::Mosaic => DT::Mosaic,
        ToolType::Text => DT::Text,
        ToolType::Number => DT::Number,
        _ => DT::None,
    }
}
