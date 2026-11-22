//! Puts text where the cursor is: paste (clipboard + ⌘V / Ctrl+V, then the
//! old clipboard comes back), typing, or copy only.

use crate::settings::InsertMethod;
use anyhow::{anyhow, Result};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::AppHandle;

/// What the clipboard held before we used it.
enum Saved {
    Text(String),
    Image(arboard::ImageData<'static>),
    Nothing,
}

fn save(cb: &mut arboard::Clipboard) -> Saved {
    if let Ok(t) = cb.get_text() {
        return Saved::Text(t);
    }
    if let Ok(img) = cb.get_image() {
        return Saved::Image(img.to_owned_img());
    }
    Saved::Nothing
}

fn restore(cb: &mut arboard::Clipboard, saved: Saved) {
    let _ = match saved {
        Saved::Text(t) => cb.set_text(t),
        Saved::Image(i) => cb.set_image(i),
        Saved::Nothing => cb.clear(),
    };
}

pub fn copy(text: &str) -> Result<()> {
    arboard::Clipboard::new()?.set_text(text.to_string())?;
    Ok(())
}

/// Returns `Ok(true)` when the text was inserted, `Ok(false)` when it was
/// only copied.
pub fn insert(app: &AppHandle, text: &str, method: InsertMethod, restore_clipboard: bool) -> Result<bool> {
    match method {
        InsertMethod::Copy => copy(text).map(|_| false),
        InsertMethod::Type => {
            let t = text.to_string();
            on_main(app, move |enigo| enigo.text(&t).map_err(|e| anyhow!("{e}")))?;
            Ok(true)
        }
        InsertMethod::Paste => {
            let mut cb = arboard::Clipboard::new()?;
            let saved = restore_clipboard.then(|| save(&mut cb));
            cb.set_text(text.to_string())?;
            // Give the clipboard a moment to settle before pasting.
            thread::sleep(Duration::from_millis(40));
            on_main(app, press_paste)?;
            if let Some(saved) = saved {
                // Apps read the clipboard asynchronously after ⌘V; wait before
                // putting the old contents back.
                thread::sleep(Duration::from_millis(350));
                restore(&mut cb, saved);
            }
            Ok(true)
        }
    }
}

fn press_paste(enigo: &mut Enigo) -> Result<()> {
    #[cfg(target_os = "macos")]
    let (modifier, v) = (Key::Meta, Key::Other(9)); // kVK_ANSI_V
    #[cfg(target_os = "windows")]
    let (modifier, v) = (Key::Control, Key::Other(0x56)); // VK_V
    #[cfg(target_os = "linux")]
    let (modifier, v) = (Key::Control, Key::Unicode('v'));

    let err = |e: enigo::InputError| anyhow!("Could not press paste: {e}");
    enigo.key(modifier, Direction::Press).map_err(err)?;
    thread::sleep(Duration::from_millis(12));
    let clicked = enigo.key(v, Direction::Click).map_err(err);
    thread::sleep(Duration::from_millis(12));
    enigo.key(modifier, Direction::Release).map_err(err)?;
    clicked
}

/// Runs keyboard simulation on the main thread (required on macOS for
/// keyboard-layout lookups) and waits for it to finish.
fn on_main(app: &AppHandle, f: impl FnOnce(&mut Enigo) -> Result<()> + Send + 'static) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    app.run_on_main_thread(move || {
        let result = Enigo::new(&Settings::default())
            .map_err(|e| anyhow!("Keyboard control is not available: {e}"))
            .and_then(|mut enigo| f(&mut enigo));
        let _ = tx.send(result);
    })?;
    rx.recv_timeout(Duration::from_secs(10))
        .map_err(|_| anyhow!("Typing timed out"))?
}
