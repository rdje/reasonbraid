//! Supervisor × Codex-stub flow tests (`PHASE-0.4.2`): the real first-adapter
//! semantics driven through the REAL supervisor + journal with a stub binary — the
//! streamed provider handle is attached to the attempt, a lost response is an honest
//! `outcome_unknown` (never a retry recommendation), and the completion path lands
//! `completed` with normalized usage. No provider spend; the live dispatch is
//! `tests/codex_live.rs` (env-gated).

use std::path::PathBuf;

use chrono::Utc;
use reasonbraid_adapter::{Adapter, CodexCliAdapter, RunRequest, StatusLookupOutcome};
use reasonbraid_core::{BudgetDimensions, ReservationReference};
use reasonbraid_node::{execute_attempt, CommandInput, Journal, LocalBudget, SupervisorError};
use serde_json::json;

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

fn stub_binary(name: &str) -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let unique = uuid::Uuid::now_v7();
    let dir = base.join("codex-stubs").join(format!("{name}-{unique}"));
    std::fs::create_dir_all(dir.parent().expect("the fixture parent")).unwrap();
    // Exclusive: an existing directory belongs to another fixture or an
    // earlier run, and must never be adopted.
    std::fs::DirBuilder::new()
        .create(&dir)
        .expect("the fixture directory is new");
    let path = dir.join("codex");
    let script = r#"#!/bin/sh
for last in "$@"; do :; done
case "$last" in
  *lose*)
    echo '{"type":"thread.started","thread_id":"stub_lose"}'
    exit 0
    ;;
  *)
    echo '{"type":"thread.started","thread_id":"stub_ok"}'
    echo '{"type":"item.completed","item":{"type":"agent_message","text":"done"}}'
    echo '{"type":"turn.completed","usage":{"input_tokens":7,"output_tokens":3}}'
    exit 0
    ;;
esac
"#;
    std::fs::write(&path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}

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

fn request_with(prompt: &str) -> RunRequest {
    RunRequest {
        payload: json!({ "prompt": prompt }),
        deadline: None,
        budget_hint: None,
    }
}

async fn seed_operation(journal: &Journal, tag: &str) -> String {
    let command_id = format!("cmd_{tag}");
    let payload = json!({ "operation": "contribute" });
    journal
        .record_command(
            &CommandInput {
                command_id: &command_id,
                tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                thread_id: "thr_00000000-0000-7000-8000-000000000000",
                payload: &payload,
                authz_ref: None,
                policy_digest: None,
                decided_at: None,
                revocation_epoch: None,
                server_cursor: "1",
            },
            Utc::now(),
        )
        .await
        .expect("record command");
    journal
        .ensure_operation(&command_id, Utc::now())
        .await
        .expect("ensure operation")
        .operation_id
}

/// The completion path: the streamed provider handle is attached to the attempt (the
/// Codex `thread.started` id), and the attempt lands `completed` with exact usage.
#[tokio::test]
async fn completion_attaches_the_streamed_provider_handle_and_lands_completed() {
    let journal = Journal::open(journal_path("codex-complete")).await.unwrap();
    let op = seed_operation(&journal, "complete").await;
    let adapter = CodexCliAdapter::with_binary(stub_binary("complete"));

    let report = execute_attempt(
        &journal,
        &adapter,
        &op,
        &request_with("ok"),
        &reservation("bud"),
        &generous_local(),
    )
    .await
    .expect("the stub completes");
    assert_eq!(report.final_state.as_str(), "completed");
    assert_eq!(report.chunks, vec!["done".to_string()]);
    let usage = report.usage.expect("usage reported");
    assert_eq!(usage.input_tokens, Some(7));
    assert_eq!(usage.output_tokens, Some(3));

    let summary = journal.attempt_summary(&report.attempt_id).await.unwrap();
    assert_eq!(
        summary.provider_request_id.as_deref(),
        Some("stub_ok"),
        "the streamed thread id is the attempt's proof handle"
    );
    assert_eq!(summary.status, "completed");
}

/// THE acceptance's honest leg with the REAL first adapter: a lost response and an
/// unsupported status lookup land `outcome_unknown` — the error carries no retry
/// recommendation — while the provider handle from the stream is still attached.
#[tokio::test]
async fn lost_response_with_unsupported_lookup_is_outcome_unknown_never_retry() {
    let journal = Journal::open(journal_path("codex-lost")).await.unwrap();
    let op = seed_operation(&journal, "lost").await;
    let adapter = CodexCliAdapter::with_binary(stub_binary("lost"));

    let err = execute_attempt(
        &journal,
        &adapter,
        &op,
        &request_with("lose it"),
        &reservation("bud"),
        &generous_local(),
    )
    .await
    .expect_err("an indeterminate attempt must not report success");
    let SupervisorError::OutcomeUnknown { attempt_id, .. } = &err else {
        panic!("expected OutcomeUnknown, got {err}");
    };
    assert!(
        !err.to_string().contains("retry"),
        "no retry language: {err}"
    );

    let summary = journal.attempt_summary(attempt_id).await.unwrap();
    assert_eq!(summary.status, "outcome_unknown");
    assert_eq!(
        summary.provider_request_id.as_deref(),
        Some("stub_lose"),
        "the provider handle survives into the ambiguous attempt (proof handle for adjudication)"
    );

    // The boundary's own declaration: status lookup is genuinely unsupported here.
    assert!(matches!(
        adapter.query_status(&op).await,
        StatusLookupOutcome::Unsupported
    ));
}
