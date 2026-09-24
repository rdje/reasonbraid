//! Service and dependency health (`SIGNOFF-REPAIR.4.6.1.4`; ROADMAP §18.5).
//!
//! Run with `scripts/run_pg_tests.sh health`. Without `DATABASE_URL` these
//! skip, so `make check` stays green offline.
//!
//! ⭐ Every `down` below is produced by a control that REALLY stops the
//! dependency — a database dropped under its pool, a directory removed, a
//! certificate whose validity window has closed, a store asked for material it
//! does not hold. None is an asserted value, because a health surface that is
//! only ever shown `up` has not been shown to work.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::sync::Arc;
use std::time::Duration;

use reasonbraid_core::fixture::Fixture;
use reasonbraid_server::ca::ensure_server_ca;
use reasonbraid_server::health::{self, Dependency, HealthMonitor};
use reasonbraid_server::secret_store::SecretStore;
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

async fn pool() -> Option<PgPool> {
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    Some(pool)
}

/// A real certificate whose validity window closed a day ago.
/// A CA whose own window closed yesterday, as the set holds one
/// (`SIGNOFF-REPAIR.4.1.8.2`: the probe reads the set's issuer, not bytes).
fn expired_ca() -> Arc<reasonbraid_server::ca::CaSet> {
    let mut params = rcgen::CertificateParams::new(vec![]).expect("params");
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    let now = time::OffsetDateTime::now_utc();
    params.not_before = now - time::Duration::days(30);
    params.not_after = now - time::Duration::days(1);
    let key = rcgen::KeyPair::generate().expect("key");
    let cert_der = params
        .self_signed(&key)
        .expect("self-signed")
        .der()
        .to_vec();
    let not_after = reasonbraid_server::ca::validity_window(&cert_der)
        .expect("parses")
        .1;
    let key_der = key.serialize_der();
    Arc::new(reasonbraid_server::ca::CaSet::single(Arc::new(
        reasonbraid_server::ca::ServerCa {
            issuer: rcgen::Issuer::new(params, key),
            cert_der,
            key_der,
            not_after,
        },
    )))
}

async fn read(client: &reqwest::Client, base: &str) -> (u16, Value) {
    let response = client
        .get(format!("{base}/v1/health"))
        .send()
        .await
        .expect("health request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("health json"))
}

fn state_of<'a>(body: &'a Value, name: &str) -> &'a Value {
    body["dependencies"]
        .as_array()
        .expect("the dependency list")
        .iter()
        .find(|d| d["name"] == json!(name))
        .unwrap_or_else(|| panic!("`{name}` is reported: {body}"))
}

/// THE acceptance: each dependency is reported `up` while it is, and `down`
/// once something actually stops it — with the instant it was last up, and
/// without the failed probe's own words.
#[tokio::test]
async fn a_stopped_dependency_is_reported_down_with_its_last_up_instant() {
    let Some(pool) = pool().await else { return };
    let ca = ensure_server_ca(&pool).await.expect("server CA");

    // A database of this control's own, so dropping it stops nothing else.
    let scratch = format!(
        "reasonbraid_health_{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_micros()
    );
    sqlx::query(&format!("CREATE DATABASE {scratch}"))
        .execute(&pool)
        .await
        .expect("create the scratch database on the verified server");
    let scratch_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(pool.connect_options().as_ref().clone().database(&scratch))
        .await
        .expect("connect to the scratch database");

    let fixture = Fixture::create("health-tests", "publication-root").expect("fixture");
    let root = fixture.join("repositories");
    std::fs::create_dir(&root).expect("the publication root");

    let dependencies = vec![
        Dependency::Postgres(scratch_pool.clone()),
        Dependency::SecretStore {
            store: SecretStore::dev(),
            pool: pool.clone(),
        },
        Dependency::ServerCa {
            cas: Arc::new(reasonbraid_server::ca::CaSet::single(Arc::new(ca))),
        },
        Dependency::PublicationRoot(root.clone()),
    ];
    let monitor = HealthMonitor::new(&dependencies, Duration::from_secs(60));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let base = format!("http://{}", listener.local_addr().unwrap());
    let router = health::health_router(monitor.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });
    let client = reqwest::Client::new();

    // (0) Before any probe: nothing is claimed up.
    let (status, body) = read(&client, &base).await;
    assert_eq!(status, 503, "{body}");
    assert_eq!(state_of(&body, "postgres")["state"], json!("unobserved"));

    // (1) Every dependency answers.
    health::probe_all(&monitor, &dependencies).await;
    let (status, body) = read(&client, &base).await;
    assert_eq!(status, 200, "every dependency is up: {body}");
    assert_eq!(body["status"], json!("ok"));
    for name in ["postgres", "secret_store", "server_ca", "publication_root"] {
        let dependency = state_of(&body, name);
        assert_eq!(dependency["state"], json!("up"), "{dependency}");
        assert!(dependency["age_ms"].as_i64().unwrap() >= 0, "{dependency}");
        assert_eq!(dependency["last_up_at"], dependency["observed_at"]);
    }
    let postgres_up_at = state_of(&body, "postgres")["last_up_at"].clone();

    // (2) Stop two of them for real: the database goes away under its pool,
    // and the root directory is removed.
    sqlx::query(&format!("DROP DATABASE {scratch} WITH (FORCE)"))
        .execute(&pool)
        .await
        .expect("drop the scratch database");
    std::fs::remove_dir(&root).expect("remove the publication root");
    health::probe_all(&monitor, &dependencies).await;

    let (status, body) = read(&client, &base).await;
    assert_eq!(status, 503, "a down dependency is not healthy: {body}");
    assert_eq!(body["status"], json!("degraded"));
    for name in ["postgres", "publication_root"] {
        let dependency = state_of(&body, name);
        assert_eq!(dependency["state"], json!("down"), "{dependency}");
        assert_ne!(
            dependency["last_up_at"], dependency["observed_at"],
            "down keeps the instant it was last up: {dependency}"
        );
    }
    assert_eq!(state_of(&body, "postgres")["last_up_at"], postgres_up_at);
    assert_eq!(state_of(&body, "secret_store")["state"], json!("up"));
    assert_eq!(state_of(&body, "server_ca")["state"], json!("up"));
    let text = body.to_string();
    assert!(
        !text.contains(&scratch) && !text.contains("repositories"),
        "the response names no database and no path — a probe's words go to the log: {body}"
    );

    // (3) The two dependencies still up can each be stopped too. A CA whose
    // window has closed is a real certificate, signed with a past `not_after`.
    // A store with no CA material is a second, UNMIGRATED database of this
    // control's own: it answers, and it holds no `server_ca` table — so the
    // store is down for its own reason, not because its database is gone.
    let empty = format!("{scratch}_empty");
    sqlx::query(&format!("CREATE DATABASE {empty}"))
        .execute(&pool)
        .await
        .expect("create the empty database on the verified server");
    let empty_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(pool.connect_options().as_ref().clone().database(&empty))
        .await
        .expect("connect to the empty database");
    let stopped = vec![
        Dependency::ServerCa { cas: expired_ca() },
        Dependency::SecretStore {
            store: SecretStore::dev(),
            pool: empty_pool.clone(),
        },
    ];
    assert!(
        Dependency::Postgres(empty_pool.clone())
            .probe()
            .await
            .is_ok(),
        "the empty database itself answers"
    );
    for dependency in &stopped {
        assert!(
            dependency.probe().await.is_err(),
            "`{}` is down when stopped",
            dependency.name()
        );
    }
    empty_pool.close().await;
    sqlx::query(&format!("DROP DATABASE {empty}"))
        .execute(&pool)
        .await
        .expect("drop the empty database");

    server.abort();
    scratch_pool.close().await;
}

/// The prober's first round completes before `spawn_prober` returns, so a
/// server that starts listening afterwards never answers `unobserved`.
#[tokio::test]
async fn the_first_probe_round_completes_before_the_prober_is_handed_back() {
    let Some(pool) = pool().await else { return };
    let dependencies = vec![Dependency::Postgres(pool.clone())];
    let monitor = HealthMonitor::new(&dependencies, Duration::from_secs(60));
    let prober =
        health::spawn_prober(monitor.clone(), dependencies, Duration::from_secs(3600)).await;
    let (healthy, body) = monitor.report_at(chrono::Utc::now());
    assert!(healthy, "{body}");
    assert_eq!(state_of(&body, "postgres")["state"], json!("up"));
    prober.abort();
}

/// A round takes the SLOWEST probe, not the sum: two dependencies that hang
/// until `PROBE_TIMEOUT` are both reported down within one timeout, not two.
///
/// Found by a real `rb-server` run: with PostgreSQL stopped, sequential
/// probing held every later dependency's observation for the whole timeout.
/// The hang here is real — a listener that accepts and never answers, so the
/// driver waits exactly as it does on a stopped server's half-open socket.
#[tokio::test]
async fn one_hung_dependency_does_not_delay_the_others() {
    let silent = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = silent.local_addr().unwrap().port();
    let accepted = tokio::spawn(async move {
        let mut held = Vec::new();
        while let Ok((socket, _)) = silent.accept().await {
            held.push(socket); // accept, keep open, never answer
        }
    });
    let hung = || {
        let options = sqlx::postgres::PgConnectOptions::new()
            .host("127.0.0.1")
            .port(port)
            .username("nobody")
            .database("nothing");
        Dependency::Postgres(
            PgPoolOptions::new()
                .max_connections(1)
                .acquire_timeout(Duration::from_secs(30))
                .connect_lazy_with(options),
        )
    };
    let dependencies = vec![hung(), hung()];
    let monitor = HealthMonitor::new(&dependencies, Duration::from_secs(60));

    let started = std::time::Instant::now();
    health::probe_all(&monitor, &dependencies).await;
    let elapsed = started.elapsed();

    let (healthy, body) = monitor.report_at(chrono::Utc::now());
    assert!(!healthy, "{body}");
    assert_eq!(body["dependencies"][0]["state"], json!("down"), "{body}");
    assert!(
        elapsed >= health::PROBE_TIMEOUT,
        "each probe really hung until its timeout: {elapsed:?}"
    );
    assert!(
        elapsed < health::PROBE_TIMEOUT * 2,
        "two hung probes took {elapsed:?} — the round is the slowest probe, not the sum"
    );
    accepted.abort();
}
