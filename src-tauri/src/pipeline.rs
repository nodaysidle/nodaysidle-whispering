use crate::buffer::RollingBuffer;
use crate::input::TextInserter;
use crate::types::{AudioFrame, DictationStatus, InsertionMode, TranscriptionUpdate, SAMPLE_RATE};
use crate::vad::SpeechDetector;
use crate::whisper::{WhisperEngine, WhisperOutput};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const ROLLING_SECONDS: usize = 10;
const WINDOW_SECONDS: u64 = 6;
const PRE_ROLL_MS: u64 = 240;
const INFER_INTERVAL_MS: u64 = 500;
const SILENCE_FINALIZE_MS: u64 = 900;
const MIN_INFERENCE_MS: u64 = 600;
const STABLE_OVERLAP_MS: u64 = 1_200;

#[derive(Default)]
pub struct PipelineControl {
    pub recording: AtomicBool,
    pub continuous_mode: AtomicBool,
    pub vad_enabled: AtomicBool,
    pub finalize_requested: AtomicBool,
    pub reset_requested: AtomicBool,
    pub reset_generation: AtomicU64,
    pub running: AtomicBool,
}

impl PipelineControl {
    pub fn request_stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn request_reset(&self) {
        self.reset_generation.fetch_add(1, Ordering::SeqCst);
        self.reset_requested.store(true, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

pub struct PipelineHandles {
    pub control: Arc<PipelineControl>,
    pub status: Arc<Mutex<DictationStatus>>,
    pub engine: Arc<Mutex<Option<WhisperEngine>>>,
    pub pipeline_thread: thread::JoinHandle<()>,
    pub inference_thread: thread::JoinHandle<()>,
}

#[derive(Debug)]
struct InferenceJob {
    generation: u64,
    samples: Vec<f32>,
    window_start_sample: u64,
    window_end_sample: u64,
    is_final: bool,
}

#[derive(Debug)]
struct InferenceResult {
    job: InferenceJob,
    output: Result<WhisperOutput, String>,
}

pub fn spawn_pipeline(app: AppHandle, audio_rx: Receiver<AudioFrame>) -> PipelineHandles {
    let control = Arc::new(PipelineControl {
        vad_enabled: AtomicBool::new(true),
        running: AtomicBool::new(true),
        ..PipelineControl::default()
    });
    let status = Arc::new(Mutex::new(DictationStatus::default()));
    let engine = Arc::new(Mutex::new(None));
    let (job_tx, job_rx) = bounded::<InferenceJob>(2);
    let (result_tx, result_rx) = bounded::<InferenceResult>(4);

    let inference_engine = Arc::clone(&engine);
    let inference_control = Arc::clone(&control);
    let inference_thread = thread::Builder::new()
        .name("dictation-inference".to_string())
        .spawn(move || run_inference_worker(job_rx, result_tx, inference_engine, inference_control))
        .expect("failed to spawn dictation inference thread");

    let worker_control = Arc::clone(&control);
    let worker_status = Arc::clone(&status);

    let pipeline_thread = thread::Builder::new()
        .name("dictation-pipeline".to_string())
        .spawn(move || {
            run_pipeline(
                app,
                audio_rx,
                job_tx,
                result_rx,
                worker_control,
                worker_status,
            )
        })
        .expect("failed to spawn dictation pipeline thread");

    PipelineHandles {
        control,
        status,
        engine,
        pipeline_thread,
        inference_thread,
    }
}

fn run_pipeline(
    app: AppHandle,
    audio_rx: Receiver<AudioFrame>,
    inference_tx: Sender<InferenceJob>,
    inference_rx: Receiver<InferenceResult>,
    control: Arc<PipelineControl>,
    status: Arc<Mutex<DictationStatus>>,
) {
    let mut buffer = RollingBuffer::new(ROLLING_SECONDS * SAMPLE_RATE as usize);
    let mut detector = SpeechDetector::new();
    let mut inserter = TextInserter;
    let mut in_speech = false;
    let mut speech_start = 0_u64;
    let mut last_voice = 0_u64;
    let mut last_infer = 0_u64;
    let mut committed_until_ms = 0_u64;
    let mut insertion_buffer = String::new();
    let mut inference_inflight = false;
    let mut final_after_inflight = false;
    let mut clear_after_final = false;
    let mut generation = control.reset_generation.load(Ordering::SeqCst);

    if !control.is_running() {
        return;
    }

    loop {
        if !control.is_running() {
            break;
        }

        if control.reset_requested.swap(false, Ordering::SeqCst) {
            generation = control.reset_generation.load(Ordering::SeqCst);
            in_speech = false;
            last_voice = 0;
            last_infer = 0;
            committed_until_ms = 0;
            insertion_buffer.clear();
            inference_inflight = false;
            final_after_inflight = false;
            clear_after_final = false;
            inserter.reset_incremental();
            buffer.clear();
            while inference_rx.try_recv().is_ok() {}
        }

        while let Ok(result) = inference_rx.try_recv() {
            if result.job.generation != generation {
                continue;
            }

            inference_inflight = false;
            apply_inference_result(
                &app,
                &status,
                &mut inserter,
                &mut committed_until_ms,
                &mut insertion_buffer,
                result,
            );

            if clear_after_final {
                in_speech = false;
                buffer.clear();
                clear_after_final = false;
            }

            if final_after_inflight {
                final_after_inflight = false;
                inference_inflight = enqueue_inference(
                    &inference_tx,
                    &buffer,
                    speech_start,
                    buffer.next_sample(),
                    true,
                    generation,
                );
                clear_after_final = inference_inflight;
            }
        }

        if control.finalize_requested.swap(false, Ordering::SeqCst) {
            if inference_inflight {
                final_after_inflight = true;
            } else {
                inference_inflight = enqueue_inference(
                    &inference_tx,
                    &buffer,
                    speech_start,
                    buffer.next_sample(),
                    true,
                    generation,
                );
                clear_after_final = inference_inflight;
                if !inference_inflight {
                    in_speech = false;
                    buffer.clear();
                }
            }
        }

        let frame = match audio_rx.recv_timeout(Duration::from_millis(20)) {
            Ok(frame) => frame,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                if !control.is_running() {
                    break;
                }
                continue;
            }
            Err(_) => break,
        };

        let active = control.recording.load(Ordering::Relaxed)
            || control.continuous_mode.load(Ordering::Relaxed);
        if !active {
            if !inference_inflight && !clear_after_final {
                in_speech = false;
                inserter.reset_incremental();
                buffer.clear();
            }
            continue;
        }

        buffer.push(&frame.samples);
        let now_sample = buffer.next_sample();
        let vad_enabled = control.vad_enabled.load(Ordering::Relaxed);
        let vad = detector.analyze(&frame.samples, vad_enabled);
        let speech_gate = if vad_enabled { vad.speech } else { true };

        if speech_gate {
            if !in_speech {
                let pre_roll_samples = ms_to_samples(PRE_ROLL_MS);
                speech_start = now_sample.saturating_sub(pre_roll_samples);
                committed_until_ms = samples_to_ms(speech_start);
                insertion_buffer.clear();
                in_speech = true;
                inserter.reset_incremental();
            }
            last_voice = now_sample;
        }

        if !in_speech {
            continue;
        }

        let elapsed_since_infer = samples_to_ms(now_sample.saturating_sub(last_infer));
        let speech_len = samples_to_ms(now_sample.saturating_sub(speech_start));
        if !inference_inflight
            && elapsed_since_infer >= INFER_INTERVAL_MS
            && speech_len >= MIN_INFERENCE_MS
        {
            inference_inflight = enqueue_inference(
                &inference_tx,
                &buffer,
                speech_start,
                now_sample,
                false,
                generation,
            );
            last_infer = now_sample;
        }

        let silence_ms = samples_to_ms(now_sample.saturating_sub(last_voice));
        if vad_enabled && silence_ms >= SILENCE_FINALIZE_MS {
            if inference_inflight {
                final_after_inflight = true;
            } else {
                inference_inflight = enqueue_inference(
                    &inference_tx,
                    &buffer,
                    speech_start,
                    now_sample,
                    true,
                    generation,
                );
                clear_after_final = inference_inflight;
                if !inference_inflight {
                    in_speech = false;
                    buffer.clear();
                }
            }
        }
    }
}

fn run_inference_worker(
    job_rx: Receiver<InferenceJob>,
    result_tx: Sender<InferenceResult>,
    engine: Arc<Mutex<Option<WhisperEngine>>>,
    control: Arc<PipelineControl>,
) {
    loop {
        if !control.is_running() {
            break;
        }

        let job = match job_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(job) => job,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
            Err(_) => break,
        };

        let output = {
            let guard = match engine.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    let _ = result_tx.try_send(InferenceResult {
                        job,
                        output: Err("Could not lock Whisper engine state.".to_string()),
                    });
                    continue;
                }
            };

            match guard.as_ref() {
                Some(engine) => engine.transcribe(&job.samples),
                None => Err("Load a whisper.cpp model before starting dictation.".to_string()),
            }
        };

        let _ = result_tx.try_send(InferenceResult { job, output });
    }
}

fn enqueue_inference(
    inference_tx: &Sender<InferenceJob>,
    buffer: &RollingBuffer,
    speech_start: u64,
    now_sample: u64,
    is_final: bool,
    generation: u64,
) -> bool {
    let window_samples = WINDOW_SECONDS * SAMPLE_RATE as u64;
    let window_start = speech_start
        .max(now_sample.saturating_sub(window_samples))
        .max(buffer.start_sample());
    let samples = buffer.slice(window_start, now_sample);
    if samples.len() < SAMPLE_RATE as usize / 2 {
        return false;
    }

    inference_tx
        .try_send(InferenceJob {
            generation,
            samples,
            window_start_sample: window_start,
            window_end_sample: now_sample,
            is_final,
        })
        .is_ok()
}

fn apply_inference_result(
    app: &AppHandle,
    status: &Arc<Mutex<DictationStatus>>,
    inserter: &mut TextInserter,
    committed_until_ms: &mut u64,
    insertion_buffer: &mut String,
    result: InferenceResult,
) {
    let job = result.job;
    let output = match result.output {
        Ok(output) => output,
        Err(error) => {
            if job.is_final {
                set_error(status, &error);
            }
            return;
        }
    };

    let cleaned = clean_text(&output.text);
    if cleaned.is_empty() {
        return;
    }

    let mut status_guard = match status.lock() {
        Ok(status) => status,
        Err(_) => return,
    };
    status_guard.chunk_processing_ms = output.processing_ms;
    status_guard.latency_ms = output.processing_ms + INFER_INTERVAL_MS;
    status_guard.last_error = None;

    let insertion_mode = status_guard.insertion_mode;
    let window_start_ms = samples_to_ms(job.window_start_sample);
    let window_end_ms = samples_to_ms(job.window_end_sample);
    let stable_cutoff_ms = if job.is_final {
        window_end_ms
    } else {
        window_end_ms.saturating_sub(STABLE_OVERLAP_MS)
    };
    let (committed_text, partial_text, committed_until) = split_committed_and_partial(
        &output,
        window_start_ms,
        stable_cutoff_ms,
        *committed_until_ms,
    );

    if !committed_text.is_empty() {
        append_with_overlap(&mut status_guard.finalized_text, &committed_text);
        append_with_overlap(insertion_buffer, &committed_text);
        *committed_until_ms = (*committed_until_ms).max(committed_until);

        if insertion_mode == InsertionMode::IncrementalTyping {
            if let Err(error) = inserter.insert_committed(&committed_text, insertion_mode) {
                status_guard.last_error = Some(error);
            }
        }
    }

    if job.is_final {
        if insertion_mode == InsertionMode::FinalPaste && !insertion_buffer.is_empty() {
            if let Err(error) = inserter.insert_committed(insertion_buffer, insertion_mode) {
                status_guard.last_error = Some(error);
            }
        }
        insertion_buffer.clear();
        inserter.reset_incremental();
        status_guard.partial_text.clear();
    } else {
        status_guard.partial_text = partial_text;
    }

    let update = TranscriptionUpdate {
        text: cleaned,
        finalized_text: status_guard.finalized_text.clone(),
        partial_text: status_guard.partial_text.clone(),
        is_final: job.is_final,
        speech_active: !job.is_final,
        window_start_ms,
        window_end_ms,
        latency_ms: status_guard.latency_ms,
        chunk_processing_ms: output.processing_ms,
    };

    drop(status_guard);
    let _ = app.emit("dictation:update", update);
}

fn split_committed_and_partial(
    output: &WhisperOutput,
    window_start_ms: u64,
    stable_cutoff_ms: u64,
    committed_until_ms: u64,
) -> (String, String, u64) {
    if output.segments.is_empty() {
        return (String::new(), clean_text(&output.text), committed_until_ms);
    }

    let mut committed = Vec::new();
    let mut partial = Vec::new();
    let mut next_committed_until = committed_until_ms;

    for segment in &output.segments {
        let abs_start = window_start_ms + segment.start_ms;
        let abs_end = window_start_ms + segment.end_ms;
        if abs_end <= committed_until_ms || abs_end <= abs_start {
            continue;
        }

        if abs_end <= stable_cutoff_ms {
            committed.push(segment.text.as_str());
            next_committed_until = next_committed_until.max(abs_end);
        } else {
            partial.push(segment.text.as_str());
        }
    }

    (
        clean_text(&committed.join(" ")),
        clean_text(&partial.join(" ")),
        next_committed_until,
    )
}

fn append_with_overlap(target: &mut String, addition: &str) {
    let addition = addition.trim();
    if addition.is_empty() {
        return;
    }

    if target.trim().is_empty() {
        target.push_str(addition);
        return;
    }

    let target_words: Vec<&str> = target.split_whitespace().collect();
    let addition_words: Vec<&str> = addition.split_whitespace().collect();
    let max_overlap = target_words.len().min(addition_words.len());
    let mut overlap = 0;

    for len in (1..=max_overlap).rev() {
        if words_equal(
            &target_words[target_words.len() - len..],
            &addition_words[..len],
        ) {
            overlap = len;
            break;
        }
    }

    let remaining = addition_words[overlap..].join(" ");
    if !remaining.is_empty() {
        if !target.ends_with(' ') {
            target.push(' ');
        }
        target.push_str(&remaining);
    }
}

fn words_equal(left: &[&str], right: &[&str]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| normalize_word(left).eq_ignore_ascii_case(&normalize_word(right)))
}

fn normalize_word(word: &str) -> String {
    word.trim_matches(|c: char| c.is_ascii_punctuation())
        .to_string()
}

fn set_error(status: &Arc<Mutex<DictationStatus>>, error: &str) {
    if let Ok(mut status) = status.lock() {
        status.last_error = Some(error.to_string());
    }
}

fn clean_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn ms_to_samples(ms: u64) -> u64 {
    (ms * SAMPLE_RATE as u64) / 1_000
}

fn samples_to_ms(samples: u64) -> u64 {
    (samples * 1_000) / SAMPLE_RATE as u64
}

#[cfg(test)]
mod tests {
    use super::append_with_overlap;

    #[test]
    fn append_removes_word_overlap() {
        let mut text = "hello world".to_string();
        append_with_overlap(&mut text, "world again");
        assert_eq!(text, "hello world again");
    }

    #[test]
    fn append_keeps_non_overlapping_text() {
        let mut text = "hello".to_string();
        append_with_overlap(&mut text, "world");
        assert_eq!(text, "hello world");
    }
}
