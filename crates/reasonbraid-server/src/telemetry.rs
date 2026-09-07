//! The dev-profile observability slice (`.5.2`; ROADMAP §18.2–18.3, ADR-023):
//! structured operational logs (JSON lines on stderr — no new dependencies,
//! the OpenTelemetry sink stays the ADR-023 trigger) and a minimal in-process
//! metrics registry over the §18.3 minimums that APPLY to the dev profile.
//!
//! The four-record doctrine (ADR-023): these counters are OPERATIONAL
//! signals, never audit facts — the audit record is the durable table row;
//! the registry only counts what the server itself observed.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// One counter (a §18.3 signal).
#[derive(Debug, Default)]
struct Counter(AtomicU64);

impl Counter {
    fn incr(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
    fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

/// The in-process metrics registry: named counters the admin surface reads.
#[derive(Debug, Default)]
pub struct Metrics {
    counters: BTreeMap<&'static str, Counter>,
}

impl Metrics {
    pub fn new() -> Self {
        let mut counters = BTreeMap::new();
        for name in [
            "authorization_denials",
            "idempotency_replays",
            "handshake_refusals",
            "lease_refusals",
            "dead_letters",
            "results_folded",
            "results_rejected",
        ] {
            counters.insert(name, Counter::default());
        }
        Metrics { counters }
    }

    pub fn incr(&self, name: &'static str) {
        if let Some(c) = self.counters.get(name) {
            c.incr();
        }
    }

    pub fn snapshot(&self) -> BTreeMap<String, u64> {
        self.counters
            .iter()
            .map(|(k, c)| ((*k).to_string(), c.get()))
            .collect()
    }
}

/// The process-wide registry (the dev profile's one server process, one
/// registry). Tests read snapshots before/after for measured deltas.
pub fn metrics() -> &'static Metrics {
    static ONCE: std::sync::OnceLock<Metrics> = std::sync::OnceLock::new();
    ONCE.get_or_init(Metrics::new)
}

/// Emit one structured operational log line (JSON on stderr). The fields are
/// the §18.2 correlation surface — never secrets, prompt text, or model
/// output (ADR-023's rules bind the future sink; they bind here too).
#[macro_export]
macro_rules! log_event {
    ($event:expr, $($key:expr => $value:expr),* $(,)?) => {{
        let line = serde_json::json!({
            "ts": chrono::Utc::now().to_rfc3339(),
            "event": $event,
            $($key: $value),*
        });
        eprintln!("{}", line);
    }};
}
