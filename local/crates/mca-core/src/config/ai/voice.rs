//! Speech-side configuration: which language the caller defaults to and which
//! voice/format the TTS adapter requests.

use crate::config::env::var;
use crate::config::provider::RemoteProviderConfig;
use crate::validation::ValidationError;

/// STT-specific knobs.
#[derive(Debug, Clone, PartialEq)]
pub struct SttConfig {
    pub remote: RemoteProviderConfig,
    pub default_language: String,
}

impl SttConfig {
    pub fn from_env() -> Result<Self, ValidationError> {
        Ok(Self {
            remote: RemoteProviderConfig::from_env("STT")?,
            default_language: var("STT_LANGUAGE").unwrap_or_else(|| "ru".to_string()),
        })
    }
}

/// TTS-specific knobs.
#[derive(Debug, Clone, PartialEq)]
pub struct TtsConfig {
    pub remote: RemoteProviderConfig,
    pub voice: String,
    pub format: String,
}

impl TtsConfig {
    pub fn from_env() -> Result<Self, ValidationError> {
        // Reject an unusable format at boot rather than at the first reply.
        let format = var("TTS_FORMAT").unwrap_or_else(|| "pcm16".to_string());
        if crate::ports::stt::AudioFormat::from_name(&format).is_none() {
            return Err(ValidationError::invalid("TTS_FORMAT", format));
        }
        Ok(Self {
            remote: RemoteProviderConfig::from_env("TTS")?,
            voice: var("TTS_VOICE").unwrap_or_else(|| "alloy".to_string()),
            format,
        })
    }
}
