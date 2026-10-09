//! Domain types for a call in the voice engine.

use std::time::{SystemTime, UNIX_EPOCH};

use mca_core::domain::language::Language;
use mca_core::ports::stt::AudioFormat;

/// Wire-prefix-stripped `mca.voice.v1.CallStatus` values.
pub const CALL_IN_PROGRESS: i32 = 2;
pub const CALL_FINISHED: i32 = 3;

/// Upper bound on buffered audio awaiting an end-of-utterance flag. A hunk of
/// audio that never ends cannot grow the heap forever.
pub const MAX_AUDIO_BYTES: usize = 1024 * 1024;

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Internal call record. `status`/`stage` keep the wire enum integers so the
/// proto mapping is identity and nothing is lost in a round trip.
#[derive(Debug, Clone)]
pub struct CallRecord {
    pub call_id: String,
    pub session_id: String,
    pub external_call_id: String,
    pub from_number: String,
    pub to_number: String,
    pub status: i32,
    pub stage: i32,
    pub started_at_unix_ms: i64,
    pub ended_at_unix_ms: i64,
    pub turn_count: i64,
    pub speaking: bool,
    pub last_transcript: String,
    pub last_reply: String,
    pub language: Language,
    /// PCM buffered between end-of-utterance flags.
    pub(crate) buffer: Vec<u8>,
    pub(crate) next_seq: u64,
}

/// Result of `open`: the registered call plus the greeting to play.
#[derive(Debug, Clone)]
pub struct OpenOutcome {
    pub record: CallRecord,
    pub greeting_text: String,
    pub greeting_audio: Vec<u8>,
}

/// A synthesised reply ready to be streamed back to the caller.
#[derive(Debug, Clone)]
pub struct EventOut {
    pub reply: String,
    pub audio: Vec<u8>,
    pub audio_format: AudioFormat,
    pub stage: i32,
    pub latency_ms: i64,
    pub seq: u64,
    pub end_of_utterance: bool,
}
