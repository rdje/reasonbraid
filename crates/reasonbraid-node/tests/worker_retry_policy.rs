//! Worker retry-gate tests (`PHASE-2.2.3`; §14.6): the PURE decision rides the
//! worker before any gate — a reserved pre-dispatch refusal re-dispatches
//! bounded, a budget-denied item is terminal (no reservation = the server said
//! no, retrying cannot change it), an ambiguous outcome refuses without the
//! explicit possible-duplicate authorization and re-dispatches with it. All
//! legs never contact a channel: the dummy node is enough.

use std::path::PathBuf;
use std::time::Duration;

use chrono::Utc;
use reasonbraid_adapter::{
    AdapterCapabilities, CancellationStrength, FakeAdapter, PolicyInjectionMode, ScriptStep,
    StatusLookupSpec,
};
use reasonbraid_core::BudgetDimensions;
use reasonbraid_node::{CommandInput, Journal, LocalBudget, Node, Worker, WorkerError};
use serde_json::{json, Value};

fn journal_path(name: &str) -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let unique = uuid::Uuid::now_v7();
    let dir = base
        .join("retry-policy-tests")
        .join(format!("{name}-{unique}"));
    std::fs::create_dir_all(dir.parent().expect("the fixture parent")).unwrap();
    // Exclusive: an existing directory belongs to another fixture or an
    // earlier run, and must never be adopted.
    std::fs::DirBuilder::new()
        .create(&dir)
        .expect("the fixture directory is new");
    dir.join("node.db")
}

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
    journal.set_revocation_epoch(7).await.expect("set epoch");
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

#[tokio::test]
async fn a_reserved_pre_dispatch_refusal_redispatches() {
    let node = dummy_node("retry-pre-dispatch").await;
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
    // The re-dispatch RUNS (the fresh attempt reaches the adapter and
    // completes; the result emission fails on the dummy URL — that failure
    // proves the dispatch happened, a refused one returns Ok silently).
    match worker.process(&item).await {
        Err(WorkerError::Channel(_)) | Err(WorkerError::Node(_)) => {}
        other => panic!(
            "a reserved refusal re-dispatches; the emit fails on the dummy URL — got {other:?}"
        ),
    }
    assert_eq!(
        attempt_count(node.journal(), &operation_id).await,
        2,
        "the retry gate re-dispatched exactly one more attempt"
    );
}

#[tokio::test]
async fn a_budget_denied_item_is_never_redispatched() {
    let node = dummy_node("retry-budget").await;
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
    let node = dummy_node("retry-ambiguous").await;
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
    let node = dummy_node("retry-authorized").await;
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
    match worker.process(&item).await {
        Err(WorkerError::Channel(_)) | Err(WorkerError::Node(_)) => {}
        other => panic!(
            "the authorized retry re-dispatches; the emit fails on the dummy URL — got {other:?}"
        ),
    }
    assert_eq!(
        attempt_count(node.journal(), &operation_id).await,
        2,
        "the authorized ambiguous outcome re-dispatched exactly one more attempt"
    );
}
