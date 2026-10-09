//! Scripted TTS fake: records requested text, optional partial chunking, and
//! deterministic PCM derived from the text (see [`crate::audio`]).

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use mca_core::domain::language::Language;
use mca_core::error::ProviderError;
use mca_core::ports::stt::AudioFormat;
use mca_core::ports::tts::{Synthesis, TtsProvider};

use crate::audio::frame_for;
struct State {
    chunks: usize,
    latency: Duration,
    spoken: Vec<String>,
}

#[derive(Clone)]
pub struct FakeTtsProvider {
    state: Arc<Mutex<State>>,
}

impl FakeTtsProvider {
    pub fn with_latency(self, latency: Duration) -> Self {
        self.state.lock().expect("tts lock").latency = latency;
        self
    }

    pub fn with_chunks(self, n: usize) -> Self {
        self.state.lock().expect("tts lock").chunks = n.max(1);
        self
    }

    pub fn spoken(&self) -> Vec<String> {
        self.state.lock().expect("tts lock").spoken.clone()
    }
}

impl Default for FakeTtsProvider {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                chunks: 1,
                latency: Duration::ZERO,
                spoken: Vec::new(),
            })),
        }
    }
}
fn chunks_of(text: &str, chunks: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec![String::new()];
    }
    words
        .chunks(words.len().div_ceil(chunks))
        .map(|g| g.join(" "))
        .collect()
}

#[async_trait]
impl TtsProvider for FakeTtsProvider {
    async fn synthesize(&self, text: &str, language: Language) -> Result<Synthesis, ProviderError> {
        let mut parts = self.synthesize_chunked(text, language).await?;
        Ok(parts.pop().expect("at least one chunk"))
    }

    async fn synthesize_chunked(
        &self,
        text: &str,
        _language: Language,
    ) -> Result<Vec<Synthesis>, ProviderError> {
        let (chunks, latency) = {
            let mut state = self.state.lock().expect("tts lock");
            state.spoken.push(text.to_string());
            (state.chunks, state.latency)
        };
        if !latency.is_zero() {
            tokio::time::sleep(latency).await;
        }
        let parts = chunks_of(text, chunks);
        let count = parts.len();
        Ok(parts
            .into_iter()
            .enumerate()
            .map(|(index, part)| Synthesis {
                audio: frame_for(&part, AudioFormat::PcmS16Le, 8_000, 240),
                text: part,
                latency_ms: latency.as_millis() as i64,
                is_partial: index + 1 < count,
            })
            .collect())
    }

    fn name(&self) -> &'static str {
        "fake_tts"
    }
}
