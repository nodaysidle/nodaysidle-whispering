use crate::types::HotkeyUpdate;
use crate::AppState;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use log::warn;
use std::str::FromStr;
use tauri::{AppHandle, Emitter, Manager};

pub struct HotkeyController {
    manager: Option<GlobalHotKeyManager>,
    current: Option<global_hotkey::hotkey::HotKey>,
}

impl HotkeyController {
    pub fn new() -> Self {
        let manager = match GlobalHotKeyManager::new() {
            Ok(manager) => Some(manager),
            Err(error) => {
                warn!("Global hotkeys unavailable: {error}");
                None
            }
        };

        Self {
            manager,
            current: None,
        }
    }

    pub fn register(&mut self, value: &str) -> Result<(), String> {
        let Some(manager) = self.manager.as_ref() else {
            return Err(
                "Global hotkeys are unavailable. Grant Accessibility permission to NoDaysIdle Whispering in macOS System Settings, then restart the app.".to_string(),
            );
        };

        let hotkey = global_hotkey::hotkey::HotKey::from_str(value)
            .map_err(|error| format!("Invalid hotkey \"{value}\": {error}"))?;
        manager
            .register(hotkey)
            .map_err(|error| format!("Could not register hotkey {value}: {error}"))?;

        if let Some(current) = self.current.replace(hotkey) {
            let _ = manager.unregister(current);
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
