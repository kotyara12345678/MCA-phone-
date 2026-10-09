use serde::{Deserialize, Serialize};
use std::fmt;

mod verdict;

pub use verdict::QualificationVerdict;

/// Outcome of the deterministic qualification rules. This is the *only* place
/// that may set a qualification; the LLM has no write access to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Qualification {
    /// Not enough information yet — keep collecting.
    #[default]
    Pending,
    /// All mandatory fields present and in scope.
    Qualified,
    /// Clearly out of scope for a transport request.
    Rejected,
    /// In scope, but a human must finish the job.
    NeedsHuman,
}

impl Qualification {
    pub const ALL: [Qualification; 4] = [
        Self::Pending,
        Self::Qualified,
        Self::Rejected,
        Self::NeedsHuman,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Qualified => "qualified",
            Self::Rejected => "rejected",
            Self::NeedsHuman => "needs_human",
        }
    }

    /// Russian label rendered in the manager email.
    pub fn label_ru(&self) -> &'static str {
        match self {
            Self::Pending => "Требует уточнения",
            Self::Qualified => "Квалифицирована",
            Self::Rejected => "Не относится к перевозкам",
            Self::NeedsHuman => "Передана менеджеру",
        }
    }

    /// Whether the call should still generate a manager handover.
    pub fn produces_handover(&self) -> bool {
        matches!(self, Self::Qualified | Self::Pending | Self::NeedsHuman)
    }

    pub fn is_final(&self) -> bool {
        !matches!(self, Self::Pending)
    }
}

impl fmt::Display for Qualification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests;
