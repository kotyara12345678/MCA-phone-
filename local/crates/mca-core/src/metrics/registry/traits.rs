//! The trait every metric sink implements.

/// Counter/histogram abstraction with no external dependency in `mca-core`, so
/// unit tests never need a Prometheus client. The services install a concrete
/// implementation; `/metrics` and Prometheus scrape are an adapter concern.
#[async_trait::async_trait]
pub trait Metrics: Send + Sync + 'static {
    fn increment_counter(&self, name: &str, labels: &[(&str, &str)]);
    fn observe(&self, name: &str, value_ms: i64, labels: &[(&str, &str)]);
    fn set_gauge(&self, name: &str, value: i64, labels: &[(&str, &str)]);
    /// Renders the exposition format. `None` when metrics are disabled.
    fn render(&self) -> Option<String> {
        None
    }
}

pub type SharedMetrics = std::sync::Arc<dyn Metrics>;
