//! Streaming behavior over the wire: events arrive tagged with the session
//! they belong to. Server setup is shared in `common`.

mod common;
mod common_grpc;

use futures::StreamExt;
use mca_proto::ai::v1::{StartSessionRequest, StreamSessionEventsRequest};

#[tokio::test]
async fn event_stream_is_session_filtered() {
    let mut client = common_grpc::spawn().await;
    let stream = client
        .stream_session_events(StreamSessionEventsRequest { session_id: None })
        .await
        .unwrap()
        .into_inner();
    let mut stream = Box::pin(stream);

    let started = client
        .start_session(StartSessionRequest {
            external_call_id: "ext-stream".into(),
            caller_phone: String::new(),
            language: String::new(),
            correlation_id: String::new(),
            metadata: Default::default(),
        })
        .await
        .unwrap()
        .into_inner();

    let first = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next())
        .await
        .expect("stream stalled")
        .expect("stream ended")
        .expect("rpc error");
    assert_eq!(first.session_id, started.session_id);
    assert!(!first.kind.is_empty());
}
