use crate::error::ProviderError;
use crate::provider::retry::RetryPolicy;
use std::future::Future;
use std::time::{Duration, Instant};
use tracing::warn;

/// Runs `op` under the policy: bounded attempts, exponential backoff and a hard
/// timeout per attempt. Returns the value plus its elapsed wall time so the
/// caller records latency without a second clock read.
pub async fn with_retry<F, Fut, T>(
    provider: &'static str,
    policy: &RetryPolicy,
    timeout: Duration,
    mut op: F,
) -> Result<(T, i64), ProviderError>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, ProviderError>>,
{
    let mut attempt = 1;
    loop {
        let started = Instant::now();
        let result = match tokio::time::timeout(timeout, op(attempt)).await {
            Ok(inner) => inner,
            Err(_) => {
                let err = ProviderError::Timeout {
                    provider,
                    timeout_ms: timeout.as_millis() as u64,
                };
                if attempt < policy.max_attempts {
                    warn!(provider, attempt, "provider timeout, retrying");
                    sleep_backoff(policy, attempt).await;
                    attempt += 1;
                    continue;
                }
                return Err(err);
            }
        };
        match result {
            Ok(value) => return Ok((value, started.elapsed().as_millis() as i64)),
            Err(err) => {
                if attempt < policy.max_attempts && err.is_retryable() {
                    warn!(
                        provider,
                        attempt,
                        kind = ?err.kind(),
                        error = %err,
                        "provider call failed, retrying"
                    );
                    sleep_backoff(policy, attempt).await;
                    attempt += 1;
                    continue;
                }
                return Err(err);
            }
        }
    }
}

async fn sleep_backoff(policy: &RetryPolicy, attempt: u32) {
    let delay = policy.delay_for(attempt);
    if !delay.is_zero() {
        tokio::time::sleep(delay).await;
    }
}
