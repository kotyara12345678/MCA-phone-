//! `finish_session`: close the dialogue, hand the lead to the CRM, and return
//! the stored application plus the manager email when one was drafted.

use std::sync::Arc;

use mca_proto::ai::v1 as proto;
use tonic::{Request, Response, Status};

use crate::engine::DialogueEngine;
use crate::grpc::support::{map_err, optional, parse_session};
use crate::map;

pub async fn finish_session(
    engine: &Arc<DialogueEngine>,
    request: Request<proto::FinishSessionRequest>,
) -> Result<Response<proto::FinishSessionResponse>, Status> {
    let input = request.into_inner();
    let session_id = parse_session(&input.session_id)?;
    let outcome = engine
        .finish_session(session_id, optional(&input.reason), None)
        .await
        .map_err(map_err)?;
    Ok(Response::new(proto::FinishSessionResponse {
        session_id: outcome.session.id.to_string(),
        session: Some(map::session_to_proto(&outcome.session)),
        application: Some(map::application_to_proto(&outcome.application)),
        email: outcome.email.as_ref().map(map::email_to_proto),
    }))
}
