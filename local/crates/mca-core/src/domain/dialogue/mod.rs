//! Explicit dialogue state machine. The LLM proposes content; it never picks
//! this value — the policy derives it deterministically.

mod fields;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// One conversation stage. Edges are owned by the policy module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DialogueStage {
    #[default]
    Greeting,
    CollectingCargo,
    CollectingWeight,
    CollectingVolume,
    CollectingOrigin,
    CollectingDestination,
    CollectingReadyDate,
    CollectingContact,
    ReadyForQualification,
    Qualified,
    Rejected,
    NeedsHuman,
    Completed,
}

impl DialogueStage {
    /// Every stage, in the canonical order used by progress reporting.
    pub const ALL: [DialogueStage; 13] = [
        Self::Greeting,
        Self::CollectingCargo,
        Self::CollectingWeight,
        Self::CollectingVolume,
        Self::CollectingOrigin,
        Self::CollectingDestination,
        Self::CollectingReadyDate,
        Self::CollectingContact,
        Self::ReadyForQualification,
        Self::Qualified,
        Self::Rejected,
        Self::NeedsHuman,
        Self::Completed,
    ];

    /// The field a collecting stage asks about.
    pub fn collecting_field(&self) -> Option<crate::domain::missing::MissingField> {
        fields::collecting_field(*self)
    }

    pub fn is_terminal(&self) -> bool {
        fields::is_terminal(*self)
    }

    /// Whether the call is still interactive.
    pub fn is_active(&self) -> bool {
        !self.is_terminal()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Greeting => "greeting",
            Self::CollectingCargo => "collecting_cargo",
            Self::CollectingWeight => "collecting_weight",
            Self::CollectingVolume => "collecting_volume",
            Self::CollectingOrigin => "collecting_origin",
            Self::CollectingDestination => "collecting_destination",
            Self::CollectingReadyDate => "collecting_ready_date",
            Self::CollectingContact => "collecting_contact",
            Self::ReadyForQualification => "ready_for_qualification",
            Self::Qualified => "qualified",
            Self::Rejected => "rejected",
            Self::NeedsHuman => "needs_human",
            Self::Completed => "completed",
        }
    }
}

impl fmt::Display for DialogueStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DialogueStage {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|s| s.as_str() == value)
            .ok_or_else(|| format!("unknown dialogue stage `{value}`"))
    }
}

#[cfg(test)]
mod tests;
