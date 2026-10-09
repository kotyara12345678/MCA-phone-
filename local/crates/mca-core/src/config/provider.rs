//! Provider selection and the shared remote-provider settings block.

pub use super::env::{millis, number, var, var_or};

mod kind;

pub use kind::ProviderKind;

use std::time::Duration;

use crate::validation::ValidationError;

/// Shared remote-provider settings: endpoint, credentials, timeouts, retries.
/// Identical shape for LLM / STT / TTS / CRM so one parser covers all of them.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteProviderConfig {
    pub kind: ProviderKind,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub retry_base_delay_ms: u64,
    pub retry_max_delay_ms: u64,
}

impl RemoteProviderConfig {
    /// Environment-variable prefix, e.g. `LLM`, `STT`, `TTS`.
    pub fn from_env(prefix: &str) -> Result<Self, ValidationError> {
        let kind = match var(&format!("{prefix}_PROVIDER")) {
            Some(raw) => ProviderKind::parse(&raw)?,
            None => ProviderKind::Fake,
        };
        let base_url = var(&format!("{prefix}_BASE_URL"));
        if kind.is_remote() && base_url.is_none() {
            return Err(ValidationError::invalid(
                "base_url",
                format!("{prefix}_PROVIDER=http requires {prefix}_BASE_URL"),
            ));
        }
        Ok(Self {
            kind,
            base_url,
            api_key: var(&format!("{prefix}_API_KEY")),
            model: var(&format!("{prefix}_MODEL")),
            timeout: millis(&format!("{prefix}_TIMEOUT_MS"), 8_000),
            retry_attempts: number(&format!("{prefix}_RETRY_ATTEMPTS"), 2),
            retry_base_delay_ms: 120,
            retry_max_delay_ms: 1_500,
        })
    }

    pub fn retry_policy(&self) -> crate::provider::retry::RetryPolicy {
        crate::provider::retry::RetryPolicy::new(
            self.retry_attempts,
            self.retry_base_delay_ms,
            self.retry_max_delay_ms,
        )
    }

    /// Redacted description for the `/ready` payload. Secrets never appear.
    pub fn redacted(&self) -> serde_json::Value {
        serde_json::json!({
            "kind": self.kind.as_str(),
            "base_url": self.base_url,
            "model": self.model,
            "has_api_key": self.api_key.is_some(),
            "timeout_ms": self.timeout.as_millis() as u64,
            "retry_attempts": self.retry_attempts,
        })
    }
}
