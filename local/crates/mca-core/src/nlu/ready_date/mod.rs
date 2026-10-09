//! Ready-date parsing.
//!
//! Four strategies run in order of specificity: relative phrases ("завтра"),
//! weekdays ("в следующий понедельник"), explicit numbers ("15.10") and named
//! months ("15 октября"). Whatever the source, the result is one ISO-8601 date
//! in the application.

use chrono::Datelike;

mod month_day;
mod relative;
mod weekday;

pub(crate) use month_day::parse_month_day;
pub(crate) use relative::parse_relative;
pub(crate) use weekday::parse_weekday;

/// Extracts a ready date from phrases like "на следующую неделю", "завтра",
/// "15 октября" and returns an ISO-8601 date (YYYY-MM-DD).
pub fn extract_ready_date(text: &str) -> Option<String> {
    parse_relative(text)
        .or_else(|| parse_weekday(text))
        .or_else(|| parse_numeric(text))
        .or_else(|| parse_month_day(text))
        .map(|d| d.format("%Y-%m-%d").to_string())
}

/// "15.10" / "15.10.2026" — only trusted when a date separator is present, so a
/// bare "15" never becomes a date on its own.
fn parse_numeric(text: &str) -> Option<chrono::NaiveDate> {
    let digits: String = text
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    for token in digits.split(['.', ' ', '-']) {
        if let Ok(value) = token.parse::<i64>() {
            if (1..=31).contains(&value) && (text.contains('.') || text.contains('/')) {
                return from_day_of_year(value as u32);
            }
        }
    }
    None
}

fn from_day_of_year(day: u32) -> Option<chrono::NaiveDate> {
    let today = today();
    chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), day)
}

pub(crate) fn today() -> chrono::NaiveDate {
    chrono::Utc::now().date_naive()
}

#[cfg(test)]
mod tests;
