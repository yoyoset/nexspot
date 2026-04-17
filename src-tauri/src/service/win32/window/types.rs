use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};

pub struct SafeHWND(pub(crate) HWND);

/// Trait for handling Win32 window messages.
pub trait WindowEventHandler {
    fn on_message(
        &mut self,
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT>;
}

/// A thin wrapper to store a trait object in GWLP_USERDATA.
/// This avoids the "fat pointer" storage problem in 64-bit pointers.
pub(crate) struct Dispatcher {
    pub(crate) handler: *mut dyn WindowEventHandler,
}
