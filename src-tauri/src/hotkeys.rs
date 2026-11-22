//! Global shortcuts via handy-keys: the dictation key (hold or tap) and Esc
//! to cancel, which is only registered while recording so Esc keeps working
//! everywhere else.

use crate::{session, App};
use anyhow::{anyhow, Result};
use handy_keys::{Hotkey, HotkeyId, HotkeyManager, HotkeyState, Key, KeyboardListener, Modifiers};
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct Inner {
    manager: HotkeyManager,
    main: Option<HotkeyId>,
    escape: Option<HotkeyId>,
}

#[derive(Default)]
pub struct Hotkeys {
    inner: Mutex<Option<Inner>>,
    /// Last error, shown in settings (e.g. missing permission).
    pub error: Mutex<Option<String>>,
}

impl Hotkeys {
    pub fn is_running(&self) -> bool {
        self.inner.lock().is_some()
    }

    /// Creates the listener and registers the dictation shortcut.
    pub fn start(&self, shortcut: &str) -> Result<()> {
        let mut inner = self.inner.lock();
        if inner.is_none() {
            let manager = HotkeyManager::new_with_blocking()
                .or_else(|_| HotkeyManager::new())
                .map_err(|e| anyhow!("{e}"))?;
            *inner = Some(Inner {
                manager,
                main: None,
                escape: None,
            });
        }
        let i = inner.as_mut().unwrap();
        if let Some(id) = i.main.take() {
            let _ = i.manager.unregister(id);
        }
        let hotkey: Hotkey = shortcut.parse().map_err(|e| anyhow!("Invalid shortcut: {e}"))?;
        i.main = Some(i.manager.register(hotkey).map_err(|e| anyhow!("{e}"))?);
        *self.error.lock() = None;
        Ok(())
    }

    pub fn set_escape(&self, on: bool) {
        let mut inner = self.inner.lock();
        let Some(i) = inner.as_mut() else { return };
        match (on, i.escape) {
            (true, None) => {
                if let Ok(hk) = Hotkey::new(Modifiers::empty(), Key::Escape) {
                    i.escape = i.manager.register(hk).ok();
                }
            }
            (false, Some(id)) => {
                let _ = i.manager.unregister(id);
                i.escape = None;
            }
            _ => {}
        }
    }

    /// Drains pending events. Returns (is_main, pressed) pairs.
    fn poll(&self) -> Vec<(bool, bool)> {
        let inner = self.inner.lock();
        let Some(i) = inner.as_ref() else { return Vec::new() };
        let mut out = Vec::new();
        while let Some(ev) = i.manager.try_recv() {
            let pressed = ev.state == HotkeyState::Pressed;
            if Some(ev.id) == i.main {
                out.push((true, pressed));
            } else if Some(ev.id) == i.escape {
                out.push((false, pressed));
            }
        }
        out
    }
}

/// Starts the shortcut thread. It keeps retrying until the listener can be
/// created (on macOS that needs the Accessibility permission), then routes
/// key events to the session.
pub fn spawn(app: Arc<App>) {
    thread::Builder::new()
        .name("yaptap-hotkeys".into())
        .spawn(move || loop {
            if !app.hotkeys.is_running() {
                let shortcut = app.settings.get().shortcut;
                if let Err(e) = app.hotkeys.start(&shortcut) {
                    *app.hotkeys.error.lock() = Some(e.to_string());
                    thread::sleep(Duration::from_secs(2));
                    continue;
                }
                log::info!("Shortcut ready: {shortcut}");
                app.emit_status();
            }
            for (is_main, pressed) in app.hotkeys.poll() {
                if is_main {
                    session::on_shortcut(&app, pressed);
                } else if pressed {
                    session::cancel(&app);
                }
            }
            // Poll faster while a recording is running.
            let busy = session::is_recording(&app);
            thread::sleep(Duration::from_millis(if busy { 8 } else { 25 }));
        })
        .expect("failed to start the shortcut thread");
}

/// Waits for the user to press a key combination and returns it in
/// handy-keys notation. A combination of only modifiers (like Fn or right
/// Option) is recorded when they are released.
pub fn record(timeout: Duration) -> Result<String> {
    let listener = KeyboardListener::new().map_err(|e| anyhow!("{e}"))?;
    let deadline = std::time::Instant::now() + timeout;
    let mut peak = Modifiers::empty();
    loop {
        let left = deadline.saturating_duration_since(std::time::Instant::now());
        if left.is_zero() {
            return Err(anyhow!("No key pressed"));
        }
        let Ok(ev) = listener.recv_timeout(left) else { continue };
        if let Some(key) = ev.key {
            if ev.is_key_down {
                if key == Key::Escape && ev.modifiers.is_empty() {
                    return Err(anyhow!("cancelled"));
                }
                let hk = Hotkey::new(ev.modifiers, key).map_err(|e| anyhow!("{e}"))?;
                return Ok(hk.to_handy_string());
            }
        } else {
            if ev.modifiers.bits().count_ones() >= peak.bits().count_ones() {
                peak = ev.modifiers;
            }
            if ev.modifiers.is_empty() && !peak.is_empty() {
                let hk = Hotkey::new(peak, None::<Key>).map_err(|e| anyhow!("{e}"))?;
                return Ok(hk.to_handy_string());
            }
        }
    }
}
