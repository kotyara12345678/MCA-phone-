use std::time::{SystemTime, UNIX_EPOCH};

/// Milliseconds since the Unix epoch. Saturates to 0 if the clock is before it.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Saturating elapsed-milliseconds helper used by latency instrumentation.
pub fn elapsed_ms(start: std::time::Instant) -> i64 {
    start.elapsed().as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_ms_is_positive() {
        assert!(now_ms() > 1_600_000_000_000);
    }

    #[test]
    fn elapsed_ms_grows() {
        let start = std::time::Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(elapsed_ms(start) >= 4);
    }
}
