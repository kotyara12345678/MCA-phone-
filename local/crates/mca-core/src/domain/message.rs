use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::application::ApplicationState;
use super::speaker::Speaker;
use crate::ids::{CorrelationId, MessageId, SessionId};

/// One persisted conversation turn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub session_id: SessionId,
    pub speaker: Speaker,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub correlation_id: Option<CorrelationId>,
    pub processing_latency_ms: i64,
    /// Snapshot of the structured state *after* this turn, so a transcript
    /// replay reproduces exactly what the agent believed at the time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_after: Option<Box<ApplicationState>>,
}

impl Message {
    pub fn new(
        session_id: SessionId,
        speaker: Speaker,
        text: impl Into<String>,
        correlation_id: Option<CorrelationId>,
    ) -> Self {
        Self {
            id: MessageId::new(),
            session_id,
            speaker,
            text: text.into(),
            created_at: Utc::now(),
            correlation_id,
            processing_latency_ms: 0,
            state_after: None,
        }
    }

    pub fn with_state(mut self, state: ApplicationState) -> Self {
        self.state_after = Some(Box::new(state));
        self
    }

    pub fn with_latency(mut self, latency_ms: i64) -> Self {
        self.processing_latency_ms = latency_ms;
        self
    }

    /// Number of Unicode scalar values, used for payload size limits.
    pub fn text_len(&self) -> usize {
        self.text.chars().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_carries_state_snapshot() {
        let state = ApplicationState {
            cargo: Some("оборудование".into()),
            ..Default::default()
        };
        let msg = Message::new(SessionId::new(), Speaker::Customer, "тест", None).with_state(state);
        assert_eq!(
            msg.state_after.as_ref().unwrap().cargo.as_deref(),
            Some("оборудование")
        );
    }

    #[test]
    fn text_len_counts_characters_not_bytes() {
        let msg = Message::new(SessionId::new(), Speaker::Customer, "груз", None);
        assert_eq!(msg.text_len(), 4);
    }
}
