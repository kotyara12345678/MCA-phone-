//! Fixed-bucket in-memory recorder used in production and tests alike.
//!
//! No allocation on the hot path, bounded memory, and enough fidelity for the
//! latency SLOs we actually care about. The enabled bit is an atomic so the
//! trait impl stays `&self` and services can share one registry.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use super::histogram::Histogram;
use super::render::series_key;
use super::traits::Metrics;

#[derive(Debug, Default)]
pub struct MetricsRegistry {
    pub(super) counters: Mutex<HashMap<String, u64>>,
    pub(super) gauges: Mutex<HashMap<String, i64>>,
    pub(super) histograms: Mutex<HashMap<String, Histogram>>,
    pub(super) enabled: AtomicU64,
}

impl MetricsRegistry {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: AtomicU64::new(u64::from(enabled)),
            ..Default::default()
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) == 1
    }

    /// Current counter value, for assertions in tests.
    pub fn counter_value(&self, name: &str, labels: &[(&str, &str)]) -> u64 {
        let key = series_key(name, labels);
        self.counters
            .lock()
            .map(|c| c.get(&key).copied().unwrap_or(0))
            .unwrap_or(0)
    }

    /// Histogram snapshot for assertions in tests.
    pub fn histogram(&self, name: &str, labels: &[(&str, &str)]) -> Option<Histogram> {
        let key = series_key(name, labels);
        self.histograms
            .lock()
            .ok()
            .and_then(|h| h.get(&key).cloned())
    }
}

#[async_trait::async_trait]
impl Metrics for MetricsRegistry {
    fn increment_counter(&self, name: &str, labels: &[(&str, &str)]) {
        if !self.is_enabled() {
            return;
        }
        let key = series_key(name, labels);
        if let Ok(mut c) = self.counters.lock() {
            *c.entry(key).or_insert(0) += 1;
        }
    }

    fn observe(&self, name: &str, value_ms: i64, labels: &[(&str, &str)]) {
        if !self.is_enabled() {
            return;
        }
        let key = series_key(name, labels);
        if let Ok(mut h) = self.histograms.lock() {
            h.entry(key)
                .or_insert_with(Histogram::new)
                .observe(value_ms);
        }
    }

    fn set_gauge(&self, name: &str, value: i64, labels: &[(&str, &str)]) {
        if !self.is_enabled() {
            return;
        }
        let key = series_key(name, labels);
        if let Ok(mut g) = self.gauges.lock() {
            g.insert(key, value);
        }
    }

    fn render(&self) -> Option<String> {
        self.is_enabled().then(|| self.to_prometheus())
    }
}
