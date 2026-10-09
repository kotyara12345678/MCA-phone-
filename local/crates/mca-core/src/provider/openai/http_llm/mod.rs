//! LLM provider backed by any OpenAI-compatible HTTP endpoint.
//!
//! On any failure it does **not** propagate to the dialogue engine directly:
//! the caller wraps it in [`RuleBasedLlmProvider`], so an outage silently turns
//! into a rule-based turn instead of a dropped call.

mod extraction;
mod response;

use std::sync::Arc;

use super::client::HttpChatClient;
use super::rule_llm::RuleBasedLlmProvider;
use crate::error::ProviderError;
use crate::ports::llm::{ExtractionRequest, LlmProvider};

pub(crate) const PROVIDER: &str = "llm";

pub struct HttpLlmProvider {
    pub(super) client: Arc<HttpChatClient>,
    pub(super) fallback: Arc<RuleBasedLlmProvider>,
}

impl HttpLlmProvider {
    pub fn new(client: HttpChatClient, fallback: Arc<RuleBasedLlmProvider>) -> Self {
        Self {
            client: Arc::new(client),
            fallback,
        }
    }

    pub fn fallback(&self) -> &Arc<RuleBasedLlmProvider> {
        &self.fallback
    }
}

#[async_trait::async_trait]
impl LlmProvider for HttpLlmProvider {
    async fn extract(
        &self,
        request: ExtractionRequest,
    ) -> Result<crate::ports::llm::ExtractionResult, ProviderError> {
        extraction::extract(self, request).await
    }

    async fn respond(
        &self,
        request: crate::ports::llm::ResponseRequest,
    ) -> Result<crate::ports::llm::ResponseDraft, ProviderError> {
        response::respond(self, request).await
    }

    fn name(&self) -> &'static str {
        PROVIDER
    }

    fn is_live(&self) -> bool {
        true
    }
}
