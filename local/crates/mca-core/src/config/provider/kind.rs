//! Which implementation to build for a port.

/// `Http` needs credentials; `Fake` is fully offline and deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Http,
    Fake,
    Log,
    Disabled,
}

impl ProviderKind {
    pub fn parse(raw: &str) -> Result<Self, crate::validation::ValidationError> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "http" | "openai" | "remote" | "external" => Ok(Self::Http),
            "fake" | "mock" | "rule" | "rule_based" | "local" => Ok(Self::Fake),
            "log" | "logging" | "null" => Ok(Self::Log),
            "none" | "disabled" | "off" => Ok(Self::Disabled),
            other => Err(crate::validation::ValidationError::invalid(
                "provider",
                format!("unknown provider `{other}`"),
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Fake => "fake",
            Self::Log => "log",
            Self::Disabled => "disabled",
        }
    }

    /// True when the provider talks to a real external service.
    pub fn is_remote(&self) -> bool {
        matches!(self, Self::Http)
    }
}
