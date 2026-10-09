//! `process_turn`: one customer utterance through extract → merge → decide →
//! respond, persisted with the post-turn state snapshot on both messages.
//! Provider calls live in `reason.rs`; durable output in `output.rs`.

use std::time::Instant;

use mca_core::domain::latency::LatencySample;
use mca_core::domain::merge;
use mca_core::domain::speaker::Speaker;
use mca_core::error::AppError;
use mca_core::ids::{CorrelationId, SessionId};
use mca_core::nlu::intent::is_hangup;
use mca_core::policy::decide;
use mca_core::policy::questions::QuestionTopic;

use super::{recent_turns, DialogueEngine, TurnOutcome};

impl DialogueEngine {
    pub async fn process_turn(
        &self,
        session_id: SessionId,
        transcript: String,
        correlation_id: Option<CorrelationId>,
        client_received_at_unix_ms: Option<i64>,
    ) -> Result<TurnOutcome, AppError> {
        let trimmed = transcript.trim().to_string();
        if trimmed.is_empty() {
            return Err(AppError::InvalidRequest("transcript is empty".into()));
        }
        if trimmed.chars().count() > self.max_transcript_chars {
            return Err(AppError::PayloadTooLarge("transcript too large".into()));
        }

        let mut session = self
            .sessions
            .load_session(session_id)
            .await?
            .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;
        if !session.status.is_active() {
            return Err(AppError::SessionNotActive(session_id.to_string()));
        }

        let all = self.sessions.load_messages(session_id, None).await?;
        let recent = recent_turns(&all, self.context_recent_turns.max(1) * 2);
        let last_agent_question = all
            .iter()
            .rev()
            .find(|m| m.speaker == Speaker::Agent)
            .map(|m| m.text.clone());

        let started = Instant::now();
        let extraction = self
            .extract_from(&session.state, &trimmed, &recent, last_agent_question)
            .await?;
        let llm_ms = started.elapsed().as_millis() as i64;

        merge::apply(&mut session.state, &extraction);

        let directive = decide::decide(
            &session.state,
            &QuestionTopic::detect_all(&trimmed),
            session.turn_count == 0,
            is_hangup(&trimmed),
        );
        let response = self
            .answer_with(&directive, &session.state, &trimmed, &recent)
            .await?;

        let total_ms = super::reason::total_latency(llm_ms, client_received_at_unix_ms);
        let latency = LatencySample {
            stt_ms: 0,
            llm_ms,
            tts_ms: 0,
            total_ms,
            first_token_ms: llm_ms,
        };
        session.turn_count += 1;
        session.stage = super::stage::of(&session.state, &directive);
        session.latency = latency;

        let agent = self
            .persist_messages(
                &session,
                &trimmed,
                correlation_id,
                &response,
                llm_ms,
                total_ms,
            )
            .await?;
        self.emit_turn(&session, &agent.text, llm_ms, total_ms);

        Ok(TurnOutcome {
            session,
            reply: agent.text.clone(),
            agent_message: agent,
            latency,
        })
    }
}
