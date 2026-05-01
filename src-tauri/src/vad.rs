use crate::types::FRAME_SAMPLES;
use webrtc_vad::{SampleRate, Vad, VadMode};

#[derive(Debug, Clone, Copy)]
pub struct VadDecision {
    pub speech: bool,
}

pub struct SpeechDetector {
    vad: Vad,
    energy_floor: f32,
}

impl SpeechDetector {
    pub fn new() -> Self {
        Self {
            vad: Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, VadMode::Aggressive),
            energy_floor: 0.006,
        }
    }

    pub fn analyze(&mut self, frame: &[f32], enabled: bool) -> VadDecision {
        let rms = rms(frame);
        if !enabled {
            return VadDecision {
                speech: rms > self.energy_floor,
            };
        }

        if frame.len() != FRAME_SAMPLES {
            return VadDecision {
                speech: rms > self.energy_floor,
            };
        }

        let pcm: Vec<i16> = frame
            .iter()
            .map(|sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .collect();
        let speech = self.vad.is_voice_segment(&pcm).unwrap_or(false) || rms > 0.02;
        VadDecision { speech }
    }
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let energy = samples.iter().map(|sample| sample * sample).sum::<f32>();
    (energy / samples.len() as f32).sqrt()
}
