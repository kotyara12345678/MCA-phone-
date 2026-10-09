use super::*;

#[test]
fn maps_almaty() {
    assert_eq!(canonical_city("в алматы"), Some("Almaty"));
}

#[test]
fn maps_inflected_form() {
    assert_eq!(canonical_city("из гамбурга"), Some("Hamburg"));
}

#[test]
fn unknown_city_is_none() {
    assert_eq!(canonical_city("в нирване"), None);
}

#[test]
fn russian_label_is_explicit() {
    assert_eq!(city_ru("Almaty"), "Алматы");
    assert_eq!(city_ru("Neverland"), "Neverland");
}
