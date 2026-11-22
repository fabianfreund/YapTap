//! User settings, stored as `settings.json` in the app data folder.
//!
//! Every field has a default, so older files keep loading after new
//! settings are added.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerMode {
    /// Hold to talk, or tap once to keep listening hands-free.
    Auto,
    /// Only push-to-talk: recording ends when the key is released.
    Hold,
    /// Press once to start, press again to stop.
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InsertMethod {
    /// Put the text on the clipboard and press paste, then restore the clipboard.
    Paste,
    /// Type the text key by key (slower, but works where paste is blocked).
    Type,
    /// Only copy to the clipboard.
    Copy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverlayPosition {
    Bottom,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistoryRetention {
    Forever,
    Week,
    Day,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replacement {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub onboarded: bool,
    /// Shortcut in handy-keys notation, e.g. `Fn`, `OptRight`, `Ctrl+Win`.
    pub shortcut: String,
    pub trigger_mode: TriggerMode,
    /// Catalog id of the speech model.
    pub model: String,
    /// ISO language code, or `None` for automatic detection.
    pub language: Option<String>,
    /// Input device id, or `None` for the system default.
    pub microphone: Option<String>,
    pub insert_method: InsertMethod,
    pub restore_clipboard: bool,
    pub trailing_space: bool,
    pub sounds: bool,
    pub overlay_position: OverlayPosition,
    pub accent: String,
    /// Words the model should prefer (names, jargon).
    pub vocabulary: Vec<String>,
    /// "heard → write" corrections applied after transcription.
    pub replacements: Vec<Replacement>,
    pub history_retention: HistoryRetention,
    /// Blocks every network request (model downloads).
    pub offline: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            onboarded: false,
            shortcut: default_shortcut().into(),
            trigger_mode: TriggerMode::Auto,
            model: crate::models::DEFAULT_MODEL.into(),
            language: None,
            microphone: None,
            insert_method: InsertMethod::Paste,
            restore_clipboard: true,
            trailing_space: true,
            sounds: true,
            overlay_position: OverlayPosition::Bottom,
            accent: "#7B61FF".into(),
            vocabulary: Vec::new(),
            replacements: Vec::new(),
            history_retention: HistoryRetention::Forever,
            offline: false,
        }
    }
}

pub fn default_shortcut() -> &'static str {
    if cfg!(target_os = "macos") {
        "Fn"
    } else if cfg!(target_os = "windows") {
        "Ctrl+Win"
    } else {
        "CtrlRight"
    }
}

pub struct SettingsStore {
    path: PathBuf,
    current: RwLock<Settings>,
}

impl SettingsStore {
    pub fn load(dir: &std::path::Path) -> Self {
        let path = dir.join("settings.json");
        let current = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self {
            path,
            current: RwLock::new(current),
        }
    }

    pub fn get(&self) -> Settings {
        self.current.read().clone()
    }

    pub fn set(&self, settings: Settings) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(&settings)?;
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, &self.path)?;
        *self.current.write() = settings;
        Ok(())
    }

    pub fn update(&self, f: impl FnOnce(&mut Settings)) -> anyhow::Result<Settings> {
        let mut s = self.get();
        f(&mut s);
        self.set(s.clone())?;
        Ok(s)
    }
}
