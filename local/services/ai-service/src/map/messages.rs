//! Message, latency and event conversions — the streaming half of the map.

use crate::engine::SessionEvent;
use mca_core::domain::latency::LatencySample;
use mca_core::domain::message::Message;
use mca_proto::ai::v1 as proto;

use super::enums;
use super::state_to_proto;

pub fn message_to_proto(message: &Message) -> proto::Message {
    proto::Message {
        id: message.id.to_string(),
        session_id: message.session_id.to_string(),
        speaker: enums::speaker(message.speaker) as i32,
        text: message.text.clone(),
        created_at_unix_ms: message.created_at.timestamp_millis(),
        correlation_id: message
            .correlation_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
        processing_latency_ms: message.processing_latency_ms,
        state_after: message.state_after.as_deref().map(state_to_proto),
    }
}

pub fn latency_to_proto(latency: &LatencySample) -> proto::LatencyBreakdown {
    proto::LatencyBreakdown {
        stt_ms: latency.stt_ms,
        llm_ms: latency.llm_ms,
        tts_ms: latency.tts_ms,
        total_ms: latency.total_ms,
        first_token_ms: latency.first_token_ms,
    }
}

pub fn event_to_proto(event: &SessionEvent) -> proto::SessionEvent {
    proto::SessionEvent {
        id: event.id.clone(),
        session_id: event.session_id.to_string(),
        kind: event.kind.clone(),
        at_unix_ms: event.at_unix_ms,
        speaker: event
            .speaker
            .map(enums::speaker)
            .unwrap_or(proto::Speaker::Unspecified) as i32,
        text: event.text.clone().unwrap_or_default(),
        stage: event
            .stage
            .map(enums::stage)
            .unwrap_or(proto::DialogueStage::Unspecified) as i32,
        qualification: event
            .qualification
            .map(enums::qualification)
            .unwrap_or(proto::Qualification::Unspecified) as i32,
        latency_ms: event.latency_ms,
    }
}
