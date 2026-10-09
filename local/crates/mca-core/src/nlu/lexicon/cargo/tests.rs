use super::*;

#[test]
fn maps_equipment() {
    assert_eq!(
        canonical_cargo("нужно перевезти оборудование").as_deref(),
        Some("Оборудование")
    );
}

#[test]
fn generic_cargo_is_none() {
    assert_eq!(canonical_cargo("что-то"), None);
}

#[test]
fn detects_special_requirement() {
    assert_eq!(
        special_requirement("оборудование хрупкое").as_deref(),
        Some("хрупк")
    );
}
