//! Cross-service flow over real gRPC: open a call, push one scripted
//! utterance, hang up — asserting the ai-service dialogue ran and the
//! telephony fake saw the whole call.

mod common_grpc;

use common_grpc::pair;
use mca_proto::voice::v1::{AudioChunk, HangupRequest, OpenCallRequest};
use mca_testing::audio::utterance;

#[tokio::test]
async fn full_call_roundtrip() {
    let env = pair().await;
    let mut client = env.client;
    env.stt.expect("нужна доставка груза 120 кг");

    let open = client
        .open_call(OpenCallRequest {
            external_call_id: "ext-1".into(),
            from_number: "+79000000000".into(),
            to_number: "+78000000000".into(),
            language: "ru".into(),
            correlation_id: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    let call = open.call.unwrap();
    assert!(call.status == 2, "call should be IN_PROGRESS");
    assert!(!open.greeting_text.is_empty());
    assert!(!open.greeting_audio_base64.is_empty());
    assert_eq!(env.telephony.answered_calls().len(), 1);

    let event = client
        .push_audio(AudioChunk {
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
        .unwrap()
        .into_inner();
    assert_eq!(event.kind, "reply");
    assert!(!event.text.is_empty());
    assert!(!event.audio.is_empty());

    let hangup = client
        .hangup(HangupRequest {
            call_id: call.call_id.clone(),
            reason: "customer_hangup".into(),
            correlation_id: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    let done = hangup.call.unwrap();
    assert_eq!(done.status, 3, "call should be FINISHED");
    assert!(done.turn_count >= 1);
    assert!(!hangup.application_id.is_empty());

    assert_eq!(env.tts.spoken().len(), 2, "greeting + reply synthesised");
    assert_eq!(env.telephony.media_for("ext-1").len(), 2);
    assert_eq!(env.telephony.hangups().len(), 1);
}
