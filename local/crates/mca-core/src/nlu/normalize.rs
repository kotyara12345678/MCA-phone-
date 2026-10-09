//! Text normalisation shared by every deterministic extractor: lowercase, ё→е,
//! punctuation collapsed to spaces, and Russian number words folded to digits
//! so that "двенадцать тонн" behaves exactly like "12 тонн".

mod numbers;

pub use numbers::first_number;

use numbers::fold_number_words;

/// Normalises free-form speech into a token string the lexicons can match.
pub fn normalize(text: &str) -> String {
    let lowered = fold_e(text.to_lowercase());
    let punctuated = replace_punctuation(&lowered);
    let digits = fold_number_words(&punctuated);
    collapse_spaces(&digits)
}

fn fold_e(text: String) -> String {
    text.chars()
        .map(|c| if c == 'ё' { 'е' } else { c })
        .collect()
}

fn replace_punctuation(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '.' {
                c
            } else {
                ' '
            }
        })
        .collect()
}

fn collapse_spaces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev_space = true;
    for ch in text.chars() {
        if ch == ' ' {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests;
