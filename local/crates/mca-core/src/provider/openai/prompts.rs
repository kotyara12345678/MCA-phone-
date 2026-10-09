use crate::domain::language::Language;
use crate::nlu::result::ExtractionResult;
use crate::policy::company::COMPANY_NAME;

/// System prompt for the extraction call. Short on purpose: cheap models, low
/// tokens, and the schema is enforced by JSON mode plus `deny_unknown_fields`.
pub fn extraction_system_prompt() -> String {
    format!(
        "You are a strict information extractor for a logistics voice agent of {COMPANY_NAME}.\n\
         Return ONLY a JSON object with these optional fields:\n\
         cargo (string), weight_kg (number, kilograms), volume_m3 (number),\n\
         origin_country (string, English name), origin_city (string, English name),\n\
         destination_country (string, English name), destination_city (string, English name),\n\
         ready_date (string, ISO date YYYY-MM-DD), contact (string as spoken),\n\
         special_requirements (string), intent (one of: unknown, greeting, shipping_request,\n\
         price_inquiry, tracking, unrelated), unknown_fields (array of: cargo, weight, volume,\n\
         origin, destination, ready_date, contact), hangup_requested (bool), greeting_only (bool).\n\
         Rules: never invent values; use null when the customer did not say it.\n\
         Never answer questions, never quote prices, never decide qualification."
    )
}

/// The user half of the extraction request. Only a compact window of the
/// conversation is included, plus the structured state, so token cost stays
/// proportional to the turn rather than the call.
pub fn extraction_user_prompt(
    utterance: &str,
    state_json: &str,
    recent_turns: &[String],
    last_question: Option<&str>,
    language: Language,
) -> String {
    let mut prompt = String::with_capacity(utterance.len() + state_json.len() + 256);
    prompt.push_str(&format!("Language: {}.\n", language.llm_tag()));
    prompt.push_str(&format!("Current structured state (JSON): {state_json}\n"));
    if let Some(question) = last_question {
        prompt.push_str(&format!("Last question the agent asked: {question}\n"));
    }
    if !recent_turns.is_empty() {
        prompt.push_str("Recent dialogue:\n");
        for turn in recent_turns {
            prompt.push_str(turn);
            prompt.push('\n');
        }
    }
    prompt.push_str(&format!("Customer utterance: {utterance}\n"));
    prompt.push_str("JSON:");
    prompt
}

/// Guards against a model that returns prose or an empty string where JSON is
/// required. Never panics; a failure here is handled by the fallback path.
pub fn parse_extraction(raw: &str) -> Result<ExtractionResult, String> {
    let json = extract_json_object(raw).ok_or_else(|| "no json object in response".to_string())?;
    serde_json::from_str::<ExtractionResult>(&json).map_err(|e| e.to_string())
}

/// Finds the outermost `{...}` span, tolerating markdown fences and prose.
pub fn extract_json_object(raw: &str) -> Option<String> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(raw[start..=end].to_string())
}
