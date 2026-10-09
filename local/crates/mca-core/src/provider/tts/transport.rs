//! Request body and HTTP round trip for TTS synthesis.

use super::HttpTtsProvider;
use crate::error::ProviderError;
use crate::provider::http_error::{classify_status, classify_transport};
use crate::provider::retry::with_retry;

/// OpenAI-compatible speech path.
const SPEECH_PATH: &str = "/audio/speech";

/// Provider contract: one attempt per retry pass, the policy owns the backoff.
const ATTEMPTS: u32 = 1;

fn request_body(
    provider: &HttpTtsProvider,
    text: &str,
    language: &str,
) -> Result<Vec<u8>, ProviderError> {
    let body = serde_json::json!({
        "model": provider.model,
        "voice": provider.voice,
        "input": text,
        "response_format": provider.format.as_str(),
        "speed": 1.0,
        "language": language,
    });
    serde_json::to_vec(&body).map_err(|e| ProviderError::InvalidResponse {
        provider: super::PROVIDER,
        message: e.to_string(),
    })
}

pub(super) async fn fetch(
    provider: &HttpTtsProvider,
    text: &str,
    language: &str,
) -> Result<(Vec<u8>, i64), ProviderError> {
    let payload = request_body(provider, text, language)?;
    let url = format!("{}{SPEECH_PATH}", provider.base_url);
    with_retry(
        super::PROVIDER,
        &provider.retry,
        provider.timeout,
        |_attempt| {
            let http = provider.http.clone();
            let url = url.clone();
            let payload = payload.clone();
            let key = provider.api_key.clone();
            async move { send(http, url, payload, key).await }
        },
    )
    .await
}

async fn send(
    http: reqwest::Client,
    url: String,
    payload: Vec<u8>,
    key: Option<String>,
) -> Result<Vec<u8>, ProviderError> {
    let mut req = http
        .post(&url)
        .header("content-type", "application/json")
        .body(payload);
    if let Some(key) = key {
        req = req.bearer_auth(key);
    }
    let response = req
        .send()
        .await
        .map_err(|e| classify_transport(super::PROVIDER, e))?;
    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(classify_status(
            super::PROVIDER,
            status.as_u16(),
            &text,
            ATTEMPTS,
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| classify_transport(super::PROVIDER, e))?;
    Ok(bytes.to_vec())
}
