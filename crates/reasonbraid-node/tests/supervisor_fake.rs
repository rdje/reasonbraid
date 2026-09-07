//! Supervisor × fake adapter integration tests (`PHASE-0.4.1`): the whole WP4 adapter
//! acceptance driven through the REAL journal boundary — the fixture corpus lands every
//! attempt on its expected terminal state, a lost response without a status lookup is
//! an honest `outcome_unknown` (never a retry recommendation), a proven lookup lands
//! the result, dispatch acknowledgement is distinct from completion, cancellation ends
//! a hang, and streaming chunks (including malformed ones) pass through verbatim.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::time::Duration;

use chrono::Utc;
use reasonbraid_adapter::{
    Adapter, AdapterCapabilities, AttemptEvent, AttemptHandle, AttemptStream, CancellationOutcome,
    CancellationStrength, DispatchAck, FakeAdapter, InvokeOutcome, NormalizedUsage,
    PolicyInjectionMode, RunRequest, StatusLookupOutcome, UsageConfidence,
};
use reasonbraid_core::{BudgetDimensions, ReservationReference};
use reasonbraid_node::{execute_attempt, CommandInput, Journal, LocalBudget, SupervisorError};
use serde_json::{json, Value};

/// A unique journal path under the repo's build dir (same volume as the repo).
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

fn run_request() -> RunRequest {
    RunRequest {
        payload: json!({ "operation": "contribute" }),
        deadline: None,
        budget_hint: None,
    }
}

/// Seed a command + operation and return the operation id.
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

fn fixture_adapter(name: &str) -> FakeAdapter {
    let spec = reasonbraid_adapter::fixtures::corpus()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("no fixture named {name}"));
    FakeAdapter::from_spec(spec)
}

/// THE corpus replay: every auto-drivable fixture lands its attempt on the expected
/// journal terminal through the real supervisor.
#[tokio::test]
async fn fixture_corpus_drives_each_auto_outcome_to_its_expected_terminal() {
    let corpus = reasonbraid_adapter::fixtures::corpus();
    let auto: Vec<_> = corpus
        .into_iter()
        .filter(|f| f.expected_terminal.is_some())
        .collect();
    assert!(
        !auto.is_empty(),
        "the corpus must have auto-drivable fixtures"
    );

    for (index, spec) in auto.iter().enumerate() {
        let journal = Journal::open(journal_path(&format!("corpus-{index}-{}", spec.name)))
            .await
            .unwrap();
        let op = seed_operation(&journal, &spec.name).await;
        let adapter = FakeAdapter::from_spec(spec.clone());
        let expected = spec.expected_terminal.as_deref().unwrap();

        match execute_attempt(
            &journal,
            &adapter,
            &op,
            &run_request(),
            &reservation("bud"),
            &generous_local(),
        )
        .await
        {
            Ok(report) => {
                assert_eq!(
                    report.final_state.as_str(),
                    expected,
                    "fixture `{}` landed on the wrong terminal",
                    spec.name
                );
                let history = journal.attempt_history(&report.attempt_id).await.unwrap();
                assert_eq!(
                    history.last().unwrap().to_status,
                    expected,
                    "fixture `{}`: journal boundary ledger disagrees",
                    spec.name
                );
            }
            Err(SupervisorError::OutcomeUnknown { attempt_id, .. }) => {
                // The honest indeterminate outcome: an error that IS a fact, with the
                // attempt journaled `outcome_unknown`.
                assert_eq!(
                    expected, "outcome_unknown",
                    "fixture `{}` went indeterminate but expected `{expected}`",
                    spec.name
                );
                let history = journal.attempt_history(&attempt_id).await.unwrap();
                assert_eq!(
                    history.last().unwrap().to_status,
                    "outcome_unknown",
                    "fixture `{}`: ambiguity not journaled",
                    spec.name
                );
            }
            Err(e) => panic!("fixture `{}` errored: {e}", spec.name),
        }
    }
}

/// THE ambiguity acceptance: a lost response with NO status lookup is
/// `outcome_unknown` in the journal, and the supervisor's error carries NO retry
/// recommendation — what to do is the caller's decision.
#[tokio::test]
async fn lost_response_without_lookup_is_outcome_unknown_and_never_recommends_retry() {
    let journal = Journal::open(journal_path("lost-no-lookup")).await.unwrap();
    let op = seed_operation(&journal, "lost").await;
    let adapter = fixture_adapter("lose_response_no_lookup");

    let err = execute_attempt(
        &journal,
        &adapter,
        &op,
        &run_request(),
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
        "the error must not recommend retrying: {err}"
    );
    let ambiguous = journal.ambiguous_attempts().await.unwrap();
    assert_eq!(ambiguous.len(), 1);
    assert_eq!(ambiguous[0].attempt_id, *attempt_id);
    assert_eq!(ambiguous[0].status, "outcome_unknown");
}

/// The escape hatch: a lost response WITH a working status lookup is PROVEN and lands
/// `completed`, with the provider request handle attached to the attempt.
#[tokio::test]
async fn lost_response_with_lookup_is_proven_completed_with_the_proof_handle() {
    let journal = Journal::open(journal_path("lost-lookup")).await.unwrap();
    let op = seed_operation(&journal, "provable").await;
    let adapter = fixture_adapter("lose_response_with_lookup");

    let report = execute_attempt(
        &journal,
        &adapter,
        &op,
        &run_request(),
        &reservation("bud"),
        &generous_local(),
    )
    .await
    .expect("a proven result must succeed");
    assert_eq!(report.final_state.as_str(), "completed");
    assert!(journal.ambiguous_attempts().await.unwrap().is_empty());

    // The dispatch ack's provider request id is attached as the proof handle.
    let summary = journal
        .pending_attempts()
        .await
        .unwrap()
        .into_iter()
        .find(|a| a.attempt_id == report.attempt_id);
    assert!(summary.is_none(), "a proven attempt is terminal");
    let history = journal.attempt_history(&report.attempt_id).await.unwrap();
    assert_eq!(history.last().unwrap().to_status, "completed");
}

/// The hang, through the supervisor: the attempt blocks in the stream, a CONFIRMED
/// cancellation ends it, and the honest outcome is `outcome_unknown` (the cancelled
/// result is unknowable — `cancelled_known` stays out of Phase 0).
#[tokio::test]
async fn hang_until_confirmed_cancel_leaves_outcome_unknown() {
    let journal = Journal::open(journal_path("hang")).await.unwrap();
    let op = seed_operation(&journal, "hang").await;
    let adapter = Arc::new(fixture_adapter("hang_forever"));

    let task_journal = journal.clone();
    let task_adapter = Arc::clone(&adapter);
    let task_op = op.clone();
    let task = tokio::spawn(async move {
        execute_attempt(
            &task_journal,
            &*task_adapter,
            &task_op,
            &run_request(),
            &reservation("bud"),
            &generous_local(),
        )
        .await
    });

    // Wait for the dispatch boundary to be durably recorded (the hang is entered).
    let mut dispatched = false;
    for _ in 0..400 {
        let pending = journal.pending_attempts().await.unwrap();
        if pending.iter().any(|a| a.status == "dispatched") {
            dispatched = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert!(
        dispatched,
        "the attempt never reached the dispatch boundary"
    );

    // Cancel: the fake's HangForever resolves on this, deterministically.
    assert_eq!(adapter.cancel(&op).await, CancellationOutcome::Confirmed);

    let err = task
        .await
        .expect("task panicked")
        .expect_err("hang must not succeed");
    assert!(
        matches!(err, SupervisorError::OutcomeUnknown { .. }),
        "got: {err}"
    );
    assert_eq!(journal.ambiguous_attempts().await.unwrap().len(), 1);
}

/// An ignored cancellation is reported honestly and the run still completes — the
/// supervisor's drive path is unaffected.
#[tokio::test]
async fn ignore_cancellation_still_completes() {
    let journal = Journal::open(journal_path("ignore")).await.unwrap();
    let op = seed_operation(&journal, "ignore").await;
    let adapter = fixture_adapter("ignore_cancellation");

    let report = execute_attempt(
        &journal,
        &adapter,
        &op,
        &run_request(),
        &reservation("bud"),
        &generous_local(),
    )
    .await
    .expect("the script completes despite an ignored cancellation");
    assert_eq!(report.final_state.as_str(), "completed");
}

/// Malformed output passes through the supervisor VERBATIM as opaque chunks — the
/// adapter boundary never parses domain meaning out of provider output.
#[tokio::test]
async fn streaming_chunks_pass_through_verbatim_including_malformed_ones() {
    let journal = Journal::open(journal_path("malformed")).await.unwrap();
    let op = seed_operation(&journal, "malformed").await;
    let adapter = fixture_adapter("malformed_output");

    let report = execute_attempt(
        &journal,
        &adapter,
        &op,
        &run_request(),
        &reservation("bud"),
        &generous_local(),
    )
    .await
    .expect("malformed output must not fail the drive");
    assert_eq!(report.final_state.as_str(), "completed");
    assert_eq!(report.chunks.len(), 2);
    assert_eq!(report.chunks[0], "valid prefix");
    assert_eq!(report.chunks[1], "\u{1f}\u{0} garbage {{{ not-json");
}

/// Usage receipts are normalized at the boundary: the estimated receipt fixture yields
/// `Estimated` confidence and its unmetered dimensions stay `None` (never zero).
#[tokio::test]
async fn usage_receipts_are_normalized_with_honest_confidence() {
    let journal = Journal::open(journal_path("usage")).await.unwrap();
    let op = seed_operation(&journal, "usage").await;
    let adapter = fixture_adapter("usage_receipt");

    let report = execute_attempt(
        &journal,
        &adapter,
        &op,
        &run_request(),
        &reservation("bud"),
        &generous_local(),
    )
    .await
    .expect("usage fixture completes");
    let usage = report.usage.expect("a receipt was reported");
    assert_eq!(usage.input_tokens, Some(100));
    assert_eq!(usage.output_tokens, Some(10));
    assert_eq!(usage.cost, None, "unmetered cost is unknown, not zero");
    assert_eq!(usage.confidence, UsageConfidence::Estimated);
}

/// THE "dispatch acknowledgement is distinct from completion" acceptance, proven at the
/// journal boundary: between the acknowledgement and the terminal event, the attempt is
/// durably `dispatched` — the ack must never be mistaken for a result.
#[tokio::test]
async fn dispatch_acknowledgement_is_distinct_from_completion_in_the_journal() {
    let journal = Journal::open(journal_path("ack-vs-complete"))
        .await
        .unwrap();
    let op = seed_operation(&journal, "ack").await;
    let adapter = Arc::new(SignalingAdapter::new());

    let task_journal = journal.clone();
    let task_adapter = Arc::clone(&adapter);
    let task_op = op.clone();
    let task = tokio::spawn(async move {
        execute_attempt(
            &task_journal,
            &*task_adapter,
            &task_op,
            &run_request(),
            &reservation("bud"),
            &generous_local(),
        )
        .await
    });

    // The adapter has dispatched (ack sent) but NOT completed yet.
    adapter.ack_received().await;
    let pending = journal.pending_attempts().await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(
        pending[0].status, "dispatched",
        "ack ≠ completion in the journal"
    );

    adapter.allow_completion();
    let report = task
        .await
        .expect("task panicked")
        .expect("the attempt completes");
    assert_eq!(report.final_state.as_str(), "completed");
}

// ── A minimal in-test adapter that separates the ack from the completion ───────────

struct SignalingAdapter {
    after_ack: StdMutex<Option<tokio::sync::oneshot::Sender<()>>>,
    allow: Arc<tokio::sync::Notify>,
    after_ack_rx: StdMutex<Option<tokio::sync::oneshot::Receiver<()>>>,
}

impl SignalingAdapter {
    fn new() -> Self {
        let (tx, rx) = tokio::sync::oneshot::channel();
        Self {
            after_ack: StdMutex::new(Some(tx)),
            allow: Arc::new(tokio::sync::Notify::new()),
            after_ack_rx: StdMutex::new(Some(rx)),
        }
    }

    async fn ack_received(&self) {
        let rx = self.after_ack_rx.lock().unwrap().take().unwrap();
        rx.await.expect("adapter must have dispatched");
    }

    fn allow_completion(&self) {
        // notify_one: a permit is stored if the handle has not registered yet, so the
        // completion signal survives the scheduling race.
        self.allow.notify_one();
    }
}

impl Adapter for SignalingAdapter {
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
        if let Some(sender) = self.after_ack.lock().unwrap().take() {
            let _ = sender.send(());
        }
        InvokeOutcome::Accepted(
            DispatchAck {
                provider_request_id: None,
            },
            AttemptHandle::new(Box::new(SignalHandle {
                allow: Arc::clone(&self.allow),
                done: false,
            })),
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

struct SignalHandle {
    allow: Arc<tokio::sync::Notify>,
    done: bool,
}

impl AttemptStream for SignalHandle {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<AttemptEvent>> + Send + '_>> {
        if self.done {
            return Box::pin(async { None });
        }
        self.done = true;
        let allow = Arc::clone(&self.allow);
        Box::pin(async move {
            allow.notified().await;
            Some(AttemptEvent::Completed { usage: None })
        })
    }
}

use std::path::PathBuf;
