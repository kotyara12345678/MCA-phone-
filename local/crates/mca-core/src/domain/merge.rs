use crate::domain::application::ApplicationState;
use crate::nlu::result::ExtractionResult;

/// Applies one turn's extraction to the accumulated state.
///
/// Rules:
/// * `Some(value)` overwrites — this is what allows a customer to *change* the
///   destination mid-call;
/// * `None` is a no-op — an LLM omission can never erase collected data;
/// * the intent escalates monotonically (Greeting → Shipping → PriceInquiry)
///   so a later "а сколько стоит?" does not demote a shipping request.
pub fn apply(state: &mut ApplicationState, extraction: &ExtractionResult) {
    overwrite(&mut state.cargo, &extraction.cargo);
    overwrite_number(&mut state.weight_kg, extraction.weight_kg);
    overwrite_number(&mut state.volume_m3, extraction.volume_m3);
    overwrite(&mut state.origin_country, &extraction.origin_country);
    overwrite(&mut state.origin_city, &extraction.origin_city);
    overwrite(
        &mut state.destination_country,
        &extraction.destination_country,
    );
    overwrite(&mut state.destination_city, &extraction.destination_city);
    overwrite(&mut state.ready_date, &extraction.ready_date);
    overwrite(&mut state.contact, &extraction.contact);
    overwrite(
        &mut state.special_requirements,
        &extraction.special_requirements,
    );

    for field in &extraction.unknown_fields {
        if !state.unknown_fields.contains(field) {
            state.unknown_fields.push(*field);
        }
    }
    state.intent = escalate(state.intent, extraction.intent);
    if extraction.language != crate::domain::language::Language::Ru
        || state.language == crate::domain::language::Language::Ru
    {
        state.language = extraction.language;
    }
    state.refresh();
}

fn overwrite(slot: &mut Option<String>, incoming: &Option<String>) {
    if let Some(value) = incoming {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            *slot = Some(trimmed.to_string());
        }
    }
}

fn overwrite_number(slot: &mut Option<f64>, incoming: Option<f64>) {
    if let Some(value) = incoming {
        if value.is_finite() && value > 0.0 {
            *slot = Some(value);
        }
    }
}

/// Intent escalation order. Tracking and Unrelated are terminal and never
/// overwritten by a later softer intent.
fn escalate(
    current: crate::domain::intent::Intent,
    incoming: crate::domain::intent::Intent,
) -> crate::domain::intent::Intent {
    use crate::domain::intent::Intent as I;
    if current == I::Unrelated || current == I::Tracking {
        return current;
    }
    if incoming == I::Unrelated || incoming == I::Tracking {
        return incoming;
    }
    if rank(incoming) > rank(current) {
        return incoming;
    }
    current
}

fn rank(intent: crate::domain::intent::Intent) -> u8 {
    use crate::domain::intent::Intent as I;
    match intent {
        I::Unknown => 0,
        I::Greeting => 1,
        I::PriceInquiry => 2,
        I::ShippingRequest => 3,
        I::Tracking | I::Unrelated => 4,
    }
}

#[cfg(test)]
mod tests;
