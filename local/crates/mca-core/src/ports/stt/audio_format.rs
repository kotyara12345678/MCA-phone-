//! Audio types shared by the STT/TTS ports.

use serde::{Deserialize, Serialize};

/// Supported wire formats on the way into and out of the voice pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    #[default]
    PcmS16Le,
    PcmS16Be,
    Ulaw,
    Alaw,
    Opus,
}

impl AudioFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PcmS16Le => "pcm_s16le",
            Self::PcmS16Be => "pcm_s16be",
            Self::Ulaw => "ulaw",
            Self::Alaw => "alaw",
            Self::Opus => "opus",
        }
    }

    /// Provider-facing name (`pcm16`, `g711ulaw`, ...) used in STT/TTS requests.
    pub fn from_name(name: &str) -> Option<Self> {
        match name
            .trim()
            .to_ascii_lowercase()
            .replace(['-', '_'], "")
            .as_str()
        {
            "pcm16" | "pcm" | "pcms16le" | "s16le" => Some(Self::PcmS16Le),
            "pcms16be" | "s16be" => Some(Self::PcmS16Be),
            "ulaw" | "g711ulaw" => Some(Self::Ulaw),
            "alaw" | "g711alaw" => Some(Self::Alaw),
            "opus" => Some(Self::Opus),
            _ => None,
        }
    }

    /// Sample rate the pipeline assumes for telephony-grade narrowband audio.
    pub fn default_sample_rate(&self) -> u32 {
        match self {
            Self::Opus => 48_000,
            _ => 8_000,
        }
    }

    /// Decodes one encoded frame into mono i16 samples. Compressed formats are
    /// not decoded here — the pipeline passes those through as opaque bytes.
    pub fn samples_from(&self, bytes: &[u8]) -> Vec<i16> {
        match self {
            Self::PcmS16Le => bytes
                .chunks_exact(2)
                .map(|c| i16::from_le_bytes([c[0], c[1]]))
                .collect(),
            Self::PcmS16Be => bytes
                .chunks_exact(2)
                .map(|c| i16::from_be_bytes([c[0], c[1]]))
                .collect(),
            Self::Ulaw | Self::Alaw => bytes.iter().map(|&b| ((b as i16) - 128) << 8).collect(),
            Self::Opus => Vec::new(),
        }
    }
}
