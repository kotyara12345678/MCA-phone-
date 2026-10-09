//! Metrics wiring: one shared sink for the engine, gRPC and HTTP layers.

use std::sync::Arc;

use mca_core::metrics::{MetricsRegistry, NoopMetrics, SharedMetrics};

/// Builds the process-wide metric sink. When metrics are disabled the engine
/// still runs, it just records nothing — `/metrics` reports the same `disabled`
/// answer for both.
pub fn build(enabled: bool) -> SharedMetrics {
    if enabled {
        Arc::new(MetricsRegistry::new(true))
    } else {
        Arc::new(NoopMetrics)
    }
}

/// Prometheus exposition text, when available.
pub fn render(metrics: &SharedMetrics) -> Option<String> {
    metrics.render()
}
