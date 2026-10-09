//! `StreamAudio`: bidirectional audio pipeline — chunks in, reply events out.
//! Each chunk is drained through the engine under a lock-free protocol (the
//! engine copies the utterance, then awaits providers without holding the
//! registry lock), so concurrent streams on the same call stay consistent.

use mca_proto::voice::v1::AudioChunk;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::engine::VoiceEngine;
use crate::grpc::rpc::FuseStream;
use crate::grpc::support;
use crate::map;

pub async fn stream_audio(
    engine: &std::sync::Arc<VoiceEngine>,
    request: Request<tonic::Streaming<AudioChunk>>,
) -> Result<Response<FuseStream>, Status> {
    let mut inbound = request.into_inner();
    let (tx, rx) =
        tokio::sync::mpsc::channel::<Result<mca_proto::voice::v1::VoiceEvent, Status>>(64);
    let engine = engine.clone();

    tokio::spawn(async move {
        while let Some(chunk) = inbound.message().await? {
            let call_id = chunk.call_id.clone();
            let event = match engine.push_audio(&chunk).await {
                Ok(Some(outcome)) => map::event(&call_id, &outcome),
                Ok(None) => map::no_reply_event(&call_id, &chunk),
                Err(err) => return Err(support::map_err(err)),
            };
            if tx.send(Ok(event)).await.is_err() {
                break;
            }
        }
        Ok::<(), tonic::Status>(())
    });

    Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
}
