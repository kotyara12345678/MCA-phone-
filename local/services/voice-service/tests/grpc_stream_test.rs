//! E2E over the bidirectional `StreamAudio` path, exercising `GetCall` and
//! `ListCalls` on the way out, with the real ai-service behind it.

mod common_grpc;

use common_grpc::pair;
use futures::StreamExt;
use mca_proto::voice::v1::{
    AudioChunk, GetCallRequest, HangupRequest, ListCallsRequest, OpenCallRequest,
};
use mca_testing::audio::utterance;
use tokio_stream::wrappers::ReceiverStream;

#[tokio::test]
async fn streamed_call_roundtrip() {
    let env = pair().await;
    let mut client = env.client;
    env.stt.expect("нужна доставка груза 120 кг");

    let open = client
        .open_call(OpenCallRequest {
            external_call_id: "ext-2".into(),
            from_number: "+79000000001".into(),
            to_number: "+78000000000".into(),
            language: "ru".into(),
            correlation_id: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    let call = open.call.unwrap();

    let (tx, rx) = tokio::sync::mpsc::channel::<AudioChunk>(8);
    let mut events = client
        .stream_audio(ReceiverStream::new(rx))
        .await
        .unwrap()
        .into_inner();
    tx.send(AudioChunk {
        call_id: call.call_id.clone(),
        audio: utterance("нужна доставка груза 120 кг").to_bytes(),
        format: 1,
        sample_rate: 8_000,
        channels: 1,
        seq: 1,
        sent_at_unix_ms: 0,
        end_of_utterance: true,
    })
    .await
    .unwrap();

    let event = events.message().await.unwrap().unwrap();
    assert_eq!(event.kind, "reply");
    assert!(!event.text.is_empty());
    drop(tx);
    assert!(events.message().await.unwrap().is_none());

    let fetched = client
        .get_call(GetCallRequest {
            call_id: call.call_id.clone(),
        })
        .await
        .unwrap()
        .into_inner()
        .call
        .unwrap();
    assert_eq!(fetched.turn_count, 1);
    assert_eq!(fetched.last_transcript, "нужна доставка груза 120 кг");

    let listed = client
        .list_calls(ListCallsRequest {})
        .await
        .unwrap()
        .into_inner();
    assert!(listed.calls.iter().any(|c| c.call_id == call.call_id));

    client
        .hangup(HangupRequest {
            call_id: call.call_id.clone(),
            reason: "customer_hangup".into(),
            correlation_id: String::new(),
        })
        .await
        .unwrap();
}
