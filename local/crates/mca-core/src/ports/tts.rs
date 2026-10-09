use async_trait::async_trait;

use super::stt::{AudioFormat, AudioFrame};
use crate::domain::language::Language;
use crate::error::ProviderError;

/// A synthesised utterance, ready to be streamed to the caller.
#[derive(Debug, Clone, PartialEq)]
pub struct Synthesis {
    pub audio: AudioFrame,
    pub text: String,
    pub latency_ms: i64,
    /// True when more chunks are expected for this utterance.
    pub is_partial: bool,
}

/// Text-to-speech port. `HttpTtsProvider` targets any OpenAI-compatible
/// `audio/speech` endpoint; `FakeTtsProvider` emits deterministic PCM so the
/// barge-in path can be exercised without audio hardware.
#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn synthesize(&self, text: &str, language: Language) -> Result<Synthesis, ProviderError>;

    /// Streaming variant used to lower time-to-first-audio. Providers that
    /// only offer a single response use the default delegation.
    async fn synthesize_chunked(
        &self,
        text: &str,
        language: Language,
    ) -> Result<Vec<Synthesis>, ProviderError> {
        Ok(vec![self.synthesize(text, language).await?])
    }

    fn name(&self) -> &'static str;

    fn is_live(&self) -> bool {
        false
    }

    fn output_format(&self) -> AudioFormat {
        AudioFormat::PcmS16Le
    }
}
