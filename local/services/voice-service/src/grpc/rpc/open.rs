//! `OpenCall`: handshake with ai-service, answer, greet.

use std::sync::Arc;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use mca_proto::voice::v1::{OpenCallRequest, OpenCallResponse};
use tonic::{Request, Response, Status};

use crate::engine::VoiceEngine;
use crate::grpc::support;
use crate::map;

pub async fn open_call(
    engine: &Arc<VoiceEngine>,
    request: Request<OpenCallRequest>,
) -> Result<Response<OpenCallResponse>, Status> {
    let outcome = engine
        .open(request.get_ref())
        .await
        .map_err(support::map_err)?;
    let response = OpenCallResponse {
        call: Some(map::call(&outcome.record)),
        greeting_audio_base64: BASE64_STANDARD.encode(outcome.greeting_audio),
        greeting_text: outcome.greeting_text,
    };
    Ok(Response::new(response))
}
