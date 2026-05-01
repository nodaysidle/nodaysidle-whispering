use crate::types::{AudioFrame, FRAME_SAMPLES, SAMPLE_RATE};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use crossbeam_channel::Sender;

pub struct AudioInput {
    _stream: Stream,
}

impl AudioInput {
    pub fn start(tx: Sender<AudioFrame>, error_tx: Sender<String>) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "No default input device is available.".to_string())?;
        let supported = select_input_config(&device)?;
        let sample_format = supported.sample_format();
        let config: StreamConfig = supported.config();
        let source_rate = config.sample_rate;
        let channels = config.channels as usize;

        let err_fn = move |error| {
            let _ = error_tx.try_send(format!("Audio input stream error: {error}"));
        };

        let stream = match sample_format {
            SampleFormat::F32 => {
                build_stream::<f32>(&device, &config, source_rate, channels, tx, err_fn)?
            }
            SampleFormat::I16 => {
                build_stream::<i16>(&device, &config, source_rate, channels, tx, err_fn)?
            }
            SampleFormat::U16 => {
                build_stream::<u16>(&device, &config, source_rate, channels, tx, err_fn)?
            }
            _ => {
                return Err(format!(
                    "Unsupported input sample format: {sample_format:?}. Expected f32, i16, or u16."
                ))
            }
        };

        stream
            .play()
            .map_err(|error| format!("Could not start microphone capture: {error}"))?;

        Ok(Self { _stream: stream })
    }
}

fn select_input_config(device: &cpal::Device) -> Result<cpal::SupportedStreamConfig, String> {
    if let Ok(mut configs) = device.supported_input_configs() {
        if let Some(range) = configs.find(|range| {
            range.channels() == 1
                && range.min_sample_rate() <= SAMPLE_RATE
                && range.max_sample_rate() >= SAMPLE_RATE
        }) {
            return Ok(range.with_sample_rate(SAMPLE_RATE));
        }
    }

    let config = device
        .default_input_config()
        .map_err(|error| format!("Could not read default input config: {error}"))?;
    Ok(config)
}

trait ToMonoSample {
    fn to_mono_sample(self) -> f32;
}

impl ToMonoSample for f32 {
    fn to_mono_sample(self) -> f32 {
        self.clamp(-1.0, 1.0)
    }
}

impl ToMonoSample for i16 {
    fn to_mono_sample(self) -> f32 {
        self as f32 / i16::MAX as f32
    }
}

impl ToMonoSample for u16 {
    fn to_mono_sample(self) -> f32 {
        (self as f32 / u16::MAX as f32) * 2.0 - 1.0
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    source_rate: u32,
    channels: usize,
    tx: Sender<AudioFrame>,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String>
where
    T: cpal::SizedSample + ToMonoSample + Copy + Send + 'static,
{
    let mut resampler = MonoResampler::new(source_rate, SAMPLE_RATE);
    let mut frame = Vec::with_capacity(FRAME_SAMPLES);

    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let mono = mix_to_mono(data, channels);
                let output = resampler.process(&mono);
                for sample in output {
                    frame.push(sample);
                    if frame.len() == FRAME_SAMPLES {
                        let ready = std::mem::take(&mut frame);
                        frame.reserve(FRAME_SAMPLES);
                        let _ = tx.try_send(AudioFrame { samples: ready });
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(|error| format!("Could not build microphone input stream: {error}"))
}

fn mix_to_mono<T: ToMonoSample + Copy>(data: &[T], channels: usize) -> Vec<f32> {
    if channels == 0 {
        return Vec::new();
    }

    data.chunks(channels)
        .map(|frame| {
            frame
                .iter()
                .map(|sample| sample.to_mono_sample())
                .sum::<f32>()
                / frame.len() as f32
        })
        .collect()
}

struct MonoResampler {
    ratio: f64,
    pos: f64,
    buffer: Vec<f32>,
}

impl MonoResampler {
    fn new(source_rate: u32, target_rate: u32) -> Self {
        Self {
            ratio: source_rate as f64 / target_rate as f64,
            pos: 0.0,
            buffer: Vec::new(),
        }
    }

    fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        self.buffer.extend_from_slice(samples);
        let mut output = Vec::new();

        while self.pos + 1.0 < self.buffer.len() as f64 {
            let i = self.pos.floor() as usize;
            let frac = (self.pos - i as f64) as f32;
            let sample = self.buffer[i] * (1.0 - frac) + self.buffer[i + 1] * frac;
            output.push(sample);
            self.pos += self.ratio;
        }

        let consumed = self.pos.floor() as usize;
        if consumed > 0 {
            let keep_from = consumed.saturating_sub(1);
            self.buffer.drain(0..keep_from);
            self.pos -= keep_from as f64;
        }

        output
    }
}
