//! Thin HTTP adapter over any OpenAI-compatible `/chat/completions` endpoint.
//!
//! No vendor SDK on purpose: the request/response shapes are stable, small and
//! fully typed, and this keeps cheap compatible providers (Groq, Together,
//! DeepSeek, vLLM, Ollama, llama.cpp) a one-variable switch.

mod transport;

use std::time::Duration;

use super::config::ChatClientConfig;
use super::types::{ChatCompletionRequest, ChatMessage, ResponseFormat};
use crate::error::ProviderError;
use crate::provider::retry::with_retry;

#[derive(Debug, Clone)]
pub struct HttpChatClient {
    pub(super) http: reqwest::Client,
    pub(super) base_url: String,
    pub(super) api_key: Option<String>,
    pub(super) model: String,
    pub(super) temperature: f32,
    pub(super) max_tokens: u32,
    pub(super) timeout: Duration,
    pub(super) retry: crate::provider::retry::RetryPolicy,
}

impl HttpChatClient {
    pub fn new(http: reqwest::Client, config: ChatClientConfig) -> Self {
        Self {
            http,
            base_url: config.base_url,
            api_key: config.api_key,
            model: config.model,
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            timeout: config.timeout,
            retry: config.retry,
        }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// One chat completion round trip, with bounded retry and a hard timeout.
    /// Returns the assistant text and the measured upstream latency.
    pub async fn complete(
        &self,
        messages: Vec<ChatMessage>,
        json_mode: bool,
    ) -> Result<(String, i64), ProviderError> {
        let body = self.encode(messages, json_mode)?;
        let url = format!("{}{}", self.base_url, transport::CHAT_PATH);
        let (value, latency) =
            with_retry(transport::PROVIDER, &self.retry, self.timeout, |_attempt| {
                let req = transport::request(self, url.clone(), body.clone());
                async move { transport::send(req).await }
            })
            .await?;
        Ok((value.into_message()?, latency))
    }

    fn encode(
        &self,
        messages: Vec<ChatMessage>,
        json_mode: bool,
    ) -> Result<Vec<u8>, ProviderError> {
        let mut request = ChatCompletionRequest::new(self.model.clone(), messages);
        request.temperature = Some(self.temperature);
        request.max_tokens = Some(self.max_tokens);
        if json_mode {
            request.response_format = Some(ResponseFormat {
                kind: "json_object".to_string(),
            });
        }
        serde_json::to_vec(&request).map_err(|e| ProviderError::InvalidResponse {
            provider: transport::PROVIDER,
            message: format!("cannot serialize request: {e}"),
        })
    }
}
