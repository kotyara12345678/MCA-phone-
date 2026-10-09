mod run;

pub use run::with_retry;

use std::time::Duration;

/// Bounded retry with exponential backoff for outbound AI calls.
///
/// Deliberately not generic middleware: the classification comes from
/// [`crate::error::ProviderError::is_retryable`], so a malformed model response
/// is *not* retried (it would fail identically) while a 503 or timeout is.
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    /// Two attempts, 200ms apart, capped at 2s: enough to ride out a provider
    /// blip without ever adding noticeable latency to a phone conversation.
    fn default() -> Self {
        Self::new(2, 200, 2_000)
    }
}

impl RetryPolicy {
    pub fn new(max_attempts: u32, base_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            base_delay: Duration::from_millis(base_delay_ms.max(1)),
            max_delay: Duration::from_millis(max_delay_ms.max(1)),
        }
    }

    /// A policy that never retries — used for non-idempotent calls.
    pub fn none() -> Self {
        Self::new(1, 1, 1)
    }

    /// Delay before attempt `attempt` (1-based), capped at `max_delay`.
    pub fn delay_for(&self, attempt: u32) -> Duration {
        if attempt <= 1 {
            return Duration::ZERO;
        }
        let factor = 1u32 << (attempt - 2).min(10);
        self.base_delay.saturating_mul(factor).min(self.max_delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_delay_before_first_retry() {
        let p = RetryPolicy::new(3, 100, 1000);
        assert_eq!(p.delay_for(1), Duration::ZERO);
    }

    #[test]
    fn exponential_backoff_capped() {
        let p = RetryPolicy::new(5, 100, 250);
        assert_eq!(p.delay_for(2), Duration::from_millis(100));
        assert_eq!(p.delay_for(3), Duration::from_millis(200));
        assert_eq!(p.delay_for(4), Duration::from_millis(250));
    }

    #[test]
    fn minimum_one_attempt() {
        assert_eq!(RetryPolicy::new(0, 10, 10).max_attempts, 1);
    }
}
