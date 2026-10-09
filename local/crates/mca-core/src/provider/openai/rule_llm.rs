use async_trait::async_trait;
use std::time::Duration;

use crate::error::ProviderError;
use crate::nlu;
use crate::policy::compose;
use crate::ports::llm::{
    ExtractionRequest, ExtractionResult, LlmProvider, ResponseDraft, ResponseRequest,
};

/// Deterministic stand-in for an LLM.
///
/// Two jobs:
/// * `mock` mode — lets the whole platform (and every test) run with no API
///   keys, so the same utterance always yields the same result;
/// * production fallback — `HttpLlmProvider` delegates here on timeout, 5xx or
///   malformed JSON, so a provider outage never costs collected data.
pub struct RuleBasedLlmProvider {
    /// Simulated per-call cost, used to exercise timeout handling in tests
    /// without actually sleeping for seconds.
    simulated_latency: Duration,
    fail_with: Option<ProviderError>,
}

impl RuleBasedLlmProvider {
    pub fn new() -> Self {
        Self {
            simulated_latency: Duration::ZERO,
            fail_with: None,
        }
    }

    pub fn with_latency(latency: Duration) -> Self {
        Self {
            simulated_latency: latency,
            fail_with: None,
        }
    }

    /// Always returns this error, for failure-path tests.
    pub fn failing(err: ProviderError) -> Self {
        Self {
            simulated_latency: Duration::ZERO,
            fail_with: Some(err),
        }
    }
}

impl Default for RuleBasedLlmProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for RuleBasedLlmProvider {
    async fn extract(&self, request: ExtractionRequest) -> Result<ExtractionResult, ProviderError> {
        self.simulate().await;
        if let Some(err) = &self.fail_with {
            return Err(err.clone());
        }
        Ok(nlu::extract(&request.utterance))
    }

    async fn respond(&self, request: ResponseRequest) -> Result<ResponseDraft, ProviderError> {
        self.simulate().await;
        if let Some(err) = &self.fail_with {
            return Err(err.clone());
        }
        Ok(ResponseDraft {
            text: compose::compose(&request.directive),
        })
    }

    fn name(&self) -> &'static str {
        "rule_based"
    }
}

impl RuleBasedLlmProvider {
    async fn simulate(&self) {
        if !self.simulated_latency.is_zero() {
            tokio::time::sleep(self.simulated_latency).await;
        }
    }
}
