use super::{
    contact, direction, normalize, place, quantity, ready_date, result::ExtractionResult, volume,
};
use crate::domain::language::Language;
use crate::nlu::lexicon::{cargo as cargo_lexicon, sanitize_fragment};

/// Deterministic field extraction from a single customer utterance.
///
/// This is the offline engine (used by `FakeLlmProvider` and by `mock` mode)
/// **and** the production fallback whenever the real LLM is unavailable, which
/// is why it is deliberately thorough: a provider outage must degrade the tone
/// of the conversation, never the amount of data collected.
pub fn extract(text: &str) -> ExtractionResult {
    let raw = text.trim();
    let n = normalize::normalize(raw);
    let (origin_seg, dest_seg) = direction::split_directions(&n);
    let origin = place::detect_place(&origin_seg);
    let destination = place::detect_place(&dest_seg);
    let directed = !origin_seg.is_empty() || !dest_seg.is_empty();
    let out_of_band = place::detect_place(&n);

    ExtractionResult {
        intent: super::intent::detect_intent(&n, &out_of_band),
        language: Language::detect(raw),
        cargo: cargo_lexicon::canonical_cargo(&n),
        weight_kg: quantity::extract_weight_kg(&n),
        volume_m3: volume::extract_volume_m3(&n),
        origin_country: origin.country.map(str::to_string),
        origin_city: origin.city.map(str::to_string),
        destination_country: if directed {
            destination.country.map(str::to_string)
        } else {
            out_of_band.country.map(str::to_string)
        },
        destination_city: if directed {
            destination.city.map(str::to_string)
        } else {
            out_of_band.city.map(str::to_string)
        },
        ready_date: ready_date::extract_ready_date(&n),
        contact: contact::extract_contact(raw),
        special_requirements: extract_special(&n),
        unknown_fields: unknown_fields(&n),
        hangup_requested: super::intent::is_hangup(&n),
        greeting_only: super::intent::is_greeting(&n),
    }
}

/// Special requirements are stored as the customer's own trimmed fragment, not
/// a rephrasing, so nothing is added to the application that the client did
/// not say.
fn extract_special(text: &str) -> Option<String> {
    let marker = cargo_lexicon::special_requirement(text)?;
    let fragment = fragment_around(text, &marker);
    Some(sanitize_fragment(&fragment))
}

fn fragment_around(text: &str, marker: &str) -> String {
    let idx = text.find(marker).unwrap_or(0);
    let start = text[..idx]
        .char_indices()
        .rev()
        .take(4)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    let end = text[idx..]
        .char_indices()
        .take(12)
        .last()
        .map(|(i, c)| idx + i + c.len_utf8())
        .unwrap_or(text.len());
    text[start..end].trim().to_string()
}

/// Maps "не знаю" onto the field the agent was last asking about, so a refusal
/// is attached to a field instead of being discarded.
fn unknown_fields(text: &str) -> Vec<crate::domain::missing::MissingField> {
    if !quantity::is_unknown_reply(text) {
        return Vec::new();
    }
    use crate::domain::missing::MissingField as F;
    // A bare "не знаю" applies to whatever is still open; the dialogue engine
    // narrows it down using the current stage.
    vec![F::Weight]
}
