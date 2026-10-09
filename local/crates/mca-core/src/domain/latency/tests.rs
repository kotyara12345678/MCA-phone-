use super::*;

#[test]
fn averages_across_turns() {
    let w = LatencyWindow::new();
    w.record(LatencySample {
        llm_ms: 100,
        total_ms: 200,
        ..Default::default()
    });
    w.record(LatencySample {
        llm_ms: 300,
        total_ms: 400,
        ..Default::default()
    });
    assert_eq!(w.average(LatencyKind::Llm), 200);
    assert_eq!(w.average(LatencyKind::Total), 300);
    assert_eq!(w.count(), 2);
}

#[test]
fn negative_values_are_clamped() {
    let w = LatencyWindow::new();
    w.record(LatencySample {
        llm_ms: -50,
        total_ms: -1,
        ..Default::default()
    });
    assert_eq!(w.average(LatencyKind::Llm), 0);
    assert_eq!(w.max_total_ms(), 0);
}

#[test]
fn empty_window_reports_zero() {
    let w = LatencyWindow::new();
    assert_eq!(w.average(LatencyKind::Total), 0);
    assert_eq!(w.count(), 0);
}
