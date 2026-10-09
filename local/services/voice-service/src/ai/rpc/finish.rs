//! `finish`: end the session and read the persisted application back out.

use mca_proto::ai::v1::FinishSessionRequest;

use super::super::grpc_ai::GrpcAiGateway;
use super::super::FinishOutcome;
use crate::error::VoiceError;

pub(crate) async fn finish(
    gateway: &GrpcAiGateway,
    session_id: &str,
    reason: &str,
) -> Result<FinishOutcome, VoiceError> {
    let request = FinishSessionRequest {
        session_id: session_id.to_string(),
        reason: reason.to_string(),
        correlation_id: String::new(),
    };
    let data = gateway
        .call(async { gateway.client.clone().finish_session(request).await })
        .await?
        .into_inner();
    let application = data
        .application
        .ok_or_else(|| VoiceError::Upstream("no application in response".into()))?;
    let qualification = crate::map::qualification(&application);
    Ok(FinishOutcome {
        session_id: data.session_id,
        application_id: application.id,
        qualification,
    })
}
