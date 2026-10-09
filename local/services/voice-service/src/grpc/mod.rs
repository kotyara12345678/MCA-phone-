//! gRPC surface: the `VoiceService` implementation plus the health server.
//!
//! Handlers are thin delegators; per-RPC logic lives in `rpc` so this file
//! stays small and errors stay mapped via `VoiceError::code()` in `support`.

pub mod health;
pub mod rpc;
pub mod support;

use std::sync::Arc;

use mca_proto::voice::v1::voice_service_server::VoiceService;
use mca_proto::voice::v1::{
    AudioChunk, GetCallRequest, GetCallResponse, HangupRequest, HangupResponse, ListCallsRequest,
    ListCallsResponse, OpenCallRequest, OpenCallResponse, VoiceEvent,
};
use tonic::{Request, Response, Status};

pub use health::HealthGrpc;

use crate::engine::VoiceEngine;

pub struct VoiceGrpc {
    engine: Arc<VoiceEngine>,
}

impl VoiceGrpc {
    pub fn new(engine: Arc<VoiceEngine>) -> Self {
        Self { engine }
    }
}

#[tonic::async_trait]
impl VoiceService for VoiceGrpc {
    type StreamAudioStream = rpc::FuseStream;

    async fn open_call(
        &self,
        request: Request<OpenCallRequest>,
    ) -> Result<Response<OpenCallResponse>, Status> {
        rpc::open_call(&self.engine, request).await
    }

    async fn push_audio(
        &self,
        request: Request<AudioChunk>,
    ) -> Result<Response<VoiceEvent>, Status> {
        rpc::push_audio(&self.engine, request).await
    }

    async fn hangup(
        &self,
        request: Request<HangupRequest>,
    ) -> Result<Response<HangupResponse>, Status> {
        rpc::hangup(&self.engine, request).await
    }

    async fn get_call(
        &self,
        request: Request<GetCallRequest>,
    ) -> Result<Response<GetCallResponse>, Status> {
        rpc::get_call(&self.engine, request).await
    }

    async fn list_calls(
        &self,
        request: Request<ListCallsRequest>,
    ) -> Result<Response<ListCallsResponse>, Status> {
        rpc::list_calls(&self.engine, request).await
    }

    async fn stream_audio(
        &self,
        request: Request<tonic::Streaming<AudioChunk>>,
    ) -> Result<Response<Self::StreamAudioStream>, Status> {
        rpc::stream_audio(&self.engine, request).await
    }
}
