use super::normalize::first_number;

/// Weight in kilograms from text like "12 тонн", "12000 кг", "1.5 т".
/// Returns `None` when no weight-like number is present, so an unrelated
/// number (a price, a date) is never mistaken for a weight.
pub fn extract_weight_kg(text: &str) -> Option<f64> {
    let unit = detect_unit(text)?;
    let value = first_number(text)?;
    let kg = match unit {
        WeightUnit::Kilograms => value,
        WeightUnit::Tons => value * 1000.0,
    };
    if !kg.is_finite() || kg <= 0.0 || kg > 500_000.0 {
        return None;
    }
    Some(kg)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightUnit {
    Kilograms,
    Tons,
}

const KG_MARKERS: &[&str] = &["кг", "килограмм", "kg", "кило"];
const TON_MARKERS: &[&str] = &["тонн", "тонна", "т.", "т ", " т", "ton", "тн", "центнер"];

pub fn detect_unit(text: &str) -> Option<WeightUnit> {
    if KG_MARKERS.iter().any(|m| text.contains(m)) {
        return Some(WeightUnit::Kilograms);
    }
    if TON_MARKERS.iter().any(|m| text.contains(m)) {
        return Some(WeightUnit::Tons);
    }
    None
}

/// Whether the customer explicitly said they do not know the value.
pub fn is_unknown_reply(text: &str) -> bool {
    const UNKNOWN: &[&str] = &[
        "не знаю",
        "незнаю",
        "не известно",
        "неизвестно",
        "не определили",
        "не определено",
        "пока нет",
        "нет данных",
        "не скажу",
        "unknown",
    ];
    UNKNOWN.iter().any(|m| text.contains(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tons_are_converted_to_kg() {
        assert_eq!(extract_weight_kg("около 12 тонн"), Some(12_000.0));
    }

    #[test]
    fn kilograms_pass_through() {
        assert_eq!(extract_weight_kg("12000 кг"), Some(12_000.0));
    }

    #[test]
    fn decimal_tons() {
        assert_eq!(extract_weight_kg("1.5 т"), Some(1_500.0));
    }

    #[test]
    fn no_unit_means_no_weight() {
        assert_eq!(extract_weight_kg("в алматы 5 этаж"), None);
    }

    #[test]
    fn absurd_weight_is_rejected() {
        assert_eq!(extract_weight_kg("900000 тонн"), None);
    }

    #[test]
    fn unknown_phrase_detected() {
        assert!(is_unknown_reply("не знаю"));
        assert!(is_unknown_reply("пока нет"));
        assert!(!is_unknown_reply("12 тонн"));
    }
}
