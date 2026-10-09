//! Response shapes for the OpenAI-compatible chat API.

use serde::{Deserialize, Serialize};

use crate::error::ProviderError;

const PROVIDER: &str = "llm";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    #[serde(default)]
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<Usage>,
    /// Cheap compatible providers sometimes return an error with HTTP 200.
    #[serde(default)]
    pub error: Option<ApiError>,
}

impl ChatCompletionResponse {
    /// Pulls the assistant message out, mapping every failure mode onto
    /// `ProviderError` so callers never see a raw transport shape.
    pub fn into_message(self) -> Result<String, ProviderError> {
        if let Some(err) = self.error {
            return Err(ProviderError::Status {
                provider: PROVIDER,
                status: 200,
                message: err.to_message(),
            });
        }
        self.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or(ProviderError::InvalidResponse {
                provider: PROVIDER,
                message: "response contained no choices".to_string(),
            })
    }

    pub fn total_tokens(&self) -> Option<u32> {
        self.usage.as_ref().map(|u| u.total_tokens)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    #[serde(default)]
    pub index: u32,
    #[serde(default)]
    pub message: super::request::ChatMessage,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: u32,
    #[serde(default)]
    pub completion_tokens: u32,
    #[serde(default)]
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    #[serde(default)]
    pub message: String,
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
}

impl ApiError {
    pub fn to_message(&self) -> String {
        if !self.message.is_empty() {
            return self.message.clone();
        }
        if let Some(kind) = &self.kind {
            return kind.clone();
        }
        self.code
            .clone()
            .unwrap_or_else(|| "unknown api error".to_string())
    }
}
