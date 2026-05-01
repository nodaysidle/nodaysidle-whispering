use crate::types::SAMPLE_RATE;
use std::path::{Path, PathBuf};
use std::time::Instant;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const DEFAULT_MODEL_FILENAMES: &[&str] = &["ggml-base.en-q5_1.bin"];

#[derive(Debug, Clone)]
pub struct WhisperSegment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Debug, Clone)]
pub struct WhisperOutput {
    pub text: String,
    pub segments: Vec<WhisperSegment>,
    pub processing_ms: u64,
}

pub struct WhisperEngine {
    context: WhisperContext,
    language: String,
    threads: i32,
}

impl WhisperEngine {
    pub fn load(model_path: &str, language: &str) -> Result<Self, String> {
        let path = resolve_model_path(model_path)?;

        let params = WhisperContextParameters::default();
        let context = WhisperContext::new_with_params(&path, params)
            .map_err(|error| format!("Could not load whisper.cpp model: {error:?}"))?;

        Ok(Self {
            context,
            language: normalize_language(language),
            threads: default_thread_count(),
        })
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn transcribe(&self, pcm_16k_mono: &[f32]) -> Result<WhisperOutput, String> {
        if pcm_16k_mono.len() < SAMPLE_RATE as usize / 2 {
            return Err(
                "Skipping inference until at least 500ms of audio is buffered.".to_string(),
            );
        }

        let started = Instant::now();
        let mut state = self
            .context
            .create_state()
            .map_err(|error| format!("Could not create Whisper state: {error:?}"))?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        params.set_n_threads(self.threads);
        params.set_translate(false);
        params.set_no_context(true);
        params.set_no_timestamps(false);
        params.set_single_segment(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_temperature(0.0);
        params.set_no_speech_thold(0.58);
        params.set_suppress_blank(true);

        if self.language == "auto" {
            params.set_detect_language(true);
            params.set_language(None);
        } else {
            params.set_detect_language(false);
            params.set_language(Some(&self.language));
        }

        state
            .full(params, pcm_16k_mono)
            .map_err(|error| format!("Whisper inference failed: {error:?}"))?;

        let mut segments = Vec::new();
        for segment in state.as_iter() {
            let text = segment
                .to_str_lossy()
                .map_err(|error| format!("Could not read Whisper segment: {error:?}"))?
                .trim()
                .to_string();
            if text.is_empty() {
                continue;
            }
            segments.push(WhisperSegment {
                text,
                start_ms: (segment.start_timestamp().max(0) as u64) * 10,
                end_ms: (segment.end_timestamp().max(0) as u64) * 10,
            });
        }

        let text = segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        Ok(WhisperOutput {
            text,
            segments,
            processing_ms: started.elapsed().as_millis() as u64,
        })
    }
}

pub fn resolve_model_path(model_path: &str) -> Result<PathBuf, String> {
    let trimmed = model_path.trim();
    let path = PathBuf::from(trimmed);
    if path.is_file() {
        return Ok(path);
    }

    if let Some(corrected_path) = corrected_ggml_typo_path(&path) {
        if corrected_path.is_file() {
            return Ok(corrected_path);
        }
    }

    Err(format!("Whisper model was not found at {trimmed}."))
}

pub fn default_model_path(resource_dir: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = resource_dir.and_then(default_model_path_from_resource_dir) {
        return Some(path);
    }

    let current_dir = std::env::current_dir().ok()?;
    default_model_path_from(&current_dir)
}

fn default_model_path_from_resource_dir(resource_dir: &Path) -> Option<PathBuf> {
    for filename in DEFAULT_MODEL_FILENAMES {
        let candidate = resource_dir.join("models").join(filename);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

fn default_model_path_from(start_dir: &Path) -> Option<PathBuf> {
    for dir in start_dir.ancestors() {
        for filename in DEFAULT_MODEL_FILENAMES {
            let candidate = dir.join("models").join(filename);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

fn corrected_ggml_typo_path(path: &Path) -> Option<PathBuf> {
    let filename = path.file_name()?.to_str()?;
    let corrected_filename = filename.strip_prefix("gglm-")?;
    let corrected_filename = format!("ggml-{corrected_filename}");

    match path.parent() {
        Some(parent) => Some(parent.join(corrected_filename)),
        None => Some(PathBuf::from(corrected_filename)),
    }
}

fn normalize_language(language: &str) -> String {
    let trimmed = language.trim().to_lowercase();
    if trimmed.is_empty() || trimmed == "auto" {
        return "auto".to_string();
    }

    trimmed
        .split(['-', '_'])
        .next()
        .unwrap_or("auto")
        .to_string()
}

fn default_thread_count() -> i32 {
    num_cpus::get_physical().clamp(2, 6) as i32
}

#[cfg(test)]
mod tests {
    use super::{default_model_path, default_model_path_from, resolve_model_path};
    use std::fs::{self, File};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn resolve_model_path_accepts_existing_model() {
        let temp_dir = temp_test_dir();
        let model_path = temp_dir.join("models").join("ggml-base.en-q5_1.bin");
        fs::create_dir_all(model_path.parent().unwrap()).unwrap();
        File::create(&model_path).unwrap();

        let resolved = resolve_model_path(model_path.to_str().unwrap()).unwrap();
        assert_eq!(resolved, model_path);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn resolve_model_path_corrects_common_gglm_typo_when_file_exists() {
        let temp_dir = temp_test_dir();
        let model_path = temp_dir.join("models").join("ggml-base.en-q5_1.bin");
        let typo_path = temp_dir.join("models").join("gglm-base.en-q5_1.bin");
        fs::create_dir_all(model_path.parent().unwrap()).unwrap();
        File::create(&model_path).unwrap();

        let resolved = resolve_model_path(typo_path.to_str().unwrap()).unwrap();
        assert_eq!(resolved, model_path);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn default_model_path_finds_models_directory_from_child_dir() {
        let temp_dir = temp_test_dir();
        let child_dir = temp_dir.join("src-tauri").join("src");
        let model_path = temp_dir.join("models").join("ggml-base.en-q5_1.bin");
        fs::create_dir_all(&child_dir).unwrap();
        fs::create_dir_all(model_path.parent().unwrap()).unwrap();
        File::create(&model_path).unwrap();

        let resolved = default_model_path_from(&child_dir).unwrap();
        assert_eq!(resolved, model_path);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn default_model_path_prefers_bundled_resource_model() {
        let temp_dir = temp_test_dir();
        let resource_model_path = temp_dir
            .join("Resources")
            .join("models")
            .join("ggml-base.en-q5_1.bin");
        fs::create_dir_all(resource_model_path.parent().unwrap()).unwrap();
        File::create(&resource_model_path).unwrap();

        let resolved = default_model_path(Some(temp_dir.join("Resources").as_path())).unwrap();
        assert_eq!(resolved, resource_model_path);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    fn temp_test_dir() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "nodaysidle-whispering-test-{}-{suffix}-{sequence}",
            std::process::id()
        ))
    }
}
