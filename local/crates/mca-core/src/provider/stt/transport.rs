//! HTTP transport for the STT call: request assembly, error classification and
//! response decoding. Split from `mod.rs` so the retry/orchestration logic there
//! stays readable.

use super::{HttpSttProvider, PROVIDER};
use crate::error::ProviderError;
use crate::ports::stt::Transcript;
use crate::provider::http_error::{classify_status, classify_transport};

/// Provider contract: 1 attempt per retry pass, the policy owns the backoff.
const ATTEMPTS: u32 = 1;

pub(super) fn request(
    provider: &HttpSttProvider,
    url: String,
    body: Vec<u8>,
) -> reqwest::RequestBuilder {
    let content_type = format!(
        "multipart/form-data; boundary={}",
        super::multipart::BOUNDARY
    );
    let mut req = provider
        .http
        .post(&url)
        .header("content-type", content_type)
        .body(body);
    if let Some(key) = provider.api_key.clone() {
        req = req.bearer_auth(key);
    }
    req
}

pub(super) async fn send(req: reqwest::RequestBuilder) -> Result<String, ProviderError> {
    let response = req
        .send()
        .await
        .map_err(|e| classify_transport(PROVIDER, e))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| classify_transport(PROVIDER, e))?;
    if !status.is_success() {
        return Err(classify_status(PROVIDER, status.as_u16(), &text, ATTEMPTS));
    }
    Ok(text)
}

#[derive(serde::Deserialize)]
struct TranscriptionResponse {
    #[serde(default)]
    text: String,
}

/// Decodes the provider payload. An unparsable body is a provider bug, not a
/// caller error, so it is reported as `InvalidResponse` rather than a retryable
/// transport failure.
pub(super) fn decode(raw: &str, latency_ms: i64) -> Result<Transcript, ProviderError> {
    let value: TranscriptionResponse =
        serde_json::from_str(raw).map_err(|e| ProviderError::InvalidResponse {
            provider: PROVIDER,
            message: format!("cannot parse stt response: {e}"),
        })?;
    Ok(Transcript {
        text: value.text,
        language: None,
        confidence: 1.0,
        is_final: true,
        latency_ms,
    })
}
