//! LIVE Claude qualification test (`PHASE-1.4.2`): one bounded REAL dispatch through
//! the REAL `claude` binary, the REAL supervisor, and the REAL journal — proving the
//! acceptance on the actual harness: dispatch ack (system/init session id) ≠ completion
//! (result event), exact usage normalization INCLUDING money cost (`total_cost_usd` —
//! the one leg Codex cannot prove), the streamed session id attached as the provider
//! handle, and an unsupported status lookup reported honestly.
//!
//! Deliberately NOT run by default: it dispatches to the user's ambient Claude login
//! and consumes a few tokens. Run it explicitly:
//!
//! ```text
//! RB_LIVE_CLAUDE=1 cargo test -p reasonbraid-node --test claude_live -- --ignored --nocapture
//! ```
//!
//! The prompt is tiny and bounded ("Reply with exactly: ok"); no files are touched
//! (`--restricted --tools ''` — content-only). A generous reservation + local budget
//! gate the dispatch (`.5.2`).

use std::path::PathBuf;

use chrono::Utc;
use reasonbraid_adapter::{Adapter, ClaudeCliAdapter, RunRequest, StatusLookupOutcome};
use reasonbraid_core::{BudgetDimensions, ReservationReference};
use reasonbraid_node::{execute_attempt, CommandInput, Journal, LocalBudget};
use serde_json::json;

fn reservation(tag: &str) -> ReservationReference {
    ReservationReference {
        reservation_id: format!("res_{tag}"),
        dimensions: BudgetDimensions {
            calls: Some(1),
            input_tokens: Some(1_000_000),
            output_tokens: Some(1_000_000),
            wall_clock_seconds: Some(3600),
        },
        issued_at: Utc::now(),
    }
}

fn generous_local() -> LocalBudget {
    LocalBudget::new(BudgetDimensions {
        calls: Some(1000),
        input_tokens: Some(100_000_000),
        output_tokens: Some(100_000_000),
        wall_clock_seconds: Some(10_000_000),
    })
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
        .join(format!("claude-live-{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("node.db")
}

#[tokio::test]
#[ignore = "live harness dispatch — run with RB_LIVE_CLAUDE=1 cargo test -p reasonbraid-node --test claude_live -- --ignored --nocapture"]
async fn live_claude_dispatch_completes_with_usage_cost_and_an_honest_unsupported_lookup() {
    if std::env::var("RB_LIVE_CLAUDE").is_err() {
        eprintln!("SKIP: RB_LIVE_CLAUDE is unset — this test dispatches to the real Claude CLI");
        return;
    }

    let journal = Journal::open(live_journal_path()).await.unwrap();
    let command_id = "cmd_live_claude";
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

    let adapter = ClaudeCliAdapter::new();
    let request = RunRequest {
        payload: json!({ "prompt": "Reply with exactly: ok" }),
        deadline: None,
        budget_hint: None,
    };

    let report = execute_attempt(
        &journal,
        &adapter,
        &op,
        &request,
        &reservation("live"),
        &generous_local(),
    )
    .await
    .expect("the live Claude dispatch completes");

    // The acceptance, on the REAL harness.
    assert_eq!(report.final_state.as_str(), "completed");
    assert!(
        report.chunks.iter().any(|c| c.contains("ok")),
        "the model's reply streams through: {:?}",
        report.chunks
    );
    let usage = report
        .usage
        .expect("the result-event receipt is normalized");
    assert!(
        usage.input_tokens.unwrap_or(0) > 0,
        "a real dispatch bills input tokens"
    );
    // The leg Codex cannot prove: Claude reports MONEY.
    assert!(
        usage.cost.is_some(),
        "total_cost_usd rides the receipt — the cost is known, not unknown"
    );

    // The streamed session id is attached as the provider handle.
    let summary = journal.attempt_summary(&report.attempt_id).await.unwrap();
    let provider_id = summary
        .provider_request_id
        .as_deref()
        .expect("system/init surfaced a session id");
    assert!(!provider_id.is_empty());

    // The honest unsupported lookup: no status query exists on this boundary.
    assert!(matches!(
        adapter.query_status(&op).await,
        StatusLookupOutcome::Unsupported
    ));

    eprintln!(
        "LIVE CLAUDE OK: attempt {} completed via session {provider_id}",
        report.attempt_id
    );
}
