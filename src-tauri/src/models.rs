//! Speech models: the built-in catalog, downloads and storage.
//!
//! Files are fetched from Hugging Face at a pinned commit, so the bytes always
//! match the SHA-256 in the catalog. GGUF builds are published by the
//! transcribe.cpp project (handy-computer on Hugging Face).

use anyhow::{anyhow, bail, Context, Result};
use futures_util::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

pub const DEFAULT_MODEL: &str = "parakeet-v3";
pub const VAD_MODEL: &str = "silero-vad";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Speech,
    Vad,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub kind: Kind,
    /// Number of languages, for display.
    pub languages: u32,
    /// Relative scores 1–5, for display.
    pub speed: u8,
    pub accuracy: u8,
    pub license: &'static str,
    #[serde(skip)]
    pub repo: &'static str,
    #[serde(skip)]
    pub revision: &'static str,
    pub filename: &'static str,
    pub size: u64,
    #[serde(skip)]
    pub sha256: &'static str,
}

pub const CATALOG: &[ModelInfo] = &[
    ModelInfo {
        id: "parakeet-v3",
        name: "Parakeet v3",
        description: "Fast and accurate. 25 European languages including English and German. Recommended.",
        kind: Kind::Speech,
        languages: 25,
        speed: 5,
        accuracy: 5,
        license: "CC-BY-4.0 (NVIDIA)",
        repo: "handy-computer/parakeet-tdt-0.6b-v3-gguf",
        revision: "85ac09ea12fc4b1112fa76810059364bc6adc9de",
        filename: "parakeet-tdt-0.6b-v3-Q8_0.gguf",
        size: 739_508_576,
        sha256: "5859f77944efcd8eafa23a6350731960b2b55b2203df51f319665c807d802cc7",
    },
    ModelInfo {
        id: "whisper-turbo",
        name: "Whisper Large v3 Turbo",
        description: "99 languages, and you can lock the language. Slower than Parakeet.",
        kind: Kind::Speech,
        languages: 99,
        speed: 3,
        accuracy: 5,
        license: "MIT (OpenAI)",
        repo: "handy-computer/whisper-large-v3-turbo-gguf",
        revision: "5eaf945c7978e564bae5b28a5b1639dd93c2bfb1",
        filename: "whisper-large-v3-turbo-Q5_K_M.gguf",
        size: 619_628_128,
        sha256: "977b5db4e004349dffd1ab9caa10ba5aaba3fc3edd3ba72cadb84328a3203e36",
    },
    ModelInfo {
        id: "whisper-small",
        name: "Whisper Small",
        description: "Small and light, 99 languages. For older or slower computers.",
        kind: Kind::Speech,
        languages: 99,
        speed: 4,
        accuracy: 3,
        license: "MIT (OpenAI)",
        repo: "handy-computer/whisper-small-gguf",
        revision: "c0214bd34be9296695486f838e0142f900803159",
        filename: "whisper-small-Q8_0.gguf",
        size: 269_751_136,
        sha256: "9b9c8811bbcc82a7766f0fb0925614bdacb0923b2cc630daeac17108b655b860",
    },
    ModelInfo {
        id: VAD_MODEL,
        name: "Silero VAD",
        description: "Finds the speech in a recording and trims silence.",
        kind: Kind::Vad,
        languages: 0,
        speed: 5,
        accuracy: 5,
        license: "MIT",
        repo: "handy-computer/silero-vad-v6.2-gguf",
        revision: "97bbd75179f535e4708bccdf61e9278b8b60dfc7",
        filename: "silero-vad-v6.2-F32.gguf",
        size: 1_241_056,
        sha256: "0d871a800c420fdd5d1b17a3aea4296d1c39608f0ff4a9295eb76457eb1c88e6",
    },
];

pub fn find(id: &str) -> Option<&'static ModelInfo> {
    CATALOG.iter().find(|m| m.id == id)
}

impl ModelInfo {
    pub fn url(&self) -> String {
        format!(
            "https://huggingface.co/{}/resolve/{}/{}",
            self.repo, self.revision, self.filename
        )
    }
}

pub struct ModelStore {
    dir: PathBuf,
}

impl ModelStore {
    pub fn new(data_dir: &Path) -> Self {
        let dir = data_dir.join("models");
        let _ = std::fs::create_dir_all(&dir);
        Self { dir }
    }

    pub fn path(&self, model: &ModelInfo) -> PathBuf {
        self.dir.join(model.filename)
    }

    pub fn is_installed(&self, model: &ModelInfo) -> bool {
        std::fs::metadata(self.path(model))
            .map(|m| m.len() == model.size)
            .unwrap_or(false)
    }

    pub fn delete(&self, model: &ModelInfo) -> Result<()> {
        let path = self.path(model);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Downloads a model, resuming a partial `.part` file, and verifies its
    /// SHA-256 before moving it into place. `progress(downloaded, total)` is
    /// called as bytes arrive.
    pub async fn download(
        &self,
        model: &ModelInfo,
        cancel: Arc<AtomicBool>,
        mut progress: impl FnMut(u64, u64),
    ) -> Result<()> {
        let final_path = self.path(model);
        let part = final_path.with_extension("gguf.part");
        let mut have = tokio::fs::metadata(&part).await.map(|m| m.len()).unwrap_or(0);
        if have > model.size {
            tokio::fs::remove_file(&part).await.ok();
            have = 0;
        }

        let client = reqwest::Client::builder()
            .user_agent(concat!("YapTap/", env!("CARGO_PKG_VERSION")))
            .build()?;
        let mut req = client.get(model.url());
        if have > 0 {
            req = req.header(reqwest::header::RANGE, format!("bytes={have}-"));
        }
        let resp = req.send().await.context("could not reach Hugging Face")?;
        let status = resp.status();
        if status == reqwest::StatusCode::OK {
            have = 0; // server ignored the range: start over
        } else if status != reqwest::StatusCode::PARTIAL_CONTENT {
            bail!("download failed: HTTP {status}");
        }

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(have > 0)
            .truncate(have == 0)
            .open(&part)
            .await?;

        let mut downloaded = have;
        progress(downloaded, model.size);
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            if cancel.load(Ordering::Relaxed) {
                file.flush().await?;
                bail!("cancelled");
            }
            let chunk = chunk.context("connection lost")?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            progress(downloaded, model.size);
        }
        file.flush().await?;
        drop(file);

        if downloaded != model.size {
            bail!("download incomplete ({downloaded} of {} bytes)", model.size);
        }
        let expected = model.sha256.to_string();
        let part_for_hash = part.clone();
        let actual = tokio::task::spawn_blocking(move || sha256_file(&part_for_hash)).await??;
        if actual != expected {
            tokio::fs::remove_file(&part).await.ok();
            return Err(anyhow!("checksum mismatch, the file was removed. Please try again."));
        }
        tokio::fs::rename(&part, &final_path).await?;
        Ok(())
    }
}

fn sha256_file(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}
