//! Transport-level test: a real tonic gRPC roundtrip over an ephemeral port,
//! proving proto mapping and the application handover. The streaming path
//! lives in `grpc_stream_test.rs`; the server setup in `common`.

mod common;
mod common_grpc;

use mca_proto::ai::v1::{
    FinishSessionRequest, GetSessionRequest, ProcessTurnRequest, StartSessionRequest,
};

#[tokio::test]
async fn start_turn_get_finish_roundtrip() {
    let mut client = common_grpc::spawn().await;
    let started = client
        .start_session(StartSessionRequest {
            external_call_id: "ext-call-9".into(),
            caller_phone: "+70001234567".into(),
            language: "ru".into(),
            correlation_id: String::new(),
            metadata: Default::default(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(!started.session_id.is_empty());
    assert!(started.state.is_some());
    assert!(started.greeting.contains("MCA Logistics"));

    let turned = client
        .process_turn(ProcessTurnRequest {
            session_id: started.session_id.clone(),
            transcript: "нужно отправить документацию тонн три из Германии в Санкт-Петербург"
                .into(),
            correlation_id: String::new(),
            client_received_at_unix_ms: 0,
            is_final: true,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(turned.session_id, started.session_id);
    assert!(!turned.reply_text.is_empty());
    assert!(turned.agent_message.is_some());

    let fetched = client
        .get_session(GetSessionRequest {
            session_id: started.session_id.clone(),
            include_messages: true,
            message_limit: 10,
        })
        .await
        .unwrap()
        .into_inner();
    assert!(fetched.session.is_some());
    assert!(fetched.messages.len() >= 2);

    let finished = client
        .finish_session(FinishSessionRequest {
            session_id: started.session_id.clone(),
            reason: "test".into(),
            correlation_id: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(finished.application.is_some());
    assert!(finished.email.is_some());
}
