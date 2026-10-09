//! Latency dimensions recorded per dialogue turn.

use serde::{Deserialize, Serialize};

/// Latency dimensions tracked per turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyKind {
    Stt,
    Llm,
    Tts,
    Total,
    FirstToken,
}

impl LatencyKind {
    pub const ALL: [LatencyKind; 5] = [
        Self::Stt,
        Self::Llm,
        Self::Tts,
        Self::Total,
        Self::FirstToken,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stt => "stt",
            Self::Llm => "llm",
            Self::Tts => "tts",
            Self::Total => "total",
            Self::FirstToken => "first_token",
        }
    }
}

/// One turn's latency breakdown, in milliseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatencySample {
    pub stt_ms: i64,
    pub llm_ms: i64,
    pub tts_ms: i64,
    pub total_ms: i64,
    pub first_token_ms: i64,
}

impl LatencySample {
    /// Clamps every dimension to zero; a failed sub-call may report -1.
    pub fn sanitized(mut self) -> Self {
        self.stt_ms = self.stt_ms.max(0);
        self.llm_ms = self.llm_ms.max(0);
        self.tts_ms = self.tts_ms.max(0);
        self.total_ms = self.total_ms.max(0);
        self.first_token_ms = self.first_token_ms.max(0);
        self
    }
}
