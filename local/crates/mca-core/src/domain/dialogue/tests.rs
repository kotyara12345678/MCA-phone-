use super::*;
use crate::domain::missing::MissingField;

#[test]
fn collecting_stages_map_to_fields() {
    assert_eq!(
        DialogueStage::CollectingWeight.collecting_field(),
        Some(MissingField::Weight)
    );
    assert_eq!(DialogueStage::Greeting.collecting_field(), None);
}

#[test]
fn terminal_stages() {
    assert!(DialogueStage::Completed.is_terminal());
    assert!(DialogueStage::NeedsHuman.is_terminal());
    assert!(!DialogueStage::CollectingCargo.is_terminal());
}

#[test]
fn stage_roundtrips_through_str() {
    for stage in DialogueStage::ALL {
        assert_eq!(DialogueStage::from_str(stage.as_str()).unwrap(), stage);
    }
}
