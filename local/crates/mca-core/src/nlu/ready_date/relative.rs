use chrono::{Duration, NaiveDate};

use super::today;

/// Relative phrases, longest first so "через 2 недели" cannot be swallowed by
/// the "через неделю" rule.
const PHRASES: &[(&[&str], i64)] = &[
    (&["сегодня", "сейчас", "сегодня же"], 0),
    (&["послезавтра", "на послезавтра", "через 2 дня"], 2),
    (&["завтра", "на следующий день", "на след день"], 1),
    (
        &[
            "через неделю",
            "на следующую неделю",
            "на следующей неделе",
            "следующая неделя",
        ],
        7,
    ),
    (&["через 2 недели", "через две недели", "через 14 дней"], 14),
    (&["через месяц", "через 30 дней"], 30),
];

pub fn parse_relative(text: &str) -> Option<NaiveDate> {
    let hit = PHRASES
        .iter()
        .find(|(needles, _)| needles.iter().any(|n| text.contains(n)))?;
    Some(today() + Duration::days(hit.1))
}
