//! Command line, for scripting and testing without the UI:
//!
//!   yaptap models                    list models and whether they're installed
//!   yaptap download <model>          download a model (and the VAD)
//!   yaptap transcribe <file.wav>     print the transcript of a WAV file
//!   yaptap --toggle                  start/stop dictation in the running app
//!
//! Returns `None` when the arguments aren't a CLI command, so the app starts.

use crate::{engine, models, settings::SettingsStore, text};
use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const IDENTIFIER: &str = "app.yaptap";

pub fn run(args: &[String]) -> Option<i32> {
    let cmd = args.get(1)?.as_str();
    let result = match cmd {
        "models" => list(),
        "download" => download(args.get(2).map(String::as_str).unwrap_or(models::DEFAULT_MODEL)),
        "transcribe" => match args.get(2) {
            Some(file) => transcribe(file),
            None => Err(anyhow!("usage: yaptap transcribe <file.wav>")),
        },
        "help" | "--help" | "-h" => {
            println!("yaptap [models | download <model> | transcribe <file.wav> | --toggle]");
            Ok(())
        }
        _ => return None,
    };
    Some(match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: {e:#}");
            1
        }
    })
}

/// Same folder Tauri uses for `app_data_dir()`.
fn data_dir() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"));
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "linux")]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")));
    let dir = base.context("no home folder")?.join(IDENTIFIER);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn list() -> Result<()> {
    let store = models::ModelStore::new(&data_dir()?);
    for m in models::CATALOG {
        let mark = if store.is_installed(m) { "✓" } else { " " };
        println!("{mark} {:<16} {:>5} MB  {}", m.id, m.size / 1_000_000, m.name);
    }
    Ok(())
}

fn download(id: &str) -> Result<()> {
    let store = models::ModelStore::new(&data_dir()?);
    let rt = tokio::runtime::Runtime::new()?;
    for wanted in [models::VAD_MODEL, id] {
        let m = models::find(wanted).ok_or_else(|| anyhow!("unknown model {wanted}"))?;
        if store.is_installed(m) {
            println!("{} is already installed", m.name);
            continue;
        }
        let mut last = 0;
        rt.block_on(store.download(m, Arc::new(AtomicBool::new(false)), |done, total| {
            let pct = done * 100 / total.max(1);
            if pct != last {
                last = pct;
                eprint!("\r{}: {pct}%", m.name);
            }
        }))?;
        eprintln!("\r{}: done, checksum OK", m.name);
    }
    Ok(())
}

fn transcribe(file: &str) -> Result<()> {
    let dir = data_dir()?;
    let settings = SettingsStore::load(&dir).get();
    let store = models::ModelStore::new(&dir);
    let model = models::find(&settings.model).context("unknown model in settings")?;
    if !store.is_installed(model) {
        return Err(anyhow!("{} is not installed. Run: yaptap download {}", model.name, model.id));
    }
    let vad = models::find(models::VAD_MODEL)
        .filter(|v| store.is_installed(v))
        .map(|v| store.path(v));
    let pcm = read_wav_16k_mono(file)?;

    let engine = engine::Engine::spawn();
    let t = std::time::Instant::now();
    engine.load(store.path(model), vad)?;
    eprintln!("loaded {} in {:?}", model.name, t.elapsed());
    let t = std::time::Instant::now();
    let raw = engine.transcribe(pcm, settings.language, settings.vocabulary)?;
    eprintln!("transcribed in {:?}", t.elapsed());
    match raw {
        Some(raw) => println!(
            "{}",
            text::process(&raw, &text::Options { replacements: &settings.replacements, trailing_space: false })
        ),
        None => eprintln!("(no speech found)"),
    }
    engine.shutdown();
    Ok(())
}

/// Reads a WAV file and converts it to 16 kHz mono f32.
fn read_wav_16k_mono(path: &str) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path).with_context(|| format!("can't open {path}"))?;
    let spec = reader.spec();
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<_, _>>()?,
        hound::SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / max))
                .collect::<Result<_, _>>()?
        }
    };
    let ch = spec.channels as usize;
    let mono: Vec<f32> = samples.chunks(ch).map(|f| f.iter().sum::<f32>() / ch as f32).collect();
    if spec.sample_rate == 16_000 {
        return Ok(mono);
    }
    // Linear resampling is fine for speech recognition input.
    let ratio = spec.sample_rate as f64 / 16_000.0;
    let n = (mono.len() as f64 / ratio) as usize;
    Ok((0..n)
        .map(|i| {
            let pos = i as f64 * ratio;
            let j = pos as usize;
            let frac = (pos - j as f64) as f32;
            let a = mono[j];
            let b = *mono.get(j + 1).unwrap_or(&a);
            a + (b - a) * frac
        })
        .collect())
}
