//! Environment-variable readers shared by every config parser.
//!
//! Typed at the edge on purpose: a missing or malformed variable is a start-up
//! error with a named field, never a runtime surprise. Empty and whitespace-only
//! values are treated as absent, because that is what an unset shell variable
//! looks like to a process.

use std::time::Duration;

/// Reads an environment variable, treating whitespace-only as absent.
pub fn var(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub fn var_or(name: &str, fallback: &str) -> String {
    var(name).unwrap_or_else(|| fallback.to_string())
}

/// Parses a boolean env var; accepts `1/true/yes/on` and `0/false/no/off`.
pub fn flag(name: &str, fallback: bool) -> bool {
    match var(name).map(|v| v.to_ascii_lowercase()) {
        Some(v) => matches!(v.as_str(), "1" | "true" | "yes" | "on"),
        None => fallback,
    }
}

pub fn number(name: &str, fallback: u32) -> u32 {
    var(name).and_then(|v| v.parse().ok()).unwrap_or(fallback)
}

pub fn millis(name: &str, fallback: u64) -> Duration {
    Duration::from_millis(u64::from(number(name, fallback as u32)))
}

pub fn seconds(name: &str, fallback: u64) -> Duration {
    Duration::from_secs(u64::from(number(name, fallback as u32)))
}

pub fn port(name: &str, fallback: u16) -> u16 {
    var(name).and_then(|v| v.parse().ok()).unwrap_or(fallback)
}
