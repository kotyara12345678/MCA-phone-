//! Read RPCs: `get_session` for a transcript and `get_application` for the
//! finished lead and its email, straight from the store.

use std::sync::Arc;

use mca_core::ids::ApplicationId;
use mca_proto::ai::v1 as proto;
use tonic::{Request, Response, Status};

use crate::engine::DialogueEngine;
use crate::grpc::support::{invalid_arg, map_err, parse_session};
use crate::map;

pub async fn get_session(
    engine: &Arc<DialogueEngine>,
    request: Request<proto::GetSessionRequest>,
) -> Result<Response<proto::GetSessionResponse>, Status> {
    let input = request.into_inner();
    let session_id = parse_session(&input.session_id)?;
    let limit = input
        .include_messages
        .then(|| input.message_limit.max(0) as usize);
    let (session, messages) = engine
        .get_session(session_id, limit)
        .await
        .map_err(map_err)?;
    Ok(Response::new(proto::GetSessionResponse {
        session: Some(map::session_to_proto(&session)),
        messages: messages.iter().map(map::message_to_proto).collect(),
        state: Some(map::state_to_proto(&session.state)),
    }))
}

pub async fn get_application(
    engine: &Arc<DialogueEngine>,
    request: Request<proto::GetApplicationRequest>,
) -> Result<Response<proto::GetApplicationResponse>, Status> {
    let input = request.into_inner();
    let application_id = input
        .application_id
        .parse::<ApplicationId>()
        .map_err(|err| invalid_arg(err.to_string()))?;
    let (application, email) = engine
        .get_application(application_id)
        .await
        .map_err(map_err)?;
    Ok(Response::new(proto::GetApplicationResponse {
        application: Some(map::application_to_proto(&application)),
        email: email.as_ref().map(map::email_to_proto),
    }))
}
