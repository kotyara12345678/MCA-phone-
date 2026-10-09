use crate::domain::application::ApplicationState;
use crate::domain::intent::Intent;
use crate::domain::missing::{order_missing, MissingField};
use crate::domain::qualification::QualificationVerdict;

use super::reasons;

/// The single authoritative qualification decision for a state snapshot.
///
/// Evaluation order is fixed and deterministic:
/// 1. out-of-scope intent        → Rejected
/// 2. tracking request          → NeedsHuman (needs a carrier system)
/// 3. no request signal yet     → Pending
/// 4. mandatory fields missing  → Pending
/// 5. customer refused a field  → NeedsHuman
/// 6. otherwise                 → Qualified
///
/// Only validated fields and the extracted intent are read, so the LLM can
/// never talk its way into a business decision.
pub fn evaluate(state: &ApplicationState) -> QualificationVerdict {
    if state.intent.is_out_of_scope() {
        return QualificationVerdict::rejected(reasons::out_of_scope(state.intent));
    }
    if state.intent == Intent::Tracking {
        return QualificationVerdict::needs_human(reasons::tracking());
    }
    if !has_request_signal(state) {
        return QualificationVerdict::pending(reasons::no_signal());
    }

    let missing = order_missing(&state.missing_fields);
    let blocking: Vec<MissingField> = missing
        .iter()
        .copied()
        .filter(|f| f.is_mandatory())
        .collect();
    if !blocking.is_empty() {
        return QualificationVerdict::pending(reasons::missing_mandatory(&blocking));
    }
    if let Some(field) = refused_field(state, &missing) {
        return QualificationVerdict::needs_human(reasons::declined(field));
    }
    QualificationVerdict::qualified(reasons::qualified(state))
}

/// True once the customer has said anything that looks like a transport
/// request. Greetings and small talk alone must stay `Pending`, never
/// `Rejected`, so the agent keeps listening.
pub fn has_request_signal(state: &ApplicationState) -> bool {
    state.intent == Intent::ShippingRequest
        || state.intent == Intent::PriceInquiry
        || state.cargo.is_some()
        || state.origin_country.is_some()
        || state.origin_city.is_some()
        || state.destination_country.is_some()
        || state.destination_city.is_some()
        || state.weight_kg.is_some()
        || state.volume_m3.is_some()
}

/// The customer explicitly refused a field ("не знаю") → stop asking, escalate.
pub fn refused_field(state: &ApplicationState, missing: &[MissingField]) -> Option<MissingField> {
    missing
        .iter()
        .copied()
        .find(|f| state.unknown_fields.contains(f))
}
