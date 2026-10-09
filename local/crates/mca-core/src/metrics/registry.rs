//! Metric surface used by both services: a small abstraction (`Metrics`) plus a
//! fixed-bucket registry. Kept dependency-free so unit tests never need a
//! Prometheus client — `/metrics` and scraping stay adapter concerns.

mod histogram;
mod noop;
mod render;
mod shield;
mod store;
mod traits;

pub use histogram::{Histogram, BUCKETS_MS};
pub use noop::{NoopMetrics, TurnCounter};
pub use render::{render_counters, render_gauges, render_histograms, series_key};
pub use store::MetricsRegistry;
pub use traits::{Metrics, SharedMetrics};

#[cfg(test)]
mod tests;
