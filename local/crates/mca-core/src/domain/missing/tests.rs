use super::*;

#[test]
fn mandatory_set_excludes_volume_and_contact() {
    assert!(!MissingField::Volume.is_mandatory());
    assert!(!MissingField::Contact.is_mandatory());
    assert!(MissingField::Cargo.is_mandatory());
}

#[test]
fn ordering_is_business_priority() {
    let unordered = vec![
        MissingField::Contact,
        MissingField::Weight,
        MissingField::Cargo,
    ];
    assert_eq!(
        order_missing(&unordered),
        vec![
            MissingField::Cargo,
            MissingField::Weight,
            MissingField::Contact
        ]
    );
}

#[test]
fn all_labels_are_populated() {
    for field in MissingField::ALL {
        assert!(!field.label_ru().is_empty(), "{field} label missing");
    }
}
