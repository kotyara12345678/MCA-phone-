//! HTTP transport for the chat completions call: shared constants, request
//! assembly and the single round trip. Split from `mod.rs` so retry logic there
//! stays readable.

use super::super::types::ChatCompletionResponse;
use super::HttpChatClient;
use crate::error::ProviderError;
use crate::provider::http_error::{classify_status, classify_transport};

pub(super) const PROVIDER: &str = "llm";
pub(super) const CHAT_PATH: &str = "/chat/completions";

/// Provider contract: 1 attempt per retry pass, the policy owns the backoff.
const ATTEMPTS: u32 = 1;

pub(super) fn request(
    client: &HttpChatClient,
    url: String,
    body: Vec<u8>,
) -> reqwest::RequestBuilder {
    let mut req = client
        .http
        .post(&url)
        .header("content-type", "application/json")
        .body(body);
    if let Some(key) = client.api_key.clone() {
        req = req.bearer_auth(key);
    }
    req
}

pub(super) async fn send(
    req: reqwest::RequestBuilder,
) -> Result<ChatCompletionResponse, ProviderError> {
    let response = req
        .send()
        .await
        .map_err(|e| classify_transport(PROVIDER, e))?;
    let status = response.status();
    if !status.is_success() {
        // Read the body only on failure; the happy path stays allocation-light.
        let text = response.text().await.unwrap_or_default();
        return Err(classify_status(PROVIDER, status.as_u16(), &text, ATTEMPTS));
    }
    response
        .json()
        .await
        .map_err(|e| classify_transport(PROVIDER, e))
}
