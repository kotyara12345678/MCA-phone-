//! `PushAudio`: one unary audio chunk; a reply event only materialises when
//! the utterance actually ended and produced text.

use std::sync::Arc;

use mca_proto::voice::v1::{AudioChunk, VoiceEvent};
use tonic::{Request, Response, Status};

use crate::engine::VoiceEngine;
use crate::grpc::support;
use crate::map;

pub async fn push_audio(
    engine: &Arc<VoiceEngine>,
    request: Request<AudioChunk>,
) -> Result<Response<VoiceEvent>, Status> {
    let chunk = request.into_inner();
    let call_id = chunk.call_id.clone();
    match engine.push_audio(&chunk).await {
        Ok(Some(outcome)) => Ok(Response::new(map::event(&call_id, &outcome))),
        Ok(None) => Ok(Response::new(map::no_reply_event(&call_id, &chunk))),
        Err(err) => Err(support::map_err(err)),
    }
}
