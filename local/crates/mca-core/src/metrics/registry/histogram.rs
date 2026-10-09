/// Latency buckets in milliseconds. Coarse on purpose: a phone conversation
/// needs percentiles, not microsecond precision.
pub const BUCKETS_MS: [i64; 10] = [25, 50, 100, 200, 400, 800, 1200, 2000, 3000, 5000];

#[derive(Debug, Default, Clone)]
pub struct Histogram {
    pub counts: Vec<u64>,
    pub sum_ms: u64,
    pub count: u64,
}

impl Histogram {
    pub(super) fn new() -> Self {
        Self {
            counts: vec![0; BUCKETS_MS.len() + 1],
            sum_ms: 0,
            count: 0,
        }
    }

    pub(super) fn observe(&mut self, value_ms: i64) {
        let v = value_ms.max(0) as u64;
        self.sum_ms += v;
        self.count += 1;
        let idx = BUCKETS_MS
            .iter()
            .position(|b| v <= *b as u64)
            .unwrap_or(BUCKETS_MS.len());
        self.counts[idx] += 1;
    }

    /// Linear-interpolated percentile estimate from the bucket counts.
    pub fn percentile(&self, p: f64) -> i64 {
        if self.count == 0 {
            return 0;
        }
        let target = (self.count as f64 * p).ceil().max(1.0) as u64;
        let mut cumulative = 0u64;
        for (i, c) in self.counts.iter().enumerate() {
            cumulative += c;
            if cumulative >= target {
                let upper = BUCKETS_MS
                    .get(i)
                    .copied()
                    .unwrap_or(*BUCKETS_MS.last().unwrap());
                let lower = if i == 0 { 0 } else { BUCKETS_MS[i - 1] };
                return (lower + upper) / 2;
            }
        }
        *BUCKETS_MS.last().unwrap()
    }
}
