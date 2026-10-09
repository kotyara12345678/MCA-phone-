use std::time::Duration;

use crate::config::provider::ProviderKind;

/// Builds the single shared `reqwest::Client` for outbound AI traffic.
///
/// Connection pooling and TLS handshakes are expensive; creating a client per
/// request would dominate latency. Timeouts are also enforced here so no call
/// can outlive them even if a provider-specific timeout is larger.
pub fn build_http_client(
    kind: ProviderKind,
    default_timeout: Duration,
) -> Result<reqwest::Client, crate::error::ProviderError> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_millis(2_000))
        .pool_max_idle_per_host(8)
        .pool_idle_timeout(Duration::from_secs(60))
        .timeout(default_timeout)
        .user_agent(concat!("mca-logistics/", env!("CARGO_PKG_VERSION")));
    if kind == ProviderKind::Fake {
        // Fakes never reach the network, but keep the client constructible.
        builder = builder.no_proxy();
    }
    builder
        .build()
        .map_err(|e| crate::error::ProviderError::Transport {
            provider: "http",
            message: format!("cannot build http client: {e}"),
        })
}

/// Streaming variant used for chunked TTS output.
pub fn build_streaming_client(
    default_timeout: Duration,
) -> Result<reqwest::Client, crate::error::ProviderError> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_millis(2_000))
        .pool_max_idle_per_host(8)
        .timeout(default_timeout)
        .user_agent(concat!("mca-logistics/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| crate::error::ProviderError::Transport {
            provider: "http",
            message: format!("cannot build streaming client: {e}"),
        })
}
