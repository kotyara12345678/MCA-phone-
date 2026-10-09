//! `start_session`: create the aggregate, persist it and answer with the
//! company-approved greeting (no model output on the very first turn).

use mca_core::domain::language::Language;
use mca_core::domain::message::Message;
use mca_core::domain::session::Session;
use mca_core::domain::speaker::Speaker;
use mca_core::error::AppError;
use mca_core::ids::{CallId, CorrelationId};
use mca_core::metrics::names;
use mca_core::policy::compose;

use super::{DialogueEngine, SessionEvent, StartOutcome};

impl DialogueEngine {
    pub async fn start_session(
        &self,
        external_call_id: Option<String>,
        caller_phone: Option<String>,
        language: Option<Language>,
        correlation_id: Option<CorrelationId>,
    ) -> Result<StartOutcome, AppError> {
        let mut session = Session::new(CallId::new(), external_call_id, caller_phone);
        if let Some(lang) = language {
            if !Language::ALL.contains(&lang) {
                return Err(AppError::InvalidRequest(format!(
                    "unsupported language: {lang:?}"
                )));
            }
            session.state.language = lang;
        }
        let greeting = compose::greeting().to_string();

        self.sessions.insert_session(&session).await?;
        let greeting_message = Message::new(session.id, Speaker::Agent, &greeting, correlation_id)
            .with_state(session.state.clone());
        self.sessions.insert_message(&greeting_message).await?;

        self.events.emit(SessionEvent::new(
            session.id,
            "session_started",
            Some(Speaker::Agent),
            Some(greeting.clone()),
            Some(session.stage),
            Some(session.state.qualification),
            0,
        ));
        self.metrics.increment_counter(names::SESSIONS_STARTED, &[]);
        self.metrics.set_gauge(names::ACTIVE_SESSIONS, 1, &[]);
        tracing::info!(session_id = %session.id, "session started");
        Ok(StartOutcome { session, greeting })
    }
}
