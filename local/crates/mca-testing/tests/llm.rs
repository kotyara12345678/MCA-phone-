use mca_core::domain::application::ApplicationState;
use mca_core::domain::language::Language;
use mca_core::nlu::result::ExtractionResult;
use mca_core::policy::directive::ReplyDirective;
use mca_core::ports::llm::{ExtractionRequest, LlmProvider, ResponseRequest};
use mca_testing::FakeLlmProvider;

fn extraction_request(utterance: &str) -> ExtractionRequest {
    ExtractionRequest {
        utterance: utterance.to_string(),
        current_state: ApplicationState::default(),
        recent_turns: Vec::new(),
        last_agent_question: None,
        language: Language::Ru,
    }
}

fn response_request(text: &str) -> ResponseRequest {
    ResponseRequest {
        directive: ReplyDirective::Greeting,
        current_state: ApplicationState::default(),
        last_customer_text: text.to_string(),
        recent_turns: Vec::new(),
        language: Language::Ru,
    }
}

#[tokio::test]
async fn fake_llm_falls_back_to_deterministic_rules() {
    let llm = FakeLlmProvider::default();
    let extraction = llm
        .extract(extraction_request("привет, хочу перевезти 200 кг в Алматы"))
        .await
        .unwrap();
    assert_eq!(extraction.weight_kg, Some(200.0));

    let draft = llm.respond(response_request("привет")).await.unwrap();
    assert!(!draft.text.is_empty());
}

#[tokio::test]
async fn scripted_llm_wins_over_rules() {
    let llm = FakeLlmProvider::default();
    llm.expect_extract(ExtractionResult {
        destination_city: Some("Чимкент".to_string()),
        ..ExtractionResult::default()
    });
    llm.expect_respond("специальный ответ");

    let extraction = llm.extract(extraction_request("что угодно")).await.unwrap();
    assert_eq!(extraction.destination_city.as_deref(), Some("Чимкент"));

    let draft = llm.respond(response_request("что угодно")).await.unwrap();
    assert_eq!(draft.text, "специальный ответ");
}
