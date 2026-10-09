//! Extraction turn: turn a transcript line into a JSON state patch.

use super::HttpLlmProvider;
use crate::error::ProviderError;
use crate::ports::llm::{ExtractionRequest, ExtractionResult as LlmExtractionResult, LlmProvider};
use crate::provider::openai::prompts;
use crate::provider::openai::types::ChatMessage;

/// One extraction round trip. Every failure — transport, HTTP status,
/// malformed JSON — falls back to the rule-based extractor, so the dialogue
/// engine never sees a `ProviderError` for a normal turn.
pub(super) async fn extract(
    provider: &HttpLlmProvider,
    request: ExtractionRequest,
) -> Result<LlmExtractionResult, ProviderError> {
    let state_json = serde_json::to_string(&request.current_state).unwrap_or_else(|_| "{}".into());
    let user = prompts::extraction_user_prompt(
        &request.utterance,
        &state_json,
        &request.recent_turns,
        request.last_agent_question.as_deref(),
        request.language,
    );
    let messages = vec![
        ChatMessage::system(prompts::extraction_system_prompt()),
        ChatMessage::user(user),
    ];
    match provider.client.complete(messages, true).await {
        Ok((raw, latency_ms)) => {
            tracing::info!(
                latency_ms,
                provider = provider.client.model(),
                "llm extraction ok"
            );
            match prompts::parse_extraction(&raw) {
                Ok(result) => Ok(result),
                Err(err) => {
                    tracing::warn!(error = %err, "llm returned malformed extraction, using rules");
                    Ok(provider.fallback.extract(request).await?)
                }
            }
        }
        Err(err) => {
            tracing::warn!(error = %err, kind = ?err.kind(), "llm extraction failed, using rules");
            Ok(provider.fallback.extract(request).await?)
        }
    }
}
