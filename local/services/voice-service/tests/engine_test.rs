//! Engine-level behaviour over the deterministic providers: unknown calls
//! are rejected and silence yields no reply.

mod common;

use common::{chunk, test_engine};
use mca_proto::voice::v1::OpenCallRequest;

#[tokio::test]
async fn push_rejects_unknown_call() {
    let engine = test_engine();
    let err = engine
        .push_audio(&chunk("missing", Vec::new(), true))
        .await
        .unwrap_err();
    assert_eq!(err.code(), "not_found");
}

#[tokio::test]
async fn open_then_silence_yields_no_reply() {
    let engine = test_engine();
    let open = engine
        .open(&OpenCallRequest {
            external_call_id: "ext-1".into(),
            from_number: "+79000000000".into(),
            to_number: "+78000000000".into(),
            language: "ru".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    let got = engine
        .push_audio(&chunk(
            &open.record.call_id,
            mca_testing::audio::silence(8_000).to_bytes(),
            true,
        ))
        .await
        .unwrap();
    assert!(got.is_none());
}

#[tokio::test]
async fn hangup_rejects_unknown_call() {
    let engine = test_engine();
    let err = engine.hangup("missing", "test").await.unwrap_err();
    assert_eq!(err.code(), "not_found");
}
