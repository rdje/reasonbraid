//! Integration tests for the migration-0007 identity store (`PHASE-1.1.2`; backlog 10).
//!
//! The identity tables are the identity records themselves (`tenants`,
//! `human_principals`, `agent_roles`, `hosts`, `nodes`, `incarnations`, `runs`);
//! the `.6.1` `enrollments` table stays the dev bootstrap's name map. Enroll now
//! writes the enrollment row AND the identity row in ONE transaction — these tests
//! prove the rows land together, that a re-enroll duplicates nothing, and that the
//! foreign keys fail closed. Like the other PostgreSQL suites, they skip without
//! `DATABASE_URL` (`scripts/run_pg_tests.sh` / the `pg-tests` CI job).

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::api_router;
use serde_json::{json, Value};
use sqlx::PgPool;

/// The identity tests own the same control-API tables as the command-API suite:
/// tests never run concurrently against the same PG database.
static IDENTITY_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn identity_guard() -> tokio::sync::MutexGuard<'static, ()> {
    IDENTITY_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    // Purge in FK order (children first): this suite exclusively owns these tables.
    pg_cleanup::delete_tables(
        &pool,
        &[
            "outbox_delivery",
            "outbox",
            "node_events",
            "node_inbox",
            "budget_reservations",
            "budget_ceilings",
            "spend_breakers",
            "administrative_effects",
            "node_enrollment_tokens",
            "authorization_records",
            "authority_grants",
            "enrollments",
            "enrollment_boundaries",
            "node_enroll_audit",
            "node_keys",
            "node_certificates",
            "node_leases",
            "runs",
            "incarnations",
            "node_proof_nonces",
            "nodes",
            "hosts",
            "profile_versions",
            "agent_profiles",
            "recruitment_panels",
            "recruitment_responses",
            "recruitment_offers",
            "recruitment_calls",
            "agent_roles",
            "human_principals",
            "evidence_citations",
            "claim_assessments",
            "derivations",
            "evidence_snapshots",
            "resource_references",
            "quota_events",
            "usage_quotas",
            "federation_agreements",
            "cross_domain_receipts",
            "mcp_listen_state",
            "tenant_bootstrap_requests",
            "tenants",
            "idempotency",
            "event_log",
            "aggregate_state",
        ],
    )
    .await
    .expect("purge checked fixture plan");
    Some(pool)
}

struct TestServer {
    addr: SocketAddr,
    _handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn start(pool: &PgPool) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().unwrap();
        let router = api_router(pool.clone());
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.expect("serve");
        });
        Self {
            addr,
            _handle: handle,
        }
    }

    fn base(&self) -> String {
        format!("http://{}", self.addr)
    }
}

async fn enroll(client: &reqwest::Client, base: &str, body: Value) -> (u16, Value) {
    let response = client
        .post(format!("{base}/v1/enrollments"))
        .json(&body)
        .send()
        .await
        .expect("enroll request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("enroll json"))
}

/// The bootstrap commits the tenant row, the human's identity row, and the
/// enrollment row together — a principal exists iff its identity record does
/// (read back from a SEPARATE connection after the commit).
#[tokio::test]
async fn bootstrap_commits_tenant_and_human_identity_together() {
    let _guard = identity_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &server.base(),
        json!({ "kind": "human", "name": "alice" }),
    )
    .await;
    assert_eq!(status, 200, "enroll alice: {alice}");
    assert_eq!(alice["replayed"], json!(false));
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let principal = alice["principal_id"].as_str().unwrap().to_string();

    let (n_tenants,): (i64,) = sqlx::query_as("SELECT count(*) FROM tenants WHERE tenant_id = $1")
        .bind(&tenant)
        .fetch_one(&pool)
        .await
        .expect("count tenants");
    assert_eq!(n_tenants, 1, "the tenant's identity row exists");

    let (row_name, row_tenant): (String, String) =
        sqlx::query_as("SELECT name, tenant_id FROM human_principals WHERE principal_id = $1")
            .bind(&principal)
            .fetch_one(&pool)
            .await
            .expect("read human identity row");
    assert_eq!(row_name, "alice");
    assert_eq!(row_tenant, tenant, "the identity row carries its tenant");

    let (n_enrollments,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM enrollments WHERE principal_id = $1 AND tenant_id = $2",
    )
    .bind(&principal)
    .bind(&tenant)
    .fetch_one(&pool)
    .await
    .expect("count enrollments");
    assert_eq!(n_enrollments, 1);
}

/// A role enrolls into an existing tenant: its `agent_roles` identity row lands
/// in the same transaction, and a re-enroll (the idempotent bootstrap replay)
/// creates NOTHING new — not an identity row, not an enrollment row.
#[tokio::test]
async fn role_enroll_creates_identity_and_replay_duplicates_nothing() {
    let _guard = identity_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();

    let (_, alice) = enroll(
        &client,
        &server.base(),
        json!({ "kind": "human", "name": "alice" }),
    )
    .await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();

    let (status, reviewer) = enroll(
        &client,
        &server.base(),
        json!({ "kind": "role", "name": "reviewer", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enroll reviewer: {reviewer}");
    assert_eq!(reviewer["replayed"], json!(false));
    let role_id = reviewer["principal_id"].as_str().unwrap().to_string();

    let (row_name, row_tenant): (String, String) =
        sqlx::query_as("SELECT name, tenant_id FROM agent_roles WHERE role_id = $1")
            .bind(&role_id)
            .fetch_one(&pool)
            .await
            .expect("read role identity row");
    assert_eq!(row_name, "reviewer");
    assert_eq!(row_tenant, tenant);

    // The re-enroll replays the ORIGINAL ids and duplicates nothing.
    let (_, again) = enroll(
        &client,
        &server.base(),
        json!({ "kind": "role", "name": "reviewer", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(again["replayed"], json!(true));
    assert_eq!(again["principal_id"].as_str().unwrap(), role_id);
    let (n_roles,): (i64,) = sqlx::query_as("SELECT count(*) FROM agent_roles WHERE role_id = $1")
        .bind(&role_id)
        .fetch_one(&pool)
        .await
        .expect("count roles after replay");
    assert_eq!(n_roles, 1, "a re-enroll creates no second identity row");

    let (_, alice_again) = enroll(
        &client,
        &server.base(),
        json!({ "kind": "human", "name": "alice", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(alice_again["replayed"], json!(true));
    let (n_humans,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM human_principals WHERE tenant_id = $1")
            .bind(&tenant)
            .fetch_one(&pool)
            .await
            .expect("count humans after replay");
    assert_eq!(n_humans, 1);
}

/// The foreign keys fail closed: an identity row whose tenant (or host) does not
/// exist is refused by the database — never a silent half-identity.
#[tokio::test]
async fn identity_fks_fail_closed() {
    let _guard = identity_guard().await;
    let Some(pool) = pool().await else { return };

    let ghost = sqlx::query(
        "INSERT INTO human_principals (principal_id, tenant_id, name) VALUES ($1, $2, $3)",
    )
    .bind("hpr_00000000-0000-7000-8000-000000000999")
    .bind("ten_00000000-0000-7000-8000-000000000999")
    .bind("ghost")
    .execute(&pool)
    .await;
    assert!(
        ghost.is_err(),
        "a principal with no tenant row must be refused"
    );

    let orphan_node =
        sqlx::query("INSERT INTO nodes (node_id, host_id, tenant_id) VALUES ($1, $2, $3)")
            .bind("nod_00000000-0000-7000-8000-000000000999")
            .bind("hst_00000000-0000-7000-8000-000000000999")
            .bind("ten_00000000-0000-7000-8000-000000000999")
            .execute(&pool)
            .await;
    assert!(
        orphan_node.is_err(),
        "a node with no host row must be refused"
    );
}

/// A preceding node suite can leave certificates referencing the identity tree.
/// Cleanup must handle that residue while preserving the database's FK checks.
#[tokio::test]
async fn fixture_cleanup_removes_certificates_before_their_identity_parents() {
    let _guard = identity_guard().await;
    let Some(seeded) = pool().await else { return };
    let mut transaction = seeded.begin().await.expect("begin fixture seed");
    // Opaque certificate bytes are sufficient for this storage dependency test.
    sqlx::raw_sql(
        "INSERT INTO tenants (tenant_id, name)
             VALUES ('ten_00000000-0000-7000-8000-000000000901', 'fixture residue');
         INSERT INTO hosts (host_id, tenant_id)
             VALUES ('hst_00000000-0000-7000-8000-000000000901',
                     'ten_00000000-0000-7000-8000-000000000901');
         INSERT INTO nodes (node_id, host_id, tenant_id)
             VALUES ('nod_00000000-0000-7000-8000-000000000901',
                     'hst_00000000-0000-7000-8000-000000000901',
                     'ten_00000000-0000-7000-8000-000000000901');
         INSERT INTO node_certificates
             (cert_fingerprint, node_id, cert_der, key_der, issued_at, expires_at)
             VALUES (repeat('a', 64), 'nod_00000000-0000-7000-8000-000000000901',
                     decode('00', 'hex'), decode('00', 'hex'), now(), now() + interval '1 hour');",
    )
    .execute(&mut *transaction)
    .await
    .expect("seed certified-node identity hierarchy");
    transaction.commit().await.expect("commit fixture seed");

    let refused = sqlx::query("DELETE FROM nodes")
        .execute(&seeded)
        .await
        .expect_err("a referenced node cannot be deleted first");
    let database_error = refused.as_database_error().expect("database FK refusal");
    assert_eq!(database_error.code().as_deref(), Some("23503"));
    assert_eq!(
        database_error.constraint(),
        Some("node_certificates_node_id_fkey")
    );

    let cleaned = pool().await.expect("reopen the owned identity fixture");
    let remaining: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM node_certificates),
                (SELECT count(*) FROM nodes),
                (SELECT count(*) FROM hosts),
                (SELECT count(*) FROM tenants)",
    )
    .fetch_one(&cleaned)
    .await
    .expect("inspect the cleaned hierarchy");
    assert_eq!(remaining, (0, 0, 0, 0));
}
