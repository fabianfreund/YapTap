//! Short, quiet feedback tones, synthesized so no audio files are shipped.
//! The output device is opened per sound, so it never stays busy.

use rodio::buffer::SamplesBuffer;
use std::num::NonZero;
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy)]
pub enum Sound {
    Start,
    Stop,
    Error,
}

const RATE: u32 = 44_100;

pub fn play(sound: Sound) {
    thread::spawn(move || {
        let samples = synth(sound);
        let ms = samples.len() as u64 * 1000 / RATE as u64;
        let Ok(mut sink) = rodio::DeviceSinkBuilder::open_default_sink() else { return };
        sink.log_on_drop(false);
        sink.mixer().add(SamplesBuffer::new(
            NonZero::new(1).unwrap(),
            NonZero::new(RATE).unwrap(),
            samples,
        ));
        thread::sleep(Duration::from_millis(ms + 80));
    });
}

/// Two soft sine notes with a quick attack and exponential decay.
fn synth(sound: Sound) -> Vec<f32> {
    let notes: &[(f32, f32)] = match sound {
        Sound::Start => &[(784.0, 0.0), (1046.5, 0.06)], // G5 → C6, rising
        Sound::Stop => &[(1046.5, 0.0), (784.0, 0.06)],  // falling
        Sound::Error => &[(330.0, 0.0), (262.0, 0.09)],
    };
    let len = (RATE as f32 * 0.22) as usize;
    let mut out = vec![0.0f32; len];
    for &(freq, offset) in notes {
        let start = (offset * RATE as f32) as usize;
        for (i, s) in out.iter_mut().enumerate().skip(start) {
            let t = (i - start) as f32 / RATE as f32;
            let attack = (t / 0.004).min(1.0);
            let env = attack * (-t * 28.0).exp();
            *s += 0.12 * env * (std::f32::consts::TAU * freq * t).sin();
        }
    }
    out
}
