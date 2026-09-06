//! LIVE Codex qualification test (`PHASE-0.4.2`): one bounded REAL dispatch through
//! the REAL `codex` binary, the REAL supervisor, and the REAL journal — proving the
//! acceptance on the actual harness: dispatch ack (thread.started) ≠ completion
//! (turn.completed), exact usage normalization, the streamed thread id attached as the
//! provider handle, and an unsupported status lookup reported honestly.
//!
//! Deliberately NOT run by default: it dispatches to the user's ambient Codex login
//! and consumes a few tokens. Run it explicitly:
//!
//! ```text
//! RB_LIVE_CODEX=1 cargo test -p reasonbraid-node --test codex_live -- --ignored --nocapture
//! ```
//!
//! The prompt is tiny and bounded ("Reply with exactly: ok"); no files are touched
//! (`--skip-git-repo-check --ephemeral --sandbox read-only`).

use std::path::PathBuf;

use chrono::Utc;
use reasonbraid_adapter::{Adapter, CodexCliAdapter, RunRequest, StatusLookupOutcome};
use reasonbraid_node::{execute_attempt, CommandInput, Journal};
use serde_json::json;

#[tokio::test]
#[ignore = "live harness dispatch — run with RB_LIVE_CODEX=1 cargo test -p reasonbraid-node --test codex_live -- --ignored --nocapture"]
async fn live_codex_dispatch_completes_with_usage_and_an_honest_unsupported_lookup() {
    if std::env::var("RB_LIVE_CODEX").is_err() {
        eprintln!("SKIP: RB_LIVE_CODEX is unset — this test dispatches to the real Codex CLI");
        return;
    }

    let journal = Journal::open(live_journal_path()).await.unwrap();
    let command_id = "cmd_live_codex";
    let payload = json!({ "operation": "contribute" });
    journal
        .record_command(
            &CommandInput {
                command_id,
                tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                thread_id: "thr_00000000-0000-7000-8000-000000000000",
                payload: &payload,
                authz_ref: None,
                server_cursor: "1",
            },
            Utc::now(),
        )
        .await
        .expect("record command");
    let op = journal
        .ensure_operation(command_id, Utc::now())
        .await
        .expect("ensure operation")
        .operation_id;

    let adapter = CodexCliAdapter::new();
    let request = RunRequest {
        payload: json!({ "prompt": "Reply with exactly: ok" }),
        deadline: None,
        budget_hint: None,
    };

    let report = execute_attempt(&journal, &adapter, &op, &request)
        .await
        .expect("the live Codex dispatch completes");

    // The acceptance, on the REAL harness.
    assert_eq!(report.final_state.as_str(), "completed");
    assert!(
        report.chunks.iter().any(|c| c.contains("ok")),
        "the model's reply streams through: {:?}",
        report.chunks
    );
    let usage = report
        .usage
        .expect("the turn.completed receipt is normalized");
    assert!(
        usage.input_tokens.unwrap_or(0) > 0,
        "a real dispatch bills input tokens"
    );

    // The streamed thread id is attached as the provider handle.
    let summary = journal.attempt_summary(&report.attempt_id).await.unwrap();
    let provider_id = summary
        .provider_request_id
        .as_deref()
        .expect("thread.started surfaced a thread id");
    assert!(!provider_id.is_empty());

    // The honest unsupported lookup: no status query exists on this boundary.
    assert!(matches!(
        adapter.query_status(&op).await,
        StatusLookupOutcome::Unsupported
    ));

    eprintln!(
        "LIVE CODEX OK: attempt {} completed via thread {provider_id}",
        report.attempt_id
    );
}

fn live_journal_path() -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = base
        .join("journal-tests")
        .join(format!("codex-live-{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("node.db")
}
