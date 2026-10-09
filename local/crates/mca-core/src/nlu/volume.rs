use super::normalize::first_number;

/// Volume in cubic metres from "20 м3", "20 кубов", "20 кубометров".
pub fn extract_volume_m3(text: &str) -> Option<f64> {
    if !has_volume_unit(text) {
        return None;
    }
    let value = first_number(text)?;
    if !value.is_finite() || value <= 0.0 || value > 5_000.0 {
        return None;
    }
    Some(value)
}

const VOLUME_MARKERS: &[&str] = &["м3", "м 3", "м^3", "куб", "кубометр", "cbm", "cubic", "m3"];

pub fn has_volume_unit(text: &str) -> bool {
    VOLUME_MARKERS.iter().any(|m| text.contains(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cubic_metres() {
        assert_eq!(extract_volume_m3("20 кубов"), Some(20.0));
        assert_eq!(extract_volume_m3("12.5 м3"), Some(12.5));
    }

    #[test]
    fn requires_a_volume_unit() {
        assert_eq!(extract_volume_m3("20 единиц"), None);
    }

    #[test]
    fn implausible_volume_rejected() {
        assert_eq!(extract_volume_m3("90000 м3"), None);
    }
}
