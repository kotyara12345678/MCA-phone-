//! Language guessing by script ratio, with tests that pin the thresholds.

use super::Language;

/// Guesses the language of a customer utterance from its Cyrillic/Latin mix.
/// An empty utterance defaults to Russian because the pipeline is Russian-first.
pub fn detect(text: &str) -> Language {
    let mut cyrillic = 0u32;
    let mut latin = 0u32;
    for ch in text.chars() {
        if ('\u{0400}'..='\u{04FF}').contains(&ch) {
            cyrillic += 1;
        } else if ch.is_ascii_alphabetic() {
            latin += 1;
        }
    }
    if cyrillic == 0 && latin == 0 {
        return Language::Ru;
    }
    let cyrillic_ratio = cyrillic as f32 / (cyrillic + latin) as f32;
    if cyrillic_ratio > 0.85 {
        Language::Ru
    } else if cyrillic_ratio > 0.2 {
        Language::Kk
    } else {
        Language::En
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_russian() {
        assert_eq!(detect("перевезти оборудование"), Language::Ru);
    }

    #[test]
    fn detects_english() {
        assert_eq!(detect("need to ship equipment"), Language::En);
    }

    #[test]
    fn empty_defaults_to_russian() {
        assert_eq!(detect("   "), Language::Ru);
    }
}
