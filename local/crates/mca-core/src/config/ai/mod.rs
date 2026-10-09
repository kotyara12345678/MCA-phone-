//! Per-port AI configuration: LLM, STT, TTS and CRM.
//!
//! Every struct embeds a [`RemoteProviderConfig`], so endpoint/credentials/
//! timeout/retry behave identically no matter which vendor is plugged in.
//! Ports the deployment does not use (`LLM_PROVIDER=disabled`) still validate
//! cleanly, which keeps a single `.env.example` honest for every profile.

mod crm;
mod voice;

pub use crm::CrmConfig;
pub use voice::{SttConfig, TtsConfig};

use std::time::Duration;

use super::env::{number, var};
use super::provider::{ProviderKind, RemoteProviderConfig};
use crate::validation::ValidationError;

/// LLM-specific knobs. Everything provider-specific lives in
/// [`RemoteProviderConfig`]; this only adds what is unique to text generation.
#[derive(Debug, Clone, PartialEq)]
pub struct LlmConfig {
    pub remote: RemoteProviderConfig,
    pub temperature: f32,
    pub max_tokens: u32,
    /// How many recent transcript lines are sent with each request. The bound is
    /// what keeps cost per turn roughly constant regardless of call length.
    pub context_recent_turns: usize,
}

impl LlmConfig {
    pub fn from_env() -> Result<Self, ValidationError> {
        let temperature = var("LLM_TEMPERATURE")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0_f32);
        if !(0.0..=2.0).contains(&temperature) {
            return Err(ValidationError::invalid(
                "LLM_TEMPERATURE",
                "must be within 0.0..=2.0",
            ));
        }
        Ok(Self {
            remote: RemoteProviderConfig::from_env("LLM")?,
            temperature,
            max_tokens: number("LLM_MAX_TOKENS", 700),
            context_recent_turns: number("LLM_CONTEXT_RECENT_TURNS", 8) as usize,
        })
    }

    pub fn fake() -> Self {
        Self {
            remote: RemoteProviderConfig {
                kind: ProviderKind::Fake,
                base_url: None,
                api_key: None,
                model: Some("rule_based".into()),
                timeout: Duration::from_millis(1),
                retry_attempts: 1,
                retry_base_delay_ms: 1,
                retry_max_delay_ms: 1,
            },
            temperature: 0.0,
            max_tokens: 0,
            context_recent_turns: 8,
        }
    }
}
