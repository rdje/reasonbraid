//! The supervisor's two bounds on a provider attempt (`SIGNOFF-REPAIR.4.4.6`):
//! the request's DEADLINE, and the size of the output it collects. Before this
//! leaf the deadline was computed and consumed by nothing, so a provider that
//! never answered held the worker for ever; and the output grew without limit,
//! into a result the control plane would refuse to receive.

use std::time::Duration;

use chrono::Utc;
use reasonbraid_adapter::{
    Adapter, AdapterCapabilities, CancellationOutcome, CancellationStrength, FakeAdapter,
    InvokeOutcome, NormalizedUsage, PolicyInjectionMode, RunRequest, ScriptStep,
    StatusLookupOutcome, StatusLookupSpec,
};
use reasonbraid_core::fixture::Fixture;
use reasonbraid_core::{BudgetDimensions, ProviderAttemptState, ReservationReference};
use reasonbraid_node::{execute_attempt, CommandInput, Journal, LocalBudget, SupervisorError};
use serde_json::json;

/// The output bound under test (256 KiB), pinned to the library's below.
const BOUND: usize = 256 * 1024;

/// The bound these controls drive is the one the supervisor enforces.
#[test]
fn the_bound_under_test_is_the_supervisors() {
    assert_eq!(reasonbraid_node::MAX_OUTPUT_BYTES, BOUND);
}

fn fixture(name: &str) -> Fixture {
    Fixture::create("supervisor-bounds-tests", name).expect("a new fixture")
}

fn reservation() -> ReservationReference {
    ReservationReference {
        reservation_id: "res_bounds".to_string(),
        dimensions: BudgetDimensions {
            calls: Some(1),
            input_tokens: Some(1_000_000),
            output_tokens: Some(1_000_000),
            wall_clock_seconds: Some(3600),
        },
        issued_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::minutes(10),
    }
}

fn local() -> LocalBudget {
    LocalBudget::new(BudgetDimensions {
        calls: Some(1000),
        input_tokens: Some(100_000_000),
        output_tokens: Some(100_000_000),
        wall_clock_seconds: Some(10_000_000),
    })
}

fn adapter(script: Vec<ScriptStep>) -> FakeAdapter {
    FakeAdapter::new(
        script,
        StatusLookupSpec::Unsupported,
        AdapterCapabilities {
            streaming: true,
            cancellation: CancellationStrength::BestEffort,
            provider_idempotency: false,
            status_lookup: false,
            tool_support: false,
            policy_injection: PolicyInjectionMode::None,
        },
    )
}

fn request(deadline: Option<chrono::DateTime<Utc>>) -> RunRequest {
    RunRequest {
        payload: json!({ "operation": "contribute" }),
        deadline,
        budget_hint: None,
    }
}

async fn operation(journal: &Journal, tag: &str) -> String {
    let command_id = format!("cmd_{tag}");
    journal
        .record_command(
            &CommandInput {
                command_id: &command_id,
                tenant_id: "ten_00000000-0000-7000-8000-000000000000",
                thread_id: "thr_00000000-0000-7000-8000-000000000000",
                payload: &json!({ "operation": "contribute" }),
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

/// THE deadline: a provider that accepts and then never answers is cancelled at
/// the request's deadline, and the attempt lands on the honest `outcome_unknown`
/// (the provider may have run), naming the deadline. The outer bound only keeps a
/// regression from hanging the suite.
#[tokio::test]
async fn a_provider_that_never_answers_is_abandoned_at_the_deadline() {
    let fixture = fixture("deadline");
    let journal = Journal::open(fixture.join("node.db")).await.unwrap();
    let op = operation(&journal, "deadline").await;
    let deadline = Utc::now() + chrono::Duration::milliseconds(200);

    let outcome = tokio::time::timeout(
        Duration::from_secs(10),
        execute_attempt(
            &journal,
            &adapter(vec![ScriptStep::HangForever]),
            &op,
            &request(Some(deadline)),
            &reservation(),
            &local(),
        ),
    )
    .await
    .expect("the attempt ended at its deadline, not at the outer bound");
    let Err(SupervisorError::OutcomeUnknown { attempt_id, .. }) = outcome else {
        panic!("expected an honest outcome_unknown, got {outcome:?}");
    };
    let attempt = journal
        .attempts_for_operation(&op)
        .await
        .unwrap()
        .into_iter()
        .find(|a| a.attempt_id == attempt_id)
        .expect("the attempt");
    assert_eq!(attempt.status, "outcome_unknown");
    let evidence = attempt.evidence.unwrap_or_default();
    assert!(
        evidence.contains("deadline"),
        "the evidence names the deadline: {evidence}"
    );
    // The provider was TOLD to stop, and answered: the fake's hang ends only on
    // a cancellation, which it confirms.
    assert!(
        evidence.contains("cancellation: Confirmed"),
        "the evidence names the cancellation's answer: {evidence}"
    );
}

/// A fake whose cancellation takes a moment to answer, as a real one does (a CLI
/// adapter cancels by ending a process). Everything else is the wrapped fake's.
struct SlowCancel(FakeAdapter);

impl Adapter for SlowCancel {
    fn capabilities(&self) -> AdapterCapabilities {
        self.0.capabilities()
    }

    async fn invoke(&self, request: &RunRequest, operation_id: &str) -> InvokeOutcome {
        self.0.invoke(request, operation_id).await
    }

    async fn cancel(&self, operation_id: &str) -> CancellationOutcome {
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.0.cancel(operation_id).await
    }

    async fn query_status(&self, operation_id: &str) -> StatusLookupOutcome {
        self.0.query_status(operation_id).await
    }

    fn normalize_usage(&self, raw_receipt: &serde_json::Value) -> NormalizedUsage {
        self.0.normalize_usage(raw_receipt)
    }
}

/// The cancellation is WAITED FOR, within its grace: an answer that takes a
/// moment still arrives and is recorded, rather than being dropped unsent.
#[tokio::test]
async fn a_cancellation_that_takes_a_moment_is_waited_for() {
    let fixture = fixture("slow-cancel");
    let journal = Journal::open(fixture.join("node.db")).await.unwrap();
    let op = operation(&journal, "slow-cancel").await;
    let deadline = Utc::now() + chrono::Duration::milliseconds(100);

    let outcome = tokio::time::timeout(
        Duration::from_secs(10),
        execute_attempt(
            &journal,
            &SlowCancel(adapter(vec![ScriptStep::HangForever])),
            &op,
            &request(Some(deadline)),
            &reservation(),
            &local(),
        ),
    )
    .await
    .expect("the attempt ended");
    let Err(SupervisorError::OutcomeUnknown { attempt_id, .. }) = outcome else {
        panic!("expected an honest outcome_unknown, got {outcome:?}");
    };
    let evidence = journal
        .attempts_for_operation(&op)
        .await
        .unwrap()
        .into_iter()
        .find(|a| a.attempt_id == attempt_id)
        .and_then(|a| a.evidence)
        .unwrap_or_default();
    assert!(
        evidence.contains("cancellation: Confirmed"),
        "the slow answer was waited for: {evidence}"
    );
}

/// THE output bound: a result larger than the bound is a definitive failure that
/// names the bound, and none of its excess is kept. It is never delivered cut
/// short, and never delivered whole to a control plane that would refuse it.
#[tokio::test]
async fn an_output_over_the_bound_fails_the_attempt_and_names_the_bound() {
    let fixture = fixture("over-bound");
    let journal = Journal::open(fixture.join("node.db")).await.unwrap();
    let op = operation(&journal, "over-bound").await;
    let script = vec![
        ScriptStep::EmitChunk {
            chunk: "x".repeat(BOUND),
        },
        ScriptStep::EmitChunk {
            chunk: "y".to_string(),
        },
        ScriptStep::Complete { usage: None },
    ];
    let report = execute_attempt(
        &journal,
        &adapter(script),
        &op,
        &request(None),
        &reservation(),
        &local(),
    )
    .await
    .expect("a bounded failure is a result, not an error");
    assert_eq!(report.final_state, ProviderAttemptState::FailedKnown);
    assert!(
        report.chunks.iter().map(String::len).sum::<usize>() <= BOUND,
        "nothing past the bound is kept"
    );
    let attempt = journal
        .attempts_for_operation(&op)
        .await
        .unwrap()
        .into_iter()
        .find(|a| a.attempt_id == report.attempt_id)
        .expect("the attempt");
    assert_eq!(attempt.status, "failed_known");
    let evidence = attempt.evidence.unwrap_or_default();
    assert!(
        evidence.contains(&BOUND.to_string()),
        "the failure names the bound: {evidence}"
    );
}

/// The edge: output of EXACTLY the bound completes, whole.
#[tokio::test]
async fn an_output_at_the_bound_completes_whole() {
    let fixture = fixture("at-bound");
    let journal = Journal::open(fixture.join("node.db")).await.unwrap();
    let op = operation(&journal, "at-bound").await;
    let script = vec![
        ScriptStep::EmitChunk {
            chunk: "x".repeat(BOUND),
        },
        ScriptStep::Complete { usage: None },
    ];
    let report = execute_attempt(
        &journal,
        &adapter(script),
        &op,
        &request(None),
        &reservation(),
        &local(),
    )
    .await
    .expect("completes");
    assert_eq!(report.final_state, ProviderAttemptState::Completed);
    assert_eq!(report.chunks.iter().map(String::len).sum::<usize>(), BOUND);
}

/// A provider that takes the request and never even acknowledges it.
struct SilentInvoke(FakeAdapter);

impl Adapter for SilentInvoke {
    fn capabilities(&self) -> AdapterCapabilities {
        self.0.capabilities()
    }

    async fn invoke(&self, _request: &RunRequest, _operation_id: &str) -> InvokeOutcome {
        std::future::pending().await
    }

    async fn cancel(&self, operation_id: &str) -> CancellationOutcome {
        self.0.cancel(operation_id).await
    }

    async fn query_status(&self, operation_id: &str) -> StatusLookupOutcome {
        self.0.query_status(operation_id).await
    }

    fn normalize_usage(&self, raw_receipt: &serde_json::Value) -> NormalizedUsage {
        self.0.normalize_usage(raw_receipt)
    }
}

/// The deadline also bounds the wait for the ACKNOWLEDGEMENT. The dispatch
/// boundary was already recorded, so the provider may have the request: the
/// attempt is `outcome_unknown`, and the evidence says the provider never
/// acknowledged it.
#[tokio::test]
async fn a_provider_that_never_acknowledges_is_abandoned_at_the_deadline() {
    let fixture = fixture("silent-invoke");
    let journal = Journal::open(fixture.join("node.db")).await.unwrap();
    let op = operation(&journal, "silent-invoke").await;
    let deadline = Utc::now() + chrono::Duration::milliseconds(100);

    let outcome = tokio::time::timeout(
        Duration::from_secs(10),
        execute_attempt(
            &journal,
            &SilentInvoke(adapter(vec![ScriptStep::Complete { usage: None }])),
            &op,
            &request(Some(deadline)),
            &reservation(),
            &local(),
        ),
    )
    .await
    .expect("the attempt ended at its deadline, not at the outer bound");
    let Err(SupervisorError::OutcomeUnknown { attempt_id, .. }) = outcome else {
        panic!("expected an honest outcome_unknown, got {outcome:?}");
    };
    let evidence = journal
        .attempts_for_operation(&op)
        .await
        .unwrap()
        .into_iter()
        .find(|a| a.attempt_id == attempt_id)
        .and_then(|a| a.evidence)
        .unwrap_or_default();
    assert!(
        evidence.contains("before the provider acknowledged"),
        "the evidence says where the deadline passed: {evidence}"
    );
}
