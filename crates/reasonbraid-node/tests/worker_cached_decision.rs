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
async fn seed_command(
    journal: &Journal,
    tag: &str,
    decided_at: Option<chrono::DateTime<Utc>>,
    revocation_epoch: Option<i64>,
) -> String {
    let command_id = format!("cmd_{tag}");
    let payload = json!({
        "kind": "contribute",
        "reservation": {
            "reservation_id": "res_00000000-0000-7000-8000-000000000001",
            "dimensions": { "calls": 1, "wall_clock_seconds": 60 },
            "issued_at": Utc::now().to_rfc3339(),
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
            Utc::now(),
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
