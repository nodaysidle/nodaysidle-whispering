// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use nodaysidle_whispering_lib::{
    hotkey::install_hotkey_event_handler,
    types::{DictationStatus, InsertionMode},
    whisper::{default_model_path, WhisperEngine},
    AppState,
};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
fn load_model(
    app: AppHandle,
    state: State<'_, AppState>,
    model_path: String,
    language: Option<String>,
) -> Result<DictationStatus, String> {
    state.ensure_can_load_model()?;

    let model_path = model_path.trim().to_string();
    let model_path = if model_path.is_empty() {
        let resource_dir = app.path().resource_dir().ok();
        default_model_path(resource_dir.as_deref())
            .and_then(|path| path.to_str().map(str::to_string))
            .ok_or_else(|| "Choose a Whisper model before loading.".to_string())?
    } else {
        model_path
    };
    let language = language.unwrap_or_else(|| "auto".to_string());
    let engine = WhisperEngine::load(&model_path, &language)?;
    let normalized_language = engine.language().to_string();

    *state
        .engine
        .lock()
        .map_err(|_| "Could not lock Whisper engine state.".to_string())? = Some(engine);

    let mut status = state
        .status
        .lock()
        .map_err(|_| "Could not lock dictation status.".to_string())?;
    status.model_loaded = true;
    status.model_path = Some(model_path);
    status.language = normalized_language;
    status.last_error = None;
    Ok(status.clone())
}

#[tauri::command]
fn start_recording(state: State<'_, AppState>) -> Result<DictationStatus, String> {
    state.start_recording()?;
    state.snapshot_status()
}

#[tauri::command]
fn stop_recording(state: State<'_, AppState>) -> Result<DictationStatus, String> {
    state.stop_recording()?;
    state.snapshot_status()
}

#[tauri::command]
fn set_continuous_mode(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<DictationStatus, String> {
    state.set_continuous_mode(enabled)?;
    state.snapshot_status()
}

#[tauri::command]
fn set_vad_enabled(state: State<'_, AppState>, enabled: bool) -> Result<DictationStatus, String> {
    state.set_vad_enabled(enabled);
    state.snapshot_status()
}

#[tauri::command]
fn set_insertion_mode(
    state: State<'_, AppState>,
    mode: InsertionMode,
) -> Result<DictationStatus, String> {
    state.set_insertion_mode(mode)?;
    state.snapshot_status()
}

#[tauri::command]
fn register_push_to_talk_hotkey(
    state: State<'_, AppState>,
    hotkey: String,
) -> Result<DictationStatus, String> {
    state.register_hotkey(&hotkey)?;
    state.snapshot_status()
}

#[tauri::command]
fn reset_transcript(state: State<'_, AppState>) -> Result<DictationStatus, String> {
    state.reset_transcript()
}

#[tauri::command]
fn get_status(state: State<'_, AppState>) -> Result<DictationStatus, String> {
    state.snapshot_status()
}

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = AppState::new(handle.clone()).map_err(|error| {
                let boxed: Box<dyn std::error::Error> = error.into();
                boxed
            })?;
            app.manage(state);
            install_hotkey_event_handler(handle);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                window.app_handle().state::<AppState>().shutdown();
            }
        })
        .invoke_handler(tauri::generate_handler![
            load_model,
            start_recording,
            stop_recording,
            set_continuous_mode,
            set_vad_enabled,
            set_insertion_mode,
            register_push_to_talk_hotkey,
            reset_transcript,
            get_status
        ])
        .build(tauri::generate_context!())
        .expect("error while building Tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::ExitRequested { .. } = event {
            app_handle.state::<AppState>().shutdown();
        }
    });
}
