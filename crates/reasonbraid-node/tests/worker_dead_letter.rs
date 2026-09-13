//! Worker dead-letter tests (`PHASE-2.2.4`): a terminal refusal reports the
//! dead letter ONCE (the outgoing-events dedup), and a replayed delivery's
//! FRESH admission decision resets the retry count — the re-dispatch runs
//! under the new decision, not the dead one.

use std::path::PathBuf;
use std::time::Duration;

use chrono::Utc;
use reasonbraid_adapter::{
    AdapterCapabilities, CancellationStrength, FakeAdapter, PolicyInjectionMode, ScriptStep,
    StatusLookupSpec,
};
use reasonbraid_core::BudgetDimensions;
use reasonbraid_node::{CommandInput, Journal, LocalBudget, Node, Worker};
use serde_json::{json, Value};

fn journal_path(name: &str) -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let unique = uuid::Uuid::now_v7();
    let dir = base
        .join("dead-letter-tests")
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

fn refusing_adapter() -> FakeAdapter {
    FakeAdapter::new(
        vec![ScriptStep::FailBeforeDispatch {
            reason: "provider unavailable".to_string(),
        }],
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

fn reserved_payload() -> Value {
    json!({
        "kind": "contribute",
        "reservation": {
            "reservation_id": "res_00000000-0000-7000-8000-000000000001",
            "dimensions": { "calls": 1, "wall_clock_seconds": 60 },
            "issued_at": Utc::now().to_rfc3339(),
        },
        "allow_possible_duplicate": false,
    })
}

async fn seed_command(
    journal: &Journal,
    tag: &str,
    decided_at: chrono::DateTime<Utc>,
    epoch: i64,
) -> (String, String) {
    let command_id = format!("cmd_{tag}");
    let payload = reserved_payload();
    let decided_at_s = decided_at.to_rfc3339();
    journal
        .record_command(
            &CommandInput {
                command_id: &command_id,
                tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                thread_id: "thr_00000000-0000-7000-8000-000000000000",
                payload: &payload,
                authz_ref: Some("authz_00000000-0000-7000-8000-000000000001"),
                policy_digest: Some("digest-a"),
                decided_at: Some(&decided_at_s),
                revocation_epoch: Some(epoch),
                server_cursor: "1",
            },
            Utc::now(),
        )
        .await
        .expect("record command");
    journal.set_revocation_epoch(epoch).await.expect("epoch");
    let operation_id = journal
        .ensure_operation(&command_id, Utc::now())
        .await
        .expect("operation")
        .operation_id;
    (command_id, operation_id)
}

/// THE `.2.4` dedup leg: a terminal refusal reports the dead letter exactly
/// ONCE — repeated ticks keep refusing (the retry gate) without a second
/// report.
#[tokio::test]
async fn a_terminal_refusal_reports_the_dead_letter_once() {
    let node = dummy_node("dead-letter-once").await;
    let (_command_id, operation_id) = seed_command(node.journal(), "x", Utc::now(), 7).await;

    let worker = Worker::new(
        node.clone(),
        refusing_adapter(),
        local(),
        Duration::from_secs(1),
    );
    let item = || async {
        node.journal()
            .work_items()
            .await
            .expect("work items")
            .into_iter()
            .find(|w| w.command_id == "cmd_x")
            .expect("seeded item")
    };

    // Tick 1: fresh → the adapter refuses BEFORE dispatch (attempt 1).
    worker.process(&item().await).await.expect("first refusal");
    // Tick 2: reserved pre-dispatch refusal → bounded retry (attempt 2).
    worker.process(&item().await).await.expect("second refusal");
    // Tick 3: attempt 3 — the bound is reached.
    worker.process(&item().await).await.expect("third refusal");
    // Tick 4: the retry gate refuses (budget exhausted) → the dead-letter
    // report fires.
    worker
        .process(&item().await)
        .await
        .expect("the terminal refusal reports");
    // Tick 5: still refused, but NO second report (the dedup).
    worker
        .process(&item().await)
        .await
        .expect("still refused, still silent");

    let dead_letters = node
        .journal()
        .emitted_events()
        .await
        .expect("outgoing events")
        .into_iter()
        .filter(|e| e.operation_id == operation_id)
        .filter(|e| e.payload.contains("work_dead_lettered"))
        .count();
    assert_eq!(dead_letters, 1, "the dead letter is reported exactly once");
    assert!(
        node.journal()
            .has_dead_letter(&operation_id)
            .await
            .expect("dedup"),
        "the dedup flag is set"
    );
}

/// THE `.2.4` replay leg: re-delivering the command with a FRESH admission
/// decision resets the retry count (the old refusals precede the new
/// decision) — the re-dispatch runs.
#[tokio::test]
async fn a_replayed_delivery_refreshes_the_decision_and_redispatches() {
    let node = dummy_node("dead-letter-replay").await;
    let decided_at = Utc::now() - chrono::Duration::seconds(600);
    let (command_id, operation_id) = seed_command(node.journal(), "y", decided_at, 7).await;

    let worker = Worker::new(
        node.clone(),
        refusing_adapter(),
        local(),
        Duration::from_secs(1),
    );
    let item = || async {
        node.journal()
            .work_items()
            .await
            .expect("work items")
            .into_iter()
            .find(|w| w.command_id == command_id)
            .expect("seeded item")
    };
    // Three refusals under the OLD decision (the adapter refuses).
    for _ in 0..3 {
        worker.process(&item().await).await.expect("refusal");
    }
    assert_eq!(
        node.journal()
            .attempts_for_operation(&operation_id)
            .await
            .expect("attempts")
            .len(),
        3
    );

    // THE replay: the server re-delivers the command with a FRESH decision
    // (decided_at now, the current epoch). The journal refreshes the cached
    // decision (record_command's upsert) — and the old refusals no longer
    // count toward the retry bound.
    let fresh = Utc::now();
    let fresh_s = fresh.to_rfc3339();
    let payload = reserved_payload();
    node.journal()
        .record_command(
            &CommandInput {
                command_id: &command_id,
                tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                thread_id: "thr_00000000-0000-7000-8000-000000000000",
                payload: &payload,
                authz_ref: Some("authz_00000000-0000-7000-8000-000000000001"),
                policy_digest: Some("digest-a"),
                decided_at: Some(&fresh_s),
                revocation_epoch: Some(7),
                server_cursor: "2",
            },
            Utc::now(),
        )
        .await
        .expect("the replayed delivery");

    // The re-dispatch runs under the FRESH decision: the retry gate counts 0
    // refusals after it, the adapter refuses again (attempt 4).
    worker
        .process(&item().await)
        .await
        .expect("the replayed dispatch");
    assert_eq!(
        node.journal()
            .attempts_for_operation(&operation_id)
            .await
            .expect("attempts")
            .len(),
        4,
        "the fresh decision re-armed the dispatch (one new attempt)"
    );
}

/// `SIGNOFF-REPAIR.3.4.3.1.2` — the retry gate is a THIRD cross-clock
/// comparison, and `.3.4.3.1`'s census did not find it.
///
/// The gate counts attempts made under the current decision by filtering
/// `updated_at >= decided_at`. `updated_at` is a LOCAL journal timestamp this
/// node wrote; `decided_at` is the SERVER's database clock. With this node's
/// clock BEHIND the server's, every genuine post-decision attempt reads as
/// older than the decision, is filtered out, and the count stays at zero — so
/// the retry bound never trips and the item re-dispatches without limit. That
/// is the fail-OPEN direction, which is why it is worth a control of its own.
#[tokio::test]
async fn the_retry_bound_still_trips_when_this_clock_runs_behind_the_server() {
    let node = dummy_node("retry-clock-behind").await;

    // This node is 600 s behind: the server's clock reads 600 s AFTER ours.
    let node_now = Utc::now();
    let server_time = node_now + chrono::Duration::seconds(600);
    node.journal()
        .record_server_time(server_time, node_now, node_now)
        .await
        .expect("record the server's clock");

    // Decided now, in the SERVER's terms — so every local attempt timestamp
    // below is numerically smaller than `decided_at`.
    let (_command_id, operation_id) = seed_command(node.journal(), "skew", server_time, 7).await;

    let worker = Worker::new(
        node.clone(),
        refusing_adapter(),
        local(),
        Duration::from_secs(1),
    );
    let item = || async {
        node.journal()
            .work_items()
            .await
            .expect("work items")
            .into_iter()
            .find(|w| w.command_id == "cmd_skew")
            .expect("seeded item")
    };
    // Three attempts, then the bound must be reached and the dead letter fire.
    for _ in 0..5 {
        worker.process(&item().await).await.expect("a tick");
    }

    let dead_letters = node
        .journal()
        .emitted_events()
        .await
        .expect("outgoing events")
        .into_iter()
        .filter(|e| e.operation_id == operation_id)
        .filter(|e| e.payload.contains("work_dead_lettered"))
        .count();
    assert_eq!(
        dead_letters, 1,
        "the retry bound must trip on a node whose clock runs behind the \
         server's — an uncorrected comparison filters every attempt out and \
         re-dispatches without limit"
    );
}
