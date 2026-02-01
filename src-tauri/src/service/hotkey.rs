use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};
use tauri::{AppHandle, Emitter};

pub struct HotkeyManager {
    #[allow(dead_code)]
    manager: GlobalHotKeyManager,
    #[allow(dead_code)]
    hotkey: HotKey,
}

unsafe impl Send for HotkeyManager {}
unsafe impl Sync for HotkeyManager {}

impl HotkeyManager {
    pub fn new() -> Self {
        let manager = GlobalHotKeyManager::new().unwrap();
        // Register Ctrl + Shift + S
        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyS);
        manager.register(hotkey).unwrap();
        Self { manager, hotkey }
    }
}

pub fn listen_hotkeys(app: AppHandle) {
    use global_hotkey::GlobalHotKeyEvent;

    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                if event.state == global_hotkey::HotKeyState::Pressed {
                    let _ = app.emit("hotkey-pressed", ());
                }
            }
        }
    });
}
