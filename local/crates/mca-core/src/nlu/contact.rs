/// Contact extraction: e-mail, Russian/international phone numbers, and
/// `имя + телефон` pairs. The customer's own words are preserved in the
/// `contact` field; a manager reads it verbatim.
pub fn extract_contact(text: &str) -> Option<String> {
    extract_email(text)
        .or_else(|| extract_phone(text))
        .or_else(|| extract_name(text))
}

pub fn extract_email(text: &str) -> Option<String> {
    let mut token = String::new();
    let mut found: Option<String> = None;
    for c in text.chars() {
        token.push(c);
        if c.is_whitespace() {
            if let Some(email) = as_email(&token) {
                found = Some(email);
            }
            token.clear();
        }
    }
    if found.is_none() {
        found = as_email(&token);
    }
    found
}

fn as_email(token: &str) -> Option<String> {
    let cleaned: String = token
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || "@._-+".contains(*c))
        .collect();
    let (local, domain) = cleaned.split_once('@')?;
    let valid = !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.');
    valid.then_some(cleaned)
}

/// Recognises `+7 701 234 56 78`, `87012345678`, `8 (7172) 00-00-00`.
pub fn extract_phone(text: &str) -> Option<String> {
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 10 {
        return None;
    }
    let has_phone_word = ["телефон", "тел", "звони", "номер", "phone", "связаться"]
        .iter()
        .any(|w| text.contains(w));
    if !has_phone_word && !text.contains('+') {
        return None;
    }
    Some(digits)
}

const NAME_HINTS: &[&str] = &["меня зовут", "мое имя", "моё имя", "звоню", "это ", "имя"];

pub fn extract_name(text: &str) -> Option<String> {
    let hint = NAME_HINTS.iter().find(|h| text.contains(**h))?;
    let rest = text.split_once(hint)?.1.trim();
    let words: Vec<&str> = rest
        .split_whitespace()
        .take_while(|w| w.chars().all(|c| c.is_alphabetic() || c == '-'))
        .collect();
    let name = words.join(" ");
    (!name.is_empty()).then_some(name)
}

#[cfg(test)]
mod tests;
