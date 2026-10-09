//! Domain → proto conversions for the ai-service API surface.
//!
//! One direction only: the client drives the FSM; the server never accepts a
//! state, so there is no proto → domain path to accidentally trust.

pub mod enums;
pub mod messages;

use mca_core::domain::application::ApplicationState;
use mca_core::domain::application_record::Application;
use mca_core::domain::session::Session;
use mca_core::email::EmailDraft;
use mca_proto::ai::v1 as proto;

pub use messages::{event_to_proto, latency_to_proto, message_to_proto};

/// Error while converting a value a caller supplied (usually an id or enum).
#[derive(Debug)]
pub struct MapError(pub String);

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for MapError {}

pub fn state_to_proto(state: &ApplicationState) -> proto::ApplicationState {
    proto::ApplicationState {
        cargo: state.cargo.clone().unwrap_or_default(),
        weight_kg: state.weight_kg,
        volume_m3: state.volume_m3,
        origin_country: state.origin_country.clone().unwrap_or_default(),
        origin_city: state.origin_city.clone().unwrap_or_default(),
        destination_country: state.destination_country.clone().unwrap_or_default(),
        destination_city: state.destination_city.clone().unwrap_or_default(),
        ready_date: state.ready_date.clone().unwrap_or_default(),
        contact: state.contact.clone().unwrap_or_default(),
        special_requirements: state.special_requirements.clone().unwrap_or_default(),
        intent: state.intent.as_str().to_string(),
        language: state.language.as_str().to_string(),
        qualification: enums::qualification(state.qualification) as i32,
        qualification_reason: state.qualification_reason.clone().unwrap_or_default(),
        missing_fields: state
            .missing_fields
            .iter()
            .map(|field| field.as_str().to_string())
            .collect(),
    }
}

pub fn session_to_proto(session: &Session) -> proto::Session {
    proto::Session {
        session_id: session.id.to_string(),
        call_id: session.call_id.to_string(),
        external_call_id: session.external_call_id.clone().unwrap_or_default(),
        status: enums::session_status(session.status) as i32,
        stage: enums::stage(session.stage) as i32,
        started_at_unix_ms: session.started_at.timestamp_millis(),
        ended_at_unix_ms: session.ended_at.map(|t| t.timestamp_millis()).unwrap_or(0),
        state: Some(state_to_proto(&session.state)),
        application_id: session
            .application_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
        finish_reason: session.finish_reason.clone().unwrap_or_default(),
        avg_llm_latency_ms: session.latency.llm_ms,
        avg_stt_latency_ms: session.latency.stt_ms,
        avg_tts_latency_ms: session.latency.tts_ms,
        avg_total_latency_ms: session.latency.total_ms,
    }
}

pub fn application_to_proto(application: &Application) -> proto::Application {
    proto::Application {
        id: application.id.to_string(),
        session_id: application.session_id.to_string(),
        call_id: application.call_id.to_string(),
        status: enums::application_status(application.state.qualification) as i32,
        state: Some(state_to_proto(&application.state)),
        summary: application.summary.clone(),
        missing_fields: application.missing_fields.clone(),
        created_at_unix_ms: application.created_at.timestamp_millis(),
        updated_at_unix_ms: application.updated_at.timestamp_millis(),
        turn_count: application.turn_count,
    }
}

pub fn email_to_proto(email: &EmailDraft) -> proto::EmailDraft {
    proto::EmailDraft {
        id: email.id.to_string(),
        application_id: email.application_id.to_string(),
        to: email.to.clone(),
        subject: email.subject.clone(),
        plain_text_body: email.plain_text_body.clone(),
        html_body: email.html_body.clone(),
        generated_at_unix_ms: email.generated_at.timestamp_millis(),
    }
}
