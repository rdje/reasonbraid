//! Every ROADMAP §18.5 operator surface is SERVED by the app `rb-server` runs
//! (`SIGNOFF-REPAIR.4.6.1.7`).
//!
//! Run with `scripts/run_pg_tests.sh operator_surfaces`. Without
//! `DATABASE_URL` this skips.
//!
//! `OPERATOR-SURFACES` (`scripts/census_operator_surfaces.py`) proves each
//! mapped route is REGISTERED in the source. That is not the same fact: two of
//! the surfaces live in routers (`health_router`, `backup_router`) that reach
//! the listener only because the app composition merges them, and a router left
//! out of that merge keeps the census green. This control builds the app
//! through [`control_plane_app`] — the one function `rb-server`'s `main` calls —
//! and calls every route in `.doctrine/operator_surfaces.tsv`, the mapping the
//! census reads too.
//!
//! ⭐ The request carries NO principal, deliberately. Every product handler
//! answers that with a typed refusal carrying a body; only a path the router
//! does not know answers `404` with an EMPTY body. So "served" needs no fixture
//! data and cannot pass because some fixture happened to exist.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::sync::Arc;
use std::time::Duration;

use reasonbraid_server::app::{control_plane_app, ControlPlane};
use reasonbraid_server::ca::ensure_server_ca;
use reasonbraid_server::health::{Dependency, HealthMonitor};

/// The shared mapping, compiled in: a tracked data file that travels with the
/// source, so the test and the census cannot read different lists.
const MAPPING: &str = include_str!("../../../.doctrine/operator_surfaces.tsv");

/// Every route the mapping names, with each path parameter given a value.
fn mapped_routes() -> Vec<(u32, String)> {
    let mut routes = Vec::new();
    for line in MAPPING.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cells: Vec<&str> = line.split('\t').collect();
        assert_eq!(cells.len(), 5, "a mapping row has five cells: {line}");
        let bullet: u32 = cells[0].parse().expect("a numbered bullet");
        for route in cells[2].split_whitespace() {
            let concrete = route
                .split('/')
                .map(|segment| {
                    if segment.starts_with('{') {
                        "id_00000000-0000-7000-8000-000000000001"
                    } else {
                        segment
                    }
                })
                .collect::<Vec<_>>()
                .join("/");
            routes.push((bullet, concrete));
        }
    }
    routes
}

#[tokio::test]
async fn every_mapped_operator_surface_is_served_by_the_composed_app() {
    let Some(pool) = pg_test_support::pool().await else {
        return;
    };
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    let ca = Arc::new(reasonbraid_server::ca::CaSet::single(Arc::new(
        ensure_server_ca(&pool).await.expect("server CA"),
    )));
    let dependencies = vec![Dependency::Postgres(pool.clone())];
    let app = control_plane_app(ControlPlane {
        pool: pool.clone(),
        ca,
        publication_repo_root: None,
        health: HealthMonitor::new(&dependencies, Duration::from_secs(60)),
        backup_dir: None,
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    let client = reqwest::Client::new();

    let routes = mapped_routes();
    assert!(
        routes
            .iter()
            .map(|(b, _)| *b)
            .collect::<std::collections::BTreeSet<_>>()
            == (1..=9).collect(),
        "the mapping covers all nine §18.5 bullets"
    );
    let mut unserved = Vec::new();
    for (bullet, path) in &routes {
        // GET first; a route registered for another method answers 405, and
        // then the POST is what reaches its handler.
        let mut response = client
            .get(format!("{base}{path}"))
            .send()
            .await
            .expect("request");
        if response.status().as_u16() == 405 {
            response = client
                .post(format!("{base}{path}"))
                .header("content-type", "application/json")
                .body("{}")
                .send()
                .await
                .expect("request");
        }
        let status = response.status().as_u16();
        let body = response.bytes().await.expect("body");
        if status == 404 && body.is_empty() {
            unserved.push(format!("bullet {bullet}: {path}"));
        }
    }
    assert!(
        unserved.is_empty(),
        "mapped §18.5 routes the composed app does not serve: {unserved:?}"
    );
    eprintln!(
        "operator surfaces: {} mapped routes across 9 bullets, every one served by control_plane_app",
        routes.len()
    );
    server.abort();
}
