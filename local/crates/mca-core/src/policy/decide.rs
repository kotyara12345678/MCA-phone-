use super::directive::ReplyDirective;
use super::questions::QuestionTopic;
use crate::domain::application::ApplicationState;
use crate::domain::intent::Intent;
use crate::domain::missing::order_missing;
use crate::domain::qualification::Qualification;

/// The decision core of the dialogue engine. Pure function: given the state
/// before the turn, the extracted intent and the topics the customer asked
/// about, it returns what the agent should do next.
///
/// Nothing here touches a model, a clock or a socket — which is exactly why the
/// "don't ask twice" and "never hallucinate a price" guarantees are testable.
pub fn decide(
    state: &ApplicationState,
    new_topics: &[QuestionTopic],
    first_turn: bool,
    customer_said_goodbye: bool,
) -> ReplyDirective {
    if customer_said_goodbye {
        return ReplyDirective::Closing;
    }
    if state.intent.is_out_of_scope() {
        let reason = state
            .qualification_reason
            .clone()
            .unwrap_or_else(|| "Обращение не относится к перевозкам.".to_string());
        return ReplyDirective::RejectOutOfScope { reason };
    }
    if first_turn && state.turn_is_greeting_only() {
        return ReplyDirective::Greeting;
    }
    if state.qualification == Qualification::NeedsHuman && state.has_all_mandatory() {
        let reason = state
            .qualification_reason
            .clone()
            .unwrap_or_else(|| "Требуется уточнение менеджера.".to_string());
        return ReplyDirective::HandoffToManager { reason };
    }
    if state.has_all_mandatory() && state.intent == Intent::ShippingRequest {
        return ReplyDirective::FinalConfirmation;
    }
    match next_field(state) {
        Some(field) if !new_topics.is_empty() => ReplyDirective::AnswerThenAsk {
            topics: new_topics.to_vec(),
            field: Some(field),
        },
        Some(field) => ReplyDirective::AcknowledgeAndAsk { field },
        None => ReplyDirective::FinalConfirmation,
    }
}

/// Picks the single most valuable missing field to ask about, or `None` when
/// nothing mandatory is left. Optional fields (volume, contact) are only asked
/// once the mandatory set is complete, and never twice.
pub fn next_field(state: &ApplicationState) -> Option<crate::domain::missing::MissingField> {
    let ordered = order_missing(&state.missing_fields);
    let declined: Vec<_> = ordered
        .iter()
        .filter(|f| state.unknown_fields.contains(f))
        .collect();
    if let Some(first) = declined.first() {
        return Some(**first);
    }
    ordered
        .iter()
        .find(|f| f.is_mandatory())
        .or_else(|| ordered.first())
        .copied()
}

impl ApplicationState {
    /// Helper kept next to the decision core: a greeting-only first turn has no
    /// fields and no shipping signal.
    fn turn_is_greeting_only(&self) -> bool {
        self.cargo.is_none()
            && self.weight_kg.is_none()
            && self.origin_country.is_none()
            && self.destination_country.is_none()
    }
}
