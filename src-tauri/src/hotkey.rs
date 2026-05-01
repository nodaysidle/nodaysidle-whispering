use crate::types::HotkeyUpdate;
use crate::AppState;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::str::FromStr;
use tauri::{AppHandle, Emitter, Manager};

pub struct HotkeyController {
    manager: GlobalHotKeyManager,
    current: Option<global_hotkey::hotkey::HotKey>,
}

impl HotkeyController {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            manager: GlobalHotKeyManager::new()
                .map_err(|error| format!("Could not initialize global hotkey manager: {error}"))?,
            current: None,
        })
    }

    pub fn register(&mut self, value: &str) -> Result<(), String> {
        let hotkey = global_hotkey::hotkey::HotKey::from_str(value)
            .map_err(|error| format!("Invalid hotkey \"{value}\": {error}"))?;
        self.manager
            .register(hotkey)
            .map_err(|error| format!("Could not register hotkey {value}: {error}"))?;

        if let Some(current) = self.current.replace(hotkey) {
            let _ = self.manager.unregister(current);
        }

        Ok(())
    }
}

pub fn install_hotkey_event_handler(app: AppHandle) {
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        let state = app.state::<AppState>();
        let pressed = event.state == HotKeyState::Pressed;

        let result = if pressed {
            state.start_recording()
        } else {
            state.stop_recording()
        };

        let hotkey = state
            .status
            .lock()
            .ok()
            .and_then(|status| status.hotkey.clone())
            .unwrap_or_else(|| "registered".to_string());
        if let Err(error) = result {
            if let Ok(mut status) = state.status.lock() {
                status.last_error = Some(error.clone());
            }
            let _ = app.emit("dictation:error", error);
        }
        let _ = app.emit("dictation:hotkey", HotkeyUpdate { hotkey, pressed });
    }));
}
