pub mod cargo;
pub mod cities;
pub mod countries;

use std::sync::OnceLock;

/// Sanitises a free-text fragment: trims, collapses whitespace and truncates so
/// a rambling answer can never bloat the persisted state blob.
pub fn sanitize_fragment(raw: &str) -> String {
    let joined = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if joined.chars().count() <= 240 {
        return joined;
    }
    let truncated: String = joined.chars().take(237).collect();
    format!("{truncated}...")
}

/// Cap for any single free-text field persisted in the application state.
pub const FRAGMENT_LIMIT: usize = 240;

pub fn fragment_limit() -> usize {
    static LIMIT: OnceLock<usize> = OnceLock::new();
    *LIMIT.get_or_init(|| FRAGMENT_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_long_fragments() {
        let long = "а".repeat(500);
        let out = sanitize_fragment(&long);
        assert_eq!(out.chars().count(), 240);
        assert!(out.ends_with("..."));
    }

    #[test]
    fn keeps_short_fragments_intact() {
        assert_eq!(
            sanitize_fragment("  хрупкое  оборудование "),
            "хрупкое оборудование"
        );
    }
}
