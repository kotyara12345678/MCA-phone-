//! `stream_session_events`: replay live engine events, optionally filtered to
//! one session. Subscribers attach at subscribe time — events already emitted
//! before the call are not replayed.

use std::sync::Arc;

use futures::StreamExt;
use mca_proto::ai::v1 as proto;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status};

use crate::engine::DialogueEngine;
use crate::grpc::rpc::FuseStream;
use crate::map;

pub async fn stream_session_events(
    engine: &Arc<DialogueEngine>,
    request: Request<proto::StreamSessionEventsRequest>,
) -> Result<Response<FuseStream>, Status> {
    let session_id = crate::grpc::support::optional_string(request.into_inner().session_id);
    let stream = BroadcastStream::new(engine.subscribe()).filter_map(move |item| {
        let filter = session_id.clone();
        async move {
            match item {
                Ok(event) if filter.is_none_or(|f| f == event.session_id.to_string()) => {
                    Some(Ok(map::event_to_proto(&event)))
                }
                _ => None,
            }
        }
    });
    Ok(Response::new(Box::pin(stream)))
}
