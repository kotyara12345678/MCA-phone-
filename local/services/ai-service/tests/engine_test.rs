//! Engine-level E2E against the in-memory store and the deterministic fakes:
//! one full start → turn → finish cycle plus the negative guard. Other flows
//! live in `engine_flow_test.rs`; wiring in `common`.

mod common;

use ai_service::store::ApplicationStore;
use mca_core::domain::intent::Intent;
use mca_core::error::AppError;

#[tokio::test]
async fn full_call_creates_application_and_email() {
    let (engine, store) = common::fixture();
    let started = engine
        .start_session(Some("call-1".into()), Some("+777.000".into()), None, None)
        .await
        .unwrap();
    assert!(!started.greeting.is_empty());
    assert!(started.session.status.is_active());

    let turn = engine
        .process_turn(
            started.session.id,
            "Здравствуйте, нужно перевезти оборудование весом 500 кг из Германии в Алматы".into(),
            None,
            None,
        )
        .await
        .unwrap();
    assert!(turn.session.state.intent == Intent::ShippingRequest);
    assert!(!turn.reply.is_empty());

    engine
        .process_turn(
            started.session.id,
            "Готов завтра, мой телефон +777 123-45-67".into(),
            None,
            None,
        )
        .await
        .unwrap();

    let finished = engine
        .finish_session(started.session.id, Some("done".into()), None)
        .await
        .unwrap();
    assert!(!finished.session.status.is_active());
    assert!(finished.session.application_id.is_some());
    assert!(finished.email.is_some());
    assert!(finished.lead_accepted);
    let stored_app = store
        .load_application(finished.application.id)
        .await
        .unwrap()
        .expect("application persisted");
    assert_eq!(stored_app.session_id, started.session.id);

    let email = store
        .load_email(finished.application.id)
        .await
        .unwrap()
        .expect("email persisted");
    assert!(email.subject.contains("Новая заявка"));
}

#[tokio::test]
async fn unknown_session_is_rejected_early() {
    let (engine, _) = common::fixture();
    let result = engine
        .process_turn(mca_core::ids::SessionId::new(), "тест".into(), None, None)
        .await;
    assert!(matches!(result, Err(AppError::SessionNotFound(_))));
}
