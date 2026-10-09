//! Remaining engine flows: the idempotency guard and the emitted event stream.
//! Use the same in-memory, deterministic-fake fixture as `engine_test.rs`.

mod common;

use mca_core::error::AppError;

#[tokio::test]
async fn finish_is_idempotent_guard() {
    let (engine, _) = common::fixture();
    let started = engine.start_session(None, None, None, None).await.unwrap();
    engine
        .finish_session(started.session.id, None, None)
        .await
        .unwrap();
    let second = engine.finish_session(started.session.id, None, None).await;
    assert!(matches!(second, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn stream_receives_start_and_turn_events() {
    let (engine, _) = common::fixture();
    let mut events = engine.subscribe();
    let started = engine.start_session(None, None, None, None).await.unwrap();
    engine
        .process_turn(started.session.id, "привет".into(), None, None)
        .await
        .unwrap();

    let mut kinds = Vec::new();
    while let Ok(received) =
        tokio::time::timeout(std::time::Duration::from_secs(2), events.recv()).await
    {
        let event = received.unwrap();
        if event.session_id != started.session.id {
            continue;
        }
        kinds.push(event.kind);
        if kinds.len() >= 2 {
            break;
        }
    }
    assert_eq!(kinds, vec!["session_started", "message"]);
}
