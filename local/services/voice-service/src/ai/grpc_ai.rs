//! Live gateway: one persistent gRPC channel to ai-service with a hard
//! timeout per RPC, so an unresponsive upstream degrades to a per-turn error.

use std::time::Duration;

use async_trait::async_trait;
use mca_core::domain::language::Language;
use mca_proto::ai::v1::ai_service_client::AiServiceClient;
use tokio::time::timeout;
use tonic::transport::Channel;

use super::{AiGateway, FinishOutcome, StartOutcome, TurnOutcome};
use crate::error::VoiceError;

#[derive(Clone)]
pub struct GrpcAiGateway {
    pub(crate) client: AiServiceClient<Channel>,
    timeout_ms: u64,
}

impl GrpcAiGateway {
    pub async fn connect(url: &str, timeout_ms: u64) -> Result<Self, VoiceError> {
        let channel = Channel::from_shared(url.to_string())
            .map_err(|e| VoiceError::Upstream(e.to_string()))?
            .connect()
            .await
            .map_err(|e| VoiceError::Upstream(e.to_string()))?;
        Ok(Self {
            client: AiServiceClient::new(channel),
            timeout_ms,
        })
    }

    pub(crate) async fn call<R>(
        &self,
        future: impl std::future::Future<Output = Result<tonic::Response<R>, tonic::Status>>,
    ) -> Result<tonic::Response<R>, VoiceError> {
        timeout(Duration::from_millis(self.timeout_ms), future)
            .await
            .map_err(|_| VoiceError::Upstream("upstream timeout".into()))?
            .map_err(|status| VoiceError::Upstream(status.to_string()))
    }
}

#[async_trait]
impl AiGateway for GrpcAiGateway {
    async fn start(
        &self,
        external_call_id: &str,
        from_number: &str,
        language: Language,
    ) -> Result<StartOutcome, VoiceError> {
        super::rpc::start(self, external_call_id, from_number, language).await
    }

    async fn turn(
        &self,
        session_id: &str,
        transcript: &str,
        correlation_id: &str,
        client_received_at_unix_ms: i64,
    ) -> Result<TurnOutcome, VoiceError> {
        super::rpc::turn(
            self,
            session_id,
            transcript,
            correlation_id,
            client_received_at_unix_ms,
        )
        .await
    }

    async fn finish(&self, session_id: &str, reason: &str) -> Result<FinishOutcome, VoiceError> {
        super::rpc::finish(self, session_id, reason).await
    }
}
