use serde::{Deserialize, Serialize};

use super::questions::QuestionTopic;
use crate::domain::missing::MissingField;

/// What the agent decided to say this turn. Computed deterministically; the
/// LLM receives it as a hard constraint and may only rephrase it. This is what
/// makes "never invent a price / company terms" structurally impossible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReplyDirective {
    /// Opening line of the call.
    Greeting,
    /// Acknowledge what we understood, then ask for one field.
    AcknowledgeAndAsk { field: MissingField },
    /// Answer the customer's own question(s) first, then continue collecting.
    AnswerThenAsk {
        topics: Vec<QuestionTopic>,
        field: Option<MissingField>,
    },
    /// Everything mandatory is collected — confirm and finish.
    FinalConfirmation,
    /// Mandatory data is collected but the customer declined a field.
    HandoffToManager { reason: String },
    /// The call is not about transport.
    RejectOutOfScope { reason: String },
    /// An upstream provider failed and no graceful degradation is possible.
    ServiceUnavailable,
    /// Customer said goodbye.
    Closing,
}

impl ReplyDirective {
    /// Whether the directive is allowed to contain a follow-up question.
    pub fn asks_question(&self) -> bool {
        matches!(
            self,
            Self::AcknowledgeAndAsk { .. } | Self::AnswerThenAsk { .. }
        )
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::FinalConfirmation | Self::HandoffToManager { .. } | Self::RejectOutOfScope { .. }
        )
    }

    /// Stable identifier for logs, metrics labels and tests.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Greeting => "greeting",
            Self::AcknowledgeAndAsk { .. } => "acknowledge_and_ask",
            Self::AnswerThenAsk { .. } => "answer_then_ask",
            Self::FinalConfirmation => "final_confirmation",
            Self::HandoffToManager { .. } => "handoff_to_manager",
            Self::RejectOutOfScope { .. } => "reject_out_of_scope",
            Self::ServiceUnavailable => "service_unavailable",
            Self::Closing => "closing",
        }
    }
}
