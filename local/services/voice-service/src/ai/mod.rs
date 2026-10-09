//! Gateway to ai-service over its gRPC surface.
//!
//! `GrpcAiGateway` is the live adapter; the trait exists so the engine can be
//! tested against the real ai-service (integration/E2E) or stubbed locally.

mod grpc_ai;
mod rpc;

pub use grpc_ai::GrpcAiGateway;

/// Outcome of starting a fresh dialogue session in ai-service.
#[derive(Debug, Clone)]
pub struct StartOutcome {
    pub session_id: String,
    pub call_id: String,
    pub greeting: String,
    pub stage: i32,
    pub started_at_unix_ms: i64,
}

/// Outcome of one customer utterance processed by ai-service.
#[derive(Debug, Clone)]
pub struct TurnOutcome {
    pub session_id: String,
    pub reply: String,
    pub stage: i32,
    pub finished: bool,
    pub needs_human: bool,
    pub latency_ms: i64,
}

/// Outcome of hanging up: the persisted application and its qualification.
#[derive(Debug, Clone)]
pub struct FinishOutcome {
    pub session_id: String,
    pub application_id: String,
    pub qualification: i32,
}

/// The dialogue-brains port the voice engine drives.
#[async_trait::async_trait]
pub trait AiGateway: Send + Sync {
    async fn start(
        &self,
        external_call_id: &str,
        from_number: &str,
        language: mca_core::domain::language::Language,
    ) -> Result<StartOutcome, crate::error::VoiceError>;

    async fn turn(
        &self,
        session_id: &str,
        transcript: &str,
        correlation_id: &str,
        client_received_at_unix_ms: i64,
    ) -> Result<TurnOutcome, crate::error::VoiceError>;

    async fn finish(
        &self,
        session_id: &str,
        reason: &str,
    ) -> Result<FinishOutcome, crate::error::VoiceError>;
}
