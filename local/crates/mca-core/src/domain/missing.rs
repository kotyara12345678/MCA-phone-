use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A single field of the transport application that may still be missing.
/// This is the *only* vocabulary used to ask questions, track progress and
/// render the manager email, which keeps "don't ask twice" enforceable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingField {
    Cargo,
    Weight,
    Volume,
    Origin,
    Destination,
    ReadyDate,
    Contact,
}

impl MissingField {
    pub const ALL: [MissingField; 7] = [
        Self::Cargo,
        Self::Weight,
        Self::Origin,
        Self::Destination,
        Self::ReadyDate,
        Self::Contact,
        Self::Volume,
    ];

    /// Fields without which a request cannot be handed to a manager at all.
    pub const MANDATORY: [MissingField; 5] = [
        Self::Cargo,
        Self::Weight,
        Self::Origin,
        Self::Destination,
        Self::ReadyDate,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cargo => "cargo",
            Self::Weight => "weight",
            Self::Volume => "volume",
            Self::Origin => "origin",
            Self::Destination => "destination",
            Self::ReadyDate => "ready_date",
            Self::Contact => "contact",
        }
    }

    /// Russian label used in the manager email and in qualification reasons.
    pub fn label_ru(&self) -> &'static str {
        match self {
            Self::Cargo => "характер груза",
            Self::Weight => "вес груза",
            Self::Volume => "объём груза",
            Self::Origin => "страна отправления",
            Self::Destination => "страна/город назначения",
            Self::ReadyDate => "дата готовности груза",
            Self::Contact => "контакт клиента",
        }
    }

    pub fn is_mandatory(&self) -> bool {
        Self::MANDATORY.contains(self)
    }
}

impl fmt::Display for MissingField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MissingField {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|f| f.as_str() == value)
            .ok_or_else(|| format!("unknown field `{value}`"))
    }
}

/// Ordered list of outstanding fields. Ordering is by business priority, not by
/// the order the customer happened to mention them, so the agent converges.
pub fn order_missing(fields: &[MissingField]) -> Vec<MissingField> {
    let mut out: Vec<MissingField> = Vec::with_capacity(fields.len());
    for candidate in MissingField::ALL {
        if fields.contains(&candidate) {
            out.push(candidate);
        }
    }
    out
}

#[cfg(test)]
mod tests;
