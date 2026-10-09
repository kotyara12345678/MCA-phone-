use super::store::MetricsRegistry;

/// Series identity: metric name plus a sorted label set. Sorted so the same
/// logical series always collapses to one Prometheus key regardless of the
/// caller's label order.
pub fn series_key(name: &str, labels: &[(&str, &str)]) -> String {
    if labels.is_empty() {
        return name.to_string();
    }
    let mut parts: Vec<String> = labels.iter().map(|(k, v)| format!("{k}={v}")).collect();
    parts.sort();
    format!("{name}{{{}}}", parts.join(","))
}

/// Prometheus text exposition (v0.0.4). Split per metric family so a large
/// scrape never builds one giant intermediate string per family.
pub fn render_counters(counters: &std::collections::HashMap<String, u64>) -> String {
    let mut keys: Vec<&String> = counters.keys().collect();
    keys.sort();
    let mut out = String::with_capacity(keys.len() * 48);
    for key in keys {
        out.push_str(&format!("# TYPE {key} counter\n"));
        out.push_str(&format!("{key} {}\n", counters[key]));
    }
    out
}

pub fn render_gauges(gauges: &std::collections::HashMap<String, i64>) -> String {
    let mut keys: Vec<&String> = gauges.keys().collect();
    keys.sort();
    let mut out = String::with_capacity(keys.len() * 48);
    for key in keys {
        out.push_str(&format!("# TYPE {key} gauge\n"));
        out.push_str(&format!("{key} {}\n", gauges[key]));
    }
    out
}

pub fn render_histograms(
    histograms: &std::collections::HashMap<String, super::histogram::Histogram>,
) -> String {
    let mut keys: Vec<&String> = histograms.keys().collect();
    keys.sort();
    let mut out = String::with_capacity(keys.len() * 256);
    for key in keys {
        render_one_histogram(&mut out, key, &histograms[key]);
    }
    out
}

fn render_one_histogram(out: &mut String, key: &str, histogram: &super::histogram::Histogram) {
    out.push_str(&format!("# TYPE {key} histogram\n"));
    for (i, bound) in super::histogram::BUCKETS_MS.iter().enumerate() {
        out.push_str(&format!(
            "{key}_bucket{{le=\"{bound}\"}} {}\n",
            histogram.counts.get(i).copied().unwrap_or(0)
        ));
    }
    out.push_str(&format!(
        "{key}_bucket{{le=\"+Inf\"}} {}\n",
        histogram.count
    ));
    out.push_str(&format!("{key}_sum {}\n", histogram.sum_ms));
    out.push_str(&format!("{key}_count {}\n", histogram.count));
}

impl MetricsRegistry {
    /// Prometheus text exposition, assembled from the per-family renderers.
    pub fn to_prometheus(&self) -> String {
        let mut out = String::with_capacity(4096);
        if let Ok(counters) = self.counters() {
            out.push_str(&render_counters(&counters));
        }
        if let Ok(gauges) = self.gauges() {
            out.push_str(&render_gauges(&gauges));
        }
        if let Ok(histograms) = self.histograms() {
            out.push_str(&render_histograms(&histograms));
        }
        out
    }
}
