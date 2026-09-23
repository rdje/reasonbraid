//! How the worker classifies a failure (`SIGNOFF-REPAIR.4.4.5.2`). A failure is
//! either the CHANNEL's (the node reconciles and resumes), a FACT about one
//! attempt that is already journaled (the tick goes on), or one ITEM's (the tick
//! goes on to the other items). Before this leaf a failed send and an unknown
//! outcome both stopped the node process, and one malformed item abandoned
//! every item after it.

mod support;

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::Utc;
use reasonbraid_adapter::{AdapterCapabilities, FakeAdapter, ScriptStep, StatusLookupSpec};
use reasonbraid_core::fixture::Fixture;
use reasonbraid_core::BudgetDimensions;
use reasonbraid_node::{CommandInput, Journal, LocalBudget, Node, Worker};
use serde_json::{json, Value};
use support::control_plane::{reconciled_node, StubControlPlane};

const TENANT: &str = "ten_00000000-0000-7000-8000-000000000000";

fn fixture(name: &str) -> Fixture {
    Fixture::create("worker-failure-tests", name).expect("a new fixture")
}

fn adapter(script: Vec<ScriptStep>) -> FakeAdapter {
    FakeAdapter::new(
        script,
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

fn completing() -> FakeAdapter {
    adapter(vec![
        ScriptStep::EmitChunk {
            chunk: "done".to_string(),
        },
        ScriptStep::Complete { usage: None },
    ])
}

fn worker(node: &Node, adapter: FakeAdapter) -> Worker<FakeAdapter> {
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

fn reservation() -> Value {
    json!({
        "reservation_id": "res_00000000-0000-7000-8000-000000000001",
        "dimensions": { "calls": 1, "wall_clock_seconds": 60 },
        "issued_at": Utc::now().to_rfc3339(),
        "expires_at": (Utc::now() + chrono::Duration::minutes(10)).to_rfc3339(),
    })
}

/// Journal one contribute command carrying `reservation`, freshly admitted under
/// epoch 7.
async fn seed(journal: &Journal, tag: &str, reservation: Value) -> String {
    let command_id = format!("cmd_{tag}");
    let payload = json!({ "kind": "contribute", "reservation": reservation });
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

async fn item(journal: &Journal, command_id: &str) -> reasonbraid_node::WorkItem {
    journal
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.command_id == command_id)
        .expect("the item")
}

/// (a) A send that fails is the CHANNEL's failure: the worker's error calls for
/// a reconcile (never a process exit), and the result stays pending for that
/// reconcile to re-emit.
#[tokio::test]
async fn a_failed_send_calls_for_a_reconcile_and_keeps_the_result() {
    let stub = StubControlPlane::start().await;
    let fixture = fixture("failed-send");
    let node = reconciled_node(&stub, &fixture.join("node.db")).await;
    node.journal()
        .set_revocation_epoch_for(TENANT, 7)
        .await
        .expect("epoch");
    let command_id = seed(node.journal(), "failed-send", reservation()).await;
    drop(stub); // the control plane goes away mid-work

    let failed = worker(&node, completing())
        .process(&item(node.journal(), &command_id).await)
        .await
        .expect_err("the send cannot succeed");
    assert!(
        failed.calls_for_reconcile(),
        "a failed send is a channel loss the node reconciles from: {failed}"
    );
    let pending = node.journal().pending_events().await.expect("pending");
    assert_eq!(pending.len(), 1, "the result waits for the reconcile");
}

/// (b) An unknown outcome is a FACT about one attempt, and it is already
/// journaled: the worker reports it and returns, the retry gate owns the item
/// from here, and nothing about the node needs recovering.
#[tokio::test]
async fn an_unknown_outcome_is_journaled_and_the_worker_goes_on() {
    let stub = StubControlPlane::start().await;
    let fixture = fixture("unknown-outcome");
    let node = reconciled_node(&stub, &fixture.join("node.db")).await;
    node.journal()
        .set_revocation_epoch_for(TENANT, 7)
        .await
        .expect("epoch");
    let command_id = seed(node.journal(), "unknown-outcome", reservation()).await;

    worker(&node, adapter(vec![ScriptStep::LoseResponse]))
        .process(&item(node.journal(), &command_id).await)
        .await
        .expect("an unknown outcome is journaled, not a failure of the worker");
    assert_eq!(
        item(node.journal(), &command_id)
            .await
            .latest_attempt_status
            .as_deref(),
        Some("outcome_unknown")
    );
    assert!(stub.events().is_empty(), "no result was invented");
}

/// (c) One item's malformed payload is that ITEM's failure: it is dead-lettered,
/// once, and the tick goes on, so the well-formed item after it still
/// dispatches and delivers.
#[tokio::test]
async fn a_malformed_item_does_not_abandon_the_items_after_it() {
    let stub = StubControlPlane::start_with_epochs(BTreeMap::from([(TENANT.to_string(), 7)])).await;
    let fixture = fixture("malformed-item");
    let node = reconciled_node(&stub, &fixture.join("node.db")).await;
    let broken = seed(
        node.journal(),
        "malformed",
        json!({ "reservation_id": 5 }), // not a reservation reference
    )
    .await;
    let sound = seed(node.journal(), "sound", reservation()).await;

    worker(&node, completing())
        .tick()
        .await
        .expect("an item's own failure does not fail the tick");
    assert_eq!(
        item(node.journal(), &broken).await.latest_attempt_status,
        None,
        "the malformed item was not dispatched"
    );
    assert_eq!(
        item(node.journal(), &sound)
            .await
            .latest_attempt_status
            .as_deref(),
        Some("completed"),
        "the item after it still ran"
    );
    let kinds = |stub: &StubControlPlane| -> Vec<String> {
        stub.events()
            .iter()
            .map(|e| e.payload["kind"].as_str().unwrap_or_default().to_string())
            .collect()
    };
    let mut received = kinds(&stub);
    received.sort();
    assert_eq!(
        received,
        ["work_dead_lettered", "work_result"],
        "the sound item delivered, and the malformed one was dead-lettered"
    );
    let dead_letter = stub
        .events()
        .into_iter()
        .find(|e| e.payload["kind"] == "work_dead_lettered")
        .expect("the dead letter");
    assert_eq!(dead_letter.payload["command_id"], broken);

    // A second tick sends nothing: the stuck item is not re-reported, and the
    // finished one is settled (`SIGNOFF-REPAIR.4.4.8`, which this control found).
    worker(&node, completing())
        .tick()
        .await
        .expect("the next tick");
    assert_eq!(
        stub.events().len(),
        2,
        "nothing more after the first tick: {:?}",
        kinds(&stub)
    );
}
