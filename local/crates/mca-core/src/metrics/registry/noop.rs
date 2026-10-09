use std::sync::atomic::{AtomicI64, Ordering};

/// No-op implementation for services that disable metrics entirely.
#[derive(Debug, Default, Clone)]
pub struct NoopMetrics;

#[async_trait::async_trait]
impl super::Metrics for NoopMetrics {
    fn increment_counter(&self, _name: &str, _labels: &[(&str, &str)]) {}
    fn observe(&self, _name: &str, _value_ms: i64, _labels: &[(&str, &str)]) {}
    fn set_gauge(&self, _name: &str, _value: i64, _labels: &[(&str, &str)]) {}
}

/// Total number of turns the dialogue engine has processed in this process.
#[derive(Debug, Default)]
pub struct TurnCounter(AtomicI64);

impl TurnCounter {
    pub fn inc(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> i64 {
        self.0.load(Ordering::Relaxed)
    }
}
