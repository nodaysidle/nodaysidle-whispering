use serde::{Deserialize, Serialize};

pub const SAMPLE_RATE: u32 = 16_000;
pub const FRAME_MS: u32 = 20;
pub const FRAME_SAMPLES: usize = (SAMPLE_RATE as usize * FRAME_MS as usize) / 1_000;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InsertionMode {
    Off,
    IncrementalTyping,
    #[default]
    FinalPaste,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationStatus {
    pub recording: bool,
    pub continuous_mode: bool,
    pub vad_enabled: bool,
    pub model_loaded: bool,
    pub language: String,
    pub model_path: Option<String>,
    pub hotkey: Option<String>,
    pub insertion_mode: InsertionMode,
    pub finalized_text: String,
    pub partial_text: String,
    pub latency_ms: u64,
    pub chunk_processing_ms: u64,
    pub last_error: Option<String>,
}

impl Default for DictationStatus {
    fn default() -> Self {
        Self {
            recording: false,
            continuous_mode: false,
            vad_enabled: true,
            model_loaded: false,
            language: "auto".to_string(),
            model_path: None,
            hotkey: None,
            insertion_mode: InsertionMode::FinalPaste,
            finalized_text: String::new(),
            partial_text: String::new(),
            latency_ms: 0,
            chunk_processing_ms: 0,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionUpdate {
    pub text: String,
    pub finalized_text: String,
    pub partial_text: String,
    pub is_final: bool,
    pub speech_active: bool,
    pub window_start_ms: u64,
    pub window_end_ms: u64,
    pub latency_ms: u64,
    pub chunk_processing_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyUpdate {
    pub hotkey: String,
    pub pressed: bool,
}

#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub samples: Vec<f32>,
}
