//! In-process event bus powering `StreamSessionEvents`.
//!
//! Events are broadcast to any number of subscribers; a session-scoped filter
//! (or an empty "watch all" set) is applied by the receiver side of the gRPC
//! stream, never here.

use mca_core::domain::dialogue::DialogueStage;
use mca_core::domain::qualification::Qualification;
use mca_core::domain::speaker::Speaker;
use mca_core::ids::SessionId;
use tokio::sync::broadcast;

/// One broadcast event. Fields mirror the proto `SessionEvent` shape so the
/// map layer stays a pure conversion.
#[derive(Debug, Clone)]
pub struct SessionEvent {
    pub id: String,
    pub session_id: SessionId,
    pub kind: String,
    pub at_unix_ms: i64,
    pub speaker: Option<Speaker>,
    pub text: Option<String>,
    pub stage: Option<DialogueStage>,
    pub qualification: Option<Qualification>,
    pub latency_ms: i64,
}

impl SessionEvent {
    pub fn new(
        session_id: SessionId,
        kind: impl Into<String>,
        speaker: Option<Speaker>,
        text: Option<String>,
        stage: Option<DialogueStage>,
        qualification: Option<Qualification>,
        latency_ms: i64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_id,
            kind: kind.into(),
            at_unix_ms: chrono::Utc::now().timestamp_millis(),
            speaker,
            text,
            stage,
            qualification,
            latency_ms,
        }
    }
}

/// Broadcast channel wrapper. Emission never blocks the dialogue engine: a
/// slow subscriber is dropped by the bounded channel instead of stalling a live
/// call.
#[derive(Debug, Clone)]
pub struct EventBus {
    tx: broadcast::Sender<SessionEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn emit(&self, event: SessionEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.tx.subscribe()
    }
}
