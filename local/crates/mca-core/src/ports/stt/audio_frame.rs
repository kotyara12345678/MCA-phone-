//! One audio frame and the transcript shape that comes back from STT.

use serde::{Deserialize, Serialize};

use crate::domain::language::Language;

use super::AudioFormat;

/// 16-bit samples. Bounded by construction: the pipeline only ever holds one
/// frame, never a whole call.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioFrame {
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<i16>,
}

impl AudioFrame {
    pub fn duration_ms(&self) -> f64 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        (self.samples.len() as f64 / self.channels.max(1) as f64) * 1000.0 / self.sample_rate as f64
    }

    pub fn is_silent(&self, threshold: i16) -> bool {
        self.samples.iter().all(|s| s.abs() <= threshold)
    }

    /// Compact on-the-wire encoding (raw bytes). No base64 in the hot path.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self.format {
            AudioFormat::PcmS16Be => self.samples.iter().flat_map(|s| s.to_be_bytes()).collect(),
            AudioFormat::Ulaw | AudioFormat::Alaw => {
                self.samples.iter().map(|s| (s >> 8) as u8).collect()
            }
            AudioFormat::Opus => Vec::new(),
            AudioFormat::PcmS16Le => self.samples.iter().flat_map(|s| s.to_le_bytes()).collect(),
        }
    }

    /// Builds a mono frame from one encoded chunk of bytes.
    pub fn from_bytes(format: AudioFormat, sample_rate: u32, bytes: &[u8]) -> Self {
        Self {
            format,
            sample_rate,
            channels: 1,
            samples: format.samples_from(bytes),
        }
    }
}

/// A transcription with its measured latency and detected language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transcript {
    pub text: String,
    pub language: Option<Language>,
    pub confidence: f64,
    pub is_final: bool,
    pub latency_ms: i64,
}
