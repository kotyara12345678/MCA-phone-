use chrono::{Datelike, Duration, NaiveDate};

use super::today;

/// Weekday names mapped to `chrono::Weekday` numbering (Monday = 0).
const WEEKDAYS: &[(&str, i64)] = &[
    ("понедельник", 0),
    ("вторник", 1),
    ("среда", 2),
    ("среду", 2),
    ("четверг", 3),
    ("пятница", 4),
    ("пятницу", 4),
    ("суббота", 5),
    ("субботу", 5),
    ("воскресенье", 6),
    ("воскресенья", 6),
];

/// Only "следующий/следующую <weekday>" is resolved: a bare weekday mention is
/// ambiguous, and guessing would silently corrupt the customer's date.
pub fn parse_weekday(text: &str) -> Option<NaiveDate> {
    if !text.contains("следующ") {
        return None;
    }
    let (_, target) = WEEKDAYS.iter().find(|(needle, _)| text.contains(needle))?;
    let now = today();
    let delta = (*target - now.weekday() as i64).rem_euclid(7);
    Some(now + Duration::days(if delta == 0 { 7 } else { delta }))
}
