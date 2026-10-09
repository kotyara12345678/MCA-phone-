//! Conversation language.
//!
//! Drives which prompt variant and which deterministic template set the agent
//! uses. Detection lives in `detect.rs` so the enum file stays compact.

mod detect;

use serde::{Deserialize, Serialize};
use std::fmt;

use detect::detect as detect_language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Ru,
    Kk,
    En,
    De,
}

impl Language {
    pub const ALL: [Language; 4] = [Self::Ru, Self::Kk, Self::En, Self::De];

    /// BCP-47 tag handed to STT/TTS providers.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Ru => "ru-RU",
            Self::Kk => "kk-KZ",
            Self::En => "en-US",
            Self::De => "de-DE",
        }
    }

    /// IETF language subtag used to pick the LLM prompt variant.
    pub fn llm_tag(&self) -> &'static str {
        match self {
            Self::Ru => "ru",
            Self::Kk => "kk",
            Self::En => "en",
            Self::De => "de",
        }
    }

    pub fn as_str(&self) -> &'static str {
        self.llm_tag()
    }

    /// Guesses the language of a customer utterance from Cyrillic/Latin mix.
    pub fn detect(text: &str) -> Self {
        detect_language(text)
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|l| l.llm_tag() == value.to_ascii_lowercase())
            .ok_or_else(|| format!("unsupported language `{value}`"))
    }
}
