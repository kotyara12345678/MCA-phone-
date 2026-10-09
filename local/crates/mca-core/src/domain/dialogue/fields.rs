//! Stage → missing-field mapping.

use crate::domain::dialogue::DialogueStage;
use crate::domain::missing::MissingField as F;

/// The field a collecting stage is asking about, or `None` outside the
/// collecting phase.
pub fn collecting_field(stage: DialogueStage) -> Option<F> {
    match stage {
        DialogueStage::CollectingCargo => Some(F::Cargo),
        DialogueStage::CollectingWeight => Some(F::Weight),
        DialogueStage::CollectingVolume => Some(F::Volume),
        DialogueStage::CollectingOrigin => Some(F::Origin),
        DialogueStage::CollectingDestination => Some(F::Destination),
        DialogueStage::CollectingReadyDate => Some(F::ReadyDate),
        DialogueStage::CollectingContact => Some(F::Contact),
        _ => None,
    }
}

pub fn is_terminal(stage: DialogueStage) -> bool {
    matches!(
        stage,
        DialogueStage::Completed | DialogueStage::Rejected | DialogueStage::NeedsHuman
    )
}
