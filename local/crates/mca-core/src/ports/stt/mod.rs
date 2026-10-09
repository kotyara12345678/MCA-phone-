//! Speech-to-text port.
//!
//! The trait is the seam that makes the platform testable: `HttpSttProvider`
//! targets any OpenAI-compatible `audio/transcriptions` endpoint, while
//! `FakeSttProvider` in `mca-testing` turns a scripted transcript out of the
//! audio so the whole pipeline runs without credentials.

mod audio_format;
mod audio_frame;

pub use audio_format::AudioFormat;
pub use audio_frame::{AudioFrame, Transcript};

use async_trait::async_trait;

use crate::domain::language::Language;
use crate::error::ProviderError;

#[async_trait]
pub trait SttProvider: Send + Sync {
    async fn transcribe(
        &self,
        audio: &AudioFrame,
        language: Language,
    ) -> Result<Transcript, ProviderError>;

    /// Streaming variant; the default implementation falls back to the unary
    /// call so simple providers need not implement partial results.
    async fn transcribe_stream(
        &self,
        audio: &AudioFrame,
        language: Language,
    ) -> Result<Transcript, ProviderError> {
        self.transcribe(audio, language).await
    }

    fn name(&self) -> &'static str;

    /// Whether this provider talks to a real external service. Used by
    /// `/ready` and by the E2E suite to assert fakes are active.
    fn is_live(&self) -> bool {
        false
    }
}
