use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::application::ApplicationState;
use crate::error::ProviderError;
pub use crate::nlu::result::ExtractionResult;
use crate::policy::directive::ReplyDirective;

/// Conversation turn handed to the extractor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRequest {
    /// Normalised transcript of the customer's utterance.
    pub utterance: String,
    /// Current structured state, so the model can resolve anaphora
    /// ("в Алматы" → destination) without a long history.
    pub current_state: ApplicationState,
    /// Compact transcript of the last few turns, newest last.
    pub recent_turns: Vec<String>,
    /// Question the agent asked last turn, if any — lets the model map a bare
    /// "завтра" onto the field being collected.
    pub last_agent_question: Option<String>,
    pub language: crate::domain::language::Language,
}

/// Everything the response generator is allowed to use. The directive is a hard
/// constraint decided by [`crate::policy::decide`]; the model may rephrase but
/// not choose content, which makes hallucinated prices structurally impossible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseRequest {
    pub directive: ReplyDirective,
    pub current_state: ApplicationState,
    pub last_customer_text: String,
    pub recent_turns: Vec<String>,
    pub language: crate::domain::language::Language,
}

/// The model's rendering of a directive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseDraft {
    pub text: String,
}

/// Structured-output LLM port. Two implementations ship today: `HttpLlmProvider`
/// (OpenAI-compatible chat completions) and `RuleBasedLlmProvider` (offline /
/// fallback). Both satisfy this contract, so swapping in a local model later
/// cannot change the dialogue engine.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Extracts structured fields from one customer utterance.
    async fn extract(&self, request: ExtractionRequest) -> Result<ExtractionResult, ProviderError>;

    /// Renders a deterministic [`ReplyDirective`] into natural language.
    async fn respond(&self, request: ResponseRequest) -> Result<ResponseDraft, ProviderError>;

    /// Identifier used in logs, metrics and the `/ready` payload.
    fn name(&self) -> &'static str;

    /// False when the provider is a deterministic stand-in rather than a model.
    fn is_live(&self) -> bool {
        false
    }
}
