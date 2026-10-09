//! Tests for the email draft excerpt logic.

use super::*;
use crate::domain::speaker::Speaker;
use crate::ids::MessageId;

fn msg(speaker: Speaker, text: &str) -> Message {
    Message {
        id: MessageId::new(),
        session_id: SessionId::new(),
        speaker,
        text: text.into(),
        created_at: chrono::Utc::now(),
        correlation_id: None,
        processing_latency_ms: 0,
        state_after: None,
    }
}

#[test]
fn excerpt_keeps_the_newest_turns_in_order() {
    let history = vec![
        msg(Speaker::Customer, "первый"),
        msg(Speaker::Agent, "второй"),
        msg(Speaker::Customer, "третий"),
    ];
    let excerpt = EmailDraft::transcript_excerpt(&history, 2);
    assert_eq!(excerpt, "Ассистент: второй\nКлиент: третий");
}

#[test]
fn excerpt_skips_empty_turns() {
    let history = vec![
        msg(Speaker::Customer, "   "),
        msg(Speaker::Customer, "текст"),
    ];
    assert_eq!(EmailDraft::transcript_excerpt(&history, 5), "Клиент: текст");
}
