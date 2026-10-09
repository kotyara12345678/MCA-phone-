//! gRPC surface: the `AiService` implementation plus the health server.
//!
//! Handlers are thin delegators; the per-RPC logic lives in `rpc` so this file
//! stays small and errors stay mapped via `AppError::code()` in `support`.

pub mod health;
pub mod rpc;
pub mod support;

use std::sync::Arc;

use mca_proto::ai::v1 as proto;
use mca_proto::ai::v1::ai_service_server::AiService;
use tonic::{Request, Response, Status};

pub use health::HealthGrpc;

use crate::engine::DialogueEngine;

pub struct AiGrpc {
    engine: Arc<DialogueEngine>,
}

impl AiGrpc {
    pub fn new(engine: Arc<DialogueEngine>) -> Self {
        Self { engine }
    }
}

#[tonic::async_trait]
impl AiService for AiGrpc {
    type StreamSessionEventsStream = rpc::FuseStream;

    async fn start_session(
        &self,
        request: Request<proto::StartSessionRequest>,
    ) -> Result<Response<proto::StartSessionResponse>, Status> {
        rpc::start_session(&self.engine, request).await
    }

    async fn process_turn(
        &self,
        request: Request<proto::ProcessTurnRequest>,
    ) -> Result<Response<proto::ProcessTurnResponse>, Status> {
        rpc::process_turn(&self.engine, request).await
    }

    async fn finish_session(
        &self,
        request: Request<proto::FinishSessionRequest>,
    ) -> Result<Response<proto::FinishSessionResponse>, Status> {
        rpc::finish_session(&self.engine, request).await
    }

    async fn get_session(
        &self,
        request: Request<proto::GetSessionRequest>,
    ) -> Result<Response<proto::GetSessionResponse>, Status> {
        rpc::get_session(&self.engine, request).await
    }

    async fn get_application(
        &self,
        request: Request<proto::GetApplicationRequest>,
    ) -> Result<Response<proto::GetApplicationResponse>, Status> {
        rpc::get_application(&self.engine, request).await
    }

    async fn stream_session_events(
        &self,
        request: Request<proto::StreamSessionEventsRequest>,
    ) -> Result<Response<Self::StreamSessionEventsStream>, Status> {
        rpc::stream_session_events(&self.engine, request).await
    }
}
