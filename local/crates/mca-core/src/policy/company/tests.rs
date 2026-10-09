//! Tests for company-facing formatting helpers.

use super::*;

#[test]
fn country_names_are_translated() {
    assert_eq!(country_ru("Germany"), "Германия");
    assert_eq!(country_ru("Neverland"), "Neverland");
}

#[test]
fn weight_formats_tons_and_kg() {
    assert_eq!(format_weight(Some(12_000.0)), "12 т (12 000 кг)");
    assert_eq!(format_weight(Some(450.0)), "450 кг");
    assert_eq!(format_weight(None), "Не указан");
}

#[test]
fn volume_formats_m3() {
    assert_eq!(format_volume(Some(2.5)), "2.5 м³");
}
