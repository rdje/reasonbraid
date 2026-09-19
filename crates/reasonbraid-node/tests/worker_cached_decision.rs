//! Worker dispatch-gate tests (`PHASE-2.1.5.2`; ADR-008): the cached ADMISSION
//! decision the delivery carries is evaluated at the dispatch boundary — a fresh,
//! epoch-current cached allow dispatches; an expired, epoch-stale, denied, or
//! MISSING decision refuses the irreversible write (fail-closed, journaled as
//! `failed_before_dispatch`). These legs never contact a channel: the refusal
//! happens before any adapter or transport use, so a dummy node works.

use std::path::PathBuf;
use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use reasonbraid_adapter::{AdapterCapabilities, FakeAdapter, ScriptStep, StatusLookupSpec};
use reasonbraid_core::{BudgetDimensions, CACHED_ALLOW_TTL_SECONDS};
use reasonbraid_node::{Journal, LocalBudget, Node, Worker, WorkerError};
use serde_json::{json, Value};

fn journal_path(name: &str) -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let unique = uuid::Uuid::now_v7();
    let dir = base
        .join("cached-decision-tests")
        .join(format!("{name}-{unique}"));
    std::fs::create_dir_all(dir.parent().expect("the fixture parent")).unwrap();
    // Exclusive: an existing directory belongs to another fixture or an
    // earlier run, and must never be adopted.
    std::fs::DirBuilder::new()
        .create(&dir)
        .expect("the fixture directory is new");
    dir.join("node.db")
}

/// A node whose channel base points nowhere — the refusal legs never touch it.
async fn dummy_node(name: &str) -> Node {
    let key = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).expect("keypair");
    Node::open(
        journal_path(name),
        "http://127.0.0.1:1",
        "nod_00000000-0000-7000-8000-000000000001".to_string(),
        vec![0x00, 0x01, 0x02],
        key,
    )
    .await
    .expect("open node")
}

fn completing_adapter() -> FakeAdapter {
    FakeAdapter::new(
        vec![
            ScriptStep::EmitChunk {
                chunk: "done".to_string(),
            },
            ScriptStep::Complete { usage: None },
        ],
        StatusLookupSpec::Unsupported,
        AdapterCapabilities {
            streaming: false,
            cancellation: reasonbraid_adapter::CancellationStrength::BestEffort,
            provider_idempotency: false,
            status_lookup: false,
            tool_support: false,
            policy_injection: reasonbraid_adapter::PolicyInjectionMode::None,
        },
    )
}

/// Journal one command with the given decision metadata; return its WorkItem id.
/// The receipt is NOW — the ordinary case, where the node takes the decision in
/// after the server made it.
async fn seed_command(
    journal: &Journal,
    tag: &str,
    decided_at: Option<chrono::DateTime<Utc>>,
    revocation_epoch: Option<i64>,
) -> String {
    seed_command_at(journal, tag, decided_at, revocation_epoch, Utc::now()).await
}

/// Journal one command with an explicit RECEIPT instant (`.3.4.3`). The receipt
/// is the node's own clock, and it anchors the freshness window; the tests that
/// drive the two clocks apart need to set it independently of `decided_at`.
async fn seed_command_at(
    journal: &Journal,
    tag: &str,
    decided_at: Option<chrono::DateTime<Utc>>,
    revocation_epoch: Option<i64>,
    received_at: chrono::DateTime<Utc>,
) -> String {
    let command_id = format!("cmd_{tag}");
    let payload = json!({
        "kind": "contribute",
        "reservation": {
            "reservation_id": "res_00000000-0000-7000-8000-000000000001",
            "dimensions": { "calls": 1, "wall_clock_seconds": 60 },
            "issued_at": Utc::now().to_rfc3339(),
            "expires_at": (Utc::now() + chrono::Duration::minutes(10)).to_rfc3339(),
        },
    });
    let decided_at_s = decided_at.map(|d| d.to_rfc3339());
    journal
        .record_command(
            &reasonbraid_node::CommandInput {
                command_id: &command_id,
                tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                thread_id: "thr_00000000-0000-7000-8000-000000000000",
                payload: &payload,
                authz_ref: Some("authz_00000000-0000-7000-8000-000000000001"),
                policy_digest: Some("digest-a"),
                decided_at: decided_at_s.as_deref(),
                revocation_epoch,
                server_cursor: "1",
            },
            received_at,
        )
        .await
        .expect("record command");
    command_id
}

/// The latest attempt status for a command's operation, if any.
async fn latest_status(journal: &Journal, command_id: &str) -> Option<String> {
    let item = journal
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == command_id)
        .expect("seeded work item");
    item.latest_attempt_status
}

#[tokio::test]
async fn an_expired_cached_allow_refuses_the_dispatch() {
    let node = dummy_node("expired").await;
    node.journal()
        .set_revocation_epoch(7)
        .await
        .expect("set epoch");
    let decided_at = Utc::now() - ChronoDuration::seconds(CACHED_ALLOW_TTL_SECONDS + 10);
    let command_id = seed_command(node.journal(), "expired", Some(decided_at), Some(7)).await;
    let worker = Worker::new(
        node.clone(),
        completing_adapter(),
        LocalBudget::new(BudgetDimensions {
            calls: Some(100),
            input_tokens: Some(100_000),
            output_tokens: Some(100_000),
            wall_clock_seconds: Some(10_000),
        }),
        Duration::from_secs(1),
    );

    let items = node.journal().work_items().await.expect("work items");
    worker
        .process(&items[0])
        .await
        .expect("process refuses, no error");

    let status = latest_status(node.journal(), &command_id)
        .await
        .expect("a refusal journals an attempt");
    assert_eq!(
        status, "failed_before_dispatch",
        "an expired cached allow refuses the dispatch"
    );
}

#[tokio::test]
async fn an_epoch_bump_invalidates_a_fresh_cached_allow() {
    let node = dummy_node("epoch-bump").await;
    // The cached decision was made under epoch 7; the node has since SEEN epoch 8
    // (a revocation happened after admission).
    node.journal()
        .set_revocation_epoch(8)
        .await
        .expect("set epoch");
    let command_id = seed_command(node.journal(), "bumped", Some(Utc::now()), Some(7)).await;
    let worker = Worker::new(
        node.clone(),
        completing_adapter(),
        LocalBudget::new(BudgetDimensions {
            calls: Some(100),
            input_tokens: Some(100_000),
            output_tokens: Some(100_000),
            wall_clock_seconds: Some(10_000),
        }),
        Duration::from_secs(1),
    );

    let items = node.journal().work_items().await.expect("work items");
    worker
        .process(&items[0])
        .await
        .expect("process refuses, no error");

    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("failed_before_dispatch"),
        "an epoch bump invalidates a fresh-looking cached allow"
    );
}

#[tokio::test]
async fn a_command_without_a_cached_decision_refuses_the_dispatch() {
    let node = dummy_node("no-decision").await;
    node.journal()
        .set_revocation_epoch(7)
        .await
        .expect("set epoch");
    // No decision metadata at all — a pre-0013 row or plain channel traffic.
    let command_id = seed_command(node.journal(), "plain", None, None).await;
    let worker = Worker::new(
        node.clone(),
        completing_adapter(),
        LocalBudget::new(BudgetDimensions {
            calls: Some(100),
            input_tokens: Some(100_000),
            output_tokens: Some(100_000),
            wall_clock_seconds: Some(10_000),
        }),
        Duration::from_secs(1),
    );

    let items = node.journal().work_items().await.expect("work items");
    worker
        .process(&items[0])
        .await
        .expect("process refuses, no error");

    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("failed_before_dispatch"),
        "no cached decision = no dispatch (fail-closed)"
    );
}

#[tokio::test]
async fn a_fresh_epoch_current_cached_allow_dispatches() {
    let node = dummy_node("fresh").await;
    node.journal()
        .set_revocation_epoch(7)
        .await
        .expect("set epoch");
    let command_id = seed_command(node.journal(), "fresh", Some(Utc::now()), Some(7)).await;
    let worker = Worker::new(
        node.clone(),
        completing_adapter(),
        LocalBudget::new(BudgetDimensions {
            calls: Some(100),
            input_tokens: Some(100_000),
            output_tokens: Some(100_000),
            wall_clock_seconds: Some(10_000),
        }),
        Duration::from_secs(1),
    );

    let items = node.journal().work_items().await.expect("work items");
    // The dispatch proceeds through the gate; the supervisor runs the adapter,
    // the attempt COMPLETES — and the result emission fails (no server behind
    // the dummy URL). That failure proves the dispatch WAS allowed (a refused
    // dispatch returns Ok, never a channel error).
    let result = worker.process(&items[0]).await;
    match result {
        Err(WorkerError::Channel(_)) | Err(WorkerError::Node(_)) => {}
        other => {
            panic!("a fresh allow dispatches; the emit fails on the dummy URL — got {other:?}")
        }
    }
    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("completed"),
        "a fresh, epoch-current cached allow reaches the adapter and completes"
    );
}

/// A work item with a reservation that the LOCAL budget refuses: the cached
/// decision allows, and the budget gate (unchanged) still refuses the dispatch.
#[tokio::test]
async fn the_budget_gate_still_runs_after_the_cached_decision_allows() {
    let node = dummy_node("budget").await;
    node.journal()
        .set_revocation_epoch(7)
        .await
        .expect("set epoch");
    let command_id = "cmd_budget".to_string();
    let payload: Value = json!({
        "kind": "contribute",
        "reservation": {
            "reservation_id": "",
            "dimensions": {},
            "issued_at": Utc::now().to_rfc3339(),
            "expires_at": (Utc::now() + chrono::Duration::minutes(10)).to_rfc3339(),
        },
    });
    let decided_at = Utc::now().to_rfc3339();
    node.journal()
        .record_command(
            &reasonbraid_node::CommandInput {
                command_id: &command_id,
                tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                thread_id: "thr_00000000-0000-7000-8000-000000000000",
                payload: &payload,
                authz_ref: Some("authz_00000000-0000-7000-8000-000000000002"),
                policy_digest: Some("digest-b"),
                decided_at: Some(&decided_at),
                revocation_epoch: Some(7),
                server_cursor: "1",
            },
            Utc::now(),
        )
        .await
        .expect("record command");
    let worker = Worker::new(
        node.clone(),
        completing_adapter(),
        LocalBudget::new(BudgetDimensions {
            calls: Some(100),
            input_tokens: Some(100_000),
            output_tokens: Some(100_000),
            wall_clock_seconds: Some(10_000),
        }),
        Duration::from_secs(1),
    );

    let items = node.journal().work_items().await.expect("work items");
    worker
        .process(&items[0])
        .await
        .expect("budget refusal is a normal skip");

    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("failed_before_dispatch"),
        "the budget gate still refuses (the cached decision allowed, the reservation refused)"
    );
}

// ── `.3.4.3`: the two clocks ────────────────────────────────────────────────
//
// `decided_at` is the SERVER's database clock, sampled inside the authorizing
// transaction; the freshness comparison runs against the NODE's clock. The
// window therefore runs from `min(decided_at, received_at)`, so a node running
// behind the server gets one TTL of its OWN observed time rather than
// `skew + TTL`. These three legs pin the three moving parts: the clamp, the
// replay refresh that keeps it from breaking re-delivery, and the guard that
// keeps a plain redelivery from re-anchoring it.

/// A worker whose local budget never refuses — these legs measure the cached
/// decision gate, not the budget one.
fn worker_with_ample_budget(node: &Node) -> Worker<FakeAdapter> {
    Worker::new(
        node.clone(),
        completing_adapter(),
        LocalBudget::new(BudgetDimensions {
            calls: Some(100),
            input_tokens: Some(100_000),
            output_tokens: Some(100_000),
            wall_clock_seconds: Some(10_000),
        }),
        Duration::from_secs(1),
    )
}

/// The `.2.4` replay contract, which outlived the mechanism it was written for:
/// an operator replays a command dead-lettered long ago, and the re-dispatch
/// MUST run on the strength of the FRESH decision, however old the first
/// delivery was.
///
/// ⛔ It was added by `.3.4.3` to prove its receipt-anchoring clamp had not
/// broken replay. `.3.4.3.1.2` removed that clamp, and the property this test
/// states is independent of it — so the test stays and its name changes to
/// describe what it actually proves.
#[tokio::test]
async fn a_replayed_decision_dispatches_however_old_the_first_delivery_was() {
    let node = dummy_node("replay-anchor").await;
    node.journal()
        .set_revocation_epoch(7)
        .await
        .expect("set epoch");
    // The original delivery: two days old, long expired on both clocks.
    let long_ago = Utc::now() - ChronoDuration::days(2);
    let command_id = seed_command_at(
        node.journal(),
        "replay-anchor",
        Some(long_ago),
        Some(7),
        long_ago,
    )
    .await;

    // THE replay (`.2.4`): the same command id, a FRESH admission decision.
    let fresh = Utc::now();
    seed_command_at(node.journal(), "replay-anchor", Some(fresh), Some(7), fresh).await;

    let worker = worker_with_ample_budget(&node);
    let items = node.journal().work_items().await.expect("work items");
    // The dispatch proceeds; the emit then fails against the dummy URL, which
    // is what proves the gate ALLOWED it (a refusal returns Ok).
    match worker.process(&items[0]).await {
        Err(WorkerError::Channel(_)) | Err(WorkerError::Node(_)) => {}
        other => panic!("the replayed decision must dispatch — got {other:?}"),
    }
    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("completed"),
        "a replayed admission is fresh from ITS delivery, however old the first was"
    );
}

// ── `.3.4.3.1.2`: the node evaluates server instants in the server's terms ──
//
// ⛔ Two of `.3.4.3`'s controls stood here and are SUPERSEDED, not dropped:
// `a_decision_dated_in_the_nodes_future_expires_one_ttl_after_receipt` and
// `a_plain_redelivery_does_not_re_anchor_the_freshness_window`. Both asserted
// the receipt-anchoring clamp, which is gone, and both drove a state that
// CANNOT occur in production — a known revocation epoch with no clock offset,
// when the two arrive in the same response. The property they protected (one
// TTL of real time whatever this node's clock says) is asserted below in BOTH
// skew directions, which the clamp could only do for one.

/// THE outage this leaf closes. The node's clock runs 10 minutes AHEAD of the
/// server, so every admission decision the server sends looks 10 minutes old.
/// Past `CACHED_ALLOW_TTL_SECONDS` of skew that refused EVERY dispatch — the
/// node did no work at all, fail-closed, with a reason line whose epochs match.
#[tokio::test]
async fn a_node_clock_ahead_of_the_server_still_dispatches() {
    let node = dummy_node("clock-ahead").await;
    node.journal()
        .set_revocation_epoch(7)
        .await
        .expect("set epoch");

    // This node is 600 s ahead: the server's clock reads 600 s BEFORE ours.
    let local = Utc::now();
    let server_time = local - ChronoDuration::seconds(600);
    node.journal()
        .record_server_time(server_time, local, local)
        .await
        .expect("record the server's clock");

    // The server decided just now — in ITS terms. Received now, in ours.
    let command_id = seed_command_at(
        node.journal(),
        "clock-ahead",
        Some(server_time),
        Some(7),
        local,
    )
    .await;

    let worker = worker_with_ample_budget(&node);
    let items = node.journal().work_items().await.expect("work items");
    // The dispatch proceeds; the emit then fails against the dummy URL, which
    // is what proves the gate ALLOWED it (a refusal returns Ok).
    match worker.process(&items[0]).await {
        Err(WorkerError::Channel(_)) | Err(WorkerError::Node(_)) => {}
        other => panic!("a decision fresh in the SERVER's terms must dispatch — got {other:?}"),
    }
    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("completed"),
        "a node whose clock runs ahead must still do its work"
    );
}

/// The other direction, which `.3.4.3` bounded and this leaf now corrects: the
/// node's clock is 10 minutes BEHIND, so decisions arrive dated in its future.
/// The window must still be one TTL of real time — not `skew + TTL`, and not
/// zero either.
#[tokio::test]
async fn a_node_clock_behind_the_server_gets_exactly_one_window() {
    let node = dummy_node("clock-behind").await;
    node.journal()
        .set_revocation_epoch(7)
        .await
        .expect("set epoch");

    let local = Utc::now();
    let server_time = local + ChronoDuration::seconds(600);
    node.journal()
        .record_server_time(server_time, local, local)
        .await
        .expect("record the server's clock");

    // Decided in the server's terms one full TTL ago: stale in real time.
    let command_id = seed_command_at(
        node.journal(),
        "clock-behind",
        Some(server_time - ChronoDuration::seconds(CACHED_ALLOW_TTL_SECONDS + 10)),
        Some(7),
        local,
    )
    .await;

    let worker = worker_with_ample_budget(&node);
    let items = node.journal().work_items().await.expect("work items");
    worker
        .process(&items[0])
        .await
        .expect("the gate refuses; a refusal is Ok, never a channel error");
    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("failed_before_dispatch"),
        "a decision older than one TTL in the server's terms is stale, whatever \
         this node's clock says"
    );
}

/// And the same backward skew with a FRESH decision must dispatch — the pair
/// above and this one together pin that the correction is a shift, not a
/// blanket allow or a blanket refusal.
#[tokio::test]
async fn a_node_clock_behind_the_server_still_dispatches_a_fresh_decision() {
    let node = dummy_node("clock-behind-fresh").await;
    node.journal()
        .set_revocation_epoch(7)
        .await
        .expect("set epoch");

    let local = Utc::now();
    let server_time = local + ChronoDuration::seconds(600);
    node.journal()
        .record_server_time(server_time, local, local)
        .await
        .expect("record the server's clock");

    let command_id = seed_command_at(
        node.journal(),
        "clock-behind-fresh",
        Some(server_time),
        Some(7),
        local,
    )
    .await;

    let worker = worker_with_ample_budget(&node);
    let items = node.journal().work_items().await.expect("work items");
    match worker.process(&items[0]).await {
        Err(WorkerError::Channel(_)) | Err(WorkerError::Node(_)) => {}
        other => panic!("a fresh decision must dispatch under backward skew — got {other:?}"),
    }
    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("completed"),
    );
}
