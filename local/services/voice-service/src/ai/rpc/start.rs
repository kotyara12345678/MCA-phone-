//! `start`: open a dialogue session and get back the greeting to play.

use mca_core::domain::language::Language;
use mca_proto::ai::v1::StartSessionRequest;

use super::super::grpc_ai::GrpcAiGateway;
use super::super::StartOutcome;
use crate::error::VoiceError;

pub(crate) async fn start(
    gateway: &GrpcAiGateway,
    external_call_id: &str,
    from_number: &str,
    language: Language,
) -> Result<StartOutcome, VoiceError> {
    let request = StartSessionRequest {
        external_call_id: external_call_id.to_string(),
        caller_phone: from_number.to_string(),
        language: language.as_str().to_string(),
        correlation_id: String::new(),
        metadata: Default::default(),
    };
    let data = gateway
        .call(async { gateway.client.clone().start_session(request).await })
        .await?
        .into_inner();
    Ok(StartOutcome {
        session_id: data.session_id,
        call_id: data.call_id,
        greeting: data.greeting,
        stage: data.stage,
        started_at_unix_ms: data.started_at_unix_ms,
    })
}
