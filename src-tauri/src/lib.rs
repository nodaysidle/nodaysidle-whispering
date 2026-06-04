mod audio;
mod buffer;

pub mod hotkey;
mod input;
mod pipeline;
pub mod types;
mod vad;
pub mod whisper;

use audio::AudioInput;

use crossbeam_channel::bounded;
use hotkey::HotkeyController;
use log::{error, info, warn};
use pipeline::{spawn_pipeline, PipelineControl};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use tauri::Emitter;
use types::{DictationStatus, InsertionMode};
use whisper::WhisperEngine;

pub struct AppState {
    _audio: Mutex<Option<AudioInput>>,
    pub control: Arc<PipelineControl>,
    pub status: Arc<Mutex<DictationStatus>>,
    pub engine: Arc<Mutex<Option<WhisperEngine>>>,
    pipeline_thread: Mutex<Option<JoinHandle<()>>>,
    inference_thread: Mutex<Option<JoinHandle<()>>>,
    audio_error_thread: Mutex<Option<JoinHandle<()>>>,
    hotkeys: Mutex<HotkeyController>,
    shutting_down: AtomicBool,
}

impl AppState {
    pub fn new(app: tauri::AppHandle) -> Result<Self, String> {
        info!("Initializing application state");
        let (audio_tx, audio_rx) = bounded(128);
        let (audio_error_tx, audio_error_rx) = bounded(8);
        let handles = spawn_pipeline(app.clone(), audio_rx);
        let audio = match AudioInput::start(audio_tx, audio_error_tx) {
            Ok(audio) => Some(audio),
            Err(error) => {
                error!("Failed to start audio input: {}", error);
                if let Ok(mut status) = handles.status.lock() {
                    status.last_error = Some(error);
                }
                warn!("Audio input is not available");
                None
            }
        };
        let hotkeys = HotkeyController::new();
        let error_status = Arc::clone(&handles.status);
        let error_control = Arc::clone(&handles.control);
        let audio_error_thread = thread::Builder::new()
            .name("audio-error-monitor".to_string())
            .spawn(move || {
                while error_control.is_running() {
                    match audio_error_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                        Ok(error) => {
                            error!("Audio input error: {}", error);
                            if let Ok(mut status) = error_status.lock() {
                                status.last_error = Some(error.clone());
                            }
                            let _ = app.emit("dictation:error", error);
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                        Err(_) => {
                            info!("Audio error channel closed");
                            break;
                        }
                    }
                }
            })
            .map_err(|error| format!("Could not spawn audio error monitor: {error}"))?;

        Ok(Self {
            _audio: Mutex::new(audio),
            control: handles.control,
            status: handles.status,
            engine: handles.engine,
            pipeline_thread: Mutex::new(Some(handles.pipeline_thread)),
            inference_thread: Mutex::new(Some(handles.inference_thread)),
            audio_error_thread: Mutex::new(Some(audio_error_thread)),
            hotkeys: Mutex::new(hotkeys),
            shutting_down: AtomicBool::new(false),
        })
    }

    pub fn shutdown(&self) {
        if self.shutting_down.swap(true, Ordering::SeqCst) {
            return;
        }
        info!("Shutting down application");

        self.control.request_stop();

        if let Ok(mut audio) = self._audio.lock() {
            info!("Stopping audio input");
            *audio = None;
        }

        if let Ok(mut pipeline_thread) = self.pipeline_thread.lock() {
            if let Some(thread) = pipeline_thread.take() {
                info!("Waiting for pipeline thread to join");
                let _ = thread.join();
                info!("Pipeline thread joined");
            }
        }

        if let Ok(mut inference_thread) = self.inference_thread.lock() {
            if let Some(thread) = inference_thread.take() {
                info!("Waiting for inference thread to join");
                let _ = thread.join();
                info!("Inference thread joined");
            }
        }

        if let Ok(mut audio_error_thread) = self.audio_error_thread.lock() {
            if let Some(thread) = audio_error_thread.take() {
                info!("Waiting for audio error thread to join");
                let _ = thread.join();
                info!("Audio error thread joined");
            }
        }

        if let Ok(mut engine) = self.engine.lock() {
            info!("Dropping Whisper engine");
            *engine = None;
        }
        info!("Shutdown complete");
    }

    fn is_shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::SeqCst)
    }

    pub fn snapshot_status(&self) -> Result<DictationStatus, String> {
        self.status
            .lock()
            .map(|status| status.clone())
            .map_err(|_| "Could not lock dictation status.".to_string())
    }

    pub fn ensure_ready_to_record(&self) -> Result<(), String> {
        if self.is_shutting_down() {
            return Err("Application is shutting down.".to_string());
        }

        let has_model = self
            .engine
            .lock()
            .map_err(|_| "Could not lock Whisper engine state.".to_string())?
            .is_some();
        if !has_model {
            return Err("Load a Whisper model before starting dictation.".to_string());
        }

        Ok(())
    }

    pub fn ensure_can_load_model(&self) -> Result<(), String> {
        if self.is_shutting_down() {
            return Err("Application is shutting down.".to_string());
        }

        if self.control.recording.load(Ordering::SeqCst)
            || self.control.continuous_mode.load(Ordering::SeqCst)
        {
            return Err(
                "Stop recording and continuous mode before loading another model.".to_string(),
            );
        }

        Ok(())
    }

    pub fn start_recording(&self) -> Result<(), String> {
        self.ensure_ready_to_record()?;
        self.control.recording.store(true, Ordering::SeqCst);
        if let Ok(mut status) = self.status.lock() {
            status.recording = true;
            status.last_error = None;
        }
        Ok(())
    }

    pub fn stop_recording(&self) -> Result<(), String> {
        if self.is_shutting_down() {
            return Ok(());
        }

        self.control.recording.store(false, Ordering::SeqCst);
        self.control
            .finalize_requested
            .store(true, Ordering::SeqCst);
        if let Ok(mut status) = self.status.lock() {
            status.recording = false;
        }
        Ok(())
    }

    pub fn set_continuous_mode(&self, enabled: bool) -> Result<(), String> {
        if self.is_shutting_down() {
            return Ok(());
        }

        if enabled {
            self.ensure_ready_to_record()?;
        }

        self.control
            .continuous_mode
            .store(enabled, Ordering::SeqCst);
        if !enabled {
            self.control
                .finalize_requested
                .store(true, Ordering::SeqCst);
        }
        if let Ok(mut status) = self.status.lock() {
            status.continuous_mode = enabled;
        }
        Ok(())
    }

    pub fn set_vad_enabled(&self, enabled: bool) {
        if self.is_shutting_down() {
            return;
        }

        self.control.vad_enabled.store(enabled, Ordering::SeqCst);
        if let Ok(mut status) = self.status.lock() {
            status.vad_enabled = enabled;
        }
    }

    pub fn set_insertion_mode(&self, mode: InsertionMode) -> Result<(), String> {
        let mut status = self
            .status
            .lock()
            .map_err(|_| "Could not lock dictation status.".to_string())?;

        if self.is_shutting_down() {
            return Ok(());
        }

        status.insertion_mode = mode;
        Ok(())
    }

    pub fn reset_transcript(&self) -> Result<DictationStatus, String> {
        if self.is_shutting_down() {
            return self.snapshot_status();
        }

        self.control.request_reset();
        let mut status = self
            .status
            .lock()
            .map_err(|_| "Could not lock dictation status.".to_string())?;
        status.finalized_text.clear();
        status.partial_text.clear();
        status.latency_ms = 0;
        status.chunk_processing_ms = 0;
        status.last_error = None;
        Ok(status.clone())
    }

    pub fn register_hotkey(&self, value: &str) -> Result<(), String> {
        if self.is_shutting_down() {
            return Ok(());
        }

        self.hotkeys
            .lock()
            .map_err(|_| "Could not lock hotkey manager.".to_string())?
            .register(value)?;

        let mut status = self
            .status
            .lock()
            .map_err(|_| "Could not lock dictation status.".to_string())?;
        status.hotkey = Some(value.to_string());
        Ok(())
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        info!("AppState dropped, shutting down");
        self.shutdown();
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::sync::atomic::Ordering;
//     use tauri::Manager;
//
//     fn setup_test_app() -> tauri::App {
//         let app = tauri::Builder::default()
//             .setup(|app| {
//                 let handle = app.handle().clone();
//                 // In a test environment, AudioInput::start will fail, which is expected.
//                 let state = AppState::new(handle).unwrap();
//                 app.manage(state);
//                 Ok(())
//             })
//             .build(tauri::test::mock_context(tauri::utils::assets::NoOpAssets))
//             .expect("error while building Tauri application");
//         app
//     }
//
//     #[test]
//     fn test_snapshot_status() {
//         let app = setup_test_app();
//         let state = app.state::<AppState>();
//         let status = state.snapshot_status().unwrap();
//         assert_eq!(status.recording, false);
//         assert_eq!(status.continuous_mode, false);
//         assert_eq!(status.model_loaded, false);
//         // Audio input is expected to fail in test environment, so we expect an error.
//         assert!(status.last_error.is_some());
//     }
//
//     #[test]
//     fn test_start_recording_without_model() {
//         let app = setup_test_app();
//         let state = app.state::<AppState>();
//         let result = state.start_recording();
//         assert!(result.is_err());
//         assert_eq!(
//             result.err().unwrap(),
//             "Load a Whisper model before starting dictation.".to_string()
//         );
//     }
//
//     #[test]
//     fn test_load_model_while_recording_fails() {
//         let app = setup_test_app();
//         let state = app.state::<AppState>();
//
//         // Manually set recording state to true for test purposes
//         state.control.recording.store(true, Ordering::SeqCst);
//
//         let result = state.ensure_can_load_model();
//         assert!(result.is_err());
//         assert_eq!(
//             result.err().unwrap(),
//             "Stop recording and continuous mode before loading another model.".to_string()
//         );
//     }
// }
