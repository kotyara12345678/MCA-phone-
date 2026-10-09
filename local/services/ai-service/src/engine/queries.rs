//! Read-side queries: a session with its transcript, or a finished application
//! with its manager email. Both map straight onto the store traits.

use mca_core::domain::application_record::Application;
use mca_core::domain::message::Message;
use mca_core::domain::session::Session;
use mca_core::email::EmailDraft;
use mca_core::error::AppError;
use mca_core::ids::{ApplicationId, SessionId};

use super::DialogueEngine;

impl DialogueEngine {
    pub async fn get_session(
        &self,
        session_id: SessionId,
        message_limit: Option<usize>,
    ) -> Result<(Session, Vec<Message>), AppError> {
        let session = self
            .sessions
            .load_session(session_id)
            .await?
            .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;
        let messages = self
            .sessions
            .load_messages(session_id, message_limit)
            .await?;
        Ok((session, messages))
    }

    pub async fn get_application(
        &self,
        application_id: ApplicationId,
    ) -> Result<(Application, Option<EmailDraft>), AppError> {
        let application = self
            .applications
            .load_application(application_id)
            .await?
            .ok_or_else(|| AppError::ApplicationNotFound(application_id.to_string()))?;
        let email = self.applications.load_email(application_id).await?;
        Ok((application, email))
    }
}
