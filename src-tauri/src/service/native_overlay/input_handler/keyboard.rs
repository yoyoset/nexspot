use crate::service::native_overlay::state;
use std::sync::{Arc, RwLock};

pub fn handle_keyboard(
    state_arc: &Arc<RwLock<state::OverlayState>>,
    _toolbar: &mut crate::service::native_overlay::render::toolbar::Toolbar,
    vk: u16,
    is_down: bool,
) -> bool {
    if !is_down {
        return false;
    }

    let mut state = match state_arc.write() {
        Ok(s) => s,
        Err(_) => return false,
    };

    if let Some(drawing) = state.objects.iter_mut().rev().find(|o| o.is_editing_text) {
        let char_code = vk as u32;
        let c = char::from_u32(char_code).unwrap_or('\0');
        
        if c == '\u{8}' { // Backspace
            if let Some(text) = &mut drawing.text {
                text.pop();
            }
        } else if !c.is_control() {
            if drawing.text.is_none() {
                drawing.text = Some(String::new());
            }
            drawing.text.as_mut().unwrap().push(c);
        }
        return true;
    }
    false
}

// Deprecated: used by old events, keeping for compatibility if needed elsewhere but updated to use RwLock
pub fn handle_char(state_arc: &Arc<RwLock<state::OverlayState>>, char_code: u32) -> bool {
    let mut state = match state_arc.write() {
        Ok(s) => s,
        Err(_) => return false,
    };
    if let Some(drawing) = state.objects.iter_mut().rev().find(|o| o.is_editing_text) {
        let c = char::from_u32(char_code).unwrap_or('\0');
        if c == '\u{8}' {
            if let Some(text) = &mut drawing.text {
                text.pop();
            }
        } else if !c.is_control() {
            if drawing.text.is_none() {
                drawing.text = Some(String::new());
            }
            drawing.text.as_mut().unwrap().push(c);
        }
        return true;
    }
    false
}
