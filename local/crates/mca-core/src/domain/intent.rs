use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// What the customer is trying to achieve. Determined by the extractor (LLM in
/// online mode, deterministic rules in offline/fallback mode) and only ever
/// *consumed* by the business rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    #[default]
    Unknown,
    Greeting,
    /// "перевезти груз из X в Y"
    ShippingRequest,
    /// "сколько стоит" — must never be answered with a made-up price
    PriceInquiry,
    /// "где мой груз" — needs a carrier system we do not have yet
    Tracking,
    /// out of scope for a logistics agent
    Unrelated,
}

impl Intent {
    pub const ALL: [Intent; 6] = [
        Self::Unknown,
        Self::Greeting,
        Self::ShippingRequest,
        Self::PriceInquiry,
        Self::Tracking,
        Self::Unrelated,
    ];

    /// True when the call should result in a transport application.
    pub fn is_shipping(&self) -> bool {
        matches!(self, Self::ShippingRequest | Self::Unknown)
    }

    pub fn is_out_of_scope(&self) -> bool {
        matches!(self, Self::Unrelated)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Greeting => "greeting",
            Self::ShippingRequest => "shipping_request",
            Self::PriceInquiry => "price_inquiry",
            Self::Tracking => "tracking",
            Self::Unrelated => "unrelated",
        }
    }
}

impl fmt::Display for Intent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Intent {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|i| i.as_str() == value)
            .ok_or_else(|| format!("unknown intent `{value}`"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipping_intents() {
        assert!(Intent::ShippingRequest.is_shipping());
        assert!(Intent::Unknown.is_shipping());
        assert!(!Intent::Tracking.is_shipping());
    }

    #[test]
    fn out_of_scope_detection() {
        assert!(Intent::Unrelated.is_out_of_scope());
        assert!(!Intent::PriceInquiry.is_out_of_scope());
    }
}
