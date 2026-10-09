//! Tests for contact extraction.

use super::*;

#[test]
fn extracts_email() {
    assert_eq!(
        extract_contact("почта ivan@mail.ru"),
        Some("ivan@mail.ru".into())
    );
}

#[test]
fn extracts_phone_with_hint() {
    assert_eq!(
        extract_contact("телефон +7 701 234 56 78"),
        Some("77012345678".into())
    );
}

#[test]
fn extracts_phone_with_plus_only() {
    assert_eq!(extract_contact("+77012345678"), Some("77012345678".into()));
}

#[test]
fn bare_number_is_not_a_phone() {
    assert_eq!(extract_contact("12 тонн"), None);
}

#[test]
fn extracts_name() {
    assert_eq!(extract_contact("меня зовут Иван"), Some("Иван".into()));
}
