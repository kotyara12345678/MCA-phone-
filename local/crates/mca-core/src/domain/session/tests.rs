use super::*;

#[test]
fn new_session_starts_in_greeting() {
    let s = Session::new(CallId::new(), None, None);
    assert_eq!(s.stage, DialogueStage::Greeting);
    assert!(s.status.is_active());
}

#[test]
fn double_finish_is_rejected() {
    let mut s = Session::new(CallId::new(), None, None);
    assert!(s.mark_finished("hangup").is_ok());
    assert!(s.mark_finished("hangup").is_err());
    assert_eq!(s.status, SessionStatus::Finished);
}

#[test]
fn finished_session_stages_by_qualification() {
    let mut s = Session::new(CallId::new(), None, None);
    s.state.qualification = Qualification::Rejected;
    s.mark_finished("hangup").unwrap();
    assert_eq!(s.stage, DialogueStage::Rejected);
}
