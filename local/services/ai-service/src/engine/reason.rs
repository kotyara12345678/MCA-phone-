//! Provider I/O for a turn: one extraction call and one response call, both
//! behind the `LlmProvider` port. Kept apart from the orchestration so each
//! provider interaction stays independently mockable.

use mca_core::domain::application::ApplicationState;
use mca_core::error::AppError;
use mca_core::nlu::result::ExtractionResult;
use mca_core::ports::llm::{ExtractionRequest, ResponseDraft, ResponseRequest};

use super::DialogueEngine;

impl DialogueEngine {
    pub(crate) async fn extract_from(
        &self,
        state: &ApplicationState,
        utterance: &str,
        recent: &[String],
        last_agent_question: Option<String>,
    ) -> Result<ExtractionResult, AppError> {
        self.llm
            .extract(ExtractionRequest {
                utterance: utterance.to_string(),
                current_state: state.clone(),
                recent_turns: recent.to_vec(),
                last_agent_question,
                language: state.language,
            })
            .await
            .map_err(AppError::Provider)
    }

    pub(crate) async fn answer_with(
        &self,
        directive: &mca_core::policy::directive::ReplyDirective,
        state: &ApplicationState,
        last_customer_text: &str,
        recent: &[String],
    ) -> Result<ResponseDraft, AppError> {
        self.llm
            .respond(ResponseRequest {
                directive: directive.clone(),
                current_state: state.clone(),
                last_customer_text: last_customer_text.to_string(),
                recent_turns: recent.to_vec(),
                language: state.language,
            })
            .await
            .map_err(AppError::Provider)
    }
}

pub(crate) fn total_latency(llm_ms: i64, client_received_at: Option<i64>) -> i64 {
    match client_received_at {
        Some(at) => (chrono::Utc::now().timestamp_millis() - at).max(llm_ms),
        None => llm_ms,
    }
}
