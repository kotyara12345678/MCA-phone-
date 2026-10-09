//! Failures of an outbound AI/CRM dependency. Never fatal on their own — the
//! dialogue engine always has a deterministic fallback path.

/// Failures of an outbound AI/CRM dependency. Never fatal on their own — the
/// dialogue engine always has a deterministic fallback path.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ProviderError {
    #[error("provider `{provider}` timed out after {timeout_ms}ms")]
    Timeout {
        provider: &'static str,
        timeout_ms: u64,
    },
    #[error("provider `{provider}` transport failure: {message}")]
    Transport {
        provider: &'static str,
        message: String,
    },
    #[error("provider `{provider}` returned {status}: {message}")]
    Status {
        provider: &'static str,
        status: u16,
        message: String,
    },
    #[error("provider `{provider}` rate limited (attempt {attempt})")]
    RateLimited {
        provider: &'static str,
        attempt: u32,
    },
    #[error("provider `{provider}` returned a malformed payload: {message}")]
    InvalidResponse {
        provider: &'static str,
        message: String,
    },
    #[error("provider `{provider}` is not configured")]
    NotConfigured { provider: &'static str },
    #[error("operation cancelled: {0}")]
    Cancelled(String),
}

impl ProviderError {
    /// Classifies the error so retry/backoff and metric labelling stay honest.
    pub fn kind(&self) -> ProviderErrorKind {
        match self {
            Self::Timeout { .. } | Self::Cancelled(_) => ProviderErrorKind::Timeout,
            Self::RateLimited { .. } => ProviderErrorKind::RateLimited,
            Self::Status { status, .. } if *status >= 500 => ProviderErrorKind::Server,
            Self::InvalidResponse { .. } => ProviderErrorKind::Malformed,
            _ => ProviderErrorKind::Other,
        }
    }

    /// Whether a bounded retry with backoff is worth attempting. A malformed
    /// payload is excluded on purpose: it would fail identically next time and
    /// only add latency to a live phone call.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind(),
            ProviderErrorKind::Timeout | ProviderErrorKind::RateLimited | ProviderErrorKind::Server
        )
    }
}

/// Retry classification buckets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderErrorKind {
    Timeout,
    RateLimited,
    Server,
    Malformed,
    Other,
}
