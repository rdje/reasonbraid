//! WP5 budget tests, node side (`PHASE-0.5.2`): the supervisor's dispatch gate — no
//! provider dispatch without an applicable reservation, local headroom enforced,
//! settlement with actual usage, and an indeterminate attempt KEEPING its hold
//! (§14.6: release only amounts not potentially consumed). These are the "denial tests
//! cross both server and node boundaries" node leg; the server leg is
//! `crates/reasonbraid-server/tests/budget.rs`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use chrono::Utc;
use reasonbraid_adapter::{
    Adapter, AdapterCapabilities, AttemptEvent, AttemptHandle, AttemptStream, CancellationOutcome,
    CancellationStrength, DispatchAck, FakeAdapter, InvokeOutcome, NormalizedUsage,
    PolicyInjectionMode, RunRequest, StatusLookupOutcome, UsageConfidence,
};
use reasonbraid_core::{BudgetDimensions, ReservationReference};
use reasonbraid_node::{execute_attempt, CommandInput, Journal, LocalBudget};
use serde_json::{json, Value};

fn journal_path(name: &str) -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = base.join("journal-tests").join(format!("{name}-{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("node.db")
}

fn run_request() -> RunRequest {
    RunRequest {
        payload: json!({ "prompt": "ok" }),
        deadline: None,
        budget_hint: None,
    }
}

fn reservation(tag: &str, calls: Option<u64>, tokens: Option<u64>) -> ReservationReference {
    ReservationReference {
        reservation_id: format!("res_{tag}"),
        dimensions: BudgetDimensions {
            calls,
            input_tokens: tokens,
            output_tokens: tokens,
            wall_clock_seconds: Some(3600),
        },
        issued_at: Utc::now(),
    }
}

fn local(ceiling_calls: u64, ceiling_tokens: u64) -> LocalBudget {
    LocalBudget::new(BudgetDimensions {
        calls: Some(ceiling_calls),
        input_tokens: Some(ceiling_tokens),
        output_tokens: Some(ceiling_tokens),
        wall_clock_seconds: Some(10_000),
    })
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

/// An adapter that COUNTS invocations and completes instantly (the "was the adapter
/// ever reached" oracle for the refusal tests).
struct CountingAdapter {
    invocations: Arc<AtomicU32>,
}

impl CountingAdapter {
    fn new() -> (Self, Arc<AtomicU32>) {
        let invocations = Arc::new(AtomicU32::new(0));
        (
            Self {
                invocations: invocations.clone(),
            },
            invocations,
        )
    }
}

impl Adapter for CountingAdapter {
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            streaming: false,
            cancellation: CancellationStrength::None,
            provider_idempotency: false,
            status_lookup: false,
            tool_support: false,
            policy_injection: PolicyInjectionMode::None,
        }
    }

    async fn invoke(&self, _request: &RunRequest, _operation_id: &str) -> InvokeOutcome {
        self.invocations.fetch_add(1, Ordering::SeqCst);
        InvokeOutcome::Accepted(
            DispatchAck {
                provider_request_id: None,
            },
            AttemptHandle::new(Box::new(InstantHandle)),
        )
    }

    async fn cancel(&self, _operation_id: &str) -> CancellationOutcome {
        CancellationOutcome::Confirmed
    }

    async fn query_status(&self, _operation_id: &str) -> StatusLookupOutcome {
        StatusLookupOutcome::Unsupported
    }

    fn normalize_usage(&self, _raw_receipt: &Value) -> NormalizedUsage {
        NormalizedUsage {
            input_tokens: None,
            output_tokens: None,
            cost: None,
            confidence: UsageConfidence::Unknown,
        }
    }
}

struct InstantHandle;

impl AttemptStream for InstantHandle {
    fn next(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<AttemptEvent>> + Send + '_>>
    {
        Box::pin(async { Some(AttemptEvent::Completed { usage: None }) })
    }
}

/// THE node-boundary denial: a reservation that covers no dispatch is refused BEFORE
/// the boundary record — the adapter is never invoked, and the journal says
/// `failed_before_dispatch` (audited).
#[tokio::test]
async fn a_reservation_covering_no_dispatch_is_refused_before_the_boundary() {
    let journal = Journal::open(journal_path("budget-no-res")).await.unwrap();
    let op = seed_operation(&journal, "nores").await;
    let (adapter, invocations) = CountingAdapter::new();

    let report = execute_attempt(
        &journal,
        &adapter,
        &op,
        &run_request(),
        &reservation("nores", None, Some(100)),
        &local(10, 10_000),
    )
    .await
    .expect("a refusal is an audited report, not an error");

    assert_eq!(report.final_state.as_str(), "failed_before_dispatch");
    assert_eq!(
        invocations.load(Ordering::SeqCst),
        0,
        "the adapter was NEVER invoked"
    );
    let history = journal.attempt_history(&report.attempt_id).await.unwrap();
    assert_eq!(history.len(), 1, "no dispatch boundary was recorded");
    assert_eq!(history[0].to_status, "failed_before_dispatch");
}

/// THE second node-boundary denial: local headroom exhausted — refused, not dispatched.
#[tokio::test]
async fn exhausted_local_headroom_refuses_the_dispatch() {
    let journal = Journal::open(journal_path("budget-local")).await.unwrap();
    let op = seed_operation(&journal, "local").await;
    let (adapter, invocations) = CountingAdapter::new();

    // The reservation asks for 1000 tokens; the node's local ceiling is 10.
    let report = execute_attempt(
        &journal,
        &adapter,
        &op,
        &run_request(),
        &reservation("local", Some(1), Some(1000)),
        &local(10, 10),
    )
    .await
    .expect("a refusal is an audited report, not an error");

    assert_eq!(report.final_state.as_str(), "failed_before_dispatch");
    assert_eq!(invocations.load(Ordering::SeqCst), 0);
    let summary = journal.attempt_summary(&report.attempt_id).await.unwrap();
    assert!(
        summary
            .evidence
            .as_deref()
            .unwrap_or_default()
            .contains("local budget"),
        "the reason is journaled: {:?}",
        summary.evidence
    );
}

/// Settlement with ACTUAL usage: the completed attempt leaves the local ledger with
/// the real usage (calls + tokens), not the reservation's hold.
#[tokio::test]
async fn a_completed_attempt_settles_actual_usage_locally() {
    let journal = Journal::open(journal_path("budget-settle")).await.unwrap();
    let op = seed_operation(&journal, "settle").await;
    let fake = FakeAdapter::from_spec(
        reasonbraid_adapter::fixtures::corpus()
            .into_iter()
            .find(|f| f.name == "usage_receipt")
            .unwrap(),
    );
    let local = local(10, 10_000);

    let report = execute_attempt(
        &journal,
        &fake,
        &op,
        &run_request(),
        &reservation("settle", Some(1), Some(5000)),
        &local,
    )
    .await
    .expect("completes");

    assert_eq!(report.final_state.as_str(), "completed");
    assert_eq!(report.reservation_id, "res_settle");
    let consumed = local.consumed().await;
    assert_eq!(consumed.calls, Some(1), "one call settled");
    assert_eq!(
        consumed.input_tokens,
        Some(100),
        "ACTUAL usage, not the 5000 hold"
    );
    assert_eq!(consumed.output_tokens, Some(10));
}

/// §14.6: an indeterminate attempt KEEPS its hold — release only amounts not
/// potentially consumed, and an ambiguous attempt may have consumed.
#[tokio::test]
async fn an_indeterminate_attempt_keeps_its_hold() {
    let journal = Journal::open(journal_path("budget-hold")).await.unwrap();
    let op = seed_operation(&journal, "hold").await;
    let fake = FakeAdapter::from_spec(
        reasonbraid_adapter::fixtures::corpus()
            .into_iter()
            .find(|f| f.name == "lose_response_no_lookup")
            .unwrap(),
    );
    let local = local(10, 10_000);

    let err = execute_attempt(
        &journal,
        &fake,
        &op,
        &run_request(),
        &reservation("hold", Some(1), Some(1000)),
        &local,
    )
    .await
    .expect_err("indeterminate");

    assert!(err.to_string().contains("outcome_unknown"), "got: {err}");
    let consumed = local.consumed().await;
    assert_eq!(
        consumed.calls,
        Some(1),
        "the hold stays in place until adjudication releases it"
    );
    assert_eq!(consumed.input_tokens, Some(1000));
}
