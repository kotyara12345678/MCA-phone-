//! STT adapter for any OpenAI-compatible `POST {base}/audio/transcriptions`.
//!
//! Multipart is assembled manually into one pre-allocated buffer per call: it
//! is a handful of fields, and `reqwest::multipart` would allocate a file part
//! for audio we already hold in memory.

mod multipart;
mod transport;

use std::time::Duration;

use multipart::TRANSCRIPTION_PATH;

use crate::error::ProviderError;
use crate::ports::stt::{AudioFrame, Transcript};
use crate::provider::http::build_http_client;
use crate::provider::retry::{with_retry, RetryPolicy};

pub(crate) const PROVIDER: &str = "stt";

pub struct HttpSttProvider {
    http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) api_key: Option<String>,
    pub(crate) model: String,
    pub(crate) timeout: Duration,
    pub(crate) retry: RetryPolicy,
}

impl HttpSttProvider {
    pub fn new(
        base_url: impl Into<String>,
        api_key: Option<String>,
        model: impl Into<String>,
        timeout: Duration,
        retry: RetryPolicy,
    ) -> Result<Self, ProviderError> {
        Ok(Self {
            http: build_http_client(crate::config::ProviderKind::Http, timeout)?,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key,
            model: model.into(),
            timeout,
            retry,
        })
    }
}

#[async_trait::async_trait]
impl crate::ports::stt::SttProvider for HttpSttProvider {
    async fn transcribe(
        &self,
        audio: &AudioFrame,
        language: crate::domain::language::Language,
    ) -> Result<Transcript, ProviderError> {
        call(self, audio, language.llm_tag()).await
    }

    fn name(&self) -> &'static str {
        PROVIDER
    }

    fn is_live(&self) -> bool {
        true
    }
}

/// One transcription round trip, with bounded retry and a hard timeout.
pub(super) async fn call(
    provider: &HttpSttProvider,
    audio: &AudioFrame,
    language: &str,
) -> Result<Transcript, ProviderError> {
    let body = multipart::build_multipart(&provider.model, language, audio);
    let url = format!("{}{TRANSCRIPTION_PATH}", provider.base_url);
    let (raw, latency_ms) = with_retry(PROVIDER, &provider.retry, provider.timeout, |_attempt| {
        let req = transport::request(provider, url.clone(), body.clone());
        async move { transport::send(req).await }
    })
    .await?;
    transport::decode(&raw, latency_ms)
}
