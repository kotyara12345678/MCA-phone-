//! Derives the stored `DialogueStage` from the deterministic directive.
//!
//! The LLM never picks this value; it is a pure function of the policy verdict
//! and the next missing field, which keeps the persisted FSM reproducible from
//! a transcript replay.

use mca_core::domain::application::ApplicationState;
use mca_core::domain::dialogue::DialogueStage;
use mca_core::domain::missing::MissingField;
use mca_core::domain::qualification::Qualification;
use mca_core::policy::directive::ReplyDirective;

/// The stage a session is in after this directive is emitted.
pub fn of(state: &ApplicationState, directive: &ReplyDirective) -> DialogueStage {
    match directive {
        ReplyDirective::Greeting => DialogueStage::Greeting,
        ReplyDirective::Closing | ReplyDirective::ServiceUnavailable => DialogueStage::Completed,
        ReplyDirective::RejectOutOfScope { .. } => DialogueStage::Rejected,
        ReplyDirective::HandoffToManager { .. } => DialogueStage::NeedsHuman,
        ReplyDirective::FinalConfirmation => by_qualification(state.qualification),
        ReplyDirective::AcknowledgeAndAsk { field }
        | ReplyDirective::AnswerThenAsk {
            field: Some(field),
            topics: _,
        } => stage_for(*field),
        ReplyDirective::AnswerThenAsk {
            field: None,
            topics: _,
        } => stage_for_none(state),
    }
}

fn by_qualification(qualification: Qualification) -> DialogueStage {
    match qualification {
        Qualification::Qualified => DialogueStage::Qualified,
        Qualification::Rejected => DialogueStage::Rejected,
        Qualification::NeedsHuman => DialogueStage::NeedsHuman,
        Qualification::Pending => DialogueStage::ReadyForQualification,
    }
}

fn stage_for(field: MissingField) -> DialogueStage {
    match field {
        MissingField::Cargo => DialogueStage::CollectingCargo,
        MissingField::Weight => DialogueStage::CollectingWeight,
        MissingField::Volume => DialogueStage::CollectingVolume,
        MissingField::Origin => DialogueStage::CollectingOrigin,
        MissingField::Destination => DialogueStage::CollectingDestination,
        MissingField::ReadyDate => DialogueStage::CollectingReadyDate,
        MissingField::Contact => DialogueStage::CollectingContact,
    }
}

fn stage_for_none(state: &ApplicationState) -> DialogueStage {
    by_qualification(state.qualification)
}
