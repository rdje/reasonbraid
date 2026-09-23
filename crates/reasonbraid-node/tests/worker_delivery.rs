//! Delivery of a completed work result (`SIGNOFF-REPAIR.4.4.4.1`, made testable
//! in-crate by `SIGNOFF-REPAIR.4.4.4.2.1`): a schedulable node sends its result at
//! once; a node that is not schedulable keeps it pending, and the next reconcile
//! delivers it under its ORIGINAL id; and a node that is not schedulable never
//! dispatches at all (`SIGNOFF-REPAIR.4.4.4.2.2`). Both run against a stub control plane the
//! node really reconciles with; until it existed only the live server suites could
//! see a result leave the node.

mod support;

use std::time::Duration;

use chrono::Utc;
use reasonbraid_adapter::{AdapterCapabilities, FakeAdapter, ScriptStep, StatusLookupSpec};
use reasonbraid_core::fixture::Fixture;
use reasonbraid_core::BudgetDimensions;
use reasonbraid_node::{
    CommandInput, EventDelivery, Journal, LocalBudget, Node, NodeError, ResultEvent, Worker,
    WorkerError,
};
use serde_json::{json, Value};
use support::control_plane::StubControlPlane;

const TENANT: &str = "ten_00000000-0000-7000-8000-000000000000";

/// A node pointed at the stub. The guard comes back FIRST: a tuple binding drops
/// right to left, so the node closes its journal before the fixture removes it.
async fn node_at(stub: &StubControlPlane, name: &str) -> (Fixture, Node) {
    let key = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).expect("keypair");
    let fixture = Fixture::create("worker-delivery-tests", name).expect("a new fixture");
    let node = Node::open(
        fixture.join("node.db"),
        stub.base_url(),
        "nod_00000000-0000-7000-8000-000000000001".to_string(),
        vec![0x00, 0x01, 0x02],
        key,
    )
    .await
    .expect("open node");
    (fixture, node)
}

/// A worker whose adapter completes with the content `delivered`.
fn worker(node: &Node) -> Worker<FakeAdapter> {
    worker_with(node, completing_adapter())
}

/// A worker driving `adapter` with an ample local budget.
fn worker_with(node: &Node, adapter: FakeAdapter) -> Worker<FakeAdapter> {
    Worker::new(
        node.clone(),
        adapter,
        LocalBudget::new(BudgetDimensions {
            calls: Some(100),
            input_tokens: Some(100_000),
            output_tokens: Some(100_000),
            wall_clock_seconds: Some(10_000),
        }),
        Duration::from_secs(1),
    )
}

/// An adapter that completes with the content `delivered`.
fn completing_adapter() -> FakeAdapter {
    FakeAdapter::new(
        vec![
            ScriptStep::EmitChunk {
                chunk: "delivered".to_string(),
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

/// Journal one contribute command with a fresh admission decision under epoch 7,
/// and the tenant's epoch at 7, so the gate allows it.
async fn seed_allowed_work(journal: &Journal, tag: &str) -> String {
    journal
        .set_revocation_epoch_for(TENANT, 7)
        .await
        .expect("set epoch");
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
            &CommandInput {
                command_id: &command_id,
                tenant_id: TENANT,
                thread_id: "thr_00000000-0000-7000-8000-000000000000",
                payload: &payload,
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
    command_id
}

/// The one completed attempt of `command_id`'s operation, as `(operation, attempt)`.
async fn completed_attempt(journal: &Journal, command_id: &str) -> (String, String) {
    let operation_id = journal
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == command_id)
        .and_then(|w| w.operation_id)
        .expect("an operation");
    let completed: Vec<String> = journal
        .attempts_for_operation(&operation_id)
        .await
        .expect("attempts")
        .into_iter()
        .filter(|a| a.status == "completed")
        .map(|a| a.attempt_id)
        .collect();
    assert_eq!(completed.len(), 1, "exactly one completed attempt");
    (operation_id, completed[0].clone())
}

/// The harness itself: a node reconciles against the stub and becomes
/// schedulable — the state no other node-crate control can reach.
#[tokio::test]
async fn a_node_reconciles_against_the_stub_and_becomes_schedulable() {
    let stub = StubControlPlane::start().await;
    let (_fixture, node) = node_at(&stub, "reconciles").await;
    assert!(
        !node.is_schedulable().await,
        "a fresh node is not schedulable"
    );
    node.reconcile().await.expect("reconcile against the stub");
    assert!(
        node.is_schedulable().await,
        "the reconcile made it schedulable"
    );
    assert_eq!(stub.handshakes(), 1);
}

/// A schedulable node sends its result as soon as the attempt completes, and
/// nothing is left pending.
#[tokio::test]
async fn a_schedulable_nodes_result_is_delivered_at_once() {
    let stub = StubControlPlane::start().await;
    let (_fixture, node) = node_at(&stub, "at-once").await;
    node.reconcile().await.expect("reconcile");
    let command_id = seed_allowed_work(node.journal(), "at-once").await;
    let items = node.journal().work_items().await.expect("work items");
    worker(&node)
        .process(&items[0])
        .await
        .expect("the dispatch completes and delivers");

    let (operation_id, attempt_id) = completed_attempt(node.journal(), &command_id).await;
    let received = stub.events();
    assert_eq!(received.len(), 1, "one result reached the control plane");
    assert_eq!(received[0].operation_id, operation_id);
    assert_eq!(received[0].payload["kind"], "work_result");
    assert_eq!(received[0].payload["attempt_id"], attempt_id);
    assert_eq!(received[0].payload["content"], "delivered");
    assert!(
        node.journal()
            .pending_events()
            .await
            .expect("pending")
            .is_empty(),
        "a delivered result is acknowledged, not left pending"
    );
}

/// THE deferral, end to end. A completed attempt's result is journaled pending in
/// the same transaction as the completion (`SIGNOFF-REPAIR.4.4.4.1`); a node that
/// is not schedulable DEFERS it rather than erroring, sends nothing, and its next
/// reconcile delivers it exactly once under the id minted with the completion.
/// The same pending row is what a node that died just past the transaction
/// leaves behind, so this is also the crash path. (`process` can no longer reach
/// a deferral on its own: since `.4.4.4.2.2` an unschedulable node never
/// dispatches. The delivery API is driven directly.)
#[tokio::test]
async fn a_deferred_result_is_delivered_by_the_next_reconcile_under_its_id() {
    let stub = StubControlPlane::start().await;
    let (_fixture, node) = node_at(&stub, "deferred").await;
    let command_id = seed_allowed_work(node.journal(), "deferred").await;
    let operation_id = node
        .journal()
        .ensure_operation(&command_id, Utc::now())
        .await
        .expect("operation")
        .operation_id;
    node.journal()
        .prepare_attempt("patt_deferred", &operation_id, Utc::now())
        .await
        .expect("prepare");
    node.journal()
        .record_dispatch("patt_deferred", None, Utc::now())
        .await
        .expect("dispatch");
    let event = ResultEvent {
        event_id: "evt_deferred".to_string(),
        payload: json!({
            "kind": "work_result",
            "attempt_id": "patt_deferred",
            "content": "delivered",
        }),
    };
    node.journal()
        .record_completed_with_event("patt_deferred", None, &event, Utc::now())
        .await
        .expect("the completion and its result, together");

    let delivery = node
        .deliver_journaled_event(&operation_id, &event.event_id, &event.payload)
        .await
        .expect("a deferral is not an error");
    assert_eq!(delivery, EventDelivery::Deferred);
    assert!(
        stub.events().is_empty(),
        "nothing was sent while unschedulable"
    );
    assert_eq!(
        node.journal()
            .pending_events()
            .await
            .expect("pending")
            .len(),
        1,
        "the result waits in the journal"
    );

    node.reconcile().await.expect("reconcile");
    let received = stub.events();
    assert_eq!(received.len(), 1, "the reconcile delivered it exactly once");
    assert_eq!(
        received[0].event_id, "evt_deferred",
        "under its original id"
    );
    assert_eq!(received[0].operation_id, operation_id);
    assert_eq!(received[0].payload["content"], "delivered");
    assert!(
        node.journal()
            .pending_events()
            .await
            .expect("pending")
            .is_empty(),
        "delivered and acknowledged"
    );
}

/// The stub's one check: a delivery under a fencing token it never issued is
/// refused, so a control cannot reach it without a real handshake. (The issued
/// token's acceptance is what the two delivery controls above already show.)
#[tokio::test]
async fn the_stub_refuses_a_delivery_under_a_token_it_never_issued() {
    let stub = StubControlPlane::start().await;
    let client = reqwest::Client::new();
    let send = |token: &'static str| {
        client
            .post(format!("{}/v1/nodes/events", stub.base_url()))
            .json(&json!({
                "event_id": "evt_forged",
                "operation_id": "op_forged",
                "payload": {},
                "fencing_token": token,
            }))
            .send()
    };
    let forged = send("never-issued").await.expect("request");
    assert_eq!(
        forged.status().as_u16(),
        401,
        "an unissued token is refused"
    );
    assert!(stub.events().is_empty(), "and nothing is recorded");
}

/// THE refusal (`SIGNOFF-REPAIR.4.4.4.2.2`): a node that is not schedulable spends
/// NOTHING. Its local gates allow the work, and the dispatch is still refused
/// before the adapter is reached: no attempt is journaled, the item stays
/// untouched for after the reconcile, and the worker reports `NotSchedulable`
/// so the run loop reconciles. Before the repair this node invoked the provider,
/// paying for work it could not yet deliver.
#[tokio::test]
async fn an_unschedulable_node_spends_nothing() {
    let stub = StubControlPlane::start().await;
    let (_fixture, node) = node_at(&stub, "unschedulable").await;
    let command_id = seed_allowed_work(node.journal(), "unschedulable").await;
    let adapter = completing_adapter();
    let invocations = adapter.invocation_counter();
    let worker = worker_with(&node, adapter);
    let items = node.journal().work_items().await.expect("work items");

    let refused = worker.process(&items[0]).await;
    assert!(
        matches!(refused, Err(WorkerError::Node(NodeError::NotSchedulable))),
        "the worker reports the node unschedulable: {refused:?}"
    );
    assert_eq!(
        invocations.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "the provider was never reached"
    );
    let item = node
        .journal()
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == command_id)
        .expect("the item");
    assert_eq!(
        item.latest_attempt_status, None,
        "no attempt was journaled; the item waits intact"
    );

    // Once reconciled, the same item dispatches and its result is delivered.
    node.reconcile().await.expect("reconcile");
    node.journal()
        .set_revocation_epoch_for(TENANT, 7)
        .await
        .expect("the gate's epoch, after the reconcile replaced the map");
    worker
        .process(&items[0])
        .await
        .expect("a schedulable node dispatches");
    assert_eq!(invocations.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(stub.events().len(), 1, "and delivers");
}

/// A finished item is SETTLED (`SIGNOFF-REPAIR.4.4.8`): the ticks after its result
/// was delivered send nothing more. Before the repair the very next tick asked the
/// retry gate about the completed item, took its *the attempt is terminal*
/// refusal for a dead letter, and reported one, so every successful item was
/// quarantined one poll after it succeeded.
#[tokio::test]
async fn a_delivered_result_is_never_followed_by_a_dead_letter() {
    let stub = StubControlPlane::start_with_epochs(std::collections::BTreeMap::from([(
        TENANT.to_string(),
        7,
    )]))
    .await;
    let (_fixture, node) = node_at(&stub, "settled").await;
    node.reconcile().await.expect("reconcile");
    seed_allowed_work(node.journal(), "settled").await;
    let worker = worker(&node);

    for tick in 0..3 {
        worker
            .tick()
            .await
            .unwrap_or_else(|e| panic!("tick {tick}: {e}"));
    }
    let kinds: Vec<Value> = stub
        .events()
        .into_iter()
        .map(|e| e.payload["kind"].clone())
        .collect();
    assert_eq!(
        kinds,
        [json!("work_result")],
        "one result, and nothing after it"
    );
}
