use chrono::{Datelike, NaiveDate};

use super::today;

const MONTHS: &[(&str, u32)] = &[
    ("январ", 1),
    ("феврал", 2),
    ("март", 3),
    ("апрел", 4),
    ("мая", 5),
    ("май", 5),
    ("июн", 6),
    ("июл", 7),
    ("август", 8),
    ("сентябр", 9),
    ("октябр", 10),
    ("ноябр", 11),
    ("декабр", 12),
];

/// "15 октября" and "октября 15". The month that already passed rolls to next
/// year rather than failing — customers talk about next season's slot, not last.
pub fn parse_month_day(text: &str) -> Option<NaiveDate> {
    let (needle, month) = MONTHS.iter().find(|(needle, _)| text.contains(needle))?;
    let index = text.find(needle)?;
    let day = nearest_digit(index, text)?;
    let mut year = today().year();
    let mut candidate = NaiveDate::from_ymd_opt(year, *month, day);
    if candidate.is_none() || candidate < Some(today()) {
        year += 1;
        candidate = NaiveDate::from_ymd_opt(year, *month, day);
    }
    candidate
}

/// Day number located closest to the month mention, so "2 тонны 15 октября"
/// does not read the cargo quantity as the day.
fn nearest_digit(near: usize, text: &str) -> Option<u32> {
    let bytes = text.as_bytes();
    let mut best: Option<(usize, u32)> = None;
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i > start {
            if let Some(value) = day_at(text, start, i) {
                let dist = start.abs_diff(near);
                if best.is_none_or(|(bd, _)| dist < bd) {
                    best = Some((dist, value));
                }
            }
        } else {
            i += 1;
        }
    }
    best.map(|(_, value)| value)
}

fn day_at(text: &str, start: usize, end: usize) -> Option<u32> {
    let value = text.get(start..end)?.parse::<u32>().ok()?;
    (1..=31).contains(&value).then_some(value)
}
