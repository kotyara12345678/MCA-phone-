//! Stub ai gateway, deterministic fakes and helpers for engine-level tests.
//! No network: the `AiGateway` is a scripted stub.

use std::sync::Arc;

use mca_proto::voice::v1::AudioChunk;
use mca_testing::{FakeSttProvider, FakeTelephonyProvider, FakeTtsProvider};
use voice_service::ai::{AiGateway, FinishOutcome, StartOutcome, TurnOutcome};
use voice_service::engine::VoiceEngine;
use voice_service::error::VoiceError;

struct StubAi;

#[async_trait::async_trait]
impl AiGateway for StubAi {
    async fn start(
        &self,
        _external_call_id: &str,
        _from_number: &str,
        _language: mca_core::domain::language::Language,
    ) -> Result<StartOutcome, VoiceError> {
        Ok(StartOutcome {
            session_id: "session-1".into(),
            call_id: "call-1".into(),
            greeting: "Привет".into(),
            stage: 1,
            started_at_unix_ms: 0,
        })
    }
    async fn turn(
        &self,
        session_id: &str,
        transcript: &str,
        _correlation_id: &str,
        _client_received_at_unix_ms: i64,
    ) -> Result<TurnOutcome, VoiceError> {
        Ok(TurnOutcome {
            session_id: session_id.to_string(),
            reply: format!("Вы сказали: {transcript}"),
            stage: 3,
            finished: false,
            needs_human: false,
            latency_ms: 5,
        })
    }
    async fn finish(&self, session_id: &str, _reason: &str) -> Result<FinishOutcome, VoiceError> {
        Ok(FinishOutcome {
            session_id: session_id.to_string(),
            application_id: "app-1".into(),
            qualification: 2,
        })
    }
}

pub fn test_engine() -> VoiceEngine {
    VoiceEngine::new(
        Arc::new(FakeSttProvider::new()),
        Arc::new(FakeTtsProvider::default()),
        Arc::new(FakeTelephonyProvider::new()),
        Arc::new(StubAi),
        "ru".to_string(),
    )
}

pub fn chunk(call_id: &str, audio: Vec<u8>, end_of_utterance: bool) -> AudioChunk {
    AudioChunk {
        call_id: call_id.to_string(),
        audio,
        format: 1,
        sample_rate: 8_000,
        channels: 1,
        seq: 1,
        sent_at_unix_ms: 0,
        end_of_utterance,
    }
}
