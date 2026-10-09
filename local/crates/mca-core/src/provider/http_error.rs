//! Shared translation from HTTP failures into [`ProviderError`].
//!
//! Every adapter classifies the same way on purpose: one place decides what
//! counts as retryable, so `RetryPolicy` behaves identically for STT, TTS and
//! the LLM.

use crate::error::ProviderError;

/// A transport-level failure (connect, TLS, read, body decode).
pub fn classify_transport(provider: &'static str, err: reqwest::Error) -> ProviderError {
    if err.is_timeout() {
        ProviderError::Timeout {
            provider,
            timeout_ms: 0,
        }
    } else {
        ProviderError::Transport {
            provider,
            message: err.to_string(),
        }
    }
}

/// A non-2xx response. The body is truncated so a verbose HTML error page
/// cannot flood the logs.
pub fn classify_status(
    provider: &'static str,
    status: u16,
    body: &str,
    attempt: u32,
) -> ProviderError {
    if status == 429 {
        return ProviderError::RateLimited { provider, attempt };
    }
    let message = body.chars().take(300).collect::<String>();
    ProviderError::Status {
        provider,
        status,
        message,
    }
}
