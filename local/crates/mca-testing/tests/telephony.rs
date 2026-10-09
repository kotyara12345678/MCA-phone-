use mca_core::error::ProviderError;
use mca_core::ports::stt::AudioFormat;
use mca_core::ports::telephony::{InboundCall, OutboundMedia, TelephonyProvider};
use mca_testing::FakeTelephonyProvider;

fn inbound_call(id: &str) -> InboundCall {
    InboundCall {
        external_call_id: id.to_string(),
        from_number: None,
        to_number: None,
        language: None,
    }
}

#[tokio::test]
async fn telephony_records_media_and_hangups() {
    let tel = FakeTelephonyProvider::new();
    let call = inbound_call("call-1");
    let handle = tel.answer(call.clone()).await.unwrap();
    tel.send_media(
        &handle,
        OutboundMedia {
            external_call_id: "call-1".to_string(),
            seq: 0,
            audio: vec![1, 2],
            format: AudioFormat::PcmS16Le,
        },
    )
    .await
    .unwrap();
    tel.hangup(&handle, "completed").await.unwrap();

    assert_eq!(tel.answered_calls(), vec![call]);
    assert_eq!(tel.media_for("call-1").len(), 1);
    assert_eq!(
        tel.hangups(),
        vec![("call-1".to_string(), "completed".to_string())]
    );
}

#[tokio::test]
async fn telephony_media_failure_is_one_shot() {
    let tel = FakeTelephonyProvider::new();
    tel.fail_media(ProviderError::NotConfigured {
        provider: "fake_telephony",
    });
    let handle = tel.answer(inbound_call("call-2")).await.unwrap();
    let frame = OutboundMedia {
        external_call_id: "call-2".to_string(),
        seq: 0,
        audio: vec![7],
        format: AudioFormat::PcmS16Le,
    };
    assert!(tel.send_media(&handle, frame.clone()).await.is_err());
    assert!(tel.send_media(&handle, frame).await.is_ok());
    assert_eq!(tel.media_for("call-2").len(), 1);
}
