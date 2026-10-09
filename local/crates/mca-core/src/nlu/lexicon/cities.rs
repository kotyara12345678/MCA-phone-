//! City lexicon lookup. When several cities appear in one utterance the
//! earliest mention wins, matching reading order.

mod data;

use data::CITIES;

/// Canonical English name for the earliest city mentioned in `text`.
pub fn canonical_city(text: &str) -> Option<&'static str> {
    CITIES
        .iter()
        .filter_map(|(canonical, _, aliases)| {
            aliases
                .iter()
                .filter_map(|alias| text.find(alias).map(|pos| (pos, *canonical)))
                .min_by_key(|(pos, _)| *pos)
        })
        .min_by_key(|(pos, _)| *pos)
        .map(|(_, canonical)| canonical)
}

/// Russian label for a canonical city, used by the manager email.
pub fn city_ru(canonical: &str) -> String {
    CITIES
        .iter()
        .find(|(c, _, _)| *c == canonical)
        .map(|(_, ru, _)| (*ru).to_string())
        .unwrap_or_else(|| canonical.to_string())
}

#[cfg(test)]
mod tests;
