//! Integration tests for the `rb-journal` inspection CLI (`PHASE-0.3.1`).
//!
//! The WP3 acceptance: "operators can inspect pending and ambiguous journal entries
//! without opening SQLite manually." These tests run the real binary against journals
//! seeded through the library, and prove the CLI is read-only in practice: the file
//! bytes are unchanged by any inspection, and the CLI runs beside a live writer (WAL).

use std::path::PathBuf;
use std::process::Command;

use chrono::{DateTime, Utc};
use reasonbraid_node::{CommandInput, Journal};
use serde_json::{json, Value};

const BIN: &str = env!("CARGO_BIN_EXE_rb-journal");

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

async fn seed_operation(journal: &Journal, tag: &str) -> String {
    let cmd_id = format!("cmd_{tag}");
    let payload = json!({ "operation": "contribute" });
    journal
        .record_command(&command(&cmd_id, "c1", &payload), now())
        .await
        .expect("record command");
    journal
        .ensure_operation(&cmd_id, now())
        .await
        .expect("ensure operation")
        .operation_id
}

/// A journal with one pending attempt (dispatched), one ambiguous attempt
/// (outcome_unknown with boundary history), and one emitted-but-unacked event.
async fn seed_rich_journal(path: &PathBuf) {
    let journal = Journal::open(path).await.unwrap();

    let op_a = seed_operation(&journal, "pending").await;
    journal
        .prepare_attempt("patt_pending", &op_a, now())
        .await
        .unwrap();
    journal
        .record_dispatch("patt_pending", Some("prv_pending"), now())
        .await
        .unwrap();

    let op_b = seed_operation(&journal, "ambiguous").await;
    journal
        .prepare_attempt("patt_ambiguous", &op_b, now())
        .await
        .unwrap();
    journal
        .record_dispatch("patt_ambiguous", Some("prv_ambiguous"), now())
        .await
        .unwrap();
    journal
        .record_outcome_unknown("patt_ambiguous", Some("adapter timeout"), now())
        .await
        .unwrap();

    journal
        .record_outgoing_event(
            "evt_pending",
            &op_a,
            &json!({ "event_type": "contribution.ready" }),
            now(),
        )
        .await
        .unwrap();
}

/// `inspect` reports the recorded durability profile, integrity, and row counts —
/// the operator sees the journal's real settings without touching SQLite.
#[tokio::test]
async fn inspect_reports_profile_and_counts() {
    let path = journal_path("cli-inspect");
    seed_rich_journal(&path).await;

    let out = Command::new(BIN)
        .arg("inspect")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("journal_mode: wal (recorded profile)"),
        "{stdout}"
    );
    assert!(
        stdout.contains("synchronous: FULL (recorded profile)"),
        "{stdout}"
    );
    assert!(stdout.contains("quick_check: ok"), "{stdout}");
    assert!(stdout.contains("schema user_version: 3"), "{stdout}");
    assert!(
        stdout.contains("attempts: prepared=0 dispatched=1"),
        "{stdout}"
    );
    assert!(stdout.contains("outcome_unknown=1"), "{stdout}");
    assert!(
        stdout.contains("outgoing events: emitted=1 acked=0"),
        "{stdout}"
    );
}

/// `pending` lists in-flight attempts and unacked events; `--json` is machine-readable
/// and carries the same facts.
#[tokio::test]
async fn pending_lists_inflight_attempts_and_unacked_events() {
    let path = journal_path("cli-pending");
    seed_rich_journal(&path).await;

    let out = Command::new(BIN)
        .args(["pending", "--json"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let parsed: Value = serde_json::from_slice(&out.stdout).unwrap();

    let attempts = parsed["attempts"].as_array().unwrap();
    assert_eq!(
        attempts.len(),
        1,
        "only the dispatched attempt is pending: {parsed}"
    );
    assert_eq!(attempts[0]["attempt_id"], "patt_pending");
    assert_eq!(attempts[0]["status"], "dispatched");
    assert_eq!(attempts[0]["provider_request_id"], "prv_pending");

    let events = parsed["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event_id"], "evt_pending");
}

/// `ambiguous` lists outcome_unknown attempts with their before/after boundary
/// history — the evidence an operator needs to adjudicate or trigger a status lookup.
#[tokio::test]
async fn ambiguous_lists_with_boundary_history() {
    let path = journal_path("cli-ambiguous");
    seed_rich_journal(&path).await;

    let out = Command::new(BIN)
        .args(["ambiguous", "--json"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let parsed: Value = serde_json::from_slice(&out.stdout).unwrap();

    let attempts = parsed["attempts"].as_array().unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0]["attempt"]["attempt_id"], "patt_ambiguous");
    assert_eq!(attempts[0]["attempt"]["status"], "outcome_unknown");
    assert_eq!(
        attempts[0]["attempt"]["provider_request_id"],
        "prv_ambiguous"
    );

    let transitions = attempts[0]["transitions"].as_array().unwrap();
    assert_eq!(transitions.len(), 2);
    assert_eq!(transitions[0]["from_status"], "prepared");
    assert_eq!(transitions[0]["to_status"], "dispatched");
    assert_eq!(transitions[1]["from_status"], "dispatched");
    assert_eq!(transitions[1]["to_status"], "outcome_unknown");
}

/// The CLI is read-only in practice: every inspection leaves the journal file
/// byte-identical (the handle is `SQLITE_OPEN_READONLY`), so operators can never
/// damage the journal by looking at it.
#[tokio::test]
async fn inspections_never_mutate_the_journal_file() {
    let path = journal_path("cli-readonly");
    seed_rich_journal(&path).await;

    let before = std::fs::read(&path).unwrap();
    let runs: [&[&str]; 5] = [
        &["inspect"],
        &["pending"],
        &["pending", "--json"],
        &["ambiguous"],
        &["ambiguous", "--json"],
    ];
    for args in runs {
        let out = Command::new(BIN).args(args).arg(&path).output().unwrap();
        assert!(
            out.status.success(),
            "{args:?} stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let after = std::fs::read(&path).unwrap();
    assert_eq!(before, after, "inspection must not mutate the journal file");
}

/// The CLI runs BESIDE a live node (the writer holds the journal open): WAL mode
/// gives the reader the committed state without disturbing the writer.
#[tokio::test]
async fn cli_runs_beside_a_live_writer() {
    let path = journal_path("cli-live");
    let journal = Journal::open(&path).await.unwrap();
    seed_rich_journal(&path).await;
    // journal (the "node") stays open for the whole CLI run below.

    let out = Command::new(BIN)
        .arg("inspect")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("quick_check: ok"), "{stdout}");
    assert!(
        stdout.contains("attempts: prepared=0 dispatched=1"),
        "{stdout}"
    );
    drop(journal);
}

/// Missing files and garbage files fail cleanly with a nonzero exit and an error on
/// stderr — never a crash or a hang.
#[tokio::test]
async fn missing_and_garbage_files_fail_cleanly() {
    let dir = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let unique = uuid::Uuid::now_v7();
    let dir = dir
        .join("journal-tests")
        .join(format!("cli-errors-{unique}"));
    std::fs::create_dir_all(dir.parent().expect("the fixture parent")).unwrap();
    // Exclusive: an existing directory belongs to another fixture or an
    // earlier run, and must never be adopted.
    std::fs::DirBuilder::new()
        .create(&dir)
        .expect("the fixture directory is new");

    let missing = dir.join("absent.db");
    let out = Command::new(BIN)
        .arg("inspect")
        .arg(&missing)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("error:"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let garbage = dir.join("garbage.db");
    std::fs::write(&garbage, b"not a database").unwrap();
    let out = Command::new(BIN)
        .arg("inspect")
        .arg(&garbage)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("error:"));
}
