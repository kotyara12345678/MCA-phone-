use serde::{Deserialize, Serialize};

use super::application::ApplicationState;
use super::message::Message;
use super::qualification::Qualification;
use crate::ids::{ApplicationId, CallId, SessionId};

/// A finished handover to a manager. Row in `applications`; the email draft is
/// stored alongside it in `application_events`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Application {
    pub id: ApplicationId,
    pub session_id: SessionId,
    pub call_id: CallId,
    pub state: ApplicationState,
    pub summary: String,
    pub missing_fields: Vec<String>,
    pub turn_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Application {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ApplicationId,
        session_id: SessionId,
        call_id: CallId,
        state: ApplicationState,
        turn_count: i32,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            summary: crate::domain::summary::summarize(&state),
            missing_fields: state
                .missing_fields
                .iter()
                .map(|f| f.as_str().to_string())
                .collect(),
            id,
            session_id,
            call_id,
            state,
            turn_count,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn status(&self) -> Qualification {
        self.state.qualification
    }
}

/// Compact projection of a stored message, used by list endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageView {
    pub id: String,
    pub speaker: super::speaker::Speaker,
    pub text: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processing_latency_ms: i64,
}

impl From<&Message> for MessageView {
    fn from(value: &Message) -> Self {
        Self {
            id: value.id.to_string(),
            speaker: value.speaker,
            text: value.text.clone(),
            created_at: value.created_at,
            processing_latency_ms: value.processing_latency_ms,
        }
    }
}
