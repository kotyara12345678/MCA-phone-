//! `Hangup`: finish the ai-service session, hang up the telephony call, and
//! report the application that came out of the dialogue.

use std::sync::Arc;

use mca_proto::voice::v1::{HangupRequest, HangupResponse};
use tonic::{Request, Response, Status};

use crate::engine::VoiceEngine;
use crate::grpc::support;
use crate::map;

pub async fn hangup(
    engine: &Arc<VoiceEngine>,
    request: Request<HangupRequest>,
) -> Result<Response<HangupResponse>, Status> {
    let call_id = request.get_ref().call_id.clone();
    let reason = request.get_ref().reason.clone();
    let (record, outcome) = engine
        .hangup(&call_id, &reason)
        .await
        .map_err(support::map_err)?;
    let response = HangupResponse {
        call: Some(map::call(&record)),
        application_id: outcome.application_id,
        qualification: outcome.qualification,
    };
    Ok(Response::new(response))
}
