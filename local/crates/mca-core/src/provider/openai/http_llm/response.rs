//! Response turn: compose the reply text, letting the LLM restyle a
//! deterministic baseline.

use super::HttpLlmProvider;
use crate::error::ProviderError;
use crate::policy::compose;
use crate::ports::llm::{ResponseDraft, ResponseRequest};
use crate::provider::openai::response_prompt::{response_system_prompt, response_user_prompt};
use crate::provider::openai::types::ChatMessage;

/// One response round trip. The baseline is always composed first so a live
/// LLM may only *restate*, never invent: any failure or empty reply falls back
/// to the deterministic text.
pub(super) async fn respond(
    provider: &HttpLlmProvider,
    request: ResponseRequest,
) -> Result<ResponseDraft, ProviderError> {
    let baseline = compose::compose(&request.directive);
    let messages = vec![
        ChatMessage::system(response_system_prompt()),
        ChatMessage::user(response_user_prompt(&request, &baseline)),
    ];
    match provider.client.complete(messages, false).await {
        Ok((raw, _latency)) => {
            let text = raw.trim();
            if text.is_empty() {
                tracing::debug!("llm returned empty reply, using deterministic text");
                return Ok(ResponseDraft { text: baseline });
            }
            Ok(ResponseDraft {
                text: text.to_string(),
            })
        }
        Err(err) => {
            tracing::warn!(
                error = %err,
                kind = ?err.kind(),
                "llm response failed, using deterministic text"
            );
            Ok(ResponseDraft { text: baseline })
        }
    }
}
