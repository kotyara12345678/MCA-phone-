use super::*;

#[test]
fn counters_accumulate_per_label_set() {
    let m = MetricsRegistry::new(true);
    m.increment_counter("qualified_calls", &[("result", "qualified")]);
    m.increment_counter("qualified_calls", &[("result", "qualified")]);
    assert_eq!(
        m.counter_value("qualified_calls", &[("result", "qualified")]),
        2
    );
}

#[test]
fn disabled_registry_records_nothing() {
    let m = MetricsRegistry::new(false);
    m.increment_counter("x", &[]);
    assert_eq!(m.counter_value("x", &[]), 0);
    assert!(m.render().is_none());
}

#[test]
fn histogram_percentile_is_ordered() {
    let m = MetricsRegistry::new(true);
    for _ in 0..90 {
        m.observe("llm_latency_ms", 100, &[]);
    }
    for _ in 0..10 {
        m.observe("llm_latency_ms", 2000, &[]);
    }
    let h = m.histogram("llm_latency_ms", &[]).unwrap();
    assert!(h.percentile(0.5) <= h.percentile(0.99));
}

#[test]
fn prometheus_output_has_buckets() {
    let m = MetricsRegistry::new(true);
    m.observe("tts_latency_ms", 42, &[]);
    let text = m.to_prometheus();
    assert!(text.contains("tts_latency_ms_bucket{le=\"50\"}"));
    assert!(text.contains("# TYPE"));
}
