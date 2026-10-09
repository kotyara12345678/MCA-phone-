//! Errors surfaced by the voice layer.

/// Voice-service error. `code()` is the single source of truth for the gRPC
/// status translation, mirroring `AppError::code()` in ai-service.
#[derive(Debug, thiserror::Error)]
pub enum VoiceError {
    #[error("unknown call `{0}`")]
    UnknownCall(String),
    #[error("call `{0}` is `{1}`, not open for media")]
    NotOpen(String, String),
    #[error("invalid request: {0}")]
    Invalid(String),
    #[error("ai-service upstream failed: {0}")]
    Upstream(String),
    #[error("speech provider failed: {0}")]
    Speech(String),
    #[error("telephony provider failed: {0}")]
    Telephony(String),
}

impl VoiceError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownCall(_) => "not_found",
            Self::NotOpen(..) => "call_state",
            Self::Invalid(_) => "invalid_request",
            Self::Upstream(_) | Self::Speech(_) | Self::Telephony(_) => "internal",
        }
    }

    pub(crate) fn from_speech(err: mca_core::error::ProviderError) -> Self {
        Self::Speech(err.to_string())
    }

    pub(crate) fn from_telephony(err: mca_core::error::ProviderError) -> Self {
        Self::Telephony(err.to_string())
    }
}
