use crate::types::InsertionMode;
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Default)]
pub struct TextInserter;

impl TextInserter {
    pub fn reset_incremental(&mut self) {}

    pub fn insert_committed(&mut self, text: &str, mode: InsertionMode) -> Result<(), String> {
        let trimmed = text.trim();
        if trimmed.is_empty() || mode == InsertionMode::Off {
            return Ok(());
        }

        match mode {
            InsertionMode::Off => Ok(()),
            InsertionMode::IncrementalTyping => type_text_with_trailing_space(trimmed),
            InsertionMode::FinalPaste => paste_text(trimmed),
        }
    }
}

fn type_text(text: &str) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|error| format!("Could not initialize native text insertion: {error}"))?;

    enigo
        .text(text)
        .map_err(|error| format!("Native incremental typing failed: {error}"))
        .or_else(|_| applescript_keystroke(text))
}

fn type_text_with_trailing_space(text: &str) -> Result<(), String> {
    let mut payload = text.to_string();
    if !payload.ends_with(' ') {
        payload.push(' ');
    }
    type_text(&payload)
}

fn paste_text(text: &str) -> Result<(), String> {
    let previous_clipboard = get_clipboard_text().ok();
    set_clipboard_text(text)?;
    thread::sleep(Duration::from_millis(12));
    send_paste_shortcut().or_else(|_| applescript_paste())?;

    if let Some(previous) = previous_clipboard {
        let inserted = text.to_string();
        let _ = thread::Builder::new()
            .name("clipboard-restore".to_string())
            .spawn(move || {
                thread::sleep(Duration::from_millis(700));
                if get_clipboard_text().ok().as_deref() == Some(inserted.as_str()) {
                    let _ = set_clipboard_text(&previous);
                }
            });
    }

    Ok(())
}

fn send_paste_shortcut() -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|error| format!("Could not initialize native paste event: {error}"))?;
    enigo
        .key(Key::Meta, Press)
        .and_then(|_| enigo.key(Key::Unicode('v'), Click))
        .and_then(|_| enigo.key(Key::Meta, Release))
        .map_err(|error| format!("Could not send paste shortcut: {error}"))
}

fn get_clipboard_text() -> Result<String, String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|error| format!("Could not open clipboard: {error}"))?;
    clipboard
        .get_text()
        .map_err(|error| format!("Could not read clipboard text: {error}"))
}

fn set_clipboard_text(text: &str) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|error| format!("Could not open clipboard: {error}"))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|error| format!("Could not set clipboard text: {error}"))
}

#[cfg(target_os = "macos")]
fn applescript_keystroke(text: &str) -> Result<(), String> {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    let script = format!("tell application \"System Events\" to keystroke \"{escaped}\"");
    run_osascript(&script)
}

#[cfg(not(target_os = "macos"))]
fn applescript_keystroke(_text: &str) -> Result<(), String> {
    Err("AppleScript fallback is only available on macOS.".to_string())
}

#[cfg(target_os = "macos")]
fn applescript_paste() -> Result<(), String> {
    run_osascript("tell application \"System Events\" to keystroke \"v\" using command down")
}

#[cfg(not(target_os = "macos"))]
fn applescript_paste() -> Result<(), String> {
    Err("AppleScript fallback is only available on macOS.".to_string())
}

#[cfg(target_os = "macos")]
fn run_osascript(script: &str) -> Result<(), String> {
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|error| format!("Could not run AppleScript fallback: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
