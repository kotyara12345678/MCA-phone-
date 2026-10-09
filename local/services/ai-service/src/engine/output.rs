//! Durable output of a turn: writes the customer/agent message pair with its
//! post-turn state snapshot, then emits the event and latency metrics. The
//! finish-side output (email, CRM handover) lives in `handoff.rs`.

use mca_core::domain::message::Message;
use mca_core::domain::session::Session;
use mca_core::domain::speaker::Speaker;
use mca_core::error::AppError;
use mca_core::ids::CorrelationId;
use mca_core::metrics::names;
use mca_core::ports::llm::ResponseDraft;

use super::{DialogueEngine, SessionEvent};

impl DialogueEngine {
    pub(crate) async fn persist_messages(
        &self,
        session: &Session,
        trimmed: &str,
        correlation_id: Option<CorrelationId>,
        response: &ResponseDraft,
        llm_ms: i64,
        total_ms: i64,
    ) -> Result<Message, AppError> {
        let customer = Message::new(session.id, Speaker::Customer, trimmed, correlation_id)
            .with_state(session.state.clone())
            .with_latency(llm_ms);
        let agent = Message::new(session.id, Speaker::Agent, &response.text, None)
            .with_state(session.state.clone())
            .with_latency(total_ms);
        self.sessions.insert_message(&customer).await?;
        self.sessions.insert_message(&agent).await?;
        self.sessions.update_session(session).await?;
        Ok(agent)
    }

    pub(crate) fn emit_turn(
        &self,
        session: &Session,
        agent_text: &str,
        llm_ms: i64,
        total_ms: i64,
    ) {
        self.events.emit(SessionEvent::new(
            session.id,
            "message",
            Some(Speaker::Agent),
            Some(agent_text.to_string()),
            Some(session.stage),
            Some(session.state.qualification),
            total_ms,
        ));
        self.metrics.increment_counter(names::TURNS_PROCESSED, &[]);
        self.metrics.observe(names::LLM_LATENCY, llm_ms, &[]);
        self.metrics
            .observe(names::TOTAL_RESPONSE_LATENCY, total_ms, &[]);
    }
}
