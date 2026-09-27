use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use wasapi::{get_default_device, initialize_mta, Direction, SampleType, StreamMode, WaveFormat};

use crate::whisper::{TranscriptSegment, WhisperTranscriber};

#[derive(Debug, Serialize, Clone)]
pub struct AudioLevel {
    pub percent: u8,
    pub detected: bool,
}

pub struct AudioCapture {
    running: Arc<AtomicBool>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl Default for AudioCapture {
    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread: Mutex::new(None),
        }
    }
}

impl AudioCapture {
    pub fn start(&self, app: AppHandle, model_path: String) -> Result<(), String> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let running = Arc::clone(&self.running);
        let handle = thread::Builder::new()
            .name("wasapi-loopback".to_string())
            .spawn(move || {
                let result = capture_loop(&app, &running, &model_path);
                running.store(false, Ordering::SeqCst);

                if let Err(error) = result {
                    let _ = app.emit("audio-error", error);
                }
            })
            .map_err(|error| {
                self.running.store(false, Ordering::SeqCst);
                format!("Unable to start audio capture: {error}")
            })?;

        *self
            .thread
            .lock()
            .map_err(|_| "Audio capture state is unavailable.".to_string())? = Some(handle);
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self
            .thread
            .lock()
            .map_err(|_| "Audio capture state is unavailable.".to_string())?
            .take()
        {
            handle
                .join()
                .map_err(|_| "Audio capture did not stop cleanly.".to_string())?;
        }

        Ok(())
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Ok(mut thread) = self.thread.lock() {
            if let Some(handle) = thread.take() {
                let _ = handle.join();
            }
        }
    }
}

fn capture_loop(app: &AppHandle, running: &AtomicBool, model_path: &str) -> Result<(), String> {
    if initialize_mta().is_err() {
        return Err("Unable to initialize Windows audio.".to_string());
    }

    // Render direction with a capture client reads the default playback mix (WASAPI loopback).
    let device = get_default_device(&Direction::Render)
        .map_err(|_| "No Windows playback device is available.".to_string())?;
    let mut audio_client = device
        .get_iaudioclient()
        .map_err(|_| "Unable to open the Windows playback device.".to_string())?;
    let format = WaveFormat::new(32, 32, &SampleType::Float, 44_100, 2, None);
    let (_, min_period) = audio_client
        .get_device_period()
        .map_err(|_| "Unable to read the playback device settings.".to_string())?;
    let mode = StreamMode::EventsShared {
        autoconvert: true,
        buffer_duration_hns: min_period,
    };

    audio_client
        .initialize_client(&format, &Direction::Render, &mode)
        .map_err(|_| "Unable to initialize Windows audio capture.".to_string())?;
    let event = audio_client
        .set_get_eventhandle()
        .map_err(|_| "Unable to start the Windows audio event stream.".to_string())?;
    let capture_client = audio_client
        .get_audiocaptureclient()
        .map_err(|_| "Unable to read Windows playback audio.".to_string())?;
    let mut samples = VecDeque::new();
    let mut whisper = WhisperTranscriber::new(model_path)?;
    let mut transcription_audio = Vec::with_capacity(80_000);
    let mut transcription_offset_ms = 0_u64;

    audio_client
        .start_stream()
        .map_err(|_| "Unable to start Windows audio capture.".to_string())?;

    while running.load(Ordering::SeqCst) {
        capture_client
            .read_from_device_to_deque(&mut samples)
            .map_err(|_| "Windows audio capture stopped unexpectedly.".to_string())?;
        emit_level_and_collect_audio(app, &mut samples, &mut transcription_audio)?;
        if transcription_audio.len() >= 80_000 {
            let chunk = std::mem::replace(&mut transcription_audio, Vec::with_capacity(80_000));
            let segments = whisper.transcribe(&chunk, transcription_offset_ms)?;
            transcription_offset_ms += (chunk.len() as u64 * 1000) / 16_000;
            for segment in segments {
                emit_transcript(app, segment)?;
            }
        }

        if event.wait_for_event(250).is_err() {
            break;
        }
    }

    let _ = audio_client.stop_stream();
    let _ = app.emit(
        "audio-level",
        AudioLevel {
            percent: 0,
            detected: false,
        },
    );
    Ok(())
}

fn emit_level_and_collect_audio(
    app: &AppHandle,
    samples: &mut VecDeque<u8>,
    transcription_audio: &mut Vec<f32>,
) -> Result<(), String> {
    const BYTES_PER_SAMPLE: usize = 4;
    const CHANNELS: usize = 2;
    const WINDOW_BYTES: usize = 4_096 * BYTES_PER_SAMPLE * CHANNELS;

    if samples.len() < WINDOW_BYTES {
        return Ok(());
    }

    let mut peak = 0.0_f32;
    let mut sum = 0.0_f32;
    let mut count = 0;
    let mut mono_window = Vec::with_capacity(WINDOW_BYTES / (BYTES_PER_SAMPLE * CHANNELS));
    while count * BYTES_PER_SAMPLE * CHANNELS < WINDOW_BYTES {
        let left = read_float(samples);
        let right = read_float(samples);
        let mono = ((left + right) * 0.5).clamp(-1.0, 1.0);
        let value = mono.abs();
        mono_window.push(mono);
        peak = peak.max(value);
        sum += value * value;
        count += 1;
    }
    resample_to_16khz(&mono_window, transcription_audio);

    let rms = (sum / count.max(1) as f32).sqrt();
    let level = (peak.max(rms * 2.0) * 100.0).round() as u8;
    app.emit(
        "audio-level",
        AudioLevel {
            percent: level,
            detected: level > 2,
        },
    )
    .map_err(|_| "Unable to update the audio level.".to_string())
}

fn resample_to_16khz(source: &[f32], destination: &mut Vec<f32>) {
    const SOURCE_RATE: f32 = 44_100.0;
    const TARGET_RATE: f32 = 16_000.0;
    let output_len = (source.len() as f32 * TARGET_RATE / SOURCE_RATE) as usize;

    for index in 0..output_len {
        let source_position = index as f32 * SOURCE_RATE / TARGET_RATE;
        let lower = source_position.floor() as usize;
        let upper = (lower + 1).min(source.len().saturating_sub(1));
        let fraction = source_position.fract();
        destination.push(source[lower] * (1.0 - fraction) + source[upper] * fraction);
    }
}

fn read_float(samples: &mut VecDeque<u8>) -> f32 {
    let mut raw = [0_u8; 4];
    for byte in &mut raw {
        *byte = samples.pop_front().unwrap_or_default();
    }
    f32::from_le_bytes(raw)
}

fn emit_transcript(app: &AppHandle, segment: TranscriptSegment) -> Result<(), String> {
    app.emit("transcript-segment", segment)
        .map_err(|_| "Unable to update the transcript.".to_string())
}
