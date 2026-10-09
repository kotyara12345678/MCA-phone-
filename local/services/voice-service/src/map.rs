//! Domain ↔ generated-proto conversions for the voice surface.

use mca_core::domain::language::Language;
use mca_core::ports::stt::AudioFormat;
use mca_proto::ai::v1::Application;
use mca_proto::voice::v1 as proto;

use crate::engine::{now_ms, CallRecord, EventOut};
use crate::error::VoiceError;

pub fn parse_language(raw: &str) -> Result<Language, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "ru" | "ru-ru" => Ok(Language::Ru),
        "kk" | "kk-kz" => Ok(Language::Kk),
        "en" | "en-us" => Ok(Language::En),
        "de" | "de-de" => Ok(Language::De),
        other => Err(format!("unsupported language `{other}`")),
    }
}

/// Wire `AudioFormat` → core format. `UNSPECIFIED` is a client bug, not an
/// auto-detection hint.
pub fn audio_format(value: i32) -> Result<AudioFormat, VoiceError> {
    match value {
        1 => Ok(AudioFormat::PcmS16Le),
        2 => Ok(AudioFormat::PcmS16Be),
        3 => Ok(AudioFormat::Ulaw),
        4 => Ok(AudioFormat::Alaw),
        5 => Ok(AudioFormat::Opus),
        other => Err(VoiceError::Invalid(format!(
            "unsupported audio format {other}"
        ))),
    }
}

pub fn wire_format(format: AudioFormat) -> proto::AudioFormat {
    match format {
        AudioFormat::PcmS16Le => proto::AudioFormat::PcmS16le,
        AudioFormat::PcmS16Be => proto::AudioFormat::PcmS16be,
        AudioFormat::Ulaw => proto::AudioFormat::Ulaw,
        AudioFormat::Alaw => proto::AudioFormat::Alaw,
        AudioFormat::Opus => proto::AudioFormat::Opus,
    }
}

pub fn call(record: &CallRecord) -> proto::Call {
    proto::Call {
        call_id: record.call_id.clone(),
        session_id: record.session_id.clone(),
        external_call_id: record.external_call_id.clone(),
        from_number: record.from_number.clone(),
        to_number: record.to_number.clone(),
        status: record.status,
        stage: record.stage,
        started_at_unix_ms: record.started_at_unix_ms,
        ended_at_unix_ms: record.ended_at_unix_ms,
        turn_count: record.turn_count,
        speaking: record.speaking,
        last_transcript: record.last_transcript.clone(),
        last_reply: record.last_reply.clone(),
    }
}

/// Qualification number for `HangupResponse`: prefers the dialogue state's own
/// qualification, then falls back to the persisted application status.
pub fn qualification(application: &Application) -> i32 {
    if let Some(state) = &application.state {
        if state.qualification != 0 {
            return state.qualification;
        }
    }
    match application.status {
        2 => 2,
        3 => 3,
        4 => 4,
        _ => 1,
    }
}

/// A "reply" event carrying audio back to the caller.
pub fn event(call_id: &str, outcome: &EventOut) -> proto::VoiceEvent {
    proto::VoiceEvent {
        call_id: call_id.to_string(),
        seq: outcome.seq as i64,
        kind: "reply".into(),
        at_unix_ms: now_ms(),
        text: outcome.reply.clone(),
        stage: outcome.stage,
        audio: outcome.audio.clone(),
        audio_format: wire_format(outcome.audio_format) as i32,
        end_of_utterance: outcome.end_of_utterance,
        interrupted: false,
        latency_ms: outcome.latency_ms,
    }
}

/// Acknowledgement event when an utterance produced no reply (buffering or
/// silence), so unary callers always receive exactly one event.
pub fn no_reply_event(call_id: &str, chunk: &proto::AudioChunk) -> proto::VoiceEvent {
    proto::VoiceEvent {
        call_id: call_id.to_string(),
        seq: chunk.seq,
        kind: "no_reply".into(),
        at_unix_ms: now_ms(),
        audio_format: chunk.format,
        end_of_utterance: chunk.end_of_utterance,
        ..proto::VoiceEvent::default()
    }
}
