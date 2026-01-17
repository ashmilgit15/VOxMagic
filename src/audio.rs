//! Audio recording module using cpal for low-latency capture

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use hound::{WavSpec, WavWriter};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::config::{BITS_PER_SAMPLE, CHANNELS};

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("No input device available")]
    NoInputDevice,
    #[error("Failed to get default input config: {0}")]
    ConfigError(String),
    #[error("Failed to build input stream: {0}")]
    StreamError(String),
    #[error("Failed to encode WAV: {0}")]
    EncodingError(String),
    #[error("Recording already in progress")]
    AlreadyRecording,
    #[error("Not currently recording")]
    NotRecording,
}

/// Thread-safe sample buffer
pub type SampleBuffer = Arc<Mutex<Vec<i16>>>;

/// Recording state that can be shared across threads
pub struct RecordingState {
    pub is_recording: Arc<AtomicBool>,
    pub samples: SampleBuffer,
    pub actual_sample_rate: Arc<AtomicU32>,
    pub actual_channels: Arc<AtomicU32>,
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            samples: Arc::new(Mutex::new(Vec::new())),
            actual_sample_rate: Arc::new(AtomicU32::new(16000)),
            actual_channels: Arc::new(AtomicU32::new(1)),
        }
    }
}

impl Default for RecordingState {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RecordingState {
    fn clone(&self) -> Self {
        Self {
            is_recording: self.is_recording.clone(),
            samples: self.samples.clone(),
            actual_sample_rate: self.actual_sample_rate.clone(),
            actual_channels: self.actual_channels.clone(),
        }
    }
}

/// Start recording audio from the default input device
pub fn start_recording(state: &RecordingState) -> Result<cpal::Stream, AudioError> {
    if state.is_recording.load(Ordering::Relaxed) {
        return Err(AudioError::AlreadyRecording);
    }

    // Clear previous samples
    if let Ok(mut samples) = state.samples.lock() {
        samples.clear();
    }

    // Get the default host and input device
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or(AudioError::NoInputDevice)?;

    println!("🎤 Using input device: {}", device.name().unwrap_or_default());

    // Get supported config
    let config = device
        .default_input_config()
        .map_err(|e| AudioError::ConfigError(e.to_string()))?;

    println!("📊 Sample format: {:?}, Rate: {:?}", config.sample_format(), config.sample_rate());

    // Store the actual sample rate and channels from the device
    let actual_rate = config.sample_rate().0;
    let actual_channels = config.channels() as u32;
    state.actual_sample_rate.store(actual_rate, Ordering::Relaxed);
    state.actual_channels.store(actual_channels, Ordering::Relaxed);

    let is_recording = state.is_recording.clone();
    let samples = state.samples.clone();

    // Build the input stream based on sample format
    let stream = match config.sample_format() {
        SampleFormat::I16 => build_stream_i16(&device, &config.into(), samples, is_recording.clone(), actual_channels)?,
        SampleFormat::F32 => build_stream_f32(&device, &config.into(), samples, is_recording.clone(), actual_channels)?,
        format => return Err(AudioError::ConfigError(format!("Unsupported format: {:?}", format))),
    };

    stream.play().map_err(|e| AudioError::StreamError(e.to_string()))?;
    state.is_recording.store(true, Ordering::Relaxed);

    println!("🔴 Recording started...");
    Ok(stream)
}

fn build_stream_i16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: SampleBuffer,
    is_recording: Arc<AtomicBool>,
    channels: u32,
) -> Result<cpal::Stream, AudioError> {
    let err_fn = |err| eprintln!("Stream error: {}", err);

    let stream = device
        .build_input_stream(
            config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                if !is_recording.load(Ordering::Relaxed) {
                    return;
                }

                if let Ok(mut samples_guard) = samples.lock() {
                    // Mix down to mono
                    for frame in data.chunks_exact(channels as usize) {
                        let sum: i32 = frame.iter().map(|&s| s as i32).sum();
                        let mono = (sum / channels as i32) as i16;
                        samples_guard.push(mono);
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| AudioError::StreamError(e.to_string()))?;

    Ok(stream)
}

fn build_stream_f32(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: SampleBuffer,
    is_recording: Arc<AtomicBool>,
    channels: u32,
) -> Result<cpal::Stream, AudioError> {
    let err_fn = |err| eprintln!("Stream error: {}", err);

    let stream = device
        .build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !is_recording.load(Ordering::Relaxed) {
                    return;
                }

                if let Ok(mut samples_guard) = samples.lock() {
                    // Mix down to mono and convert to i16
                    for frame in data.chunks_exact(channels as usize) {
                        let sum: f32 = frame.iter().sum();
                        let avg = sum / channels as f32;
                        let mono_i16 = (avg * 32767.0).clamp(-32768.0, 32767.0) as i16;
                        samples_guard.push(mono_i16);
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| AudioError::StreamError(e.to_string()))?;

    Ok(stream)
}

/// Stop recording and return WAV data
pub fn stop_recording(state: &RecordingState) -> Result<Vec<u8>, AudioError> {
    if !state.is_recording.load(Ordering::Relaxed) {
        return Err(AudioError::NotRecording);
    }

    state.is_recording.store(false, Ordering::Relaxed);
    println!("⏹️ Recording stopped");

    // Get samples and encode to WAV
    let samples = state.samples.lock()
        .map_err(|_| AudioError::EncodingError("Failed to lock samples".to_string()))?
        .clone();

    if samples.is_empty() {
        return Err(AudioError::EncodingError("No audio captured".to_string()));
    }

    // Get the actual sample rate that was used
    let sample_rate = state.actual_sample_rate.load(Ordering::Relaxed);
    println!("📦 Encoding {} samples to WAV at {} Hz...", samples.len(), sample_rate);

    // Encode to WAV in memory with correct sample rate
    encode_wav(&samples, sample_rate)
}

fn encode_wav(samples: &[i16], original_sample_rate: u32) -> Result<Vec<u8>, AudioError> {
    // Target sample rate for Whisper API is 16000Hz
    let target_sample_rate = 16000;

    let processed_samples = if original_sample_rate != target_sample_rate {
        println!("🔄 Downsampling from {}Hz to {}Hz...", original_sample_rate, target_sample_rate);
        downsample(samples, original_sample_rate, target_sample_rate)
    } else {
        samples.to_vec()
    };

    let spec = WavSpec {
        channels: CHANNELS,
        sample_rate: target_sample_rate,
        bits_per_sample: BITS_PER_SAMPLE,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut cursor, spec)
            .map_err(|e| AudioError::EncodingError(e.to_string()))?;

        for &sample in &processed_samples {
            writer.write_sample(sample)
                .map_err(|e| AudioError::EncodingError(e.to_string()))?;
        }

        writer.finalize()
            .map_err(|e| AudioError::EncodingError(e.to_string()))?;
    }

    Ok(cursor.into_inner())
}

fn downsample(samples: &[i16], from: u32, to: u32) -> Vec<i16> {
    if from == to {
        return samples.to_vec();
    }

    let ratio = from as f64 / to as f64;
    let target_len = (samples.len() as f64 / ratio) as usize;
    let mut result = Vec::with_capacity(target_len);

    for i in 0..target_len {
        let pos = i as f64 * ratio;
        let index = pos as usize;

        if index + 1 < samples.len() {
            // Linear interpolation for clearer audio extraction
            let fract = pos - index as f64;
            let s1 = samples[index] as f64;
            let s2 = samples[index + 1] as f64;
            let interpolated = s1 + (s2 - s1) * fract;
            result.push(interpolated as i16);
        } else if index < samples.len() {
            result.push(samples[index]);
        }
    }

    result
}
