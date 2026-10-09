//! `start_session`: bind the call, pick the language, and hand the engine a
//! fresh session with the greeting to play.

use std::sync::Arc;

use mca_proto::ai::v1 as proto;
use tonic::{Request, Response, Status};

use crate::engine::DialogueEngine;
use crate::grpc::support::{invalid_arg, map_err, optional, optional_correlation};
use crate::map;

pub async fn start_session(
    engine: &Arc<DialogueEngine>,
    request: Request<proto::StartSessionRequest>,
) -> Result<Response<proto::StartSessionResponse>, Status> {
    let input = request.into_inner();
    let language = map::enums::parse_language(&input.language).map_err(invalid_arg)?;
    let outcome = engine
        .start_session(
            optional(&input.external_call_id),
            optional(&input.caller_phone),
            Some(language),
            optional_correlation(&input.correlation_id)?,
        )
        .await
        .map_err(map_err)?;
    Ok(Response::new(proto::StartSessionResponse {
        session_id: outcome.session.id.to_string(),
        call_id: outcome.session.call_id.to_string(),
        correlation_id: input.correlation_id,
        started_at_unix_ms: outcome.session.started_at.timestamp_millis(),
        state: Some(map::state_to_proto(&outcome.session.state)),
        greeting: outcome.greeting,
        stage: map::enums::stage(outcome.session.stage) as i32,
    }))
}
