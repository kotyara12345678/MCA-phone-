//! Terminal-state transitions for a session.
//!
//! These mutate the aggregate, so the service never sees a half-finished
//! session: terminal stage, status, end time and finish reason change in one
//! call or all change in none.

use crate::domain::dialogue::DialogueStage;
use crate::domain::qualification::Qualification;
use crate::domain::session::{Session, SessionStatus};

impl Session {
    /// A session may only be finished once; a second call is an error rather
    /// than a silent no-op, so duplicate RPCs surface as conflicts.
    pub fn mark_finished(&mut self, reason: impl Into<String>) -> Result<(), String> {
        if !self.status.is_active() {
            return Err(format!(
                "session {} is already {}",
                self.id,
                self.status.as_str()
            ));
        }
        self.status = SessionStatus::Finished;
        self.ended_at = Some(chrono::Utc::now());
        self.finish_reason = Some(reason.into());
        self.stage = terminal_stage(self.state.qualification);
        Ok(())
    }
}

fn terminal_stage(qualification: Qualification) -> DialogueStage {
    match qualification {
        Qualification::Qualified => DialogueStage::Qualified,
        Qualification::Rejected => DialogueStage::Rejected,
        _ => DialogueStage::Completed,
    }
}
