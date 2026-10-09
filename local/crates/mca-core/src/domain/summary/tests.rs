use super::*;
use crate::domain::missing::MissingField;

fn full() -> ApplicationState {
    ApplicationState {
        cargo: Some("Оборудование".into()),
        weight_kg: Some(12000.0),
        origin_country: Some("Germany".into()),
        destination_country: Some("Kazakhstan".into()),
        destination_city: Some("Almaty".into()),
        ..Default::default()
    }
}

#[test]
fn summary_uses_city_and_country() {
    let s = summarize(&full());
    assert!(s.contains("Оборудование"), "{s}");
    assert!(s.contains("Almaty"), "{s}");
}

#[test]
fn field_lines_cover_every_field() {
    let lines = field_lines(&full());
    assert_eq!(lines.len(), 8);
    assert_eq!(lines[1].1, "12 т (12 000 кг)");
    assert_eq!(lines[2].1, "Не указан");
}

#[test]
fn missing_sentence_two_items() {
    let s = missing_sentence_ru(&[MissingField::ReadyDate, MissingField::Contact]);
    assert_eq!(s, "Не указаны: дата готовности груза и контакт клиента.");
}

#[test]
fn missing_sentence_empty() {
    assert_eq!(missing_sentence_ru(&[]), "");
}
