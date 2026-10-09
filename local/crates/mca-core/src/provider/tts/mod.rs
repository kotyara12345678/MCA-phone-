//! TTS adapter for any OpenAI-compatible `POST {base}/audio/speech`.
//!
//! The response is raw audio rather than JSON, so it is read as bytes and
//! decoded straight into the pipeline's frame format — no base64, no copy.

mod transport;

use std::time::Duration;

use crate::domain::language::Language;
use crate::error::ProviderError;
use crate::ports::stt::{AudioFormat, AudioFrame};
use crate::ports::tts::{Synthesis, TtsProvider};
use crate::provider::http::build_http_client;
use crate::provider::retry::RetryPolicy;

pub(crate) const PROVIDER: &str = "tts";

pub struct HttpTtsProvider {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) api_key: Option<String>,
    pub(crate) model: String,
    pub(crate) voice: String,
    pub(crate) format: AudioFormat,
    pub(crate) sample_rate: u32,
    pub(crate) timeout: Duration,
    pub(crate) retry: RetryPolicy,
}

impl HttpTtsProvider {
    pub fn new(
        base_url: impl Into<String>,
        api_key: Option<String>,
        model: impl Into<String>,
        voice: impl Into<String>,
        format: AudioFormat,
        timeout: Duration,
        retry: RetryPolicy,
    ) -> Result<Self, ProviderError> {
        Ok(Self {
            http: build_http_client(crate::config::ProviderKind::Http, timeout)?,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key,
            model: model.into(),
            voice: voice.into(),
            format,
            sample_rate: format.default_sample_rate(),
            timeout,
            retry,
        })
    }
}

#[async_trait::async_trait]
impl TtsProvider for HttpTtsProvider {
    async fn synthesize(&self, text: &str, language: Language) -> Result<Synthesis, ProviderError> {
        let (bytes, latency_ms) = transport::fetch(self, text, language.llm_tag()).await?;
        Ok(Synthesis {
            audio: AudioFrame::from_bytes(self.format, self.sample_rate, &bytes),
            text: text.to_string(),
            latency_ms,
            is_partial: false,
        })
    }

    fn name(&self) -> &'static str {
        PROVIDER
    }

    fn is_live(&self) -> bool {
        true
    }

    fn output_format(&self) -> AudioFormat {
        self.format
    }
}
