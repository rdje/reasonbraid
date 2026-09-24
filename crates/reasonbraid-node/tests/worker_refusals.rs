//! A result the control plane refuses PERMANENTLY (`SIGNOFF-REPAIR.4.4.10.2`).
//!
//! Such a refusal repeats whatever the node does, because it is about the bytes:
//! `400 unrepresentable_input` for input the store cannot hold, or the body
//! extractor refusing them. Before this leaf the node read it as a lost channel.
//! The worker's send failed into a reconcile, the reconcile re-emitted the same
//! bytes, met the same refusal, and failed, so the node never became schedulable
//! again (measured live by `DOC-0154`). The refusal is now journaled as what it
//! is, and the event stops being offered.

mod support;

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::Utc;
use reasonbraid_adapter::{AdapterCapabilities, FakeAdapter, ScriptStep, StatusLookupSpec};
use reasonbraid_core::fixture::Fixture;
use reasonbraid_core::BudgetDimensions;
use reasonbraid_node::{CommandInput, Journal, LocalBudget, Node, ResultEvent, Worker};
use serde_json::json;
use support::control_plane::{reconciled_node, StubControlPlane};

const TENANT: &str = "ten_00000000-0000-7000-8000-000000000000";
const MARKER: &str = "UNSTORABLE";

fn fixture(name: &str) -> Fixture {
    Fixture::create("worker-refusal-tests", name).expect("a new fixture")
}

fn refusing_stub() -> impl std::future::Future<Output = StubControlPlane> {
    StubControlPlane::start_refusing(BTreeMap::from([(TENANT.to_string(), 7)]), MARKER)
}

async fn seed(journal: &Journal, tag: &str) -> String {
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

fn worker(node: &Node, content: &str) -> Worker<FakeAdapter> {
    Worker::new(
        node.clone(),
        FakeAdapter::new(
            vec![
                ScriptStep::EmitChunk {
                    chunk: content.to_string(),
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
        ),
        LocalBudget::new(BudgetDimensions {
            calls: Some(100),
            input_tokens: Some(100_000),
            output_tokens: Some(100_000),
            wall_clock_seconds: Some(10_000),
        }),
        Duration::from_secs(1),
    )
}

/// The worker's send: a permanently refused result is journaled refused and no
/// longer pending, the tick succeeds, and the NEXT reconcile succeeds, so the
/// node stays schedulable and goes on with its other work.
#[tokio::test]
async fn a_permanently_refused_result_is_journaled_and_does_not_wedge_the_node() {
    let stub = refusing_stub().await;
    let fixture = fixture("send");
    let node = reconciled_node(&stub, &fixture.join("node.db")).await;
    seed(node.journal(), "refused").await;

    worker(&node, &format!("an answer the store cannot hold: {MARKER}"))
        .tick()
        .await
        .expect("a permanent refusal is recorded, not raised");
    assert!(
        node.journal()
            .pending_events()
            .await
            .expect("pending")
            .is_empty(),
        "the refused result is no longer offered"
    );
    let refusals = node.journal().event_refusals().await.expect("refusals");
    assert_eq!(refusals.len(), 1, "the refusal is journaled: {refusals:?}");
    assert!(
        format!("{refusals:?}").contains("unrepresentable_input"),
        "under the control plane's own code: {refusals:?}"
    );

    node.reconcile()
        .await
        .expect("the reconcile is not wedged by the refused result");
    assert!(node.is_schedulable().await);
    assert!(
        stub.events().is_empty(),
        "the refused result was never recorded"
    );
}

/// The reconcile's re-emission: a node that finished an attempt while it could
/// not deliver (a crash, or an unschedulable node) re-emits the result at its
/// next reconcile; a permanent refusal THERE is journaled too, and the reconcile
/// completes instead of failing for ever.
#[tokio::test]
async fn a_reconcile_that_meets_a_permanent_refusal_completes() {
    let stub = refusing_stub().await;
    let key = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).expect("keypair");
    let fixture = fixture("reconcile");
    let node = Node::open(
        fixture.join("node.db"),
        stub.base_url(),
        "nod_00000000-0000-7000-8000-000000000001".to_string(),
        vec![0x00, 0x01, 0x02],
        key,
    )
    .await
    .expect("open node");
    let command_id = seed(node.journal(), "pending").await;
    let operation_id = node
        .journal()
        .ensure_operation(&command_id, Utc::now())
        .await
        .expect("operation")
        .operation_id;
    node.journal()
        .prepare_attempt("patt_refused", &operation_id, Utc::now())
        .await
        .expect("prepare");
    node.journal()
        .record_dispatch("patt_refused", None, Utc::now())
        .await
        .expect("dispatch");
    node.journal()
        .record_completed_with_event(
            "patt_refused",
            None,
            &ResultEvent {
                event_id: "evt_refused".to_string(),
                payload: json!({
                    "kind": "work_result",
                    "command_id": command_id,
                    "attempt_id": "patt_refused",
                    "content": format!("pending, and unstorable: {MARKER}"),
                }),
            },
            Utc::now(),
        )
        .await
        .expect("a result pending for the reconcile");

    node.reconcile()
        .await
        .expect("the reconcile completes past the refused result");
    assert!(node.is_schedulable().await);
    assert!(node
        .journal()
        .pending_events()
        .await
        .expect("pending")
        .is_empty());
    assert_eq!(
        node.journal()
            .event_refusals()
            .await
            .expect("refusals")
            .len(),
        1
    );
}
