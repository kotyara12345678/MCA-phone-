//! Guard accessors used by the Prometheus renderer. Kept in their own module
//! so `store.rs` and `render.rs` both stay under the file-size limit while the
//! poisoned-lock error message stays in one place.

use std::collections::HashMap;
use std::sync::MutexGuard;

use super::histogram::Histogram;
use super::store::MetricsRegistry;

impl MetricsRegistry {
    pub(crate) fn counters(&self) -> Result<MutexGuard<'_, HashMap<String, u64>>, String> {
        self.counters
            .lock()
            .map_err(|_| "counters poisoned".to_string())
    }

    pub(crate) fn gauges(&self) -> Result<MutexGuard<'_, HashMap<String, i64>>, String> {
        self.gauges
            .lock()
            .map_err(|_| "gauges poisoned".to_string())
    }

    pub(crate) fn histograms(&self) -> Result<MutexGuard<'_, HashMap<String, Histogram>>, String> {
        self.histograms
            .lock()
            .map_err(|_| "histograms poisoned".to_string())
    }
}
