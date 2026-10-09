//! Scripted, thread-safe STT fake: `transcribe` pops the next scripted result,
//! silence is never "heard", and unscripted audio yields a stable marker.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mca_core::domain::language::Language;
use mca_core::error::ProviderError;
use mca_core::ports::stt::{AudioFrame, SttProvider, Transcript};

use crate::audio::TRANSCRIPT_SILENCE_THRESHOLD;

#[derive(Default)]
struct Script {
    queue: VecDeque<Result<Transcript, ProviderError>>,
    calls: u64,
}

/// Deterministic stand-in for the transcription port.
#[derive(Clone, Default)]
pub struct FakeSttProvider {
    state: Arc<Mutex<Script>>,
}

impl FakeSttProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expect(&self, text: impl Into<String>) -> &Self {
        self.push(Ok(transcript(text.into(), None)))
    }

    pub fn expect_language(&self, text: impl Into<String>, language: Language) -> &Self {
        self.push(Ok(transcript(text.into(), Some(language))))
    }

    pub fn fail_next(&self, err: ProviderError) -> &Self {
        self.push(Err(err))
    }

    pub fn calls(&self) -> u64 {
        self.state.lock().expect("stt lock").calls
    }

    fn push(&self, item: Result<Transcript, ProviderError>) -> &Self {
        self.state.lock().expect("stt lock").queue.push_back(item);
        self
    }
}

#[async_trait]
impl SttProvider for FakeSttProvider {
    async fn transcribe(
        &self,
        audio: &AudioFrame,
        _language: Language,
    ) -> Result<Transcript, ProviderError> {
        if audio.is_silent(TRANSCRIPT_SILENCE_THRESHOLD) {
            return Ok(transcript(String::new(), None));
        }
        let mut state = self.state.lock().expect("stt lock");
        state.calls += 1;
        match state.queue.pop_front() {
            Some(result) => result,
            None => Ok(transcript("(unscripted)".to_string(), None)),
        }
    }

    fn name(&self) -> &'static str {
        "fake_stt"
    }
}

fn transcript(text: String, language: Option<Language>) -> Transcript {
    Transcript {
        text,
        language,
        confidence: 1.0,
        is_final: true,
        latency_ms: 0,
    }
}
