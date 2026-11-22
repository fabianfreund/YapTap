//! Speech recognition. One thread owns the loaded model (transcribe.cpp) and
//! the voice activity detector, and serves jobs from a channel, so native
//! code never runs on the UI thread.

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use transcribe_cpp::{Feature, Model, RunOptions, Session, VadOptions, VadSession};

pub const SAMPLE_RATE: usize = 16_000;

pub struct Job {
    pub pcm: Vec<f32>,
    pub language: Option<String>,
    pub vocabulary: Vec<String>,
    pub reply: Sender<Result<Option<String>>>,
}

enum Msg {
    Load {
        model: PathBuf,
        vad: Option<PathBuf>,
        reply: Sender<Result<()>>,
    },
    Unload(Option<Sender<()>>),
    Run(Job),
}

#[derive(Clone)]
pub struct Engine {
    tx: Sender<Msg>,
}

impl Engine {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("yaptap-engine".into())
            .spawn(move || worker(rx))
            .expect("failed to start the engine thread");
        Self { tx }
    }

    /// Loads a speech model (and the VAD if given). Blocks until done.
    pub fn load(&self, model: PathBuf, vad: Option<PathBuf>) -> Result<()> {
        let (reply, rx) = mpsc::channel();
        self.tx
            .send(Msg::Load { model, vad, reply })
            .map_err(|_| anyhow!("engine stopped"))?;
        rx.recv().map_err(|_| anyhow!("engine stopped"))?
    }

    pub fn unload(&self) {
        let _ = self.tx.send(Msg::Unload(None));
    }

    /// Frees the model and waits until it's gone. Call before the process
    /// exits: ggml's Metal backend asserts if a model is still alive while
    /// static destructors run.
    pub fn shutdown(&self) {
        let (tx, rx) = mpsc::channel();
        if self.tx.send(Msg::Unload(Some(tx))).is_ok() {
            let _ = rx.recv_timeout(std::time::Duration::from_secs(5));
        }
    }

    /// Transcribes 16 kHz mono audio. `Ok(None)` means no speech was found.
    pub fn transcribe(
        &self,
        pcm: Vec<f32>,
        language: Option<String>,
        vocabulary: Vec<String>,
    ) -> Result<Option<String>> {
        let (reply, rx) = mpsc::channel();
        self.tx
            .send(Msg::Run(Job {
                pcm,
                language,
                vocabulary,
                reply,
            }))
            .map_err(|_| anyhow!("engine stopped"))?;
        rx.recv().map_err(|_| anyhow!("engine stopped"))?
    }
}

struct Loaded {
    // The model must outlive its sessions.
    _model: Model,
    session: Session,
    supports_vocabulary: bool,
    vad: Option<(Model, VadSession)>,
}

fn worker(rx: Receiver<Msg>) {
    transcribe_cpp::init_logging();
    if let Err(e) = transcribe_cpp::init_backends_default() {
        log::error!("transcribe.cpp backends failed to start: {e}");
    }
    let mut loaded: Option<Loaded> = None;

    while let Ok(msg) = rx.recv() {
        match msg {
            Msg::Load { model, vad, reply } => {
                loaded = None; // free the old model first
                let result = load(&model, vad.as_deref()).map(|l| loaded = Some(l));
                let _ = reply.send(result);
            }
            Msg::Unload(done) => {
                loaded = None;
                if let Some(done) = done {
                    let _ = done.send(());
                }
            }
            Msg::Run(job) => {
                let result = match loaded.as_mut() {
                    Some(l) => run(l, job.pcm, job.language, job.vocabulary),
                    None => Err(anyhow!("No speech model loaded yet")),
                };
                let _ = job.reply.send(result);
            }
        }
    }
}

fn load(model_path: &std::path::Path, vad_path: Option<&std::path::Path>) -> Result<Loaded> {
    let started = std::time::Instant::now();
    let model = Model::load(model_path).map_err(|e| anyhow!("Could not load the model: {e}"))?;
    let session = model.session().map_err(|e| anyhow!("Could not start the model: {e}"))?;
    let supports_vocabulary = model.supports(Feature::Vocabulary);
    let vad = match vad_path {
        Some(p) => match Model::load(p).and_then(|m| m.vad_session().map(|s| (m, s))) {
            Ok(v) => Some(v),
            Err(e) => {
                log::warn!("VAD unavailable: {e}");
                None
            }
        },
        None => None,
    };
    log::info!(
        "Loaded {} on {} in {:?}",
        model_path.display(),
        model.backend(),
        started.elapsed()
    );
    Ok(Loaded {
        _model: model,
        session,
        supports_vocabulary,
        vad,
    })
}

fn run(
    l: &mut Loaded,
    pcm: Vec<f32>,
    language: Option<String>,
    vocabulary: Vec<String>,
) -> Result<Option<String>> {
    let pcm = match l.vad.as_mut() {
        Some((_, vad)) => match trim_to_speech(vad, &pcm) {
            Some(trimmed) => trimmed,
            None => return Ok(None),
        },
        None => pcm,
    };
    // Very short clips confuse some models; pad to at least one second.
    let mut pcm = pcm;
    if pcm.len() < SAMPLE_RATE {
        pcm.resize(SAMPLE_RATE, 0.0);
    }
    let options = RunOptions {
        language,
        vocabulary: if l.supports_vocabulary { vocabulary } else { Vec::new() },
        ..Default::default()
    };
    let started = std::time::Instant::now();
    let transcript = l
        .session
        .run(&pcm, &options)
        .map_err(|e| anyhow!("Transcription failed: {e}"))?;
    log::info!(
        "Transcribed {:.1}s of audio in {:?}",
        pcm.len() as f32 / SAMPLE_RATE as f32,
        started.elapsed()
    );
    let text = transcript.text.trim().to_string();
    Ok(if text.is_empty() { None } else { Some(text) })
}

/// Cuts the recording to the span that contains speech, keeping a little
/// padding. `None` when there is no speech at all.
fn trim_to_speech(vad: &mut VadSession, pcm: &[f32]) -> Option<Vec<f32>> {
    let result = match vad.run(pcm, &VadOptions::default()) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("VAD failed, using the full recording: {e}");
            return Some(pcm.to_vec());
        }
    };
    let first = result.segments.first()?;
    let last = result.segments.last()?;
    let pad = (SAMPLE_RATE / 4) as i64; // 250 ms
    let start = (first.start_sample - pad).max(0) as usize;
    let end = ((last.end_sample + pad) as usize).min(pcm.len());
    (start < end).then(|| pcm[start..end].to_vec())
}
