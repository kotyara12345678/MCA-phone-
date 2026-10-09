use serde::{Deserialize, Serialize};

use crate::domain::application::ApplicationState;
use crate::domain::message::Message;
use crate::ids::{ApplicationId, EmailId, SessionId};

/// A ready-to-send manager email. Typed on purpose: SMTP, an email API or a CRM
/// note can all be built on top of it without re-parsing text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailDraft {
    pub id: EmailId,
    pub application_id: ApplicationId,
    pub session_id: SessionId,
    pub to: String,
    pub subject: String,
    pub plain_text_body: String,
    pub html_body: String,
    /// One-paragraph business summary, reused by the CRM lead payload.
    pub summary: String,
    pub missing_fields: Vec<String>,
    pub qualification: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl EmailDraft {
    /// Approximate size in bytes; the transport layer uses it for size limits.
    pub fn size_bytes(&self) -> usize {
        self.plain_text_body.len() + self.html_body.len() + self.subject.len()
    }

    /// Short excerpt of the dialogue for the CRM, never more than `max` lines.
    pub fn transcript_excerpt(messages: &[Message], max: usize) -> String {
        let lines: Vec<String> = messages
            .iter()
            .filter(|m| !m.text.trim().is_empty())
            .map(|m| {
                let who = match m.speaker {
                    crate::domain::speaker::Speaker::Customer => "Клиент",
                    crate::domain::speaker::Speaker::Agent => "Ассистент",
                    crate::domain::speaker::Speaker::System => "Система",
                };
                format!("{who}: {}", m.text.trim())
            })
            .collect();
        let mut excerpt = lines.into_iter().rev().take(max).collect::<Vec<_>>();
        excerpt.reverse();
        excerpt.join("\n")
    }
}

/// Everything the composer needs. A single input struct keeps the composer
/// free of database and session lookups.
#[derive(Debug, Clone)]
pub struct EmailInput {
    pub application_id: ApplicationId,
    pub session_id: SessionId,
    pub manager_email: String,
    pub company_name: String,
    pub state: ApplicationState,
    pub transcript: Vec<Message>,
    pub transcript_line_limit: usize,
}

#[cfg(test)]
mod tests;
