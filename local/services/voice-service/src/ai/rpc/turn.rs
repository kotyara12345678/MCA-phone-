//! `turn`: run one final transcript through the active session.

use mca_proto::ai::v1::ProcessTurnRequest;

use super::super::grpc_ai::GrpcAiGateway;
use super::super::TurnOutcome;
use crate::error::VoiceError;

pub(crate) async fn turn(
    gateway: &GrpcAiGateway,
    session_id: &str,
    transcript: &str,
    correlation_id: &str,
    client_received_at_unix_ms: i64,
) -> Result<TurnOutcome, VoiceError> {
    let request = ProcessTurnRequest {
        session_id: session_id.to_string(),
        transcript: transcript.to_string(),
        correlation_id: correlation_id.to_string(),
        client_received_at_unix_ms,
        is_final: true,
    };
    let data = gateway
        .call(async { gateway.client.clone().process_turn(request).await })
        .await?
        .into_inner();
    Ok(TurnOutcome {
        session_id: data.session_id,
        reply: data.reply_text,
        stage: data.stage,
        finished: data.finished,
        needs_human: data.needs_human,
        latency_ms: data.latency.map(|latency| latency.total_ms).unwrap_or(0),
    })
}
