//! Worker dispatch-gate tests (`PHASE-2.1.5.2`; ADR-008): the cached ADMISSION
//! decision the delivery carries is evaluated at the dispatch boundary — a fresh,
//! epoch-current cached allow dispatches; an expired, epoch-stale, denied, or
//! MISSING decision refuses the irreversible write (fail-closed, journaled as
//! `failed_before_dispatch`). These legs never contact a channel: the refusal
//! happens before any adapter or transport use, so a dummy node works.

use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use reasonbraid_adapter::{AdapterCapabilities, FakeAdapter, ScriptStep, StatusLookupSpec};
use reasonbraid_core::fixture::Fixture;
use reasonbraid_core::{BudgetDimensions, CACHED_ALLOW_TTL_SECONDS};
use reasonbraid_node::{Journal, LocalBudget, Node, Worker};
use serde_json::{json, Value};

/// A fixture directory for one test, created exclusively on the repository's own
/// volume and REMOVED when the test passes (`SIGNOFF-REPAIR.11.2.1.3.2.4`). A
/// failing test keeps its fixture, and everything in it, as the diagnostic.
fn journal_fixture(name: &str) -> Fixture {
    Fixture::create("cached-decision-tests", name).expect("the fixture directory is new")
}

/// A node whose channel base points nowhere — the refusal legs never touch it.
///
/// 🔴 The guard comes back FIRST in the tuple and that is load-bearing: a `let`
/// statement drops its bindings in reverse declaration order, and a tuple
/// pattern's bindings count left to right, so `let (_fixture, node)` drops the
/// NODE first and the fixture last. The other order would remove the directory
/// while the node still held its SQLite file open.
async fn dummy_node(name: &str) -> (Fixture, Node) {
    let key = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).expect("keypair");
    let fixture = journal_fixture(name);
    let node = Node::open(
        fixture.join("node.db"),
        "http://127.0.0.1:1",
        "nod_00000000-0000-7000-8000-000000000001".to_string(),
        vec![0x00, 0x01, 0x02],
        key,
    )
    .await
    .expect("open node");
    (fixture, node)
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

/// The one work result a completed dispatch leaves waiting in the journal
/// (`SIGNOFF-REPAIR.4.4.4.1`). This node was never reconciled, so it cannot
/// deliver; the result must stay pending for the next reconcile to re-emit,
/// naming the attempt that produced it. Before the repair the completion was
/// journaled, the emission refused with an error, and the result existed
/// nowhere, so these controls accepted a paid result's loss as proof that the
/// dispatch happened.
async fn pending_result_for(journal: &Journal, command_id: &str) -> Value {
    let operation_id = journal
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == command_id)
        .and_then(|w| w.operation_id)
        .expect("the dispatched item has an operation");
    let completed: Vec<String> = journal
        .attempts_for_operation(&operation_id)
        .await
        .expect("attempts")
        .into_iter()
        .filter(|a| a.status == "completed")
        .map(|a| a.attempt_id)
        .collect();
    assert_eq!(completed.len(), 1, "exactly one completed attempt");
    let pending: Vec<Value> = journal
        .pending_events()
        .await
        .expect("pending events")
        .into_iter()
        .filter(|e| e.operation_id == operation_id)
        .map(|e| serde_json::from_str(&e.payload).expect("a JSON payload"))
        .collect();
    assert_eq!(
        pending.len(),
        1,
        "the completed dispatch left exactly one result waiting: {pending:?}"
    );
    let result = pending.into_iter().next().unwrap();
    assert_eq!(result["kind"], "work_result");
    assert_eq!(result["command_id"], command_id);
    assert_eq!(
        result["attempt_id"], completed[0],
        "the result names the attempt that produced it"
    );
    result
}

#[tokio::test]
async fn an_expired_cached_allow_refuses_the_dispatch() {
    let (_fixture, node) = dummy_node("expired").await;
    node.journal()
        .set_revocation_epoch_for("ten_00000000-0000-7000-8000-000000000000", 7)
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
    let (_fixture, node) = dummy_node("epoch-bump").await;
    // The cached decision was made under epoch 7; the node has since SEEN epoch 8
    // (a revocation happened after admission).
    node.journal()
        .set_revocation_epoch_for("ten_00000000-0000-7000-8000-000000000000", 8)
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
    let (_fixture, node) = dummy_node("no-decision").await;
    node.journal()
        .set_revocation_epoch_for("ten_00000000-0000-7000-8000-000000000000", 7)
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
    let (_fixture, node) = dummy_node("fresh").await;
    node.journal()
        .set_revocation_epoch_for("ten_00000000-0000-7000-8000-000000000000", 7)
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
    // The dispatch proceeds through the gate; the supervisor runs the adapter
    // and the attempt COMPLETES. This node was never reconciled, so the result
    // cannot be delivered: it waits in the journal, and what proves the gate
    // allowed the dispatch is the completed attempt and its waiting result.
    worker
        .process(&items[0])
        .await
        .expect("the dispatch completes; its result waits in the journal");
    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("completed"),
        "a fresh, epoch-current cached allow reaches the adapter and completes"
    );
    let result = pending_result_for(node.journal(), &command_id).await;
    assert_eq!(result["content"], "done", "the result carries the content");
}

/// A work item with a reservation that the LOCAL budget refuses: the cached
/// decision allows, and the budget gate (unchanged) still refuses the dispatch.
#[tokio::test]
async fn the_budget_gate_still_runs_after_the_cached_decision_allows() {
    let (_fixture, node) = dummy_node("budget").await;
    node.journal()
        .set_revocation_epoch_for("ten_00000000-0000-7000-8000-000000000000", 7)
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
    let (_fixture, node) = dummy_node("replay-anchor").await;
    node.journal()
        .set_revocation_epoch_for("ten_00000000-0000-7000-8000-000000000000", 7)
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
    // The dispatch proceeds and completes; the completed attempt and its
    // waiting result are what prove the gate ALLOWED it.
    worker
        .process(&items[0])
        .await
        .expect("the dispatch completes; its result waits in the journal");
    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("completed"),
        "a replayed admission is fresh from ITS delivery, however old the first was"
    );
    pending_result_for(node.journal(), &command_id).await;
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
    let (_fixture, node) = dummy_node("clock-ahead").await;
    node.journal()
        .set_revocation_epoch_for("ten_00000000-0000-7000-8000-000000000000", 7)
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
    // The dispatch proceeds and completes; the completed attempt and its
    // waiting result are what prove the gate ALLOWED it.
    worker
        .process(&items[0])
        .await
        .expect("the dispatch completes; its result waits in the journal");
    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("completed"),
        "a node whose clock runs ahead must still do its work"
    );
    pending_result_for(node.journal(), &command_id).await;
}

/// The other direction, which `.3.4.3` bounded and this leaf now corrects: the
/// node's clock is 10 minutes BEHIND, so decisions arrive dated in its future.
/// The window must still be one TTL of real time — not `skew + TTL`, and not
/// zero either.
#[tokio::test]
async fn a_node_clock_behind_the_server_gets_exactly_one_window() {
    let (_fixture, node) = dummy_node("clock-behind").await;
    node.journal()
        .set_revocation_epoch_for("ten_00000000-0000-7000-8000-000000000000", 7)
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
    let (_fixture, node) = dummy_node("clock-behind-fresh").await;
    node.journal()
        .set_revocation_epoch_for("ten_00000000-0000-7000-8000-000000000000", 7)
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
    worker
        .process(&items[0])
        .await
        .expect("the dispatch completes; its result waits in the journal");
    assert_eq!(
        latest_status(node.journal(), &command_id).await.as_deref(),
        Some("completed"),
    );
    pending_result_for(node.journal(), &command_id).await;
}

/// Journal one command of `tenant_id`, decided under `revocation_epoch`, now.
async fn seed_tenant_command(
    journal: &Journal,
    tag: &str,
    tenant_id: &str,
    revocation_epoch: i64,
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
    let decided_at = Utc::now().to_rfc3339();
    journal
        .record_command(
            &reasonbraid_node::CommandInput {
                command_id: &command_id,
                tenant_id,
                thread_id: "thr_00000000-0000-7000-8000-000000000000",
                payload: &payload,
                authz_ref: Some("authz_00000000-0000-7000-8000-000000000001"),
                policy_digest: Some("digest-a"),
                decided_at: Some(&decided_at),
                revocation_epoch: Some(revocation_epoch),
                server_cursor: "1",
            },
            Utc::now(),
        )
        .await
        .expect("record command");
    command_id
}

/// `SIGNOFF-REPAIR.5.3.5.3.2` — a node holding several tenants' work judges
/// each command's cached admission by ITS OWN tenant's epoch. A's command,
/// decided under A's current epoch, dispatches while B's is stale under B's
/// bump; C's, whose epoch no response has carried, is refused rather than
/// judged by someone else's. As found the journal kept ONE epoch and the gate
/// compared every command against it, so A's allow would have been refused by
/// B's revocation — or B's stale allow admitted under A's epoch.
#[tokio::test]
async fn each_command_is_judged_by_its_own_tenants_epoch() {
    let (_fixture, node) = dummy_node("per-tenant-epoch").await;
    let (a, b, c) = (
        "ten_00000000-0000-7000-8000-00000000000a",
        "ten_00000000-0000-7000-8000-00000000000b",
        "ten_00000000-0000-7000-8000-00000000000c",
    );
    node.journal()
        .set_revocation_epochs(&std::collections::BTreeMap::from([
            (a.to_string(), 7),
            (b.to_string(), 9),
        ]))
        .await
        .expect("set epochs");
    let cmd_b = seed_tenant_command(node.journal(), "tenant-b", b, 8).await;
    let cmd_c = seed_tenant_command(node.journal(), "tenant-c", c, 7).await;
    let cmd_a = seed_tenant_command(node.journal(), "tenant-a", a, 7).await;
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
    let item = |command_id: String| {
        let journal = node.journal().clone();
        async move {
            journal
                .work_items()
                .await
                .expect("work items")
                .into_iter()
                .find(|w| w.command_id == command_id)
                .expect("seeded work item")
        }
    };

    // B: decided under 8, B is at 9 — stale by B's own revocation.
    worker
        .process(&item(cmd_b.clone()).await)
        .await
        .expect("a refusal is not an error");
    assert_eq!(
        latest_status(node.journal(), &cmd_b).await.as_deref(),
        Some("failed_before_dispatch"),
        "B's command is stale under B's epoch"
    );
    // C: no response has carried C's epoch — refused, never judged by A's or B's.
    worker
        .process(&item(cmd_c.clone()).await)
        .await
        .expect("a refusal is not an error");
    assert_eq!(
        latest_status(node.journal(), &cmd_c).await.as_deref(),
        Some("failed_before_dispatch"),
        "a tenant with no epoch reference is refused"
    );
    // A: decided under 7, A is at 7 — dispatches despite B's bump; its
    // completed attempt and waiting result prove the gate let it through.
    worker
        .process(&item(cmd_a.clone()).await)
        .await
        .expect("the dispatch completes; its result waits in the journal");
    assert_eq!(
        latest_status(node.journal(), &cmd_a).await.as_deref(),
        Some("completed"),
        "A's command is judged by A's epoch, not B's"
    );
    pending_result_for(node.journal(), &cmd_a).await;
}

/// `SIGNOFF-REPAIR.5.3.6` — the epoch map a response carries is the COMPLETE
/// set of tenants the node may act for: a tenant left out of the next map
/// loses its reference, and a command of that tenant the node still holds is
/// refused at the gate instead of dispatching under the epoch it last saw.
#[tokio::test]
async fn a_tenant_left_out_of_the_map_loses_its_reference() {
    let (_fixture, node) = dummy_node("tenant-departs").await;
    let (a, b) = (
        "ten_00000000-0000-7000-8000-00000000000a",
        "ten_00000000-0000-7000-8000-00000000000b",
    );
    node.journal()
        .set_revocation_epochs(&std::collections::BTreeMap::from([
            (a.to_string(), 7),
            (b.to_string(), 9),
        ]))
        .await
        .expect("both served");
    let cmd_a = seed_tenant_command(node.journal(), "departed-a", a, 7).await;
    // The next response no longer names A (its binding ended).
    node.journal()
        .set_revocation_epochs(&std::collections::BTreeMap::from([(b.to_string(), 9)]))
        .await
        .expect("only B served");
    assert_eq!(
        node.journal().revocation_epoch_for(a).await.expect("read"),
        None
    );
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
    let item = node
        .journal()
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == cmd_a)
        .expect("the held command");
    worker
        .process(&item)
        .await
        .expect("a refusal is not an error");
    assert_eq!(
        latest_status(node.journal(), &cmd_a).await.as_deref(),
        Some("failed_before_dispatch"),
        "a departed tenant's held command does not dispatch"
    );
}
