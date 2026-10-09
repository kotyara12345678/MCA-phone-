//! `push_audio`: buffer an utterance, and when it ends transcribe, talk to
//! ai-service, synthesise the reply and play it back. The audio handoff in
//! `take.rs` keeps the registry lock out of every `.await`.

use mca_core::ports::stt::AudioFrame;
use mca_core::ports::telephony::{CallHandle, OutboundMedia};
use mca_proto::voice::v1::AudioChunk;

use super::types::{now_ms, EventOut};
use super::{CallRecord, VoiceEngine};
use crate::error::VoiceError;
use crate::map;

impl VoiceEngine {
    pub async fn push_audio(&self, chunk: &AudioChunk) -> Result<Option<EventOut>, VoiceError> {
        let format = map::audio_format(chunk.format)?;
        let call_id = chunk.call_id.clone();

        let Some(utterance) = self.take_utterance(chunk)? else {
            return Ok(None);
        };

        let frame = AudioFrame::from_bytes(format, utterance.sample_rate, &utterance.bytes);
        let transcript = self
            .stt
            .transcribe(&frame, utterance.language)
            .await
            .map_err(VoiceError::from_speech)?;
        if transcript.text.trim().is_empty() {
            return Ok(None);
        }

        let turn = self
            .ai
            .turn(&utterance.session_id, &transcript.text, &call_id, now_ms())
            .await?;
        let synth = self
            .tts
            .synthesize(&turn.reply, utterance.language)
            .await
            .map_err(VoiceError::from_speech)?;
        let audio = synth.audio.to_bytes();
        self.telephony
            .send_media(
                &CallHandle {
                    external_call_id: utterance.external_call_id.clone(),
                },
                OutboundMedia {
                    external_call_id: utterance.external_call_id,
                    seq: utterance.seq,
                    audio: audio.clone(),
                    format: synth.audio.format,
                },
            )
            .await
            .map_err(VoiceError::from_telephony)?;

        let mut calls = self.calls.write().expect("call registry lock poisoned");
        let record: &mut CallRecord = calls
            .get_mut(&call_id)
            .ok_or_else(|| VoiceError::UnknownCall(call_id.clone()))?;
        record.last_transcript = transcript.text;
        record.last_reply = turn.reply.clone();
        record.turn_count += 1;
        record.stage = turn.stage;
        record.speaking = false;

        Ok(Some(EventOut {
            reply: turn.reply,
            audio,
            audio_format: synth.audio.format,
            stage: turn.stage,
            latency_ms: turn.latency_ms,
            seq: utterance.seq,
            end_of_utterance: true,
        }))
    }
}
