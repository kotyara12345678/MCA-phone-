//! `process_turn`: run one customer utterance through the engine and return
//! the reply, the post-turn state and the latency breakdown.

use std::sync::Arc;

use mca_core::domain::qualification::Qualification;
use mca_proto::ai::v1 as proto;
use tonic::{Request, Response, Status};

use crate::engine::DialogueEngine;
use crate::grpc::support::{map_err, optional_correlation, parse_session, positive};
use crate::map;

pub async fn process_turn(
    engine: &Arc<DialogueEngine>,
    request: Request<proto::ProcessTurnRequest>,
) -> Result<Response<proto::ProcessTurnResponse>, Status> {
    let input = request.into_inner();
    let session_id = parse_session(&input.session_id)?;
    let outcome = engine
        .process_turn(
            session_id,
            input.transcript,
            optional_correlation(&input.correlation_id)?,
            positive(input.client_received_at_unix_ms),
        )
        .await
        .map_err(map_err)?;
    Ok(Response::new(proto::ProcessTurnResponse {
        session_id: outcome.session.id.to_string(),
        reply_text: outcome.reply,
        stage: map::enums::stage(outcome.session.stage) as i32,
        state: Some(map::state_to_proto(&outcome.session.state)),
        finished: !outcome.session.status.is_active(),
        needs_human: outcome.session.state.qualification == Qualification::NeedsHuman,
        latency: Some(map::latency_to_proto(&outcome.latency)),
        agent_message: Some(map::message_to_proto(&outcome.agent_message)),
    }))
}
