//! Finish-side effects: push the CRM lead + deliver the manager email, then
//! record the outcome. A CRM failure is logged, never fatal — the application
//! and the email draft are already persisted before this runs.

use mca_core::domain::application_record::Application;
use mca_core::domain::qualification::Qualification;
use mca_core::domain::session::Session;
use mca_core::email::EmailDraft;
use mca_core::metrics::names;
use mca_core::ports::crm::CrmLead;

use super::{DialogueEngine, SessionEvent};

impl DialogueEngine {
    pub(crate) async fn push_lead(
        &self,
        application: &Application,
        messages: &[mca_core::domain::message::Message],
        email: Option<&EmailDraft>,
    ) -> bool {
        let excerpt = EmailDraft::transcript_excerpt(messages, self.email.transcript_line_limit);
        let subject = email.map(|e| e.subject.clone()).unwrap_or_default();
        let pushed = self
            .crm
            .push_lead(CrmLead {
                application_id: application.id,
                summary: application.summary.clone(),
                state: application.state.clone(),
                qualification: application.state.qualification,
                manager_email_subject: subject,
                transcript_excerpt: excerpt,
            })
            .await
            .map(|r| r.accepted)
            .unwrap_or(false);
        if !pushed {
            tracing::warn!(application_id = %application.id, "crm lead push failed");
        }
        if let Some(draft) = email {
            let sent = self
                .crm
                .deliver_email(&draft.to, &draft.subject, &draft.plain_text_body)
                .await
                .map(|r| r.accepted)
                .unwrap_or(false);
            if !sent {
                tracing::warn!(application_id = %application.id, "manager email delivery failed");
            }
        }
        pushed
    }

    pub(crate) fn emit_finished(&self, session: &Session) {
        self.metrics.increment_counter(names::CALLS_FINISHED, &[]);
        self.metrics.set_gauge(names::ACTIVE_SESSIONS, 0, &[]);
        match session.state.qualification {
            Qualification::Qualified => self.metrics.increment_counter(names::QUALIFIED_CALLS, &[]),
            Qualification::Rejected => self.metrics.increment_counter(names::REJECTED_CALLS, &[]),
            Qualification::NeedsHuman => self
                .metrics
                .increment_counter(names::HUMAN_REVIEW_CALLS, &[]),
            _ => {}
        }
        self.events.emit(SessionEvent::new(
            session.id,
            "session_finished",
            None,
            None,
            Some(session.stage),
            Some(session.state.qualification),
            session.duration_ms(),
        ));
    }
}
