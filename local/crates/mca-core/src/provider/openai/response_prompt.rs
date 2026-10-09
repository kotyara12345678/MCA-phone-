use crate::ports::llm::ResponseRequest;

/// System prompt for the response call. The key constraint is stated explicitly:
/// the model renders a *given* directive and may not introduce facts.
pub fn response_system_prompt() -> String {
    concat!(
        "You are the voice of a logistics company's AI operator. ",
        "You are given a DIRECTIVE that has already been decided by company rules. ",
        "Rewrite it as one short, natural spoken sentence in Russian (max 2 sentences). ",
        "Hard rules: never invent prices, discounts, terms, schedules or guarantees; ",
        "never add facts not present in the directive; never ask about a field already collected; ",
        "output only the spoken text, no quotes, no markdown, no explanation."
    )
    .to_string()
}

/// User half of the response request. The deterministic baseline is included so
/// the model has something safe to compress rather than an open brief to fill.
pub fn response_user_prompt(request: &ResponseRequest, baseline: &str) -> String {
    let mut prompt = String::with_capacity(320);
    prompt.push_str(&format!("Language: {}.\n", request.language.llm_tag()));
    prompt.push_str(&format!("Directive kind: {}\n", request.directive.as_str()));
    if let Some(question) = requested_question(request) {
        prompt.push_str(&format!("Must ask exactly this: {question}\n"));
    }
    prompt.push_str(&format!(
        "Customer just said: {}\n",
        request.last_customer_text
    ));
    prompt.push_str(&format!("Approved text: {baseline}\n"));
    prompt.push_str("Spoken text:");
    prompt
}

/// The directive already decided which single field to ask about; surfacing the
/// approved wording keeps the model from drifting onto another field.
fn requested_question(request: &ResponseRequest) -> Option<&'static str> {
    use crate::policy::directive::ReplyDirective;
    match &request.directive {
        ReplyDirective::AcknowledgeAndAsk { field } => Some(approved_question(*field)),
        ReplyDirective::AnswerThenAsk {
            field: Some(field), ..
        } => Some(approved_question(*field)),
        _ => None,
    }
}

fn approved_question(field: crate::domain::missing::MissingField) -> &'static str {
    crate::policy::compose::question_ru(field)
}
