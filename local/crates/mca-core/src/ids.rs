use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Error returned when a caller-supplied identifier is not a valid UUID.
#[derive(Debug, thiserror::Error)]
#[error("invalid identifier `{raw}`: {reason}")]
pub struct InvalidId {
    pub raw: String,
    pub reason: String,
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(value: Uuid) -> Self {
                Self(value)
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = InvalidId;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(value).map(Self).map_err(|e| InvalidId {
                    raw: value.to_string(),
                    reason: e.to_string(),
                })
            }
        }
    };
}

id_type!(CallId);
id_type!(SessionId);
id_type!(ApplicationId);
id_type!(MessageId);
id_type!(EmailId);
id_type!(CorrelationId);
