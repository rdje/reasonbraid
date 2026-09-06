//! WP3 node channel integration tests (`PHASE-0.3.2`): the REAL outbound channel —
//! the axum server in this crate and the node client from `reasonbraid-node` talk over
//! actual `127.0.0.1` sockets, with the durable PostgreSQL inbox behind the server.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job. Without
//! `DATABASE_URL` these skip, so `make check` stays green offline. The node side needs
//! no service: its SQLite journal is a file.
//!
//! The acceptance: reconnect exchanges the last acknowledged server cursor and the
//! pending local operation ids; a duplicated command never creates a second local
//! operation; the node is not schedulable until reconciliation completes. Plus the WP3
//! exercises: network loss, server restart, duplicate delivery, and cursor rewind
//! (the node reporting a cursor this server cannot reproduce).

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;

use chrono::Utc;
use reasonbraid_node::{Journal, Node, NodeState};
use reasonbraid_server::{node_router, NodeChannelState};
use serde_json::{json, Value};
use sqlx::PgPool;

/// The channel tests own the inbox/event tables (like the outbox tests own the queue):
/// tests never run concurrently against the same PG database.
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
    // These tests exclusively own the channel tables for their duration.
    sqlx::query("DELETE FROM node_events")
        .execute(&pool)
        .await
        .expect("purge node_events");
    sqlx::query("DELETE FROM node_inbox")
        .execute(&pool)
        .await
        .expect("purge node_inbox");
    Some(pool)
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
        let router = node_router(pool.clone());
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
    let state = NodeChannelState::new(pool.clone());
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000001".to_string();

    for i in 1..=3 {
        enqueue(&state, &node_id, &format!("cmd_fresh_{i}")).await;
    }

    let node = Node::open(journal_path("fresh"), server.base_url(), node_id.clone())
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
    let state = NodeChannelState::new(pool.clone());
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000002".to_string();

    for i in 1..=3 {
        enqueue(&state, &node_id, &format!("cmd_tail_{i}")).await;
    }
    let node = Node::open(journal_path("tail"), server.base_url(), node_id.clone())
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
    let state = NodeChannelState::new(pool.clone());
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000003".to_string();

    for i in 1..=3 {
        enqueue(&state, &node_id, &format!("cmd_dup_{i}")).await;
    }
    let node = Node::open(
        journal_path("duplicate"),
        server.base_url(),
        node_id.clone(),
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
    let state = NodeChannelState::new(pool.clone());
    let node_id = "nod_00000000-0000-7000-8000-000000000004".to_string();

    // A port with nothing listening: the channel is unreachable.
    let dead = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dead_url = format!("http://{}", dead.local_addr().unwrap());
    drop(dead);

    let journal = journal_path("schedulable");
    let node = Node::open(&journal, dead_url, node_id.clone())
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
    let node = Node::open(&journal, server.base_url(), node_id.clone())
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

    let node = Node::open(&journal_path, server.base_url(), node_id.clone())
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
    let state = NodeChannelState::new(pool.clone());
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000006".to_string();

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

    let node = Node::open(&journal_path, server.base_url(), node_id.clone())
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

    let node = Node::open(&journal_path, server.base_url(), node_id.clone())
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
    let state = NodeChannelState::new(pool.clone());
    let node_id = "nod_00000000-0000-7000-8000-000000000008".to_string();

    let journal_path = journal_path("restart");
    {
        let server = TestServer::start(&pool).await;
        for i in 1..=2 {
            enqueue(&state, &node_id, &format!("cmd_rst_{i}")).await;
        }
        let node = Node::open(&journal_path, server.base_url(), node_id.clone())
            .await
            .unwrap();
        node.reconcile().await.unwrap();
        server.crash(); // the server process dies
    }

    // The node's next reconcile fails (nothing listening) — not schedulable.
    let dead = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dead_url = format!("http://{}", dead.local_addr().unwrap());
    drop(dead);
    let node = Node::open(&journal_path, dead_url, node_id.clone())
        .await
        .unwrap();
    assert!(node.reconcile().await.is_err());
    assert_ne!(node.state().await, NodeState::Schedulable);

    // A NEW server process, the SAME durable PostgreSQL: resume from the recorded cursor.
    let server = TestServer::start(&pool).await;
    for i in 3..=4 {
        enqueue(&state, &node_id, &format!("cmd_rst_{i}")).await;
    }
    let node = Node::open(&journal_path, server.base_url(), node_id.clone())
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
    let state = NodeChannelState::new(pool.clone());
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000009".to_string();

    enqueue(&state, &node_id, "cmd_ahead_1").await;
    enqueue(&state, &node_id, "cmd_ahead_2").await;

    let node = Node::open(journal_path("ahead"), server.base_url(), node_id.clone())
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
    let state = NodeChannelState::new(pool.clone());
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000010".to_string();

    for i in 1..=3 {
        enqueue(&state, &node_id, &format!("cmd_poll_{i}")).await;
    }
    let node = Node::open(journal_path("poll"), server.base_url(), node_id.clone())
        .await
        .unwrap();
    node.reconcile().await.unwrap();

    for i in 4..=5 {
        enqueue(&state, &node_id, &format!("cmd_poll_{i}")).await;
    }
    let tail = reasonbraid_node::NodeChannel::new(server.base_url(), node_id)
        .poll(3)
        .await
        .expect("poll");

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
            "ambiguous_attempts": []
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
    let state = NodeChannelState::new(pool.clone());
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000011".to_string();

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

    let node = Node::open(&journal_path, server.base_url(), node_id.clone())
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
    let state = NodeChannelState::new(pool.clone());
    let server = TestServer::start(&pool).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000012".to_string();

    enqueue(&state, &node_id, "cmd_dedupe").await;
    let node = Node::open(
        journal_path("event-dedupe"),
        server.base_url(),
        node_id.clone(),
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
