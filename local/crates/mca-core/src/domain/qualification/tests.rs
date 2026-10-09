use super::*;

#[test]
fn only_pending_is_open() {
    assert!(!Qualification::Pending.is_final());
    assert!(Qualification::Qualified.is_final());
    assert!(Qualification::Rejected.is_final());
}

#[test]
fn handover_generated_for_qualified_and_pending() {
    assert!(Qualification::Qualified.produces_handover());
    assert!(Qualification::Pending.produces_handover());
    assert!(!Qualification::Rejected.produces_handover());
}
