//! Connection-level configuration for the chat adapter.

use std::time::Duration;

use crate::provider::retry::RetryPolicy;

/// Everything the adapter needs to talk to one endpoint, grouped so adding a
/// knob does not turn `new` into a wall of positional arguments.
#[derive(Debug, Clone)]
pub struct ChatClientConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout: Duration,
    pub retry: RetryPolicy,
}

impl ChatClientConfig {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: None,
            model: model.into(),
            temperature: 0.2,
            max_tokens: 512,
            timeout: Duration::from_secs(30),
            retry: RetryPolicy::default(),
        }
    }

    pub fn with_api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }
}
