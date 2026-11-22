//! Transcript history, kept locally as `history.json`.

use crate::settings::HistoryRetention;
use chrono::{DateTime, Duration, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAX_ENTRIES: usize = 2_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: u64,
    pub at: DateTime<Utc>,
    /// What the model heard.
    pub raw: String,
    /// What was inserted after the text rules.
    pub text: String,
    pub model: String,
    pub duration_ms: u64,
    pub inserted: bool,
}

pub struct History {
    path: PathBuf,
    entries: Mutex<Vec<Entry>>,
}

impl History {
    pub fn load(dir: &std::path::Path) -> Self {
        let path = dir.join("history.json");
        let entries = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self {
            path,
            entries: Mutex::new(entries),
        }
    }

    pub fn list(&self) -> Vec<Entry> {
        let mut v = self.entries.lock().clone();
        v.reverse(); // newest first
        v
    }

    pub fn last(&self) -> Option<Entry> {
        self.entries.lock().last().cloned()
    }

    pub fn add(&self, mut entry: Entry, retention: HistoryRetention) {
        let mut entries = self.entries.lock();
        entry.id = entries.last().map_or(1, |e| e.id + 1);
        entries.push(entry);
        prune(&mut entries, retention);
        self.save(&entries, retention);
    }

    pub fn delete(&self, id: u64, retention: HistoryRetention) {
        let mut entries = self.entries.lock();
        entries.retain(|e| e.id != id);
        self.save(&entries, retention);
    }

    pub fn clear(&self) {
        let mut entries = self.entries.lock();
        entries.clear();
        self.save(&entries, HistoryRetention::Never);
    }

    pub fn apply_retention(&self, retention: HistoryRetention) {
        let mut entries = self.entries.lock();
        prune(&mut entries, retention);
        self.save(&entries, retention);
    }

    /// With retention "Never" nothing is written to disk; the latest entry
    /// lives in memory only, for "Paste last transcript".
    fn save(&self, entries: &[Entry], retention: HistoryRetention) {
        let entries = if retention == HistoryRetention::Never { &[][..] } else { entries };
        if let Ok(json) = serde_json::to_string(entries) {
            let tmp = self.path.with_extension("json.tmp");
            if std::fs::write(&tmp, json).is_ok() {
                let _ = std::fs::rename(&tmp, &self.path);
            }
        }
    }
}

fn prune(entries: &mut Vec<Entry>, retention: HistoryRetention) {
    let cutoff = match retention {
        HistoryRetention::Forever => None,
        HistoryRetention::Week => Some(Utc::now() - Duration::days(7)),
        HistoryRetention::Day => Some(Utc::now() - Duration::days(1)),
        // Keep only the latest, so "Paste last transcript" still works.
        HistoryRetention::Never => {
            let keep = entries.pop();
            entries.clear();
            entries.extend(keep);
            None
        }
    };
    if let Some(cutoff) = cutoff {
        entries.retain(|e| e.at >= cutoff);
    }
    if entries.len() > MAX_ENTRIES {
        let excess = entries.len() - MAX_ENTRIES;
        entries.drain(..excess);
    }
}
