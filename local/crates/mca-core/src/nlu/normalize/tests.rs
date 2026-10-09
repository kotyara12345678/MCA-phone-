use super::*;

#[test]
fn normalizes_case_and_punctuation() {
    assert_eq!(normalize("В Алматы!"), "в алматы");
}

#[test]
fn folds_simple_number_words() {
    assert_eq!(normalize("около двенадцать тонн"), "около 12 тонн");
}

#[test]
fn folds_compound_number_words() {
    assert_eq!(normalize("двадцать пять килограмм"), "25 килограмм");
}

#[test]
fn keeps_existing_digits() {
    assert_eq!(normalize("12 тонн"), "12 тонн");
}

#[test]
fn extracts_first_number() {
    assert_eq!(first_number("около 12 тонн"), Some(12.0));
    assert_eq!(first_number("1,5 куба"), Some(1.5));
    assert_eq!(first_number("нет числа"), None);
}

#[test]
fn collapses_repeated_spaces() {
    assert_eq!(normalize("а   б"), "а б");
}
