//! Service and dependency health, with freshness (`SIGNOFF-REPAIR.4.6.1.4`;
//! ROADMAP §18.5, its first bullet).
//!
//! Before this module nothing probed a dependency after boot: `rb-server`
//! connected to PostgreSQL, loaded the CA through the declared secret store and
//! validated the publication root ONCE, and from then on a dependency that went
//! away was noticed only by the request that failed on it.
//!
//! The shape, and why:
//!
//! - **A prober, not a request-time check.** [`spawn_prober`] probes every
//!   dependency on a fixed interval and records each outcome with the instant it
//!   was OBSERVED. A read reports those observations; it never probes. So a read
//!   is cheap and cannot be used to drive load onto a dependency, and — the point
//!   of §18.5's *with freshness* — an observation that stops being refreshed
//!   AGES, and the read says so. A prober that has stalled is itself a fault,
//!   and a request-time check could never show it.
//! - **Up, down, stale, unobserved.** An observation older than
//!   [`HealthMonitor::stale_after`] is `stale` whatever it said, and a dependency
//!   never probed is `unobserved`. Neither is `up`: an operator is never told a
//!   dependency is up on evidence the system no longer has.
//! - **The read is UNAUTHENTICATED, and discloses accordingly.** Every other
//!   operator read authorizes against PostgreSQL, so a health route gated the
//!   same way could never report PostgreSQL down — the one fact it most exists
//!   to report. `GET /v1/health` therefore answers without authority, and says
//!   only what that justifies: each dependency's NAME (the book documents them),
//!   its state, and its instants. ⛔ The error text of a failed probe is NOT in
//!   the response — it can carry a host, a path or a driver message — and goes
//!   to the structured log on every state change instead, where the operator
//!   who holds the terminal reads it
//!   (`docs/decisions/2026-09-22_health-is-read-without-authority-and-says-only-state-and-age.md`).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;

/// How often the shipped prober probes. The interval is the freshness a
/// healthy deployment shows: every observation is at most this old, plus the
/// probe's own duration.
pub const PROBE_INTERVAL: Duration = Duration::from_secs(10);

/// The longest one probe may take before it counts as `down`. A dependency
/// that answers too slowly to serve a request is not up.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// One runtime dependency of the control plane, with what probing it means.
#[derive(Clone)]
pub enum Dependency {
    /// The control plane's store: `SELECT 1` answers.
    Postgres(PgPool),
    /// The declared secret store answers for the CA material it holds — the
    /// read the server itself makes at boot, repeated.
    SecretStore {
        store: crate::secret_store::SecretStore,
        pool: PgPool,
    },
    /// The workload-identity CA's certificate is inside its validity window, so
    /// the leaves it signs will chain.
    ServerCa { cert_der: Arc<Vec<u8>> },
    /// The declared publication root is still a usable directory — the same
    /// check the boot makes before it mutates anything.
    PublicationRoot(PathBuf),
}

impl Dependency {
    /// The name the read reports. Stable: an operator scripts against it.
    pub fn name(&self) -> &'static str {
        match self {
            Dependency::Postgres(_) => "postgres",
            Dependency::SecretStore { .. } => "secret_store",
            Dependency::ServerCa { .. } => "server_ca",
            Dependency::PublicationRoot(_) => "publication_root",
        }
    }

    /// Probe once, bounded by [`PROBE_TIMEOUT`]. `Err` carries the reason,
    /// which is logged and never returned to a caller.
    pub async fn probe(&self) -> Result<(), String> {
        match tokio::time::timeout(PROBE_TIMEOUT, self.probe_unbounded()).await {
            Ok(outcome) => outcome,
            Err(_) => Err(format!("no answer within {} ms", PROBE_TIMEOUT.as_millis())),
        }
    }

    async fn probe_unbounded(&self) -> Result<(), String> {
        match self {
            Dependency::Postgres(pool) => sqlx::query_scalar::<_, i32>("SELECT 1")
                .fetch_one(pool)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string()),
            Dependency::SecretStore { store, pool } => match store.load_ca_material(pool).await {
                Ok(Some(_)) => Ok(()),
                Ok(None) => Err("the store holds no CA material".to_owned()),
                Err(e) => Err(e.to_string()),
            },
            Dependency::ServerCa { cert_der } => {
                let (not_before, not_after) = crate::ca::validity_window(cert_der)?;
                let now = Utc::now();
                if now < not_before {
                    Err(format!("not valid before {}", not_before.to_rfc3339()))
                } else if now >= not_after {
                    Err(format!("expired at {}", not_after.to_rfc3339()))
                } else {
                    Ok(())
                }
            }
            Dependency::PublicationRoot(root) => crate::publisher::validate_root(root)
                .map(|_| ())
                .map_err(|e| e.to_string()),
        }
    }
}

/// What the last probe of one dependency saw.
#[derive(Debug, Clone, PartialEq)]
struct Observation {
    up: bool,
    observed_at: DateTime<Utc>,
    last_up_at: Option<DateTime<Utc>>,
}

/// A dependency's state as a read reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyState {
    Up,
    Down,
    Stale,
    Unobserved,
}

impl DependencyState {
    fn as_str(self) -> &'static str {
        match self {
            DependencyState::Up => "up",
            DependencyState::Down => "down",
            DependencyState::Stale => "stale",
            DependencyState::Unobserved => "unobserved",
        }
    }
}

/// The observations, shared between the prober that writes them and the route
/// that reads them.
#[derive(Debug)]
pub struct HealthMonitor {
    names: Vec<&'static str>,
    observations: Mutex<BTreeMap<&'static str, Observation>>,
    stale_after: Duration,
}

impl HealthMonitor {
    /// A monitor over `dependencies`, calling an observation stale once it is
    /// older than `stale_after`.
    pub fn new(dependencies: &[Dependency], stale_after: Duration) -> Arc<Self> {
        Arc::new(Self {
            names: dependencies.iter().map(Dependency::name).collect(),
            observations: Mutex::new(BTreeMap::new()),
            stale_after,
        })
    }

    /// The staleness bound this monitor applies.
    pub fn stale_after(&self) -> Duration {
        self.stale_after
    }

    /// Record one probe outcome observed at `at`. A change of state is logged
    /// with the reason, which is the only place a failed probe's text goes.
    pub fn record(&self, name: &'static str, outcome: &Result<(), String>, at: DateTime<Utc>) {
        let up = outcome.is_ok();
        let mut observations = self
            .observations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = observations.get(name).cloned();
        let last_up_at = if up {
            Some(at)
        } else {
            previous.as_ref().and_then(|p| p.last_up_at)
        };
        if previous.as_ref().map(|p| p.up) != Some(up) {
            crate::log_event!(
                "dependency_health_changed",
                "dependency" => name,
                "state" => if up { "up" } else { "down" },
                "reason" => outcome.as_ref().err().cloned(),
            );
        }
        observations.insert(
            name,
            Observation {
                up,
                observed_at: at,
                last_up_at,
            },
        );
    }

    /// The report as of `now`: the JSON body and whether every dependency is
    /// up AND fresh. `now` is the reader's clock; the observations are the
    /// prober's.
    pub fn report_at(&self, now: DateTime<Utc>) -> (bool, Value) {
        let observations = self
            .observations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let stale_after =
            chrono::Duration::from_std(self.stale_after).unwrap_or(chrono::Duration::MAX);
        let mut healthy = true;
        let dependencies: Vec<Value> = self
            .names
            .iter()
            .map(|name| {
                let observation = observations.get(name);
                let state = match observation {
                    None => DependencyState::Unobserved,
                    Some(o) if now - o.observed_at > stale_after => DependencyState::Stale,
                    Some(o) if o.up => DependencyState::Up,
                    Some(_) => DependencyState::Down,
                };
                healthy &= state == DependencyState::Up;
                json!({
                    "name": name,
                    "state": state.as_str(),
                    "observed_at": observation.map(|o| o.observed_at.to_rfc3339()),
                    "age_ms": observation.map(|o| (now - o.observed_at).num_milliseconds()),
                    "last_up_at": observation.and_then(|o| o.last_up_at).map(|t| t.to_rfc3339()),
                })
            })
            .collect();
        (
            healthy,
            json!({
                "status": if healthy { "ok" } else { "degraded" },
                "checked_at": now.to_rfc3339(),
                "stale_after_ms": self.stale_after.as_millis() as u64,
                "dependencies": dependencies,
            }),
        )
    }
}

/// Probe every dependency once, CONCURRENTLY, and record each outcome at the
/// instant its own probe finished.
///
/// ⚠️ Concurrent because a real run showed what sequential probing does: with
/// PostgreSQL stopped, its probe spent the whole `PROBE_TIMEOUT`, and every
/// dependency probed after it kept showing the previous round's observation for
/// that long. The age made that visible, but one hung dependency should not
/// delay the news about the others. A round now takes the slowest probe, never
/// the sum of them.
pub async fn probe_all(monitor: &HealthMonitor, dependencies: &[Dependency]) {
    let rounds = dependencies.iter().map(|dependency| async move {
        let outcome = dependency.probe().await;
        (dependency.name(), outcome, Utc::now())
    });
    for (name, outcome, at) in futures_util::future::join_all(rounds).await {
        monitor.record(name, &outcome, at);
    }
}

/// Probe every dependency now and then every `interval`, for the life of the
/// process. The first round runs before this returns, so a server that has
/// started listening already has an observation for each dependency.
pub async fn spawn_prober(
    monitor: Arc<HealthMonitor>,
    dependencies: Vec<Dependency>,
    interval: Duration,
) -> tokio::task::JoinHandle<()> {
    probe_all(&monitor, &dependencies).await;
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        ticker.tick().await; // the first tick is immediate; that round ran above
        loop {
            ticker.tick().await;
            probe_all(&monitor, &dependencies).await;
        }
    })
}

/// `GET /v1/health` — 200 when every dependency is up and fresh, 503
/// otherwise, with the same body either way. Unauthenticated; see the module
/// comment for what that permits it to say.
async fn read_health(State(monitor): State<Arc<HealthMonitor>>) -> Response {
    let (healthy, body) = monitor.report_at(Utc::now());
    let status = if healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(body)).into_response()
}

/// The health route over `monitor`.
pub fn health_router(monitor: Arc<HealthMonitor>) -> Router {
    Router::new()
        .route("/v1/health", get(read_health))
        .with_state(monitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor() -> Arc<HealthMonitor> {
        HealthMonitor::new(
            &[Dependency::PublicationRoot(PathBuf::from("/unused"))],
            Duration::from_secs(30),
        )
    }

    /// Never probed is `unobserved`, and that is not healthy: the read never
    /// calls a dependency up without an observation saying so.
    #[test]
    fn an_unprobed_dependency_is_unobserved_and_not_healthy() {
        let (healthy, body) = monitor().report_at(Utc::now());
        assert!(!healthy);
        assert_eq!(body["status"], json!("degraded"));
        assert_eq!(body["dependencies"][0]["state"], json!("unobserved"));
        assert_eq!(body["dependencies"][0]["observed_at"], Value::Null);
    }

    /// An `up` observation reads `up` while fresh and `stale` once older than
    /// the bound — whatever it said. The age is reported either way.
    #[test]
    fn an_up_observation_goes_stale_when_it_is_not_refreshed() {
        let monitor = monitor();
        let at = Utc::now();
        monitor.record("publication_root", &Ok(()), at);
        let (healthy, body) = monitor.report_at(at + chrono::Duration::seconds(30));
        assert!(healthy, "exactly at the bound is still fresh: {body}");
        assert_eq!(body["dependencies"][0]["state"], json!("up"));
        assert_eq!(body["dependencies"][0]["age_ms"], json!(30_000));
        let (healthy, body) = monitor.report_at(at + chrono::Duration::seconds(31));
        assert!(!healthy);
        assert_eq!(body["dependencies"][0]["state"], json!("stale"));
        assert_eq!(body["dependencies"][0]["age_ms"], json!(31_000));
    }

    /// A down observation keeps the last instant the dependency WAS up, which
    /// is how long an operator has been without it.
    #[test]
    fn a_down_observation_keeps_the_last_up_instant() {
        let monitor = monitor();
        let up_at = Utc::now();
        monitor.record("publication_root", &Ok(()), up_at);
        let down_at = up_at + chrono::Duration::seconds(10);
        monitor.record("publication_root", &Err("gone".to_owned()), down_at);
        let (healthy, body) = monitor.report_at(down_at);
        assert!(!healthy);
        let dependency = &body["dependencies"][0];
        assert_eq!(dependency["state"], json!("down"));
        assert_eq!(dependency["last_up_at"], json!(up_at.to_rfc3339()));
        assert_eq!(dependency["observed_at"], json!(down_at.to_rfc3339()));
        assert!(
            !body.to_string().contains("gone"),
            "a probe's error text is never in the response: {body}"
        );
    }
}
