//! Worker retry-gate tests (`PHASE-2.2.3`; §14.6): the PURE decision rides the
//! worker before any gate — a reserved pre-dispatch refusal re-dispatches
//! bounded, a budget-denied item is terminal (no reservation = the server said
//! no, retrying cannot change it), an ambiguous outcome refuses without the
//! explicit possible-duplicate authorization and re-dispatches with it. The
//! refusing legs never contact a channel, so the dummy node is enough; the legs
//! that re-dispatch need a schedulable node (`SIGNOFF-REPAIR.4.4.4.2.2`), so they
//! reconcile against the stub control plane and read the delivered result there.

mod support;

use std::time::Duration;

use chrono::Utc;
use reasonbraid_adapter::{
    AdapterCapabilities, CancellationStrength, FakeAdapter, PolicyInjectionMode, ScriptStep,
    StatusLookupSpec,
};
use reasonbraid_core::fixture::Fixture;
use reasonbraid_core::BudgetDimensions;
use reasonbraid_node::{CommandInput, Journal, LocalBudget, Node, Worker};
use serde_json::{json, Value};
use support::control_plane::{reconciled_node, StubControlPlane};

/// A fixture directory for one test, created exclusively on the repository's own
/// volume and REMOVED when the test passes (`SIGNOFF-REPAIR.11.2.1.3.2.4`). A
/// failing test keeps its fixture, and everything in it, as the diagnostic.
fn journal_fixture(name: &str) -> Fixture {
    Fixture::create("retry-policy-tests", name).expect("the fixture directory is new")
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

/// A node reconciled against a stub control plane, so it is `Schedulable` and may
/// dispatch. The bindings drop right to left: the node, its fixture, the stub.
async fn schedulable_node(name: &str) -> (StubControlPlane, Fixture, Node) {
    let stub = StubControlPlane::start().await;
    let fixture = journal_fixture(name);
    let node = reconciled_node(&stub, &fixture.join("node.db")).await;
    (stub, fixture, node)
}

fn completing_adapter() -> FakeAdapter {
    FakeAdapter::new(
        vec![ScriptStep::Complete { usage: None }],
        StatusLookupSpec::Unsupported,
        AdapterCapabilities {
            streaming: false,
            cancellation: CancellationStrength::BestEffort,
            provider_idempotency: false,
            status_lookup: false,
            tool_support: false,
            policy_injection: PolicyInjectionMode::None,
        },
    )
}

fn local() -> LocalBudget {
    LocalBudget::new(BudgetDimensions {
        calls: Some(100),
        input_tokens: Some(100_000),
        output_tokens: Some(100_000),
        wall_clock_seconds: Some(10_000),
    })
}

/// A work payload with (or without) a reservation and the duplicate flag.
fn work_payload(reservation_present: bool, duplicate_authorized: bool) -> Value {
    json!({
        "kind": "contribute",
        "reservation": if reservation_present {
            json!({
                "reservation_id": "res_00000000-0000-7000-8000-000000000001",
                "dimensions": { "calls": 1, "wall_clock_seconds": 60 },
                "issued_at": Utc::now().to_rfc3339(),
                "expires_at": (Utc::now() + chrono::Duration::minutes(10)).to_rfc3339(),
            })
        } else {
            Value::Null
        },
        "allow_possible_duplicate": duplicate_authorized,
    })
}

/// Seed a command (with the admission decision the cache gate needs) + its
/// operation; return the command id + operation id.
async fn seed_command(journal: &Journal, tag: &str, payload: &Value) -> (String, String) {
    let command_id = format!("cmd_{tag}");
    let decided_at = Utc::now().to_rfc3339();
    journal
        .record_command(
            &CommandInput {
                command_id: &command_id,
                tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                thread_id: "thr_00000000-0000-7000-8000-000000000000",
                payload,
                authz_ref: Some("authz_00000000-0000-7000-8000-000000000001"),
                policy_digest: Some("digest-a"),
                decided_at: Some(&decided_at),
                revocation_epoch: Some(7),
                server_cursor: "1",
            },
            Utc::now(),
        )
        .await
        .expect("record command");
    journal
        .set_revocation_epoch_for("ten_00000000-0000-7000-8000-000000000000", 7)
        .await
        .expect("set epoch");
    let operation_id = journal
        .ensure_operation(&command_id, Utc::now())
        .await
        .expect("ensure operation")
        .operation_id;
    (command_id, operation_id)
}

async fn attempt_count(journal: &Journal, operation_id: &str) -> usize {
    journal
        .attempts_for_operation(operation_id)
        .await
        .expect("attempts")
        .len()
}

/// Exactly one work result was DELIVERED for `operation_id`, naming the attempt
/// that completed, and none is left pending (`SIGNOFF-REPAIR.4.4.4.1`,
/// `.4.4.4.2.2`). Before `.4.4.4.1` the undeliverable result was dropped with an
/// error, which these controls accepted as proof that the dispatch happened.
async fn assert_one_delivered_result(
    stub: &StubControlPlane,
    journal: &Journal,
    operation_id: &str,
) {
    let completed: Vec<String> = journal
        .attempts_for_operation(operation_id)
        .await
        .expect("attempts")
        .into_iter()
        .filter(|a| a.status == "completed")
        .map(|a| a.attempt_id)
        .collect();
    assert_eq!(completed.len(), 1, "exactly one completed attempt");
    let delivered: Vec<Value> = stub
        .events()
        .into_iter()
        .filter(|e| e.operation_id == operation_id)
        .map(|e| e.payload)
        .collect();
    assert_eq!(delivered.len(), 1, "one result delivered: {delivered:?}");
    assert_eq!(delivered[0]["kind"], "work_result");
    assert_eq!(delivered[0]["attempt_id"], completed[0]);
    assert!(
        journal
            .pending_events()
            .await
            .expect("pending events")
            .iter()
            .all(|e| e.operation_id != operation_id),
        "a delivered result is not left pending"
    );
}

#[tokio::test]
async fn a_reserved_pre_dispatch_refusal_redispatches() {
    let (stub, _fixture, node) = schedulable_node("retry-pre-dispatch").await;
    let payload = work_payload(true, false);
    let (_command_id, operation_id) = seed_command(node.journal(), "a", &payload).await;

    // Seed ONE failed_before_dispatch attempt (a transient refusal with a
    // VALID reservation): the retry gate must re-dispatch.
    let attempt_id = reasonbraid_core::ProviderAttemptId::new().to_string();
    node.journal()
        .prepare_attempt(&attempt_id, &operation_id, Utc::now())
        .await
        .expect("prepare");
    node.journal()
        .record_failed_before_dispatch(&attempt_id, Some("transient"), Utc::now())
        .await
        .expect("refuse");
    assert_eq!(attempt_count(node.journal(), &operation_id).await, 1);

    let worker = Worker::new(
        node.clone(),
        completing_adapter(),
        local(),
        Duration::from_secs(1),
    );
    let item = node
        .journal()
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == "cmd_a")
        .expect("seeded item");
    // The re-dispatch RUNS: the fresh attempt reaches the adapter, completes,
    // and its result is delivered.
    worker
        .process(&item)
        .await
        .expect("the re-dispatch completes and delivers its result");
    assert_one_delivered_result(&stub, node.journal(), &operation_id).await;
    assert_eq!(
        attempt_count(node.journal(), &operation_id).await,
        2,
        "the retry gate re-dispatched exactly one more attempt"
    );
}

#[tokio::test]
async fn a_budget_denied_item_is_never_redispatched() {
    let (_fixture, node) = dummy_node("retry-budget").await;
    let payload = work_payload(false, false); // no reservation — the server denied
    let (_command_id, operation_id) = seed_command(node.journal(), "b", &payload).await;

    let attempt_id = reasonbraid_core::ProviderAttemptId::new().to_string();
    node.journal()
        .prepare_attempt(&attempt_id, &operation_id, Utc::now())
        .await
        .expect("prepare");
    node.journal()
        .record_failed_before_dispatch(&attempt_id, Some("budget denied"), Utc::now())
        .await
        .expect("refuse");

    let worker = Worker::new(
        node.clone(),
        completing_adapter(),
        local(),
        Duration::from_secs(1),
    );
    let item = node
        .journal()
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == "cmd_b")
        .expect("seeded item");
    worker
        .process(&item)
        .await
        .expect("the refusal is a silent skip");

    assert_eq!(
        attempt_count(node.journal(), &operation_id).await,
        1,
        "a budget-denied item is terminal — no second attempt"
    );
}

#[tokio::test]
async fn an_ambiguous_outcome_refuses_without_the_authorization() {
    let (_fixture, node) = dummy_node("retry-ambiguous").await;
    let payload = work_payload(true, false);
    let (_command_id, operation_id) = seed_command(node.journal(), "c", &payload).await;

    let attempt_id = reasonbraid_core::ProviderAttemptId::new().to_string();
    node.journal()
        .prepare_attempt(&attempt_id, &operation_id, Utc::now())
        .await
        .expect("prepare");
    node.journal()
        .record_dispatch(&attempt_id, Some("req_1"), Utc::now())
        .await
        .expect("dispatch");
    node.journal()
        .record_outcome_unknown(&attempt_id, Some("req_1"), Utc::now())
        .await
        .expect("outcome unknown");

    let worker = Worker::new(
        node.clone(),
        completing_adapter(),
        local(),
        Duration::from_secs(1),
    );
    let item = node
        .journal()
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == "cmd_c")
        .expect("seeded item");
    worker
        .process(&item)
        .await
        .expect("the refusal is a silent skip");

    assert_eq!(
        attempt_count(node.journal(), &operation_id).await,
        1,
        "an ambiguous outcome without the authorization is never retried"
    );
}

#[tokio::test]
async fn an_authorized_ambiguous_outcome_redispatches() {
    let (stub, _fixture, node) = schedulable_node("retry-authorized").await;
    let payload = work_payload(true, true); // the explicit possible-duplicate authorization
    let (_command_id, operation_id) = seed_command(node.journal(), "d", &payload).await;

    let attempt_id = reasonbraid_core::ProviderAttemptId::new().to_string();
    node.journal()
        .prepare_attempt(&attempt_id, &operation_id, Utc::now())
        .await
        .expect("prepare");
    node.journal()
        .record_dispatch(&attempt_id, Some("req_1"), Utc::now())
        .await
        .expect("dispatch");
    node.journal()
        .record_outcome_unknown(&attempt_id, Some("req_1"), Utc::now())
        .await
        .expect("outcome unknown");

    let worker = Worker::new(
        node.clone(),
        completing_adapter(),
        local(),
        Duration::from_secs(1),
    );
    let item = node
        .journal()
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == "cmd_d")
        .expect("seeded item");
    worker
        .process(&item)
        .await
        .expect("the authorized retry completes and delivers its result");
    assert_one_delivered_result(&stub, node.journal(), &operation_id).await;
    assert_eq!(
        attempt_count(node.journal(), &operation_id).await,
        2,
        "the authorized ambiguous outcome re-dispatched exactly one more attempt"
    );
}

/// Re-deliver `command_id` the way a re-ask does: the same command, a FRESH
/// admission decision, and `payload` (`SIGNOFF-REPAIR.4.4.7.2.1`).
async fn redeliver(journal: &Journal, command_id: &str, payload: &Value) {
    let decided_at = (Utc::now() + chrono::Duration::seconds(1)).to_rfc3339();
    journal
        .record_command(
            &CommandInput {
                command_id,
                tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                thread_id: "thr_00000000-0000-7000-8000-000000000000",
                payload,
                authz_ref: Some("authz_00000000-0000-7000-8000-000000000002"),
                policy_digest: Some("digest-a"),
                decided_at: Some(&decided_at),
                revocation_epoch: Some(7),
                server_cursor: "2",
            },
            Utc::now(),
        )
        .await
        .expect("the re-delivery");
}

/// THE re-ask reaches the node (`SIGNOFF-REPAIR.4.4.7.2.1`). An `outcome_unknown`
/// attempt is re-asked: the same command comes back with a fresh decision, the
/// possible-duplicate flag and a NEW reservation (the original stays held for
/// the attempt that may have run). The node's journal takes those two
/// authorization fields, so the retry gate honours the flag, re-dispatches, and
/// charges the NEW reservation. Before this leaf the journal ignored a
/// re-delivered payload, and the gate still refused `retry_requires_authorization`.
#[tokio::test]
async fn a_reask_redelivery_authorizes_the_possible_duplicate_retry() {
    let (stub, _fixture, node) = schedulable_node("retry-reask").await;
    let original = work_payload(true, false);
    let (command_id, operation_id) = seed_command(node.journal(), "reask", &original).await;
    let attempt_id = reasonbraid_core::ProviderAttemptId::new().to_string();
    node.journal()
        .prepare_attempt(&attempt_id, &operation_id, Utc::now())
        .await
        .expect("prepare");
    node.journal()
        .record_dispatch(&attempt_id, None, Utc::now())
        .await
        .expect("dispatch");
    node.journal()
        .record_outcome_unknown(&attempt_id, Some("lost"), Utc::now())
        .await
        .expect("outcome unknown");

    let mut reasked = original.clone();
    reasked["allow_possible_duplicate"] = json!(true);
    reasked["reservation"]["reservation_id"] = json!("res_00000000-0000-7000-8000-00000000000b");
    redeliver(node.journal(), &command_id, &reasked).await;

    let worker = Worker::new(
        node.clone(),
        completing_adapter(),
        local(),
        Duration::from_secs(1),
    );
    let item = node
        .journal()
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == command_id)
        .expect("the item");
    worker
        .process(&item)
        .await
        .expect("the re-asked retry runs");
    assert_eq!(
        attempt_count(node.journal(), &operation_id).await,
        2,
        "the re-ask re-dispatched exactly one more attempt"
    );
    let delivered = stub
        .events()
        .into_iter()
        .find(|e| e.payload["kind"] == "work_result")
        .expect("the re-run's result");
    assert_eq!(
        delivered.payload["reservation_id"], "res_00000000-0000-7000-8000-00000000000b",
        "and ran under the NEW reservation"
    );
}

/// A re-delivery refreshes the work's AUTHORIZATION and never the work
/// (`SIGNOFF-REPAIR.4.4.7.2.1`): a changed `kind` or any other field is ignored,
/// so a redelivery can widen what a node may risk, never change what it was
/// asked to do.
#[tokio::test]
async fn a_redelivery_never_changes_the_work() {
    let (_stub, _fixture, node) = schedulable_node("retry-work-fixed").await;
    let original = work_payload(true, false);
    let (command_id, _operation_id) = seed_command(node.journal(), "fixed", &original).await;

    let mut altered = original.clone();
    altered["kind"] = json!("revise");
    altered["target_event_id"] = json!("evt_injected");
    altered["allow_possible_duplicate"] = json!(true);
    redeliver(node.journal(), &command_id, &altered).await;

    let item = node
        .journal()
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == command_id)
        .expect("the item");
    let held: Value = serde_json::from_str(&item.payload).expect("the payload");
    assert_eq!(
        held["kind"], original["kind"],
        "the work kind is as first delivered"
    );
    assert!(
        held.get("target_event_id").is_none(),
        "no field is added: {held}"
    );
    assert_eq!(
        held["allow_possible_duplicate"],
        json!(true),
        "the authorization is refreshed"
    );
}
