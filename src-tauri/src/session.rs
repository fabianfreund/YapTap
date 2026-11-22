//! One dictation, start to finish:
//! idle → listening (→ locked for hands-free) → processing → done | error → idle.

use crate::history::Entry;
use crate::settings::TriggerMode;
use crate::sounds::{self, Sound};
use crate::{insert, models, overlay, text, App};
use serde::Serialize;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::Emitter;

/// Held longer than this = push-to-talk. Shorter = a tap that starts
/// hands-free mode.
const HOLD_THRESHOLD: Duration = Duration::from_millis(350);
/// Recordings shorter than this are treated as accidental.
const MIN_RECORDING: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Recording {
        since: Instant,
        locked: bool,
        /// The key press that ended a hands-free recording still has a
        /// release coming; ignore it.
        ignore_release: bool,
    },
    Processing,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PillState {
    /// listening | locked | processing | done | error | hidden
    pub state: &'static str,
    pub message: Option<String>,
}

pub fn is_recording(app: &App) -> bool {
    matches!(*app.phase.lock(), Phase::Recording { .. })
}

fn pill(app: &App, state: &'static str, message: Option<String>) {
    let _ = app.handle.emit("pill", PillState { state, message });
}

pub fn on_shortcut(app: &Arc<App>, pressed: bool) {
    let mode = app.settings.get().trigger_mode;
    let phase = *app.phase.lock();
    match (phase, pressed) {
        (Phase::Idle, true) => start(app),
        (Phase::Recording { locked: true, .. }, true) => {
            if let Phase::Recording { ignore_release, .. } = &mut *app.phase.lock() {
                *ignore_release = true;
            }
            finish(app);
        }
        (Phase::Recording { since, locked: false, ignore_release: false }, false) => {
            let held = since.elapsed();
            if mode == TriggerMode::Hold || (mode == TriggerMode::Auto && held >= HOLD_THRESHOLD) {
                finish(app);
            } else {
                if let Phase::Recording { locked, .. } = &mut *app.phase.lock() {
                    *locked = true;
                }
                pill(app, "locked", None);
            }
        }
        _ => {}
    }
}

/// Starts or stops from the tray, the pill or a command.
pub fn toggle(app: &Arc<App>) {
    let phase = *app.phase.lock();
    match phase {
        Phase::Idle => {
            start(app);
            let started = match &mut *app.phase.lock() {
                Phase::Recording { locked, .. } => {
                    *locked = true;
                    true
                }
                _ => false,
            };
            if started {
                pill(app, "locked", None);
            }
        }
        Phase::Recording { .. } => finish(app),
        Phase::Processing => {}
    }
}

pub fn start(app: &Arc<App>) {
    let settings = app.settings.get();
    let model_ready = models::find(&settings.model).is_some_and(|m| app.models.is_installed(m));
    if !model_ready {
        show_error(app, "Download a speech model first".into());
        crate::open_settings(&app.handle);
        return;
    }

    log::info!("Recording starts");
    let handle = app.handle.clone();
    let on_level = Arc::new(move |level: f32| {
        let _ = handle.emit("level", level);
    });
    let failed = app.clone();
    let on_fail = Box::new(move |message: String| {
        // Only matters if this recording is still running.
        if is_recording(&failed) {
            failed.mic.cancel();
            failed.hotkeys.set_escape(false);
            *failed.phase.lock() = Phase::Idle;
            show_error(&failed, permission_hint(message));
        }
    });
    // Phase first, so a mic that fails instantly is still handled by on_fail.
    *app.phase.lock() = Phase::Recording {
        since: Instant::now(),
        locked: settings.trigger_mode == TriggerMode::Toggle,
        ignore_release: false,
    };
    app.mic.start(settings.microphone.clone(), on_level, on_fail);
    app.hotkeys.set_escape(true);
    if settings.sounds {
        sounds::play(Sound::Start);
    }
    pill(
        app,
        if settings.trigger_mode == TriggerMode::Toggle { "locked" } else { "listening" },
        None,
    );
    overlay::show(&app.handle, settings.overlay_position);
}

pub fn cancel(app: &Arc<App>) {
    if !is_recording(app) {
        return;
    }
    app.mic.cancel();
    app.hotkeys.set_escape(false);
    *app.phase.lock() = Phase::Idle;
    pill(app, "hidden", None);
    overlay::hide(&app.handle);
}

pub fn finish(app: &Arc<App>) {
    let since = match *app.phase.lock() {
        Phase::Recording { since, .. } => since,
        _ => return,
    };
    let too_short = since.elapsed() < MIN_RECORDING;
    *app.phase.lock() = Phase::Processing;
    app.hotkeys.set_escape(false);
    let settings = app.settings.get();
    if settings.sounds && !too_short {
        sounds::play(Sound::Stop);
    }
    if too_short {
        app.mic.cancel();
        *app.phase.lock() = Phase::Idle;
        pill(app, "hidden", None);
        overlay::hide(&app.handle);
        return;
    }
    pill(app, "processing", None);
    // Stopping waits for the mic to finish opening, so do it off this thread.
    let app = app.clone();
    thread::spawn(move || match app.mic.stop() {
        Ok(samples) => {
            log::info!("Recording stopped: {:.1}s", samples.len() as f32 / crate::engine::SAMPLE_RATE as f32);
            process(&app, samples);
        }
        Err(e) => {
            *app.phase.lock() = Phase::Idle;
            show_error(&app, permission_hint(e.to_string()));
        }
    });
}

fn process(app: &Arc<App>, samples: Vec<f32>) {
    let settings = app.settings.get();
    let duration_ms = (samples.len() * 1000 / crate::engine::SAMPLE_RATE) as u64;

    if let Err(e) = crate::ensure_engine(app) {
        *app.phase.lock() = Phase::Idle;
        show_error(app, e.to_string());
        return;
    }
    let result = app
        .engine
        .transcribe(samples, settings.language.clone(), settings.vocabulary.clone());

    let raw = match result {
        Ok(Some(raw)) => raw,
        Ok(None) => {
            *app.phase.lock() = Phase::Idle;
            show_error(app, "Nothing heard".into());
            return;
        }
        Err(e) => {
            *app.phase.lock() = Phase::Idle;
            show_error(app, e.to_string());
            return;
        }
    };

    let final_text = text::process(
        &raw,
        &text::Options {
            replacements: &settings.replacements,
            trailing_space: settings.trailing_space,
        },
    );
    let inserted = insert::insert(
        &app.handle,
        &final_text,
        settings.insert_method,
        settings.restore_clipboard,
    );
    let (inserted, message) = match inserted {
        Ok(true) => (true, None),
        Ok(false) => (false, Some("Copied to clipboard".to_string())),
        Err(e) => {
            log::warn!("Insert failed: {e}");
            let _ = insert::copy(&final_text);
            (false, Some("Couldn't paste. Copied to clipboard".to_string()))
        }
    };

    app.history.add(
        Entry {
            id: 0,
            at: chrono::Utc::now(),
            raw,
            text: final_text.trim_end().to_string(),
            model: settings.model.clone(),
            duration_ms,
            inserted,
        },
        settings.history_retention,
    );
    let _ = app.handle.emit("history-changed", ());

    *app.phase.lock() = Phase::Idle;
    pill(app, "done", message.clone());
    hide_after(app, if message.is_some() { 2200 } else { 700 });
}

/// A mic that never opens is almost always a missing permission.
fn permission_hint(message: String) -> String {
    use handy_recorder::Permission;
    match handy_recorder::permission_status() {
        Permission::Denied | Permission::NotDetermined => "Allow microphone access in Settings".into(),
        _ => message,
    }
}

fn show_error(app: &Arc<App>, message: String) {
    log::warn!("{message}");
    let settings = app.settings.get();
    if settings.sounds {
        sounds::play(Sound::Error);
    }
    pill(app, "error", Some(message));
    overlay::show(&app.handle, settings.overlay_position);
    hide_after(app, 2600);
}

/// Hides the pill after a delay, unless a new recording started meanwhile.
fn hide_after(app: &Arc<App>, ms: u64) {
    let app = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(ms));
        if *app.phase.lock() == Phase::Idle {
            pill(&app, "hidden", None);
            // Let the fade-out play before the window disappears.
            thread::sleep(Duration::from_millis(220));
            if *app.phase.lock() == Phase::Idle {
                overlay::hide(&app.handle);
            }
        }
    });
}
