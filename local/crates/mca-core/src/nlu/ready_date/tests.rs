use super::*;
use chrono::{Duration, Utc};

#[test]
fn parses_tomorrow() {
    let expected = (Utc::now().date_naive() + Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    assert_eq!(extract_ready_date("завтра"), Some(expected));
}

#[test]
fn parses_next_week_phrase() {
    let expected = (Utc::now().date_naive() + Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();
    assert_eq!(extract_ready_date("через неделю"), Some(expected));
}

#[test]
fn parses_next_weekday() {
    let value = extract_ready_date("в следующий понедельник").expect("weekday phrase");
    assert!(!value.is_empty());
    assert_eq!(value.len(), 10);
}

#[test]
fn parses_month_day() {
    let value = extract_ready_date("15 октября").expect("month/day");
    assert!(value.ends_with("-10-15"), "{value}");
}

#[test]
fn no_date_phrase_returns_none() {
    assert_eq!(extract_ready_date("когда-то потом"), None);
}
