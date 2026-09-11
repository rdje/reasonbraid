//! WP3 kill-point integration tests for the node journal (`PHASE-0.3.1`).
//!
//! `KICKOFF.md` WP3: "Inject termination before and after every journal transition."
//! These tests walk the `.3.1` journal seams and simulate a crash at each one by
//! dropping the journal handle without any checkpoint — what survives in the file is
//! exactly what a killed process would leave (each transition is its own committed
//! transaction; no external service is involved, so these run in plain `cargo test`).
//!
//! The acceptance: a crash **after a possible dispatch** recovers as `outcome_unknown`
//! unless the adapter can prove the result; a crash **before** the boundary is
//! `safe_to_redeliver`; and operators can see all of it without opening SQLite by hand
//! (that last leg is `tests/journal_cli.rs`).

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use reasonbraid_node::{CommandInput, Journal, ProvenStatus};
use serde_json::{json, Value};

/// A unique journal path under the repo's build dir (same volume as the repo, per the
/// data-locality policy). `CARGO_TARGET_TMPDIR` is provided to integration tests.
fn journal_path(name: &str) -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let unique = uuid::Uuid::now_v7();
    let dir = base.join("journal-tests").join(format!("{name}-{unique}"));
    std::fs::create_dir_all(dir.parent().expect("the fixture parent")).unwrap();
    // Exclusive: an existing directory belongs to another fixture or an
    // earlier run, and must never be adopted.
    std::fs::DirBuilder::new()
        .create(&dir)
        .expect("the fixture directory is new");
    dir.join("node.db")
}

fn now() -> DateTime<Utc> {
    Utc::now()
}

fn command<'a>(command_id: &'a str, cursor: &'a str, payload: &'a Value) -> CommandInput<'a> {
    CommandInput {
        command_id,
        tenant_id: "ten_00000000-0000-7000-8000-000000000000",
        thread_id: "thr_00000000-0000-7000-8000-000000000000",
        payload,
        authz_ref: None,
        policy_digest: None,
        decided_at: None,
        revocation_epoch: None,
        server_cursor: cursor,
    }
}

/// Seed command + operation and return `(operation_id, journal)`.
async fn seed_command(journal: &Journal, tag: &str) -> String {
    let cmd_id = format!("cmd_{tag}");
    let payload = json!({ "operation": "contribute" });
    journal
        .record_command(&command(&cmd_id, "c1", &payload), now())
        .await
        .expect("record command");
    let op = journal
        .ensure_operation(&cmd_id, now())
        .await
        .expect("ensure operation");
    assert!(op.created);
    op.operation_id
}

/// KP-1 — crash BEFORE the command is recorded: nothing was persisted; the node can
/// simply receive the command again.
#[tokio::test]
async fn kp1_crash_before_command_record_persists_nothing() {
    let path = journal_path("kp1");
    {
        let _journal = Journal::open(&path).await.unwrap();
    } // crash before any write

    let journal = Journal::open(&path).await.unwrap();
    assert_eq!(journal.counts().await.unwrap().commands, 0);
    assert!(journal
        .recover(now())
        .await
        .unwrap()
        .now_ambiguous
        .is_empty());
    // Redelivery after the crash records cleanly.
    let op = seed_command(&journal, "kp1-redeliver").await;
    assert!(!op.is_empty());
}

/// KP-2 — crash AFTER the command record, BEFORE the operation exists: the command is
/// durable; the node derives the operation (exactly once) after restart.
#[tokio::test]
async fn kp2_crash_between_command_and_operation() {
    let path = journal_path("kp2");
    let (cmd_id, _) = {
        let journal = Journal::open(&path).await.unwrap();
        let cmd_id = "cmd_kp2".to_string();
        let payload = json!({ "operation": "contribute" });
        journal
            .record_command(&command(&cmd_id, "c1", &payload), now())
            .await
            .unwrap();
        (cmd_id, journal)
    }; // crash before ensure_operation

    let journal = Journal::open(&path).await.unwrap();
    assert_eq!(journal.counts().await.unwrap().commands, 1);
    let op = journal.ensure_operation(&cmd_id, now()).await.unwrap();
    assert!(op.created);
    let replay = journal.ensure_operation(&cmd_id, now()).await.unwrap();
    assert!(
        !replay.created,
        "the restarted node must not double the operation"
    );
    assert_eq!(op.operation_id, replay.operation_id);
}

/// KP-3 — crash AFTER prepare, BEFORE the dispatch boundary record: recovery reports
/// `safe_to_redeliver`; the attempt stays `prepared` (the boundary was never crossed,
/// so redelivery cannot duplicate a provider effect).
#[tokio::test]
async fn kp3_crash_after_prepare_before_dispatch() {
    let path = journal_path("kp3");
    let (attempt_id, _op) = {
        let journal = Journal::open(&path).await.unwrap();
        let op = seed_command(&journal, "kp3").await;
        let attempt_id = "patt_kp3".to_string();
        journal
            .prepare_attempt(&attempt_id, &op, now())
            .await
            .unwrap();
        (attempt_id, journal)
    }; // crash before record_dispatch

    let journal = Journal::open(&path).await.unwrap();
    let report = journal.recover(now()).await.unwrap();
    assert!(report.now_ambiguous.is_empty());
    assert_eq!(report.safe_to_redeliver.len(), 1);
    assert_eq!(report.safe_to_redeliver[0].attempt_id, attempt_id);
    assert_eq!(report.safe_to_redeliver[0].status, "prepared");
    // Redelivery resumes from the durable attempt: dispatch can still be recorded.
    journal
        .record_dispatch(&attempt_id, Some("prv_kp3"), now())
        .await
        .unwrap();
    assert_eq!(
        journal.pending_attempts().await.unwrap()[0].status,
        "dispatched"
    );
}

/// KP-4 — crash AFTER the dispatch boundary record, BEFORE the adapter runs. The
/// journal cannot know whether the provider was ever contacted, so recovery reports
/// `outcome_unknown` — the honest answer is ambiguity, not a silent retry.
#[tokio::test]
async fn kp4_crash_after_dispatch_before_adapter_call() {
    let path = journal_path("kp4");
    let attempt_id = {
        let journal = Journal::open(&path).await.unwrap();
        let op = seed_command(&journal, "kp4").await;
        let attempt_id = "patt_kp4".to_string();
        journal
            .prepare_attempt(&attempt_id, &op, now())
            .await
            .unwrap();
        journal
            .record_dispatch(&attempt_id, Some("prv_kp4"), now())
            .await
            .unwrap();
        attempt_id
    }; // crash: the boundary record committed, the adapter never ran

    let journal = Journal::open(&path).await.unwrap();
    let report = journal.recover(now()).await.unwrap();
    assert_eq!(report.now_ambiguous.len(), 1);
    assert_eq!(report.now_ambiguous[0].attempt_id, attempt_id);
    assert_eq!(report.now_ambiguous[0].status, "outcome_unknown");
}

/// KP-5 — crash AFTER the adapter finished, BEFORE the result was recorded — followed
/// by the acceptance's escape hatch: the adapter CAN prove the result via status
/// lookup, so the attempt lands on `completed` instead of staying ambiguous.
#[tokio::test]
async fn kp5_crash_before_result_record_then_adapter_proves_it() {
    let path = journal_path("kp5");
    let attempt_id = {
        let journal = Journal::open(&path).await.unwrap();
        let op = seed_command(&journal, "kp5").await;
        let attempt_id = "patt_kp5".to_string();
        journal
            .prepare_attempt(&attempt_id, &op, now())
            .await
            .unwrap();
        journal
            .record_dispatch(&attempt_id, Some("prv_kp5"), now())
            .await
            .unwrap();
        // The fake adapter "did the work" and knows it — but the result was never recorded.
        attempt_id
    }; // crash before record_completed

    let journal = Journal::open(&path).await.unwrap();
    let report = journal.recover(now()).await.unwrap();
    assert_eq!(
        report.now_ambiguous.len(),
        1,
        "no result recorded → ambiguous"
    );
    assert_eq!(
        report.now_ambiguous[0].provider_request_id.as_deref(),
        Some("prv_kp5")
    );

    // The adapter proves the result by provider status lookup (§11.3).
    journal
        .prove_result(
            &attempt_id,
            ProvenStatus::Completed,
            None,
            Some(&json!({ "provider_status_lookup": "completed" })),
            now(),
        )
        .await
        .unwrap();
    assert!(journal.ambiguous_attempts().await.unwrap().is_empty());
    assert_eq!(
        journal
            .attempt_history(&attempt_id)
            .await
            .unwrap()
            .last()
            .unwrap()
            .to_status,
        "completed"
    );
}

/// KP-6 — crash AFTER the result was recorded: the terminal state is untouched, and
/// no later recovery can re-open it.
#[tokio::test]
async fn kp6_crash_after_result_record_is_terminal() {
    let path = journal_path("kp6");
    let attempt_id = {
        let journal = Journal::open(&path).await.unwrap();
        let op = seed_command(&journal, "kp6").await;
        let attempt_id = "patt_kp6".to_string();
        journal
            .prepare_attempt(&attempt_id, &op, now())
            .await
            .unwrap();
        journal
            .record_dispatch(&attempt_id, None, now())
            .await
            .unwrap();
        journal
            .record_completed(&attempt_id, Some(&json!({ "ok": true })), now())
            .await
            .unwrap();
        attempt_id
    }; // crash after the completed record

    let journal = Journal::open(&path).await.unwrap();
    let report = journal.recover(now()).await.unwrap();
    assert!(report.now_ambiguous.is_empty());
    assert!(report.safe_to_redeliver.is_empty());
    let counts = journal.counts().await.unwrap();
    assert_eq!(counts.attempts_completed, 1);
    assert_eq!(
        journal
            .attempt_history(&attempt_id)
            .await
            .unwrap()
            .last()
            .unwrap()
            .to_status,
        "completed"
    );
}

/// KP-7 — crash AFTER the ambiguity was already recorded: recovery is a no-op (the
/// attempt stays ambiguous exactly once, with one boundary ledger row per move).
#[tokio::test]
async fn kp7_crash_after_outcome_unknown_is_stable() {
    let path = journal_path("kp7");
    let attempt_id = {
        let journal = Journal::open(&path).await.unwrap();
        let op = seed_command(&journal, "kp7").await;
        let attempt_id = "patt_kp7".to_string();
        journal
            .prepare_attempt(&attempt_id, &op, now())
            .await
            .unwrap();
        journal
            .record_dispatch(&attempt_id, None, now())
            .await
            .unwrap();
        journal
            .record_outcome_unknown(&attempt_id, Some("adapter timeout"), now())
            .await
            .unwrap();
        attempt_id
    }; // crash after the ambiguity record

    let journal = Journal::open(&path).await.unwrap();
    let report = journal.recover(now()).await.unwrap();
    assert_eq!(report.now_ambiguous.len(), 1);
    assert_eq!(report.now_ambiguous[0].attempt_id, attempt_id);
    assert_eq!(journal.attempt_history(&attempt_id).await.unwrap().len(), 2);
}

/// KP-8 — crash AFTER the event was emitted, BEFORE the acknowledgement: the event
/// shows in the pending view and can be acknowledged by the restarted node (the
/// server cursor travels with the ack — reconnect evidence for `.3.2`).
#[tokio::test]
async fn kp8_crash_between_event_emission_and_ack() {
    let path = journal_path("kp8");
    let (event_id, _op) = {
        let journal = Journal::open(&path).await.unwrap();
        let op = seed_command(&journal, "kp8").await;
        let event_id = "evt_kp8".to_string();
        journal
            .record_outgoing_event(&event_id, &op, &json!({ "event_type": "ready" }), now())
            .await
            .unwrap();
        (event_id, journal)
    }; // crash before the server acknowledgement

    let journal = Journal::open(&path).await.unwrap();
    let pending = journal.pending_events().await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].event_id, event_id);

    // The restarted node re-sends the pending event and records the ack + cursor.
    assert!(journal
        .acknowledge_event(&event_id, "c42", now())
        .await
        .unwrap());
    assert!(journal.pending_events().await.unwrap().is_empty());
    let counts = journal.counts().await.unwrap();
    assert_eq!(counts.events_emitted, 1);
    assert_eq!(counts.events_acked, 1);
}

/// KP-9 — crash AFTER the acknowledgement: the ack survives; the event never returns
/// to the pending view.
#[tokio::test]
async fn kp9_crash_after_ack_is_terminal() {
    let path = journal_path("kp9");
    let event_id = {
        let journal = Journal::open(&path).await.unwrap();
        let op = seed_command(&journal, "kp9").await;
        let event_id = "evt_kp9".to_string();
        journal
            .record_outgoing_event(&event_id, &op, &json!({ "event_type": "ready" }), now())
            .await
            .unwrap();
        assert!(journal
            .acknowledge_event(&event_id, "c9", now())
            .await
            .unwrap());
        event_id
    }; // crash after the ack

    let journal = Journal::open(&path).await.unwrap();
    assert!(journal.pending_events().await.unwrap().is_empty());
    assert!(
        !journal
            .acknowledge_event(&event_id, "c9", now())
            .await
            .unwrap(),
        "redelivered ack is idempotent, not a new acknowledgement"
    );
}

/// The whole WP3 acceptance in one end-to-end replay: crash mid-attempt → ambiguous →
/// operator sees it via the journal API → adapter proves → terminal. At no point is
/// the attempt silently retried.
#[tokio::test]
async fn end_to_end_crash_recovery_never_silently_retries() {
    let path = journal_path("e2e");
    let attempt_id = {
        let journal = Journal::open(&path).await.unwrap();
        let op = seed_command(&journal, "e2e").await;
        let attempt_id = "patt_e2e".to_string();
        journal
            .prepare_attempt(&attempt_id, &op, now())
            .await
            .unwrap();
        journal
            .record_dispatch(&attempt_id, Some("prv_e2e"), now())
            .await
            .unwrap();
        attempt_id
    }; // crash after possible dispatch

    let journal = Journal::open(&path).await.unwrap();
    let ambiguous = journal.ambiguous_attempts().await.unwrap();
    assert!(
        ambiguous.is_empty(),
        "before recovery the state is `dispatched`, not yet ambiguous"
    );

    let report = journal.recover(now()).await.unwrap();
    assert_eq!(report.now_ambiguous.len(), 1);
    assert_eq!(report.now_ambiguous[0].attempt_id, attempt_id);

    // The node supervisor does NOT retry: the ambiguous view is what reconciliation
    // and the operator see. The adapter's status lookup proves success instead.
    journal
        .prove_result(
            &attempt_id,
            ProvenStatus::Completed,
            None,
            Some(&json!({ "provider_status_lookup": "completed" })),
            now(),
        )
        .await
        .unwrap();
    let counts = journal.counts().await.unwrap();
    assert_eq!(counts.attempts_completed, 1);
    assert_eq!(counts.attempts_outcome_unknown, 0);
}
