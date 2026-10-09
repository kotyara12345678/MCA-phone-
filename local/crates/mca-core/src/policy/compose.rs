use super::directive::ReplyDirective;
use super::questions::QuestionTopic;
use crate::domain::missing::MissingField;

/// Deterministic Russian text for a [`ReplyDirective`]. This is the *floor*:
/// whatever the LLM returns, this text is always available as a fallback, so a
/// provider outage degrades quality, never correctness.
pub fn compose(directive: &ReplyDirective) -> String {
    match directive {
        ReplyDirective::Greeting => greeting().to_string(),
        ReplyDirective::AcknowledgeAndAsk { field } => {
            format!("{}. {}", acknowledgement_ru(*field), question_ru(*field))
        }
        ReplyDirective::AnswerThenAsk { topics, field } => answer_then_ask(topics, *field),
        ReplyDirective::FinalConfirmation => final_confirmation(),
        ReplyDirective::HandoffToManager { reason } => handoff(reason),
        ReplyDirective::RejectOutOfScope { reason } => reject(reason),
        ReplyDirective::ServiceUnavailable => {
            "Извините, возникли затруднения. Попробуем продолжить через минуту.".to_string()
        }
        ReplyDirective::Closing => {
            "Спасибо за звонок. Заявка передана менеджеру, мы свяжемся с вами.".to_string()
        }
    }
}

pub fn greeting() -> &'static str {
    concat!(
        "Здравствуйте! Это голосовой ассистент компании MCA Logistics. ",
        "Помогу оформить заявку на перевозку. Подскажите, пожалуйста, что нужно перевезти."
    )
}

/// The question asked for each field. Kept as a table so the wording is
/// consistent, reviewable and testable.
pub fn question_ru(field: MissingField) -> &'static str {
    match field {
        MissingField::Cargo => "Что именно нужно перевезти?",
        MissingField::Weight => "Укажите, пожалуйста, примерный вес груза.",
        MissingField::Volume => "Если известен, укажите объём груза в кубометрах.",
        MissingField::Origin => "Откуда отправляется груз?",
        MissingField::Destination => "Куда необходимо доставить груз?",
        MissingField::ReadyDate => "На какую дату груз готов к отправке?",
        MissingField::Contact => "Как с вами связаться? Укажите имя и телефон.",
    }
}

fn acknowledgement_ru(field: MissingField) -> &'static str {
    match field {
        MissingField::Cargo => "Понял",
        MissingField::Weight => "Понял",
        MissingField::Volume => "Понял",
        MissingField::Origin => "Понял",
        MissingField::Destination => "Понял",
        MissingField::ReadyDate => "Понял",
        MissingField::Contact => "Понял",
    }
}

fn answer_then_ask(topics: &[QuestionTopic], field: Option<MissingField>) -> String {
    let answers: Vec<&str> = topics.iter().map(|t| t.answer_ru()).collect();
    let joined = answers.join(" ");
    match field {
        Some(f) => format!("{joined} {}", question_ru(f)),
        None => joined.to_string(),
    }
}

fn final_confirmation() -> String {
    concat!(
        "Спасибо, все обязательные данные получены. ",
        "Я передаю заявку менеджеру, он свяжется с вами для подтверждения деталей."
    )
    .to_string()
}

fn handoff(reason: &str) -> String {
    format!("{reason} Я передаю вашу заявку менеджеру, он свяжется с вами.")
}

fn reject(reason: &str) -> String {
    format!(
        "К сожалению, я могу помочь только с заявкой на перевозку. {reason} \
         Для остальных вопросов, пожалуйста, свяжитесь с нашим менеджером."
    )
}
