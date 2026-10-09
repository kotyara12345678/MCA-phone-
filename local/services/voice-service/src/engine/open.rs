//! `open`: stand up a dialogue session in ai-service, answer the call with the
//! telephony layer, and synthesise + play the greeting.

use mca_core::ports::telephony::{InboundCall, OutboundMedia};
use mca_proto::voice::v1::OpenCallRequest;
use uuid::Uuid;

use super::types::{now_ms, OpenOutcome};
use super::{CallRecord, VoiceEngine, CALL_IN_PROGRESS};
use crate::error::VoiceError;

impl VoiceEngine {
    pub async fn open(&self, request: &OpenCallRequest) -> Result<OpenOutcome, VoiceError> {
        let language = self
            .resolve_language(&request.language)
            .map_err(VoiceError::Invalid)?;
        if request.external_call_id.is_empty() {
            return Err(VoiceError::Invalid("external_call_id is required".into()));
        }
        let external_call_id = request.external_call_id.clone();
        let started_at = now_ms();

        let start = self
            .ai
            .start(&external_call_id, &request.from_number, language)
            .await?;
        let handle = self
            .telephony
            .answer(InboundCall {
                external_call_id: external_call_id.clone(),
                from_number: Some(request.from_number.clone()),
                to_number: Some(request.to_number.clone()),
                language: Some(language.as_str().to_string()),
            })
            .await
            .map_err(VoiceError::from_telephony)?;

        let greeting = self
            .tts
            .synthesize(&start.greeting, language)
            .await
            .map_err(VoiceError::from_speech)?;
        let audio = greeting.audio.to_bytes();
        self.telephony
            .send_media(
                &handle,
                OutboundMedia {
                    external_call_id: external_call_id.clone(),
                    seq: 0,
                    audio: audio.clone(),
                    format: greeting.audio.format,
                },
            )
            .await
            .map_err(VoiceError::from_telephony)?;

        let record = CallRecord {
            call_id: Uuid::new_v4().to_string(),
            session_id: start.session_id,
            external_call_id,
            from_number: request.from_number.clone(),
            to_number: request.to_number.clone(),
            status: CALL_IN_PROGRESS,
            stage: start.stage,
            started_at_unix_ms: start.started_at_unix_ms.max(started_at),
            ended_at_unix_ms: 0,
            turn_count: 0,
            speaking: true,
            last_transcript: String::new(),
            last_reply: String::new(),
            language,
            buffer: Vec::new(),
            next_seq: 1,
        };
        self.calls
            .write()
            .expect("call registry lock poisoned")
            .insert(record.call_id.clone(), record.clone());

        Ok(OpenOutcome {
            record,
            greeting_text: start.greeting,
            greeting_audio: audio,
        })
    }
}
