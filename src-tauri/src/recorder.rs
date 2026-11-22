//! Microphone capture through handy-recorder: 16 kHz mono, with a live level
//! for the overlay's bars. The mic is opened per recording and closed after,
//! so the system's "mic in use" indicator is only on while you dictate.

use anyhow::{anyhow, Result};
use handy_recorder::{AudioChunk, InputDevice, Recorder, RecorderConfig, Sink};
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::Arc;

/// Called with a 0–1 loudness value about 33 times a second.
pub type LevelFn = Arc<dyn Fn(f32) + Send + Sync>;

pub struct LevelSink {
    samples: Vec<f32>,
    on_level: LevelFn,
}

impl Sink for LevelSink {
    fn process_chunk(&mut self, chunk: AudioChunk<'_>) {
        let real = &chunk.samples[..chunk.valid_frames * chunk.channels as usize];
        self.samples.extend_from_slice(real);
        if !real.is_empty() {
            let rms = (real.iter().map(|s| s * s).sum::<f32>() / real.len() as f32).sqrt();
            // Map roughly -60..-10 dBFS onto 0..1 so normal speech fills the bars.
            let db = 20.0 * rms.max(1e-6).log10();
            (self.on_level)(((db + 60.0) / 50.0).clamp(0.0, 1.0));
        }
    }
}

/// Called once if the microphone can't be opened.
pub type FailFn = Box<dyn FnOnce(String) + Send>;

type Opening = std::thread::JoinHandle<Option<Recorder<LevelSink>>>;

/// Opening a mic can take a while (Bluetooth headsets, a pending permission
/// prompt), so it happens on a background thread and never blocks the
/// shortcut. `stop` waits for the open to finish, so no audio is lost.
#[derive(Default)]
pub struct Mic {
    opening: Mutex<Option<Opening>>,
}

impl Mic {
    pub fn start(&self, device: Option<String>, on_level: LevelFn, on_fail: FailFn) {
        let mut opening = self.opening.lock();
        if opening.is_some() {
            return;
        }
        *opening = Some(std::thread::spawn(move || match open(device, on_level) {
            Ok(r) => Some(r),
            Err(e) => {
                on_fail(e.to_string());
                None
            }
        }));
    }

    /// Stops and returns the recorded 16 kHz mono samples.
    pub fn stop(&self) -> Result<Vec<f32>> {
        let handle = self.opening.lock().take().ok_or_else(|| anyhow!("Not recording"))?;
        let recorder = handle
            .join()
            .ok()
            .flatten()
            .ok_or_else(|| anyhow!("The microphone didn't start"))?;
        let stopped = recorder.stop().map_err(|e| anyhow!("Recording failed: {e}"))?;
        let samples = stopped.sink.samples;
        let _ = recorder.close();
        Ok(samples)
    }

    /// Drops the recording without waiting for the mic.
    pub fn cancel(&self) {
        if let Some(handle) = self.opening.lock().take() {
            std::thread::spawn(move || {
                if let Ok(Some(recorder)) = handle.join() {
                    let _ = recorder.close();
                }
            });
        }
    }
}

fn open(device: Option<String>, on_level: LevelFn) -> Result<Recorder<LevelSink>> {
    let config = RecorderConfig {
        device,
        ..RecorderConfig::speech()
    };
    let recorder = Recorder::open(config.clone())
        .or_else(|e| {
            // The chosen mic may be unplugged: fall back to the default one.
            if config.device.is_some() {
                Recorder::open(RecorderConfig::speech())
            } else {
                Err(e)
            }
        })
        .map_err(|e| anyhow!("Could not open the microphone: {e}"))?;
    recorder
        .start(LevelSink {
            samples: Vec::with_capacity(16_000 * 30),
            on_level,
        })
        .map_err(|e| anyhow!("Could not start recording: {}", e.error))?;
    Ok(recorder)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MicInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

pub fn list_mics() -> Vec<MicInfo> {
    handy_recorder::list_input_devices()
        .unwrap_or_default()
        .into_iter()
        .filter(|d: &InputDevice| !d.is_monitor)
        .map(|d| MicInfo {
            id: d.id,
            name: d.name,
            is_default: d.is_default,
        })
        .collect()
}
