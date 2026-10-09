use serde::{Deserialize, Serialize};

/// Who produced an utterance. Persisted on every message row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Speaker {
    Agent,
    Customer,
    System,
}

impl Speaker {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Customer => "customer",
            Self::System => "system",
        }
    }

    pub fn is_customer(&self) -> bool {
        matches!(self, Self::Customer)
    }
}

impl std::fmt::Display for Speaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn customer_detection() {
        assert!(Speaker::Customer.is_customer());
        assert!(!Speaker::Agent.is_customer());
    }

    #[test]
    fn serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&Speaker::Customer).unwrap(),
            "\"customer\""
        );
    }
}
