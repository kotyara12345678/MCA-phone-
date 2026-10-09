//! Question topics and their detection.

mod answers;

use super::question_keywords::needles;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Topics a customer can ask about. Detected deterministically from the
/// transcript; each maps to a fixed, company-approved answer so the LLM is
/// never the source of a company claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionTopic {
    Price,
    Terms,
    Documents,
    Customs,
    Timeline,
    Insurance,
    Tracking,
    Contact,
    Capabilities,
}

impl QuestionTopic {
    pub const ALL: [QuestionTopic; 9] = [
        Self::Price,
        Self::Terms,
        Self::Documents,
        Self::Customs,
        Self::Timeline,
        Self::Insurance,
        Self::Tracking,
        Self::Contact,
        Self::Capabilities,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Price => "price",
            Self::Terms => "terms",
            Self::Documents => "documents",
            Self::Customs => "customs",
            Self::Timeline => "timeline",
            Self::Insurance => "insurance",
            Self::Tracking => "tracking",
            Self::Contact => "contact",
            Self::Capabilities => "capabilities",
        }
    }

    /// Detects every topic mentioned in one utterance. A single turn may raise
    /// several questions and all of them must be answered.
    pub fn detect_all(text: &str) -> Vec<QuestionTopic> {
        QuestionTopic::ALL
            .into_iter()
            .filter(|topic| topic.detect(text))
            .collect()
    }

    /// Keyword heuristics per topic, run over normalised text.
    pub fn detect(&self, text: &str) -> bool {
        let normalized = crate::nlu::normalize::normalize(text);
        needles(*self).iter().any(|n| normalized.contains(n))
    }

    /// Company-approved answer. Prices are never invented — the agent states
    /// that a manager calculates them.
    pub fn answer_ru(&self) -> &'static str {
        answers::answer_ru(*self)
    }
}

impl fmt::Display for QuestionTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests;
