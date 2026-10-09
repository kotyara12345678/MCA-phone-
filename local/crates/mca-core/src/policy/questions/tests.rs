use super::*;

#[test]
fn detects_price_question() {
    assert!(QuestionTopic::Price.detect("а сколько это будет стоить?"));
}

#[test]
fn detects_several_topics_at_once() {
    let found = QuestionTopic::detect_all("сколько стоит и какие документы нужны?");
    assert!(found.contains(&QuestionTopic::Price));
    assert!(found.contains(&QuestionTopic::Documents));
}

#[test]
fn price_answer_defers_to_manager() {
    assert!(QuestionTopic::Price.answer_ru().contains("менеджер"));
}
