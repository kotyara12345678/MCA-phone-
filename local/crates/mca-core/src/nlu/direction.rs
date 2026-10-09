use super::lexicon::cities;

/// Splits an utterance into an origin segment and a destination segment using
/// directional prepositions, so "из Германии в Алматы" never collapses into a
/// single ambiguous place mention.
///
/// Returns `(origin_segment, destination_segment)`; either may be empty.
pub fn split_directions(text: &str) -> (String, String) {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut origin_pos = find_marker(&chars, ORIGIN_MARKERS);
    let mut dest_pos = find_marker(&chars, DEST_MARKERS);
    if !text.is_empty() {
        if origin_pos.is_none() && markers_at_start(text, ORIGIN_MARKERS) {
            origin_pos = Some(0);
        }
        if dest_pos.is_none() && markers_at_start(text, DEST_MARKERS) {
            dest_pos = Some(0);
        }
    }

    match (origin_pos, dest_pos) {
        (Some(o), Some(d)) if d > o => (segment(&chars, o, d), segment(&chars, d, len)),
        (Some(o), Some(d)) => (segment(&chars, d, len), segment(&chars, o, len)),
        (Some(o), None) => (segment(&chars, o, len), String::new()),
        (None, Some(d)) => (String::new(), segment(&chars, d, len)),
        (None, None) => (String::new(), String::new()),
    }
}

fn markers_at_start(text: &str, markers: &[&str]) -> bool {
    markers.iter().any(|m| {
        let m = m.trim();
        !m.is_empty() && text.starts_with(&format!("{m} "))
    })
}

fn find_marker(chars: &[char], markers: &[&str]) -> Option<usize> {
    markers
        .iter()
        .filter_map(|m| {
            let needle: Vec<char> = m.chars().collect();
            window_position(chars, &needle)
        })
        .min()
}

fn window_position(haystack: &[char], needle: &[char]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    (0..=haystack.len() - needle.len()).find(|&i| haystack[i..i + needle.len()] == *needle)
}

fn segment(chars: &[char], from: usize, to: usize) -> String {
    let slice = &chars[from.min(chars.len())..to.min(chars.len())];
    slice.iter().collect::<String>().trim().to_string()
}

/// Russian label for a canonical city, re-exported so callers do not need to
/// reach into the lexicon module directly.
pub fn city_label(canonical: &str) -> String {
    cities::city_ru(canonical)
}

const ORIGIN_MARKERS: &[&str] = &[" из ", "откуда", " from ", "с ", "от "];
const DEST_MARKERS: &[&str] = &[" в ", "куда", " до ", " to ", "в/"];

#[cfg(test)]
mod tests;
