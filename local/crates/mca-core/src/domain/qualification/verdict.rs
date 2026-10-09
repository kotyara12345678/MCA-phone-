//! The verdict a qualification rule returns, with its human-readable reason.

use super::Qualification;

/// Verdict returned by [`crate::policy::rules::evaluate`].
#[derive(Debug, Clone, PartialEq)]
pub struct QualificationVerdict {
    pub qualification: Qualification,
    pub reason: Option<String>,
}

impl QualificationVerdict {
    pub fn pending(reason: impl Into<String>) -> Self {
        Self {
            qualification: Qualification::Pending,
            reason: Some(reason.into()),
        }
    }

    pub fn qualified(reason: impl Into<String>) -> Self {
        Self {
            qualification: Qualification::Qualified,
            reason: Some(reason.into()),
        }
    }

    pub fn rejected(reason: impl Into<String>) -> Self {
        Self {
            qualification: Qualification::Rejected,
            reason: Some(reason.into()),
        }
    }

    pub fn needs_human(reason: impl Into<String>) -> Self {
        Self {
            qualification: Qualification::NeedsHuman,
            reason: Some(reason.into()),
        }
    }
}
