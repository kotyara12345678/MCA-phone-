//! Fixed-window latency accumulator.
//!
//! Allocation-free, lock-free and sized for the per-session turn counts we
//! actually see (tens, not thousands), so it stays a bounded buffer rather
//! than an unbounded log.

mod sample;

pub use sample::{LatencyKind, LatencySample};

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct LatencyWindow {
    inner: Arc<WindowInner>,
}

#[derive(Debug, Default)]
struct WindowInner {
    stt: AtomicU64,
    llm: AtomicU64,
    tts: AtomicU64,
    total: AtomicU64,
    first_token: AtomicU64,
    count: AtomicU64,
    max_total: AtomicU64,
}

impl LatencyWindow {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one turn's latencies. Negative values are clamped to zero.
    pub fn record(&self, sample: LatencySample) {
        let s = sample.sanitized();
        add(&self.inner.stt, s.stt_ms);
        add(&self.inner.llm, s.llm_ms);
        add(&self.inner.tts, s.tts_ms);
        add(&self.inner.total, s.total_ms);
        add(&self.inner.first_token, s.first_token_ms);
        bump_max(&self.inner.max_total, s.total_ms);
        self.inner.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn count(&self) -> u64 {
        self.inner.count.load(Ordering::Relaxed)
    }

    pub fn average(&self, which: LatencyKind) -> i64 {
        self.sum(which).checked_div(self.count()).unwrap_or(0) as i64
    }

    pub fn max_total_ms(&self) -> i64 {
        self.inner.max_total.load(Ordering::Relaxed) as i64
    }

    fn sum(&self, which: LatencyKind) -> u64 {
        let cell = match which {
            LatencyKind::Stt => &self.inner.stt,
            LatencyKind::Llm => &self.inner.llm,
            LatencyKind::Tts => &self.inner.tts,
            LatencyKind::Total => &self.inner.total,
            LatencyKind::FirstToken => &self.inner.first_token,
        };
        cell.load(Ordering::Relaxed)
    }
}

fn add(cell: &AtomicU64, value: i64) {
    cell.fetch_add(value.max(0) as u64, Ordering::Relaxed);
}

fn bump_max(cell: &AtomicU64, value: i64) {
    cell.fetch_max(value.max(0) as u64, Ordering::Relaxed);
}

#[cfg(test)]
mod tests;
