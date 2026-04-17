pub mod builder;
pub mod constants;
pub mod events;
pub mod layout;
pub mod property_bar;
pub mod render;
pub mod tooltip;
pub mod types;
pub mod widgets;

pub use types::{ButtonState, PropertyChange, ToolType, ToolbarButton};

use crate::service::win32::gdi::SafeHDC;
use windows::Win32::Foundation::RECT;

pub struct Toolbar {
    pub main_buttons: Vec<ToolbarButton>,
    pub rect: RECT,    // Main toolbar rect
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
        let main = builder::rebuild_for_mode(app, mode, engine, registry);
        self.main_buttons = main;
    }

    pub fn draw(
        &self,
        graphics: &crate::service::win32::gdiplus::GraphicsWrapper,
        hdc: &SafeHDC, // Still needed for some legacy GDI calls if any
        app: &tauri::AppHandle,
        current_color: u32,
        current_font_size: f32,
        current_stroke: f32,
        current_is_filled: bool,
        current_opacity: f32,
        current_glow: f32,
    ) -> anyhow::Result<()> {
        if !self.visible {
            return Ok(());
        }

        // Draw Main Toolbar
        render::draw_toolbar(
            &self.main_buttons,
            &self.rect,
            true, // Already checked self.visible
            self.is_loading,
            &self.current_tool,
            self.property_bar_visible,
            &self.property_bar_rect,
            self.spacing,
            graphics,
            hdc,
            app,
            current_color,
            current_font_size,
            current_stroke,
            current_is_filled,
            current_opacity,
            current_glow,
            builder::Orientation::Horizontal,
        )?;


        Ok(())
    }

    pub fn hide(&mut self) {
        self.visible = false;
        for btn in &mut self.main_buttons {
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
        engine: crate::service::native_overlay::state::CaptureEngine,
    ) {
        layout::update_toolbar_layout(
            &mut self.main_buttons,
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
            engine,
        );
        self.visible = self.rect.right - self.rect.left > 0;
    }

    pub fn handle_mouse_move(&mut self, x: i32, y: i32) -> bool {
        if !self.visible {
            return false;
        }
        events::handle_mouse_move(&mut self.main_buttons, x, y)
    }

    pub fn handle_mouse_down(&mut self, x: i32, y: i32) -> bool {
        if !self.visible {
            return false;
        }
        events::handle_mouse_down(&mut self.main_buttons, x, y)
    }

    pub fn handle_mouse_up(&mut self, x: i32, y: i32) -> Option<ToolType> {
        if !self.visible {
            return None;
        }
        events::handle_mouse_up(&mut self.main_buttons, x, y)
    }

    pub fn handle_click(&mut self, x: i32, y: i32) -> Option<ToolType> {
        events::handle_click(&self.main_buttons, x, y)
    }

    pub fn handle_property_click(
        &self,
        x: i32,
        y: i32,
        enable_advanced_effects: bool,
        current_is_filled: bool,
        engine: crate::service::native_overlay::state::CaptureEngine,
    ) -> Option<PropertyChange> {
        events::handle_property_click(
            self.property_bar_visible,
            &self.property_bar_rect,
            &self.current_tool,
            x,
            y,
            enable_advanced_effects,
            current_is_filled,
            engine,
        )
    }

    pub fn hit_test(&self, x: i32, y: i32) -> bool {
        if !self.visible {
            return false;
        }
        
        // Check Main Toolbar
        if x >= self.rect.left && x < self.rect.right && y >= self.rect.top && y < self.rect.bottom {
            return true;
        }


        // Check Property Bar
        if self.property_bar_visible && x >= self.property_bar_rect.left && x < self.property_bar_rect.right && y >= self.property_bar_rect.top && y < self.property_bar_rect.bottom {
            return true;
        }

        false
    }

    pub fn handle_property_down(&mut self, x: i32, y: i32, enable_advanced_effects: bool, engine: crate::service::native_overlay::state::CaptureEngine) -> bool {
        let hit = events::hit_test_property_bar(
            self.property_bar_visible,
            &self.property_bar_rect,
            &self.current_tool,
            x,
            y,
            enable_advanced_effects,
            engine,
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

    pub fn reset_dragging(&mut self) {
        self.is_dragging_opacity = false;
        self.is_dragging_glow = false;
    }

    pub fn handle_property_move(
        &mut self,
        x: i32,
        y: i32,
        enable_advanced_effects: bool,
        engine: crate::service::native_overlay::state::CaptureEngine,
    ) -> Option<PropertyChange> {
        events::handle_property_move(
            self.property_bar_visible,
            &self.property_bar_rect,
            &self.current_tool,
            x,
            y,
            enable_advanced_effects,
            self.is_dragging_opacity,
            engine,
        )
    }

    pub fn new(app: &tauri::AppHandle) -> Self {
        let mut slf = Self {
            main_buttons: Vec::new(),
            rect: RECT::default(),
            current_tool: None,
            visible: false,
            margin: 4,
            button_size: 44,
            spacing: 2,
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

    /// Create a lightweight stub for parallel rendering.
    /// Industrial standard: Decouples layout calculation (Master) from drawing (Worker).
    pub fn new_stub(rect: RECT) -> Self {
        Self {
            main_buttons: Vec::new(),
            rect,
            current_tool: None,
            visible: true,
            margin: 4,
            button_size: 44,
            spacing: 2,
            property_bar_visible: false,
            property_bar_rect: RECT::default(),
            is_loading: false,
            is_dragging_opacity: false,
            is_dragging_glow: false,
        }
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
