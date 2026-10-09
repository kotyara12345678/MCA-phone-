//! Session aggregate.

mod finish;

use super::application::ApplicationState;
use super::dialogue::DialogueStage;
use super::latency::LatencySample;
use super::qualification::Qualification;
use crate::ids::{ApplicationId, CallId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Lifecycle of a conversation session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    #[default]
    Active,
    Finished,
    Failed,
}

impl SessionStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Finished => "finished",
            Self::Failed => "failed",
        }
    }
}

/// The in-memory aggregate for one call, persisted to `calls`, `messages`
/// and `applications`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub call_id: CallId,
    pub external_call_id: Option<String>,
    pub caller_phone: Option<String>,
    pub status: SessionStatus,
    pub stage: DialogueStage,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub state: ApplicationState,
    pub application_id: Option<ApplicationId>,
    pub finish_reason: Option<String>,
    pub turn_count: u32,
    pub latency: LatencySample,
    /// The customer said goodbye or asked to stop collecting.
    pub customer_hangup_requested: bool,
}

impl Session {
    pub fn new(
        call_id: CallId,
        external_call_id: Option<String>,
        caller_phone: Option<String>,
    ) -> Self {
        Self {
            id: SessionId::new(),
            call_id,
            external_call_id,
            caller_phone,
            status: SessionStatus::Active,
            stage: DialogueStage::Greeting,
            started_at: Utc::now(),
            ended_at: None,
            state: ApplicationState::default(),
            application_id: None,
            finish_reason: None,
            turn_count: 0,
            latency: LatencySample::default(),
            customer_hangup_requested: false,
        }
    }

    pub fn duration_ms(&self) -> i64 {
        self.ended_at
            .unwrap_or_else(Utc::now)
            .signed_duration_since(self.started_at)
            .num_milliseconds()
            .max(0)
    }

    pub fn qualification(&self) -> Qualification {
        self.state.qualification
    }
}

#[cfg(test)]
mod tests;
