//! `finish_session`: build the application + manager email, persist both, then
//! hand over to the CRM. The lead push is intentionally non-fatal.

use mca_core::domain::application_record::Application;
use mca_core::email::{compose as email_compose, EmailInput};
use mca_core::error::AppError;
use mca_core::ids::{ApplicationId, CorrelationId, SessionId};

use super::{DialogueEngine, FinishOutcome};

impl DialogueEngine {
    pub async fn finish_session(
        &self,
        session_id: SessionId,
        reason: Option<String>,
        _correlation_id: Option<CorrelationId>,
    ) -> Result<FinishOutcome, AppError> {
        let mut session = self
            .sessions
            .load_session(session_id)
            .await?
            .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;
        session
            .mark_finished(reason.unwrap_or_else(|| "call finished".into()))
            .map_err(AppError::Conflict)?;

        let messages = self.sessions.load_messages(session_id, None).await?;
        let application = Application::new(
            ApplicationId::new(),
            session.id,
            session.call_id,
            session.state.clone(),
            session.turn_count as i32,
        );

        let email = if self.email.enabled {
            Some(email_compose(&EmailInput {
                application_id: application.id,
                session_id: session.id,
                manager_email: self.email.manager_email.clone(),
                company_name: self.email.company_name.clone(),
                state: application.state.clone(),
                transcript: messages.clone(),
                transcript_line_limit: self.email.transcript_line_limit,
            }))
        } else {
            None
        };

        self.applications
            .insert_application(&application, email.as_ref())
            .await?;
        session.application_id = Some(application.id);
        self.sessions.update_session(&session).await?;

        let lead_accepted = self
            .push_lead(&application, &messages, email.as_ref())
            .await;
        self.emit_finished(&session);
        tracing::info!(session_id = %session.id, application_id = %application.id, "session finished");

        Ok(FinishOutcome {
            session,
            application,
            email,
            lead_accepted,
        })
    }
}
