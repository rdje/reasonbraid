//! WP3 node channel integration tests (`PHASE-0.3.2`; the `.1.2.2` authenticated
//! contract): the REAL outbound channel — the axum server in this crate and the
//! node client from `reasonbraid-node` talk over actual `127.0.0.1` sockets, with
//! the durable PostgreSQL inbox + lease store behind the server.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job. Without
//! `DATABASE_URL` these skip, so `make check` stays green offline. The node side
//! needs no service: its SQLite journal is a file.
//!
//! The acceptance: reconnect exchanges the last acknowledged server cursor and the
//! pending local operation ids; a duplicated command never creates a second local
//! operation; the node is not schedulable until reconciliation completes. Plus the WP3
//! exercises: network loss, server restart, duplicate delivery, cursor rewind
//! (the node reporting a cursor this server cannot reproduce) — and the `.1.2.2`
//! authentication: a handshake without a valid key-proof is refused, heartbeats
//! renew the lease, a newer handshake fences the old token, and expiry flips
//! presence to `offline` and refuses channel traffic.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use chrono::Utc;
use reasonbraid_node::{Journal, Node, NodeState};
use reasonbraid_server::{
    api_router, ca::ensure_server_ca, node_router, NodeChannelState, PRINCIPAL_HEADER,
};
use serde_json::{json, Value};
use sqlx::PgPool;

/// The dev signing secret every seeded node in this suite shares (the dev
/// trust-store stance: the server stores it, tests read it back from node_keys).
const DEV_SECRET: &str = "dev-secret";

/// The channel tests own the inbox/event/lease/identity tables (like the outbox
/// tests own the queue): tests never run concurrently against the same PG database.
static CHANNEL_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn channel_guard() -> tokio::sync::MutexGuard<'static, ()> {
    CHANNEL_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the real node channel proof"
            );
            return None;
        }
    };
    let pool = PgPool::connect(&url)
        .await
        .expect("connect to DATABASE_URL");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    // These tests exclusively own the channel + identity + lease tables for their
    // duration (FK order: leases and keys before nodes, nodes before hosts, every
    // tenant-referencing table before tenants).
    for table in [
        "outbox_delivery",
        "outbox",
        "node_events",
        "node_inbox",
        "node_leases",
        "budget_reservations",
        "budget_ceilings",
        "spend_breakers",
        "authorization_records",
        "authority_grants",
        "enrollments",
        "enrollment_boundaries",
        "node_enroll_audit",
        "node_keys",
        "node_certificates",
        "server_ca",
        "node_enrollment_tokens",
        "runs",
        "incarnations",
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
        "resource_references",
        "quota_events",
        "usage_quotas",
        "federation_agreements",
        "cross_domain_receipts",
        "tenants",
        "idempotency",
        "event_log",
        "aggregate_state",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&pool)
            .await
            .expect("purge table");
    }
    // The seed tenant the `seed_node` host rows reference.
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name) \
         VALUES ('ten_00000000-0000-7000-8000-000000000000', 'channel-seed') \
         ON CONFLICT (tenant_id) DO NOTHING",
    )
    .execute(&pool)
    .await
    .expect("seed tenant");
    Some(pool)
}

/// Seed an enrolled node the `.1.2.1` way (host → node → key rows) — the
/// Seed an enrolled node the `.1.2.1`/`.1.2.2` way (host → node → key → cert
/// rows): the `.1.2.2` handshake chains the presented leaf to the server CA and
/// checks the node-id → fingerprint binding. Returns the leaf + its key so each
/// test signs proofs exactly as the real node does. The enrollment ENDPOINT's
/// own semantics are proven by `node_enrollment.rs`; here the suite owns its
/// identity rows directly so each channel test starts from a known enrolled
/// state.
async fn seed_node(pool: &PgPool, node_id: &str) -> (Vec<u8>, Vec<u8>) {
    seed_node_in_tenant(pool, "ten_00000000-0000-7000-8000-000000000000", node_id).await
}

/// Seed an enrolled node into a SPECIFIC tenant (the revocation tests enroll
/// their admin first, so the node must live in the admin's tenant).
async fn seed_node_in_tenant(pool: &PgPool, tenant: &str, node_id: &str) -> (Vec<u8>, Vec<u8>) {
    let host_id = format!("hst_seed_{}", &node_id[4..]);
    let host_name = format!("seed-{node_id}");
    sqlx::query("INSERT INTO hosts (host_id, tenant_id, name) VALUES ($1, $2, $3)")
        .bind(&host_id)
        .bind("ten_00000000-0000-7000-8000-000000000000")
        .bind(&host_name)
        .execute(pool)
        .await
        .expect("seed host");
    sqlx::query("INSERT INTO nodes (node_id, host_id, tenant_id) VALUES ($1, $2, $3)")
        .bind(node_id)
        .bind(&host_id)
        .bind(tenant)
        .execute(pool)
        .await
        .expect("seed node");
    sqlx::query("INSERT INTO node_keys (node_id, key_fingerprint, key_secret) VALUES ($1, $2, $3)")
        .bind(node_id)
        .bind(format!("seed-fingerprint-{node_id}"))
        .bind(DEV_SECRET)
        .execute(pool)
        .await
        .expect("seed key");
    // The workload certificate (the `.1.2.2` identity): issued by the server's
    // own CA and stored like the enrollment transaction stores it.
    let ca = ensure_server_ca(pool).await.expect("server CA");
    let (cert_der, key_der) = reasonbraid_server::ca::issue_node_leaf(&ca, node_id, &host_name);
    let fingerprint = reasonbraid_server::ca::cert_fingerprint(&cert_der);
    sqlx::query(
        "INSERT INTO node_certificates \
         (cert_fingerprint, node_id, cert_der, key_der, issued_at, expires_at) \
         VALUES ($1, $2, $3, $4, now(), now() + interval '10 minutes')",
    )
    .bind(&fingerprint)
    .bind(node_id)
    .bind(&cert_der)
    .bind(&key_der)
    .execute(pool)
    .await
    .expect("seed certificate");
    (cert_der, key_der)
}

/// Rebuild the leaf key from its DER (tests open the Node multiple times //
/// across server generations; rcgen::KeyPair is not Clone).
fn key_from_der(key_der: &[u8]) -> rcgen::KeyPair {
    let der = rustls_pki_types::PrivateKeyDer::try_from(key_der.to_vec()).expect("leaf key DER");
    rcgen::KeyPair::from_der_and_sign_algo(&der, &rcgen::PKCS_ECDSA_P256_SHA256)
        .expect("leaf key parses")
}

/// The running server half: an axum listener on an ephemeral loopback port backed by the
/// shared pool. Dropping/aborting it is the "server crash" kill point.
struct TestServer {
    addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn start(pool: &PgPool) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().unwrap();
        let ca = Arc::new(ensure_server_ca(pool).await.expect("server CA"));
        let router = api_router(pool.clone()).merge(node_router(pool.clone(), ca));
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.expect("serve");
        });
        Self { addr, handle }
    }

    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Crash the server (ungraceful abort — a kill point).
    fn crash(self) {
        self.handle.abort();
    }
}

/// A unique node journal path under the build dir (same volume as the repo).
fn journal_path(name: &str) -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = base.join("journal-tests").join(format!("{name}-{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("node.db")
}

async fn enqueue(state: &NodeChannelState, node_id: &str, command_id: &str) -> i64 {
    state
        .enqueue(
            node_id,
            command_id,
            "ten_00000000-0000-7000-8000-000000000000",
            "thr_00000000-0000-7000-8000-000000000000",
            &json!({ "operation": "contribute", "command_id": command_id }),
        )
        .await
        .expect("enqueue command")
}

async fn acked_rows(pool: &PgPool, node_id: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM node_inbox WHERE node_id = $1 AND acknowledged_at IS NOT NULL",
    )
    .bind(node_id)
    .fetch_one(pool)
    .await
    .expect("count acknowledged rows")
}

/// A fresh node plays the whole inbox from cursor zero, journals every command exactly
/// once, acknowledges, and becomes schedulable.
#[tokio::test]
async fn fresh_node_handshake_plays_the_whole_inbox_and_becomes_schedulable() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000001".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    for i in 1..=3 {
        enqueue(&state, &node_id, &format!("cmd_fresh_{i}")).await;
    }

    let node = Node::open(
        journal_path("fresh"),
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    assert_eq!(node.state().await, NodeState::Offline);

    node.reconcile().await.expect("first reconcile");
    assert_eq!(node.state().await, NodeState::Schedulable);

    let counts = node.journal().counts().await.unwrap();
    assert_eq!(counts.commands, 3, "the whole inbox is journaled");
    assert_eq!(counts.operations, 3, "one local operation per command");
    assert_eq!(node.journal().last_acked_cursor().await.unwrap(), 3);
    assert_eq!(
        acked_rows(&pool, &node_id).await,
        3,
        "server-side delivery state"
    );
    server.crash();
}

/// THE cursor-resume acceptance: a reconnect reports the last acknowledged cursor and
/// the server replays ONLY the tail — the node never re-journals what it already holds.
#[tokio::test]
async fn reconnect_replays_only_the_tail_after_the_reported_cursor() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000002".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    for i in 1..=3 {
        enqueue(&state, &node_id, &format!("cmd_tail_{i}")).await;
    }
    let node = Node::open(
        journal_path("tail"),
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    node.reconcile().await.unwrap();

    // Two more commands arrive server-side while the node is away (the "crash").
    for i in 4..=5 {
        enqueue(&state, &node_id, &format!("cmd_tail_{i}")).await;
    }
    node.reconcile().await.expect("reconnect");

    let counts = node.journal().counts().await.unwrap();
    assert_eq!(
        counts.commands, 5,
        "only the two new commands were journaled"
    );
    assert_eq!(counts.operations, 5);
    assert_eq!(node.journal().last_acked_cursor().await.unwrap(), 5);
    assert_eq!(acked_rows(&pool, &node_id).await, 5);
    server.crash();
}

/// THE duplicate-command acceptance: the crash window between journaling the replay and
/// recording the acknowledgement cursor makes the node report an OLD cursor, the server
/// replays commands the node already holds, and the journal's dedupe keeps one local
/// operation per command — exactly the same operation ids as before.
#[tokio::test]
async fn duplicate_command_delivery_never_creates_a_second_local_operation() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000003".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    for i in 1..=3 {
        enqueue(&state, &node_id, &format!("cmd_dup_{i}")).await;
    }
    let node = Node::open(
        journal_path("duplicate"),
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    node.reconcile().await.unwrap();
    let operations_before = node.journal().operation_ids().await.unwrap();
    assert_eq!(operations_before.len(), 3);

    // Simulate the crash window: the replay WAS journaled but the acknowledgement
    // cursor was never recorded — the node's durable state reports cursor 0.
    node.journal().set_last_acked_cursor(0).await.unwrap();

    node.reconcile()
        .await
        .expect("reconnect replays everything");
    let operations_after = node.journal().operation_ids().await.unwrap();
    assert_eq!(node.journal().counts().await.unwrap().commands, 3);
    assert_eq!(
        operations_before, operations_after,
        "a duplicated command must never create a second local operation"
    );
    assert_eq!(node.journal().last_acked_cursor().await.unwrap(), 3);
    server.crash();
}

/// THE schedulability acceptance: the node refuses new work until reconciliation
/// completes; a failed reconcile (server down) leaves it NOT schedulable, and it only
/// becomes schedulable once the full handshake round-trip is applied.
#[tokio::test]
async fn node_is_not_schedulable_until_reconciliation_completes() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let node_id = "nod_00000000-0000-7000-8000-000000000004".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    // A port with nothing listening: the channel is unreachable.
    let dead = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dead_url = format!("http://{}", dead.local_addr().unwrap());
    drop(dead);

    let journal = journal_path("schedulable");
    let node = Node::open(
        &journal,
        dead_url,
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    assert_eq!(node.state().await, NodeState::Offline);

    // New work is refused before reconciliation, whatever the connection state.
    let err = node
        .emit_event("op_none", "evt_none", &json!({}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not schedulable"), "got: {err}");

    // Reconcile against a dead server: error, still NOT schedulable.
    let err = node.reconcile().await.unwrap_err();
    assert!(err.to_string().contains("transport"), "got: {err}");
    assert_ne!(node.state().await, NodeState::Schedulable);

    // Bring the server up (same journal): reconciliation completes → schedulable.
    enqueue(&state, &node_id, "cmd_sched_1").await;
    let server = TestServer::start(&pool).await;
    let node = Node::open(
        &journal,
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    node.reconcile()
        .await
        .expect("reconcile against the live server");
    assert_eq!(node.state().await, NodeState::Schedulable);
    assert_eq!(node.journal().counts().await.unwrap().commands, 1);

    // Schedulable now: emitting works (against the operation the replay created).
    let op = node.journal().operation_ids().await.unwrap().remove(0);
    node.emit_event(&op, "evt_sched", &json!({ "event_type": "ready" }))
        .await
        .expect("emit after reconciliation");
    assert!(node.journal().pending_events().await.unwrap().is_empty());
    server.crash();
}

/// Ambiguity reconciliation, negative case: the server holds NO receipt for the
/// operation, so the directive is `needs_adjudication` — the attempt stays visibly
/// `outcome_unknown` (bounded, never silently retried), and the node still becomes
/// schedulable: its part of reconciliation is complete.
#[tokio::test]
async fn ambiguous_attempt_without_server_receipt_stays_outcome_unknown() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000005".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    let journal_path = journal_path("ambiguous-unknown");
    {
        let journal = Journal::open(&journal_path).await.unwrap();
        let payload = json!({ "operation": "contribute" });
        journal
            .record_command(
                &reasonbraid_node::CommandInput {
                    command_id: "cmd_amb_1",
                    tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                    thread_id: "thr_00000000-0000-7000-8000-000000000000",
                    payload: &payload,
                    authz_ref: None,
                    policy_digest: None,
                    decided_at: None,
                    revocation_epoch: None,
                    server_cursor: "1",
                },
                Utc::now(),
            )
            .await
            .unwrap();
        let op = journal
            .ensure_operation("cmd_amb_1", Utc::now())
            .await
            .unwrap()
            .operation_id;
        journal
            .prepare_attempt("patt_amb_1", &op, Utc::now())
            .await
            .unwrap();
        // Crash after a possible dispatch, before any result reached the server.
        journal
            .record_dispatch("patt_amb_1", None, Utc::now())
            .await
            .unwrap();
    }

    let node = Node::open(
        &journal_path,
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    node.reconcile().await.expect("reconcile");

    let ambiguous = node.journal().ambiguous_attempts().await.unwrap();
    assert_eq!(ambiguous.len(), 1, "the attempt stays bounded and visible");
    assert_eq!(ambiguous[0].status, "outcome_unknown");
    assert_eq!(node.state().await, NodeState::Schedulable);
    server.crash();
}

/// Ambiguity reconciliation, positive case: the server DOES hold a receipt for the
/// operation (its event arrived before the crash), so the directive is `adjudicated` —
/// the node marks the attempt `reconciled`. The pending-operation exchange is
/// load-bearing here too: the server reports the event as known, so the node marks it
/// acknowledged locally instead of re-sending it.
#[tokio::test]
async fn ambiguous_attempt_with_server_receipt_is_adjudicated_and_events_dedupe() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000006".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    let journal_path = journal_path("ambiguous-adjudicated");
    let (op, event_id) = {
        let journal = Journal::open(&journal_path).await.unwrap();
        let payload = json!({ "operation": "contribute" });
        journal
            .record_command(
                &reasonbraid_node::CommandInput {
                    command_id: "cmd_amb_2",
                    tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                    thread_id: "thr_00000000-0000-7000-8000-000000000000",
                    payload: &payload,
                    authz_ref: None,
                    policy_digest: None,
                    decided_at: None,
                    revocation_epoch: None,
                    server_cursor: "1",
                },
                Utc::now(),
            )
            .await
            .unwrap();
        let op = journal
            .ensure_operation("cmd_amb_2", Utc::now())
            .await
            .unwrap()
            .operation_id;
        journal
            .prepare_attempt("patt_amb_2", &op, Utc::now())
            .await
            .unwrap();
        journal
            .record_dispatch("patt_amb_2", None, Utc::now())
            .await
            .unwrap();
        // The event WAS emitted and the server accepted it (receipt below) — but the
        // node crashed before journaling the acknowledgement.
        let event_id = "evt_amb_2".to_string();
        journal
            .record_outgoing_event(
                &event_id,
                &op,
                &json!({ "event_type": "ready" }),
                Utc::now(),
            )
            .await
            .unwrap();
        (op, event_id)
    };
    state
        .record_event(
            &node_id,
            &event_id,
            &op,
            &json!({ "event_type": "ready" }),
            Utc::now(),
        )
        .await
        .unwrap();

    let node = Node::open(
        &journal_path,
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    node.reconcile().await.expect("reconcile");

    // The directive landed: the attempt is reconciled, the ambiguity is closed.
    assert!(node
        .journal()
        .ambiguous_attempts()
        .await
        .unwrap()
        .is_empty());
    let history = node.journal().attempt_history("patt_amb_2").await.unwrap();
    assert_eq!(history.last().unwrap().to_status, "reconciled");

    // The pending event was reported as KNOWN by the handshake (the pending-operation
    // exchange), so the node marked it acknowledged locally instead of re-sending —
    // the server still holds exactly one receipt.
    assert!(node.journal().pending_events().await.unwrap().is_empty());
    let receipts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM node_events WHERE event_id = $1")
        .bind(&event_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(receipts, 1, "the known event was not re-sent");
    server.crash();
}

/// §17.4 step 5: pending results are re-emitted with their ORIGINAL ids after a crash —
/// the server ends up with exactly one receipt carrying the node-assigned event id.
#[tokio::test]
async fn pending_events_are_reemitted_with_their_original_ids() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000007".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    let journal_path = journal_path("reemit");
    let op = {
        let journal = Journal::open(&journal_path).await.unwrap();
        let payload = json!({ "operation": "contribute" });
        journal
            .record_command(
                &reasonbraid_node::CommandInput {
                    command_id: "cmd_reemit",
                    tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                    thread_id: "thr_00000000-0000-7000-8000-000000000000",
                    payload: &payload,
                    authz_ref: None,
                    policy_digest: None,
                    decided_at: None,
                    revocation_epoch: None,
                    server_cursor: "1",
                },
                Utc::now(),
            )
            .await
            .unwrap();
        journal
            .ensure_operation("cmd_reemit", Utc::now())
            .await
            .unwrap()
            .operation_id
    };
    // A pending event the node never got to send (crash before send).
    {
        let journal = Journal::open(&journal_path).await.unwrap();
        journal
            .record_outgoing_event(
                "evt_reemit",
                &op,
                &json!({ "event_type": "contribution.ready", "original": true }),
                Utc::now(),
            )
            .await
            .unwrap();
    }

    let node = Node::open(
        &journal_path,
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    node.reconcile().await.expect("reconcile");

    let (event_id,): (String,) =
        sqlx::query_as("SELECT event_id FROM node_events WHERE operation_id = $1")
            .bind(&op)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(event_id, "evt_reemit", "the ORIGINAL id, not a new one");
    assert!(node.journal().pending_events().await.unwrap().is_empty());
    server.crash();
}

/// Server restart (a WP3 exercise): the inbox is durable, so a node that reconciled
/// against one server process resumes against a fresh one from its recorded cursor and
/// receives exactly the tail.
#[tokio::test]
async fn server_restart_preserves_the_inbox_and_resume() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let node_id = "nod_00000000-0000-7000-8000-000000000008".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    let journal_path = journal_path("restart");
    {
        let server = TestServer::start(&pool).await;
        for i in 1..=2 {
            enqueue(&state, &node_id, &format!("cmd_rst_{i}")).await;
        }
        let node = Node::open(
            &journal_path,
            server.base_url(),
            node_id.clone(),
            cert_der.clone(),
            key_from_der(&key_der),
        )
        .await
        .unwrap();
        node.reconcile().await.unwrap();
        server.crash(); // the server process dies
    }

    // The node's next reconcile fails (nothing listening) — not schedulable.
    let dead = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dead_url = format!("http://{}", dead.local_addr().unwrap());
    drop(dead);
    let node = Node::open(
        &journal_path,
        dead_url,
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    assert!(node.reconcile().await.is_err());
    assert_ne!(node.state().await, NodeState::Schedulable);

    // A NEW server process, the SAME durable PostgreSQL: resume from the recorded cursor.
    let server = TestServer::start(&pool).await;
    for i in 3..=4 {
        enqueue(&state, &node_id, &format!("cmd_rst_{i}")).await;
    }
    let node = Node::open(
        &journal_path,
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    node.reconcile()
        .await
        .expect("resume against the restarted server");

    let counts = node.journal().counts().await.unwrap();
    assert_eq!(counts.commands, 4, "the durable inbox survived the restart");
    assert_eq!(counts.operations, 4);
    assert_eq!(node.journal().last_acked_cursor().await.unwrap(), 4);
    assert_eq!(acked_rows(&pool, &node_id).await, 4);
    server.crash();
}

/// Cursor rewind beyond the server's ledger (a journal-lost-class anomaly, §11.4): the
/// server REFUSES the handshake with a typed error and the node stays NOT schedulable —
/// it never silently re-bases on a ledger it cannot reproduce.
#[tokio::test]
async fn reporting_a_cursor_ahead_of_the_server_ledger_is_refused() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000009".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    enqueue(&state, &node_id, "cmd_ahead_1").await;
    enqueue(&state, &node_id, "cmd_ahead_2").await;

    let node = Node::open(
        journal_path("ahead"),
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    // The node claims to hold cursor 99 — this server's ledger only goes to 2.
    node.journal().set_last_acked_cursor(99).await.unwrap();

    let err = node.reconcile().await.unwrap_err();
    assert!(err.to_string().contains("ahead"), "got: {err}");
    assert_ne!(node.state().await, NodeState::Schedulable);
    assert_eq!(node.journal().counts().await.unwrap().commands, 0);
    assert_eq!(
        acked_rows(&pool, &node_id).await,
        0,
        "nothing was acknowledged"
    );
    server.crash();
}

/// The live delivery path: a schedulable node polls the tail after its cursor (the
/// channel's poll endpoint serves the same replay logic as the handshake).
#[tokio::test]
async fn poll_returns_the_tail_after_a_cursor() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000010".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    for i in 1..=3 {
        enqueue(&state, &node_id, &format!("cmd_poll_{i}")).await;
    }
    let node = Node::open(
        journal_path("poll"),
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    node.reconcile().await.unwrap();

    for i in 4..=5 {
        enqueue(&state, &node_id, &format!("cmd_poll_{i}")).await;
    }
    // A fresh client must authenticate (handshake → lease + fencing token) before
    // polling; the handshake itself rotates the lease, fencing the node's earlier
    // token — which is exactly the fencing contract (the worker's channel is the
    // one `reconcile` authenticated, and it shares the token with `node`).
    let channel = reasonbraid_node::NodeChannel::new(
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    );
    channel
        .handshake(&reasonbraid_node::HandshakeRequest {
            channel_version: reasonbraid_node::CHANNEL_VERSION,
            node_id: node_id.clone(),
            last_acked_cursor: 0,
            pending_operations: vec![],
            ambiguous_attempts: vec![],
            cert_der: String::new(),
            proof_signature: String::new(),
        })
        .await
        .expect("handshake");
    let tail = channel.poll(3).await.expect("poll");

    assert_eq!(tail.current_cursor, 5);
    assert_eq!(tail.commands.len(), 2);
    assert_eq!(tail.commands[0].cursor, 4);
    assert_eq!(tail.commands[1].cursor, 5);
    server.crash();
}

/// The handshake wire contract is versioned and `deny_unknown_fields`-strict: a future
/// version and a forged field are rejected, never silently accepted.
#[tokio::test]
async fn handshake_rejects_version_mismatch_and_unknown_fields() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();

    let bad_version = client
        .post(format!("{}/v1/nodes/handshake", server.base_url()))
        .json(&json!({
            "channel_version": 999,
            "node_id": "nod_00000000-0000-7000-8000-000000000011",
            "last_acked_cursor": 0,
            "pending_operations": [],
            "ambiguous_attempts": [],
            "cert_der": "00",
            "proof_signature": "00"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad_version.status().as_u16(), 400);
    let body: Value = bad_version.json().await.unwrap();
    assert_eq!(body["code"], "protocol_incompatible");

    let forged = client
        .post(format!("{}/v1/nodes/handshake", server.base_url()))
        .json(&json!({
            "channel_version": 1,
            "node_id": "nod_00000000-0000-7000-8000-000000000011",
            "last_acked_cursor": 0,
            "pending_operations": [],
            "ambiguous_attempts": [],
            "tenant_id": "forged"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        forged.status().as_u16(),
        422,
        "a forged authoritative field is rejected"
    );
    server.crash();
}

/// The pending-operation exchange is load-bearing: when the server reports an event as
/// already held (`known_events`), the node marks it acknowledged locally and does NOT
/// re-send it — while events the server never got are re-emitted with their original
/// ids. Both end acknowledged in the journal; the server holds exactly one receipt each.
#[tokio::test]
async fn server_known_events_skip_reemission_of_already_delivered_results() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000011".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    let journal_path = journal_path("known-events");
    let (op_known, _op_new) = {
        let journal = Journal::open(&journal_path).await.unwrap();
        let payload = json!({ "operation": "contribute" });
        journal
            .record_command(
                &reasonbraid_node::CommandInput {
                    command_id: "cmd_known",
                    tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                    thread_id: "thr_00000000-0000-7000-8000-000000000000",
                    payload: &payload,
                    authz_ref: None,
                    policy_digest: None,
                    decided_at: None,
                    revocation_epoch: None,
                    server_cursor: "1",
                },
                Utc::now(),
            )
            .await
            .unwrap();
        let op_known = journal
            .ensure_operation("cmd_known", Utc::now())
            .await
            .unwrap()
            .operation_id;
        journal
            .record_command(
                &reasonbraid_node::CommandInput {
                    command_id: "cmd_new",
                    tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                    thread_id: "thr_00000000-0000-7000-8000-000000000000",
                    payload: &payload,
                    authz_ref: None,
                    policy_digest: None,
                    decided_at: None,
                    revocation_epoch: None,
                    server_cursor: "1",
                },
                Utc::now(),
            )
            .await
            .unwrap();
        let op_new = journal
            .ensure_operation("cmd_new", Utc::now())
            .await
            .unwrap()
            .operation_id;
        journal
            .record_outgoing_event(
                "evt_known",
                &op_known,
                &json!({ "event_type": "ready" }),
                Utc::now(),
            )
            .await
            .unwrap();
        journal
            .record_outgoing_event(
                "evt_new",
                &op_new,
                &json!({ "event_type": "ready" }),
                Utc::now(),
            )
            .await
            .unwrap();
        (op_known, op_new)
    };
    // The server already holds evt_known (its receipt survived; the node's ack did not).
    state
        .record_event(
            &node_id,
            "evt_known",
            &op_known,
            &json!({ "event_type": "ready" }),
            Utc::now(),
        )
        .await
        .unwrap();

    let node = Node::open(
        &journal_path,
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    node.reconcile().await.expect("reconcile");

    let known_receipts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM node_events WHERE event_id = 'evt_known'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(known_receipts, 1, "the known event was NOT re-sent");
    let new_receipts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM node_events WHERE event_id = 'evt_new'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(new_receipts, 1, "the unknown event WAS re-emitted");
    assert!(node.journal().pending_events().await.unwrap().is_empty());
    server.crash();
}

/// Server-side event dedupe: a node emitting the same event id twice (transport
/// retry) produces exactly ONE receipt — the second submission is a duplicate.
#[tokio::test]
async fn duplicate_event_emission_dedupes_server_side() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000012".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    enqueue(&state, &node_id, "cmd_dedupe").await;
    let node = Node::open(
        journal_path("event-dedupe"),
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    )
    .await
    .unwrap();
    node.reconcile().await.unwrap();
    let op = node.journal().operation_ids().await.unwrap().remove(0);

    node.emit_event(
        &op,
        "evt_dedupe",
        &json!({ "event_type": "ready", "take": 1 }),
    )
    .await
    .unwrap();
    // A transport-level retry of the SAME emission (same id, same effect).
    node.emit_event(
        &op,
        "evt_dedupe",
        &json!({ "event_type": "ready", "take": 1 }),
    )
    .await
    .unwrap();

    let receipts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM node_events WHERE event_id = 'evt_dedupe'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(receipts, 1, "two emissions, one receipt");
    assert!(node.journal().pending_events().await.unwrap().is_empty());
    server.crash();
}

// ── The `.1.2.2` authenticated contract ─────────────────────────────────────────

/// THE `.1.2.2` acceptance: a handshake without a valid key-proof is refused —
/// a missing field is a malformed request (422 at the strict wire boundary), a
/// wrong proof and an unenrolled node fail IDENTICALLY (`401 unauthorized`, no
/// existence leak), and no ledger fact is readable without an authenticated
/// handshake.
#[tokio::test]
async fn handshake_without_a_valid_certificate_proof_is_refused() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let node_id = "nod_00000000-0000-7000-8000-000000000101".to_string();
    let _ = seed_node(&pool, &node_id).await;

    let handshake = |proof_signature: Option<String>| {
        let client = client.clone();
        let node_id = node_id.clone();
        let base = server.base_url();
        async move {
            let mut body = json!({
                "channel_version": 5,
                "node_id": node_id,
                "last_acked_cursor": 0,
                "pending_operations": [],
                "ambiguous_attempts": [],
            });
            if let Some(proof) = proof_signature {
                body["cert_der"] = json!("00");
                body["proof_signature"] = json!(proof);
            }
            client
                .post(format!("{base}/v1/nodes/handshake"))
                .json(&body)
                .send()
                .await
                .unwrap()
        }
    };

    // No proof at all: a MISSING field is a malformed request — the strict wire
    // boundary refuses it before the handler runs (422).
    let missing = handshake(None).await;
    assert_eq!(
        missing.status().as_u16(),
        422,
        "a missing proof is malformed"
    );
    // A structurally valid but wrong proof (a signature over nothing relevant):
    // the handler.s chain/status/signature ladder refuses it 401, identically to
    // an unenrolled node.
    let wrong = handshake(Some(
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
    ))
    .await;
    assert_eq!(wrong.status().as_u16(), 401, "a wrong proof is refused");
    let body: Value = wrong.json().await.unwrap();
    assert_eq!(body["code"], "unauthorized");
    assert_eq!(
        body["message"],
        "the handshake certificate proof was refused"
    );

    // An unenrolled node fails the same way (no existence leak).
    let stranger = client
        .post(format!("{}/v1/nodes/handshake", server.base_url()))
        .json(&json!({
            "channel_version": 5,
            "node_id": "nod_00000000-0000-7000-8000-0000000001ff",
            "last_acked_cursor": 0,
            "pending_operations": [],
            "ambiguous_attempts": [],
            "cert_der": "00",
            "proof_signature": "00",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(stranger.status().as_u16(), 401);

    // Channel traffic without an authenticated handshake is refused too. A
    // request that OMITS the fencing token is malformed at the strict boundary
    // (422); a WRONG token reaches the lease check and is refused 401.
    let malformed = client
        .post(format!("{}/v1/nodes/events", server.base_url()))
        .json(&json!({
            "channel_version": 5,
            "node_id": node_id,
            "event_id": "evt_00000000-0000-7000-8000-000000000101",
            "operation_id": "op_00000000-0000-7000-8000-000000000101",
            "payload": { "kind": "some_signal" },
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        malformed.status().as_u16(),
        422,
        "a missing token is malformed"
    );
    let unauthenticated = client
        .post(format!("{}/v1/nodes/events", server.base_url()))
        .json(&json!({
            "channel_version": 5,
            "node_id": node_id,
            "event_id": "evt_00000000-0000-7000-8000-000000000101",
            "operation_id": "op_00000000-0000-7000-8000-000000000101",
            "payload": { "kind": "some_signal" },
            "fencing_token": "fnc_not-the-live-token",
            "lease_epoch": 1,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(unauthenticated.status().as_u16(), 401);

    // Presence: the seeded node exists but holds no lease — observably offline.
    let presence = client
        .get(format!(
            "{}/v1/nodes/presence?node_id={node_id}",
            server.base_url()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(presence.status().as_u16(), 200);
    let p: Value = presence.json().await.unwrap();
    assert_eq!(p["online"], json!(false));
    assert_eq!(p["state"], json!("offline"), "the derived state: {p}");
    assert!(p["last_seen_at"].is_null() && p["lease_expires_at"].is_null());
    server.crash();
}

/// Heartbeats renew a LIVE lease: every renewal pushes the expiry forward and the
/// presence surface reflects it (`last_seen_at` advances, `online` stays true).
#[tokio::test]
async fn heartbeat_renews_the_lease_and_presence_shows_online() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let node_id = "nod_00000000-0000-7000-8000-000000000102".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    let channel = reasonbraid_node::NodeChannel::new(
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    );
    let handshake = channel
        .handshake(&reasonbraid_node::HandshakeRequest {
            channel_version: reasonbraid_node::CHANNEL_VERSION,
            node_id: node_id.clone(),
            last_acked_cursor: 0,
            pending_operations: vec![],
            ambiguous_attempts: vec![],
            cert_der: String::new(),
            proof_signature: String::new(),
        })
        .await
        .expect("handshake");
    let first_expiry = handshake.lease_expires_at;

    let presence = async || -> Value {
        client
            .get(format!(
                "{}/v1/nodes/presence?node_id={node_id}",
                server.base_url()
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    };
    let p = presence().await;
    assert_eq!(
        p["online"],
        json!(true),
        "the handshake made the node online"
    );
    assert!(p["last_seen_at"].is_string());

    let renewal = channel.heartbeat().await.expect("heartbeat");
    assert_eq!(renewal.fencing_token, handshake.fencing_token);
    assert!(
        renewal.lease_expires_at >= first_expiry,
        "a heartbeat pushes the expiry forward"
    );
    let p = presence().await;
    assert_eq!(p["online"], json!(true), "presence stays online");
    assert_eq!(p["state"], json!("available"), "the derived state: {p}");

    // A second heartbeat renews again — and the lease is visible through the API.
    let first_seen = p["last_seen_at"].as_str().unwrap().to_string();
    let renewal2 = channel.heartbeat().await.expect("second heartbeat");
    assert!(renewal2.lease_expires_at >= renewal.lease_expires_at);
    let p = presence().await;
    assert!(
        p["last_seen_at"].as_str().unwrap() >= first_seen.as_str(),
        "last_seen_at advances with the renewal"
    );
    server.crash();
}

/// The fencing contract: a second handshake rotates the lease — the OLD token is
/// fenced and every channel surface refuses it, while the new token works. A
/// stale process (one that missed the rotation) cannot write anything.
#[tokio::test]
async fn a_second_handshake_fences_the_previous_lease() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let node_id = "nod_00000000-0000-7000-8000-000000000103".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    let handshake = async || -> (String, i64) {
        let channel = reasonbraid_node::NodeChannel::new(
            server.base_url(),
            node_id.clone(),
            cert_der.clone(),
            key_from_der(&key_der),
        );
        let hs = channel
            .handshake(&reasonbraid_node::HandshakeRequest {
                channel_version: reasonbraid_node::CHANNEL_VERSION,
                node_id: node_id.clone(),
                last_acked_cursor: 0,
                pending_operations: vec![],
                ambiguous_attempts: vec![],
                cert_der: String::new(),
                proof_signature: String::new(),
            })
            .await
            .expect("handshake");
        (hs.fencing_token, hs.lease_epoch)
    };
    let (old_token, old_epoch) = handshake().await;
    let (new_token, _new_epoch) = handshake().await;
    assert_ne!(old_token, new_token, "every handshake rotates the token");

    let base = server.base_url();
    let probe = |path: &'static str, token: &str, epoch: i64, body: Value| {
        let client = client.clone();
        let base = base.clone();
        let node_id = node_id.clone();
        let token = token.to_string();
        async move {
            let mut body = body;
            body["fencing_token"] = json!(token);
            body["lease_epoch"] = json!(epoch);
            body["channel_version"] = json!(5);
            body["node_id"] = json!(node_id);
            client
                .post(format!("{base}{path}"))
                .json(&body)
                .send()
                .await
                .unwrap()
                .status()
                .as_u16()
        }
    };

    // The old token is fenced on EVERY surface — its epoch died with it, so even
    // a write carrying the old epoch can never match the rotated lease row.
    assert_eq!(
        probe("/v1/nodes/heartbeat", &old_token, old_epoch, json!({})).await,
        401,
        "heartbeat with a fenced token is refused"
    );
    assert_eq!(
        probe(
            "/v1/nodes/events",
            &old_token,
            old_epoch,
            json!({ "event_id": "evt_fenced", "operation_id": "op_fenced", "payload": {} }),
        )
        .await,
        401,
        "events with a fenced token are refused"
    );
    assert_eq!(
        probe(
            "/v1/nodes/ack",
            &old_token,
            old_epoch,
            json!({ "ack_cursor": 0 })
        )
        .await,
        401,
        "ack with a fenced token is refused"
    );
    assert_eq!(
        probe(
            "/v1/nodes/poll",
            &old_token,
            old_epoch,
            json!({ "after_cursor": 0 })
        )
        .await,
        401,
        "poll with a fenced token is refused"
    );

    // The new token is the live lease's.
    assert_eq!(
        probe("/v1/nodes/heartbeat", &new_token, _new_epoch, json!({})).await,
        200,
        "the current token renews"
    );
    server.crash();
}

/// Lease expiry is observable AND enforced: backdating the expiry flips presence
/// to `offline` and refuses channel traffic (a heartbeat cannot resurrect an
/// expired lease) — only a fresh handshake, a NEW key-proof, restores it.
#[tokio::test]
async fn lease_expiry_flips_presence_offline_and_refuses_channel_traffic() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let node_id = "nod_00000000-0000-7000-8000-000000000104".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;

    let channel = reasonbraid_node::NodeChannel::new(
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    );
    let handshake_req = reasonbraid_node::HandshakeRequest {
        channel_version: reasonbraid_node::CHANNEL_VERSION,
        node_id: node_id.clone(),
        last_acked_cursor: 0,
        pending_operations: vec![],
        ambiguous_attempts: vec![],
        cert_der: String::new(),
        proof_signature: String::new(),
    };
    let first = channel.handshake(&handshake_req).await.expect("handshake");
    assert_eq!(
        first.fencing_token,
        channel.heartbeat().await.expect("heartbeat").fencing_token,
        "sanity: a heartbeat echoes the live token (only the handshake rotates)"
    );

    let presence = async || -> Value {
        client
            .get(format!(
                "{}/v1/nodes/presence?node_id={node_id}",
                server.base_url()
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    };
    assert_eq!(presence().await["online"], json!(true));

    // The clock runs out (the suite fast-forwards it; a real expiry is just time).
    sqlx::query(
        "UPDATE node_leases SET lease_expires_at = now() - interval '1 second' WHERE node_id = $1",
    )
    .bind(&node_id)
    .execute(&pool)
    .await
    .expect("backdate the lease");
    assert_eq!(
        presence().await["online"],
        json!(false),
        "expiry flips presence to offline"
    );

    // Channel traffic on the expired lease is refused...
    let expired_heartbeat = channel.heartbeat().await.unwrap_err();
    assert!(
        expired_heartbeat.to_string().contains("expired"),
        "got: {expired_heartbeat}"
    );
    let expired_ack = channel.acknowledge(0).await.unwrap_err();
    assert!(
        expired_ack.to_string().contains("expired"),
        "got: {expired_ack}"
    );

    // ...and a heartbeat cannot resurrect it; only a new key-proof can.
    let second = channel
        .handshake(&handshake_req)
        .await
        .expect("re-handshake");
    assert_ne!(second.fencing_token, first.fencing_token);
    assert_eq!(presence().await["online"], json!(true));
    channel.heartbeat().await.expect("the new lease renews");
    server.crash();
}

/// THE `.1.2.2` rotation acceptance: the node presents its CURRENT certificate
/// and receives a FRESH key + certificate (a NEW fingerprint) for the same node
/// id; rotation is ADDITIVE — the old leaf stays valid until expiry/revocation
/// (`.1.3`) — and the fresh identity handshakes.
#[tokio::test]
async fn rotation_issues_a_fresh_certificate_and_both_identities_handshake() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000220".to_string();
    let (cert_der, key_der) = seed_node(&pool, &node_id).await;
    let old_fp = reasonbraid_server::ca::cert_fingerprint(&cert_der);

    let channel = reasonbraid_node::NodeChannel::new(
        server.base_url(),
        node_id.clone(),
        cert_der.clone(),
        key_from_der(&key_der),
    );
    let (fresh_cert, fresh_key) = channel.rotate().await.expect("rotate succeeds");
    let fresh_fp = reasonbraid_server::ca::cert_fingerprint(&fresh_cert);
    assert_ne!(fresh_fp, old_fp, "rotation issues a fresh fingerprint");

    // The rotation is ADDITIVE: both rows exist for the same node id.
    let (n,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM node_certificates WHERE node_id = $1 AND revoked_at IS NULL",
    )
    .bind(&node_id)
    .fetch_one(&pool)
    .await
    .expect("count certs");
    assert_eq!(n, 2, "the old leaf stays valid alongside the fresh one");

    // The FRESH identity handshakes (installed; the client fills cert + proof).
    channel.install_identity(fresh_cert, fresh_key);
    channel
        .handshake(&reasonbraid_node::HandshakeRequest {
            channel_version: 5,
            node_id: node_id.clone(),
            last_acked_cursor: 0,
            pending_operations: vec![],
            ambiguous_attempts: vec![],
            cert_der: String::new(),
            proof_signature: String::new(),
        })
        .await
        .expect("the fresh identity handshakes");

    // The OLD identity still handshakes too (no silent invalidation).
    let old_channel = reasonbraid_node::NodeChannel::new(
        server.base_url(),
        node_id.clone(),
        cert_der,
        key_from_der(&key_der),
    );
    old_channel
        .handshake(&reasonbraid_node::HandshakeRequest {
            channel_version: 5,
            node_id,
            last_acked_cursor: 0,
            pending_operations: vec![],
            ambiguous_attempts: vec![],
            cert_der: String::new(),
            proof_signature: String::new(),
        })
        .await
        .expect("the pre-rotation leaf stays valid");

    server.crash();
}

/// A rotate call without a valid certificate proof is refused 401, identically
/// to an unenrolled node (no existence leak).
#[tokio::test]
async fn rotation_without_a_valid_certificate_proof_is_refused() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let node_id = "nod_00000000-0000-7000-8000-000000000221".to_string();
    let _ = seed_node(&pool, &node_id).await;

    let forged = client
        .post(format!("{}/v1/nodes/rotate", server.base_url()))
        .json(&json!({
            "channel_version": 5,
            "node_id": node_id,
            "cert_der": "00",
            "proof_signature": "00",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        forged.status().as_u16(),
        401,
        "a forged rotate proof is refused"
    );
    let body: Value = forged.json().await.unwrap();
    assert_eq!(body["code"], "unauthorized");

    server.crash();
}

/// Bootstrap a human admin INTO THE SEED TENANT (the seeded nodes live there).
/// The revoke verbs are tenant_admin-audited — the same gate as token issuance.
async fn bootstrap_admin(client: &reqwest::Client, base: &str) -> (String, String) {
    let response = client
        .post(format!("{base}/v1/enrollments"))
        .json(&json!({
            "kind": "human",
            "name": "revoker",
        }))
        .send()
        .await
        .expect("enroll human");
    let status = response.status().as_u16();
    let body: Value = response.json().await.expect("enroll json");
    assert_eq!(status, 200, "the human enrolls: {body}");
    (
        body["tenant_id"].as_str().unwrap().to_string(),
        body["principal_id"].as_str().unwrap().to_string(),
    )
}

/// Bootstrap a ROLE into the seed tenant — a role never carries tenant_admin,
/// so it is the suite.s non-admin caller (a typed 403, audited).
async fn bootstrap_role(client: &reqwest::Client, base: &str, tenant: &str) -> String {
    let response = client
        .post(format!("{base}/v1/enrollments"))
        .json(&json!({
            "kind": "role",
            "name": "agent-r",
            "tenant_id": tenant,
        }))
        .send()
        .await
        .expect("enroll role");
    assert_eq!(response.status().as_u16(), 200, "the role enrolls");
    let body: Value = response.json().await.expect("enroll json");
    body["principal_id"].as_str().unwrap().to_string()
}

/// THE `.3.5.2` wake-gate acceptance: a role whose profile declares ZERO
/// concurrency is HELD at the delivery boundary — its inbox rows stay
/// `queued` (the replay skips them) until the policy admits work again.
#[tokio::test]
async fn the_zero_concurrency_wake_gate_holds_the_delivery() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);

    // The dev wiring's node==role: the role must exist for the profile join.
    let role_id = "rol_00000000-0000-7000-8000-0000000000c1".to_string();
    sqlx::query("INSERT INTO tenants (tenant_id, name) VALUES ('ten_00000000-0000-7000-8000-0000000000c1', 'gate')")
        .execute(&pool)
        .await
        .expect("seed tenant");
    sqlx::query("INSERT INTO agent_roles (role_id, tenant_id, name) VALUES ($1, $2, 'gate-role')")
        .bind(&role_id)
        .bind("ten_00000000-0000-7000-8000-0000000000c1")
        .execute(&pool)
        .await
        .expect("seed role");
    sqlx::query("INSERT INTO agent_profiles (role_id, current_version) VALUES ($1, 1)")
        .bind(&role_id)
        .execute(&pool)
        .await
        .expect("seed profile pointer");
    sqlx::query(
        "INSERT INTO profile_versions (version_id, role_id, version, content_hash, profile, written_by) \
         VALUES ('pver_gate_1', $1, 1, 'h', $2, 'agt_gate')",
    )
    .bind(&role_id)
    .bind(serde_json::json!({
        "display_label": "gate",
        "purpose": "probe",
        "conversation_modes": [],
        "capabilities": [],
        "interests": [],
        "languages": [],
        "structured_output_formats": [],
        "scopes": [],
        "confidentiality_classes": [],
        "availability": { "concurrency": 0, "operating_hours": null, "wake_policy": null },
        "resolver_tool_capabilities": [],
        "cost_latency_class": null,
        "resource_ceilings": null,
        "visibility": {},
        "grants_by_reference": [],
        "incarnation_id": null,
    }))
    .execute(&pool)
    .await
    .expect("seed the zero-concurrency profile");

    let command_id = "cmd_gate_1".to_string();
    state
        .enqueue(
            &role_id,
            &command_id,
            "ten_00000000-0000-7000-8000-0000000000c1",
            "thr_00000000-0000-7000-8000-000000000000",
            &json!({ "operation": "contribute", "command_id": command_id }),
        )
        .await
        .expect("enqueue");

    // The zero-concurrency gate HOLDS the delivery: the replay returns nothing.
    let held = state.replay(&role_id, 0).await.expect("the replay");
    assert!(held.is_empty(), "the held role receives nothing: {held:?}");

    // The policy admits work again: the delivery flows.
    sqlx::query(
        "UPDATE profile_versions SET profile = jsonb_set(profile, '{availability,concurrency}', '2') \
         WHERE role_id = $1 AND version = 1",
    )
    .bind(&role_id)
    .execute(&pool)
    .await
    .expect("admit work");
    let delivered = state.replay(&role_id, 0).await.expect("the replay");
    assert_eq!(delivered.len(), 1, "the admitted role receives the row");
}

/// THE `.3.5.1` acceptance: the §10.6 ladder is ONE derived truth — the
/// shipped columns (the ack, the quarantine, the work-result receipt) name
/// the states through the view: `queued` → `acknowledged` → `consumed`, and
/// the quarantine reads `dead_lettered`.
#[tokio::test]
async fn the_delivery_ladder_reads_through_the_inbox_state_view() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base_url();
    let client = reqwest::Client::new();

    let (tenant, admin_id) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-0000000000b1".to_string();
    seed_node_in_tenant(&pool, &tenant, &node_id).await;

    // Three rows at three rungs of the ladder.
    sqlx::query(
        "INSERT INTO node_inbox (node_id, cursor, command_id, tenant_id, thread_id, payload) \
         VALUES ($1, 1, 'cmd_queued', $2, 'thr_00000000-0000-7000-8000-000000000001', '{}'), \
                ($1, 2, 'cmd_consumed', $2, 'thr_00000000-0000-7000-8000-000000000001', '{}'), \
                ($1, 3, 'cmd_dead', $2, 'thr_00000000-0000-7000-8000-000000000001', '{}')",
    )
    .bind(&node_id)
    .bind(&tenant)
    .execute(&pool)
    .await
    .expect("seed the rows");

    // The consumed rung: the ack + the work-result receipt.
    sqlx::query(
        "UPDATE node_inbox SET acknowledged_at = now() \
         WHERE node_id = $1 AND command_id = 'cmd_consumed'",
    )
    .bind(&node_id)
    .execute(&pool)
    .await
    .expect("ack");
    sqlx::query(
        "INSERT INTO node_events (event_id, node_id, operation_id, payload) \
         VALUES ('evt_walk_1', $1, 'cmd_consumed', '{\"kind\":\"work_result\",\"command_id\":\"cmd_consumed\"}')",
    )
    .bind(&node_id)
    .execute(&pool)
    .await
    .expect("the receipt");

    // The dead-letter rung: the quarantine IS the dead letter.
    sqlx::query(
        "UPDATE node_inbox SET quarantined_at = now(), quarantine_reason = 'the retry budget is exhausted' \
         WHERE node_id = $1 AND command_id = 'cmd_dead'",
    )
    .bind(&node_id)
    .execute(&pool)
    .await
    .expect("quarantine");

    // The inspection reads the derived states through the view.
    let response = client
        .get(format!(
            "{base}/v1/nodes/inbox?tenant_id={tenant}&node_id={node_id}"
        ))
        .header(PRINCIPAL_HEADER, &admin_id)
        .send()
        .await
        .expect("inspection request");
    assert_eq!(response.status().as_u16(), 200, "the inspection reads");
    let body: Value = response.json().await.unwrap();
    let rows = body["rows"].as_array().expect("the rows");
    assert_eq!(rows.len(), 3, "{body}");
    let state_of = |command: &str| {
        rows.iter()
            .find(|r| r["command_id"] == json!(command))
            .map(|r| r["delivery_state"].as_str().unwrap().to_string())
            .expect("the row")
    };
    assert_eq!(state_of("cmd_queued"), "queued", "{body}");
    assert_eq!(state_of("cmd_consumed"), "consumed", "{body}");
    assert_eq!(state_of("cmd_dead"), "dead_lettered", "{body}");

    server.crash();
}

/// THE `.3.2.2` distinction, measured: the never-leased enrolled node reads
/// `offline` with null clocks (a known node, never active); the expired-lease
/// node reads `offline` WITH its past expiry visible (known, just quiet); an
/// unknown id is the typed 404 (never a fabricated offline); and the operator's
/// enumeration lists the offline-KNOWN rows with their states + expiries.
#[tokio::test]
async fn the_offline_known_distinction_and_the_operator_enumeration() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base_url();
    let client = reqwest::Client::new();

    let (tenant, admin_id) = bootstrap_admin(&client, &base).await;
    let never_leased = "nod_00000000-0000-7000-8000-0000000000a1".to_string();
    let expired = "nod_00000000-0000-7000-8000-0000000000a2".to_string();
    seed_node_in_tenant(&pool, &tenant, &never_leased).await;
    seed_node_in_tenant(&pool, &tenant, &expired).await;
    // The expired-lease node: a lease whose clock ran out (the stale handling
    // already refuses its heartbeat — the fencing test; here the PRESENCE leg).
    let past = chrono::Utc::now() - chrono::Duration::minutes(5);
    sqlx::query(
        "INSERT INTO node_leases (node_id, fencing_token, lease_expires_at, last_seen_at) \
         VALUES ($1, 'fnc_expired', $2, $3)",
    )
    .bind(&expired)
    .bind(past)
    .bind(past)
    .execute(&pool)
    .await
    .expect("seed the expired lease");

    // The one-node surface: never-leased → offline, null clocks.
    let presence = client
        .get(format!("{base}/v1/nodes/presence?node_id={never_leased}"))
        .send()
        .await
        .expect("presence request");
    assert_eq!(presence.status().as_u16(), 200);
    let p: Value = presence.json().await.unwrap();
    assert_eq!(p["state"], json!("offline"), "{p}");
    assert!(p["last_seen_at"].is_null() && p["lease_expires_at"].is_null());

    // Expired-lease → offline WITH the past expiry visible.
    let presence = client
        .get(format!("{base}/v1/nodes/presence?node_id={expired}"))
        .send()
        .await
        .expect("presence request");
    assert_eq!(presence.status().as_u16(), 200);
    let p: Value = presence.json().await.unwrap();
    assert_eq!(p["state"], json!("offline"), "{p}");
    assert!(
        p["lease_expires_at"].as_str().is_some(),
        "the expired node's expiry is visible: {p}"
    );
    assert!(
        p["lease_expires_at"].as_str().unwrap() < chrono::Utc::now().to_rfc3339().as_str(),
        "the visible expiry is in the past: {p}"
    );

    // An unknown id is the typed 404, never a fabricated offline.
    let unknown = client
        .get(format!(
            "{base}/v1/nodes/presence?node_id=nod_00000000-0000-7000-8000-0000000000ff"
        ))
        .send()
        .await
        .expect("presence request");
    assert_eq!(unknown.status().as_u16(), 404);
    let u: Value = unknown.json().await.unwrap();
    assert_eq!(
        u["code"],
        json!("unknown_node"),
        "the typed unknown, never a fabricated offline: {u}"
    );

    // The operator's enumeration: BOTH offline-known rows with their states
    // and expiries (the admin's tenant scope).
    let listed = client
        .get(format!("{base}/v1/admin/nodes/presence?tenant_id={tenant}"))
        .header(PRINCIPAL_HEADER, &admin_id)
        .send()
        .await
        .expect("enumeration request");
    assert_eq!(listed.status().as_u16(), 200);
    let l: Value = listed.json().await.unwrap();
    let nodes = l["nodes"].as_array().expect("the node list");
    assert_eq!(nodes.len(), 2, "both offline-known rows: {l}");
    for node in nodes {
        assert_eq!(node["state"], json!("offline"), "{node}");
        let expiry = &node["lease_expires_at"];
        if node["node_id"] == json!(expired) {
            assert!(
                expiry.as_str().is_some(),
                "the expiry rides the row: {node}"
            );
        } else {
            assert!(
                expiry.is_null(),
                "the never-leased row has no expiry: {node}"
            );
        }
    }

    // A non-admin role reads nothing of the enumeration.
    let role = bootstrap_role(&client, &base, &tenant).await;
    let refused = client
        .get(format!("{base}/v1/admin/nodes/presence?tenant_id={tenant}"))
        .header(PRINCIPAL_HEADER, &role)
        .send()
        .await
        .expect("enumeration request");
    assert_eq!(refused.status().as_u16(), 403, "the non-admin is refused");

    server.crash();
}

/// THE `.1.3.1` acceptance: revoking a node refuses its NEXT handshake (the
/// `.1.2.2` ladder sees `revoked_at`) and presence reads `suspended` — while
/// the live lease (if any) is untouched: suspension gates re-entry, it does
/// not pretend the running session never existed.
#[tokio::test]
async fn revoking_a_node_refuses_the_next_handshake_and_flips_presence_suspended() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, alice) = bootstrap_admin(&client, &server.base_url()).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000300".to_string();
    let (cert_der, key_der) = seed_node_in_tenant(&pool, &tenant, &node_id).await;

    // A live handshake works first (and issues a live lease).
    let channel = reasonbraid_node::NodeChannel::new(
        server.base_url(),
        node_id.clone(),
        cert_der,
        key_from_der(&key_der),
    );
    channel
        .handshake(&reasonbraid_node::HandshakeRequest {
            channel_version: 5,
            node_id: node_id.clone(),
            last_acked_cursor: 0,
            pending_operations: vec![],
            ambiguous_attempts: vec![],
            cert_der: String::new(),
            proof_signature: String::new(),
        })
        .await
        .expect("pre-revocation handshake");

    let response = client
        .post(format!("{}/v1/nodes/revoke", server.base_url()))
        .header(PRINCIPAL_HEADER, &alice)
        .json(&json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "reason": "compromised adapter output",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200, "the revocation succeeds");
    let body: Value = response.json().await.unwrap();
    assert!(body["revoked_certificates"].as_i64().unwrap() >= 1);

    // The NEXT handshake is refused: the ladder sees the revoked row.
    let refused = channel
        .handshake(&reasonbraid_node::HandshakeRequest {
            channel_version: 5,
            node_id: node_id.clone(),
            last_acked_cursor: 0,
            pending_operations: vec![],
            ambiguous_attempts: vec![],
            cert_der: String::new(),
            proof_signature: String::new(),
        })
        .await;
    assert!(
        matches!(
            refused,
            Err(reasonbraid_node::ChannelError::Server { status: 401, .. })
        ),
        "the revoked certificate is refused at the next crossing: {refused:?}"
    );

    // Presence reads suspended; the live lease (from the earlier handshake)
    // is untouched — suspension gates re-entry, it does not rewrite history.
    let presence: Value = client
        .get(format!(
            "{}/v1/nodes/presence?node_id={node_id}",
            server.base_url()
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(presence["suspended"], json!(true));
    assert_eq!(
        presence["online"],
        json!(true),
        "the live lease was not cut"
    );

    server.crash();
}

/// The revocation refusals are typed and audited: an unknown node is a 404, a
/// non-admin caller is the 403 + audit row, and revoking twice is the 409
/// (no active certificate remains).
#[tokio::test]
async fn revocation_refusals_are_typed_and_audited() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, alice) = bootstrap_admin(&client, &server.base_url()).await;
    let role = bootstrap_role(&client, &server.base_url(), &tenant).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000301".to_string();
    let _ = seed_node_in_tenant(&pool, &tenant, &node_id).await;

    let unknown = client
        .post(format!("{}/v1/nodes/revoke", server.base_url()))
        .header(PRINCIPAL_HEADER, &alice)
        .json(&json!({
            "tenant_id": tenant,
            "node_id": "nod_00000000-0000-7000-8000-0000000003ff",
            "reason": "nope",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(unknown.status().as_u16(), 404, "an unknown node is a 404");

    let non_admin = client
        .post(format!("{}/v1/nodes/revoke", server.base_url()))
        .header(PRINCIPAL_HEADER, &role) // a role never carries tenant_admin
        .json(&json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "reason": "self-serve",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        non_admin.status().as_u16(),
        403,
        "a non-admin caller is a 403"
    );
    let (n_denied,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM authorization_records WHERE tenant_id = $1 AND decision = 'denied'",
    )
    .bind(&tenant)
    .fetch_one(&pool)
    .await
    .expect("count denials");
    assert!(n_denied >= 1, "the refusal is audited");

    let revoked = client
        .post(format!("{}/v1/nodes/revoke", server.base_url()))
        .header(PRINCIPAL_HEADER, &alice)
        .json(&json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "reason": "legitimate",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(revoked.status().as_u16(), 200);
    let again = client
        .post(format!("{}/v1/nodes/revoke", server.base_url()))
        .header(PRINCIPAL_HEADER, &alice)
        .json(&json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "reason": "again",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        again.status().as_u16(),
        409,
        "a second revocation finds no active certificate"
    );

    server.crash();
}

/// THE `.2.2` renewal-race leg, deterministic at the state level: after a
/// second handshake rotates the lease (new token, bumped epoch), a renewal
/// carrying the OLD epoch matches no row — the stale heartbeat loses the race
/// instead of extending the session that fenced it. The in-tx verifier
/// refuses the stale epoch the same way (the check-vs-commit window).
#[tokio::test]
async fn a_stale_epoch_renewal_loses_the_race() {
    let _guard = channel_guard().await;
    let Some(pool) = pool().await else { return };
    let node_id = "nod_00000000-0000-7000-8000-000000000152".to_string();
    seed_node(&pool, &node_id).await;
    let ca = Arc::new(ensure_server_ca(&pool).await.expect("server CA"));
    let state = NodeChannelState::new(pool.clone(), ca);

    let (token_a, epoch_a, _expiry_a) = state
        .issue_lease(&node_id, Utc::now())
        .await
        .expect("first lease");
    let (token_b, epoch_b, _expiry_b) = state
        .issue_lease(&node_id, Utc::now())
        .await
        .expect("second lease rotates");
    assert_ne!(token_a, token_b, "rotation issues a fresh token");
    assert_eq!(
        epoch_b,
        epoch_a + 1,
        "every handshake bumps the lease epoch"
    );

    // THE race: the stale heartbeat's check passed (epoch_a was live when it
    // verified) but the write lands AFTER the rotation — the epoch guard
    // matches no row, and the renewal is refused instead of extending the
    // NEW session's lease.
    let raced = state.renew_lease(&node_id, epoch_a, Utc::now()).await;
    assert!(
        raced.is_err(),
        "a renewal from a fenced epoch matches no row"
    );

    // The epoch rides the fencing check: the honest pair passes, a stale
    // epoch fails even WITH the current token (the token is never the whole
    // story).
    state
        .verify_fencing(&node_id, &token_b, epoch_b)
        .await
        .expect("the current token + epoch verify");
    assert!(
        state
            .verify_fencing(&node_id, &token_b, epoch_a)
            .await
            .is_err(),
        "a stale epoch with the current token is refused"
    );

    // The check-vs-commit window: the same guard INSIDE a transaction.
    let mut tx = pool.begin().await.expect("begin");
    assert!(
        state
            .verify_fencing_in_tx(&mut tx, &node_id, &token_b, epoch_a)
            .await
            .is_err(),
        "the in-tx verifier refuses the stale epoch"
    );
    state
        .verify_fencing_in_tx(&mut tx, &node_id, &token_b, epoch_b)
        .await
        .expect("the in-tx verifier accepts the honest pair");
    tx.commit().await.expect("commit");
}
