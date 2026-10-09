//! Static email settings, the engine's public outcome structs, and the small
//! context helper both provider prompts rely on. No behaviour beyond
//! construction, so these stay together without bloat.

use mca_core::domain::application_record::Application;
use mca_core::domain::latency::LatencySample;
use mca_core::domain::message::Message;
use mca_core::domain::session::Session;
use mca_core::email::EmailDraft;

#[derive(Debug, Clone)]
pub struct EmailSettings {
    pub enabled: bool,
    pub manager_email: String,
    pub company_name: String,
    pub transcript_line_limit: usize,
}

pub struct StartOutcome {
    pub session: Session,
    pub greeting: String,
}

pub struct TurnOutcome {
    pub session: Session,
    pub reply: String,
    pub agent_message: Message,
    pub latency: LatencySample,
}

pub struct FinishOutcome {
    pub session: Session,
    pub application: Application,
    pub email: Option<EmailDraft>,
    pub lead_accepted: bool,
}

/// Turn/list context extractor used by both extraction and response prompts.
pub(crate) fn recent_turns(messages: &[Message], max: usize) -> Vec<String> {
    messages
        .iter()
        .rev()
        .take(max)
        .map(|m| format!("{}: {}", m.speaker.as_str(), m.text))
        .collect()
}
