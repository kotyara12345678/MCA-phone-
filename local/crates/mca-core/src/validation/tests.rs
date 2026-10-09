use super::*;

#[test]
fn clean_text_collapses_whitespace() {
    assert_eq!(clean_text("  12   тонн \n "), "12 тонн");
}

#[test]
fn control_characters_are_rejected() {
    assert!(has_control_characters("bad\u{0}text"));
    assert!(!has_control_characters("line\nbreak"));
}

#[test]
fn len_is_measured_in_chars() {
    assert!(check_len("cargo", "ok", 8).is_ok());
    assert!(check_len("cargo", "очень-длинный-груз", 8).is_err());
}

#[test]
fn quantity_bounds_are_enforced() {
    assert!(check_quantity("weight_kg", 12000.0, 0.0, 500_000.0).is_ok());
    assert!(check_quantity("weight_kg", f64::NAN, 0.0, 500_000.0).is_err());
    assert!(check_quantity("weight_kg", 900_000.0, 0.0, 500_000.0).is_err());
}
