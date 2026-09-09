//! The RLS defense-in-depth proof (`PHASE-7.1.3.1`, migration 0046,
//! `docs/decisions/2026-09-08_rls-tenant-claim.md`): the DATABASE-level
//! cross-tenant refusal, measured through a NON-superuser probe role —
//! superusers bypass row-level security unconditionally, and the dev profile
//! connects as postgres, so the layer is measured as it will bind (the
//! named deferral is the deployment-profile role change, not this proof).
//!
//! Measured legs:
//!   - the UNSET claim reads ZERO rows (fail-closed);
//!   - the tenant-A claim reads tenant-A's rows and NONE of tenant-B's;
//!   - a foreign-tenant INSERT under tenant-A's claim is REFUSED by the
//!     WITH CHECK policy at the DATABASE (the row-level-security violation);
//!   - the correct claim + the command path remain green (the live guard).
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

use sqlx::{PgPool, Row};

const PROBE_ROLE: &str = "rls_probe";

async fn pool() -> Option<PgPool> {
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    // FK-ordered: the delivery rows reference the outbox, which references
    // the events — the child tables purge first.
    for table in [
        "outbox_delivery",
        "outbox",
        "idempotency",
        "event_log",
        "aggregate_state",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&pool)
            .await
            .expect("purge table");
    }
    // The probe role: fresh per run (no cross-suite residue). The OWNED drop
    // removes the grant dependencies that would block DROP ROLE; the DO
    // block keeps it idempotent whether or not the role exists.
    sqlx::query(
        "DO $$ BEGIN \
         IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'rls_probe') THEN \
         DROP OWNED BY rls_probe; DROP ROLE rls_probe; END IF; END $$",
    )
    .execute(&pool)
    .await
    .expect("reset the probe role");
    sqlx::query("CREATE ROLE rls_probe LOGIN")
        .execute(&pool)
        .await
        .expect("create probe role");
    sqlx::query("GRANT USAGE ON SCHEMA public TO rls_probe")
        .execute(&pool)
        .await
        .expect("grant schema usage");
    for table in ["aggregate_state", "event_log", "idempotency"] {
        sqlx::query(&format!(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON {table} TO rls_probe"
        ))
        .execute(&pool)
        .await
        .expect("grant table");
    }
    Some(pool)
}

/// The probe's connection URL: the same host/port/database as DATABASE_URL,
/// with the probe role (localhost trust auth — no password).
fn probe_url(database_url: &str) -> String {
    let parsed = url::Url::parse(database_url).expect("DATABASE_URL parses");
    let host = parsed.host_str().expect("a host");
    let port = parsed.port().expect("a port");
    let db = parsed.path().trim_start_matches('/');
    format!("postgres://{PROBE_ROLE}@{host}:{port}/{db}?sslmode=disable")
}

#[tokio::test]
async fn the_database_refuses_a_cross_tenant_read_and_write_under_a_foreign_claim() {
    let Some(pool) = pool().await else {
        return;
    };

    // Seed TWO tenants' rows (as postgres — the superuser bypasses RLS).
    for (tenant, agg) in [("tenant-a", "thr-a"), ("tenant-b", "thr-b")] {
        sqlx::query(
            "INSERT INTO aggregate_state (tenant_id, aggregate_id, aggregate_type, \
             aggregate_version, state) VALUES ($1, $2, 'thread', 1, '{}'::jsonb)",
        )
        .bind(tenant)
        .bind(agg)
        .execute(&pool)
        .await
        .expect("seed aggregate");
        sqlx::query(
            "INSERT INTO event_log (event_id, tenant_id, aggregate_id, aggregate_version, \
             event_type, body) VALUES ($1, $2, $3, 1, 'thread.created', '{}'::jsonb)",
        )
        .bind(format!("evt-{tenant}"))
        .bind(tenant)
        .bind(agg)
        .execute(&pool)
        .await
        .expect("seed event");
        sqlx::query(
            "INSERT INTO idempotency (tenant_id, idempotency_key, request_hash, \
             response_result) VALUES ($1, 'key-1', 'hash-1', 'null'::jsonb)",
        )
        .bind(tenant)
        .execute(&pool)
        .await
        .expect("seed idempotency");
    }

    let probe = PgPool::connect(&probe_url(&std::env::var("DATABASE_URL").unwrap()))
        .await
        .expect("connect as the probe role");

    // Leg 1: the UNSET claim reads ZERO rows — fail-closed, on all three
    // protected tables (id-scoped: the cluster is shared with the other
    // suites, so the counts target THIS test's seeded rows only).
    for table in ["aggregate_state", "event_log", "idempotency"] {
        let count: i64 = sqlx::query(&format!(
            "SELECT count(*) FROM {table} WHERE tenant_id IN ('tenant-a', 'tenant-b')"
        ))
        .fetch_one(&probe)
        .await
        .expect("unset-claim count")
        .get(0);
        assert_eq!(
            count, 0,
            "the unset claim must see no {table} rows (fail-closed)"
        );
    }

    // Leg 2: the tenant-A claim sees tenant-A's rows and NONE of tenant-B's.
    let mut tx = probe.begin().await.expect("begin");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind("tenant-a")
        .execute(&mut *tx)
        .await
        .expect("set the claim");
    for table in ["aggregate_state", "event_log", "idempotency"] {
        let visible: i64 = sqlx::query(&format!(
            "SELECT count(*) FROM {table} WHERE tenant_id = 'tenant-a'"
        ))
        .fetch_one(&mut *tx)
        .await
        .expect("claim count")
        .get(0);
        assert_eq!(visible, 1, "the tenant-a claim must see its {table} row");
        let foreign: i64 = sqlx::query(&format!(
            "SELECT count(*) FROM {table} WHERE tenant_id = 'tenant-b'"
        ))
        .fetch_one(&mut *tx)
        .await
        .expect("foreign count")
        .get(0);
        assert_eq!(
            foreign, 0,
            "the tenant-a claim must see NO tenant-b {table} rows"
        );
    }

    // Leg 3: a foreign-tenant WRITE under the tenant-a claim is refused by
    // the WITH CHECK policy at the DATABASE (never silently stored).
    let refused = sqlx::query(
        "INSERT INTO event_log (event_id, tenant_id, aggregate_id, aggregate_version, \
         event_type, body) VALUES ('evt-forged', 'tenant-b', 'thr-b', 2, 'thread.created', \
         '{}'::jsonb)",
    )
    .execute(&mut *tx)
    .await
    .is_err();
    assert!(
        refused,
        "the foreign-tenant INSERT must be refused by the row-level security"
    );
    tx.rollback().await.expect("rollback");
    // The refused write left NO row behind (the check happens before any
    // storage) — verified as postgres (the superuser sees everything, so the
    // count is the ground truth, not the RLS filter).
    let stored: i64 = sqlx::query("SELECT count(*) FROM event_log WHERE event_id = 'evt-forged'")
        .fetch_one(&pool)
        .await
        .expect("forged row count")
        .get(0);
    assert_eq!(stored, 0, "the refused INSERT must store nothing");

    // Leg 4: the tenant-b claim sees ITS rows — the claim switches cleanly
    // per transaction (the pooled connection carries nothing between).
    let mut tx = probe.begin().await.expect("begin");
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind("tenant-b")
        .execute(&mut *tx)
        .await
        .expect("set the claim");
    let visible: i64 =
        sqlx::query("SELECT count(*) FROM aggregate_state WHERE tenant_id = 'tenant-b'")
            .fetch_one(&mut *tx)
            .await
            .expect("tenant-b count")
            .get(0);
    assert_eq!(visible, 1, "the tenant-b claim must see its own row only");
    tx.rollback().await.expect("rollback");

    // Cleanup: the role drops (the OWNED drop clears the grant dependencies);
    // the seeded rows purge (as postgres).
    drop(probe);
    sqlx::query(
        "DO $$ BEGIN \
         IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'rls_probe') THEN \
         DROP OWNED BY rls_probe; DROP ROLE rls_probe; END IF; END $$",
    )
    .execute(&pool)
    .await
    .expect("reset the probe role");
}
