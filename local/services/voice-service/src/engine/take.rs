//! Lock-free audio handoff: copies the end-of-utterance audio out of the
//! registry so `push_audio` can await provider calls unconditioned on the lock.

use mca_core::domain::language::Language;
use mca_proto::voice::v1::AudioChunk;

use super::{VoiceEngine, CALL_IN_PROGRESS, MAX_AUDIO_BYTES};
use crate::error::VoiceError;

/// Audio copied out of the registry so processing happens lock-free.
pub(crate) struct Utterance {
    pub session_id: String,
    pub language: Language,
    pub external_call_id: String,
    pub bytes: Vec<u8>,
    pub sample_rate: u32,
    pub seq: u64,
}

impl VoiceEngine {
    pub(crate) fn take_utterance(
        &self,
        chunk: &AudioChunk,
    ) -> Result<Option<Utterance>, VoiceError> {
        let call_id = chunk.call_id.clone();
        let mut calls = self.calls.write().expect("call registry lock poisoned");
        let record = calls
            .get_mut(&call_id)
            .ok_or_else(|| VoiceError::UnknownCall(call_id.clone()))?;
        if record.status != CALL_IN_PROGRESS {
            return Err(VoiceError::NotOpen(
                call_id,
                format!("status={}", record.status),
            ));
        }
        record.buffer.extend_from_slice(&chunk.audio);
        if !chunk.end_of_utterance && record.buffer.len() < MAX_AUDIO_BYTES {
            return Ok(None);
        }
        let utterance = Utterance {
            session_id: record.session_id.clone(),
            language: record.language,
            external_call_id: record.external_call_id.clone(),
            bytes: std::mem::take(&mut record.buffer),
            sample_rate: chunk.sample_rate as u32,
            seq: record.next_seq,
        };
        record.next_seq += 1;
        Ok(Some(utterance))
    }
}
