//! The execution supervisor (`PHASE-0.4.1`): the journal-driven attempt flow around the
//! adapter contract (`ROADMAP.md` §11.1: "supervises adapters"; `§17.4`: persist before
//! advancing the boundary).
//!
//! [`execute_attempt`] is the boundary discipline made executable:
//!
//! 1. `prepared` — the attempt exists before anything touches the adapter.
//! 2. `record_dispatch` — the dispatch boundary record COMMITS before `invoke` runs
//!    (the `.3.1` rule), so a crash after a possible dispatch is recoverable.
//! 3. `invoke` — [`InvokeOutcome::FailedBeforeDispatch`] is a deterministic pre-boundary
//!    failure (`failed_before_dispatch`, terminal); [`InvokeOutcome::Accepted`] is the
//!    dispatch acknowledgement — DISTINCT from completion — and its provider request id
//!    is attached to the attempt as the proof handle.
//! 4. The attempt handle streams opaque chunks (passed through verbatim, never parsed)
//!    to a terminal event: `completed` (usage normalized and journaled as evidence) or
//!    `failed_known` (a definitive, proven failure).
//! 5. A stream that ends WITHOUT a terminal event is a lost response: the attempt is
//!    journaled `outcome_unknown`, and ONLY a proven status lookup can move it —
//!    [`StatusLookupOutcome::Unsupported`] leaves it `outcome_unknown` and the caller
//!    receives [`SupervisorError::OutcomeUnknown`], which is a FACT about the attempt,
//!    not a retry recommendation (retrying `outcome_unknown` requires duplicate-risk
//!    authorization, `§14.6` — a policy decision, never made here).
//!
//! Two bounds end an attempt from the supervisor's side (`SIGNOFF-REPAIR.4.4.6`).
//! The request's DEADLINE bounds the wait for the acknowledgement and for every
//! stream event; when it passes, the supervisor asks the adapter to cancel and
//! lands the attempt on step 5's honest `outcome_unknown`, naming the deadline,
//! because the provider may have run. [`MAX_OUTPUT_BYTES`] bounds the output
//! collected; past it, the supervisor cancels and lands a definitive
//! `failed_known` naming the bound, never a silently shortened result. Until that
//! leaf the deadline was documented as the caller's to enforce, and no caller did.

use std::time::Duration;

use chrono::Utc;
use reasonbraid_adapter::{
    Adapter, AttemptEvent, AttemptResult, InvokeOutcome, NormalizedUsage, RunRequest,
    StatusLookupOutcome,
};
use reasonbraid_core::{
    BudgetDimensions, BudgetError, ProviderAttemptId, ProviderAttemptState, ReservationReference,
};
use serde_json::json;
use tokio::sync::Mutex;

use crate::journal::{Journal, JournalError, ProvenStatus, ResultEvent};

/// The most output one attempt may produce, in bytes (`SIGNOFF-REPAIR.4.4.6`).
///
/// The result travels to the control plane as one JSON body, and the control
/// plane accepts bodies up to 2 MiB (the HTTP framework's default). A JSON
/// string can cost up to six bytes per byte of content (`\u0000` escapes), so
/// 256 KiB of content fits even at the worst escaping, with room for the rest of
/// the event. A larger result could never be delivered: the node would carry an
/// event the control plane refuses for as long as it runs.
pub const MAX_OUTPUT_BYTES: usize = 256 * 1024;

/// How long a cancellation may take before the supervisor stops waiting for its
/// answer. The attempt is already being ended; this only bounds the courtesy.
const CANCEL_GRACE: Duration = Duration::from_secs(5);

/// `fut`, or `None` if `deadline` passes first. No deadline waits for `fut`.
async fn within<F: std::future::Future>(
    deadline: Option<tokio::time::Instant>,
    fut: F,
) -> Option<F::Output> {
    match deadline {
        Some(at) => tokio::time::timeout_at(at, fut).await.ok(),
        None => Some(fut.await),
    }
}

/// Ask the adapter to cancel `operation_id`, waiting at most [`CANCEL_GRACE`],
/// and say what came of it, for the attempt's evidence.
async fn cancel_within_grace(adapter: &impl Adapter, operation_id: &str) -> String {
    match within(
        Some(tokio::time::Instant::now() + CANCEL_GRACE),
        adapter.cancel(operation_id),
    )
    .await
    {
        Some(outcome) => format!("{outcome:?}"),
        None => format!("no answer within {CANCEL_GRACE:?}"),
    }
}

/// The node's LOCAL budget ledger (§14.3 step 4: "the node verifies a signed/
/// authorized reservation AND local headroom before dispatch"). Development profile:
/// in-memory per-node state — the server-side ceiling is the durable counterpart.
pub struct LocalBudget {
    ceiling: BudgetDimensions,
    consumed: Mutex<BudgetDimensions>,
}

impl LocalBudget {
    pub fn new(ceiling: BudgetDimensions) -> Self {
        Self {
            ceiling,
            consumed: Mutex::new(BudgetDimensions::default()),
        }
    }

    /// Reserve against local headroom; fails closed when the ceiling does not cover
    /// (consumed + dims) — the second boundary of the WP5 acceptance.
    pub async fn try_reserve(&self, dims: &BudgetDimensions) -> Result<(), BudgetError> {
        let mut consumed = self.consumed.lock().await;
        // A sum past `u64` covers nothing (`SIGNOFF-REPAIR.4.5.2`).
        let next = consumed.add(dims).map_err(|e| BudgetError::Unavailable {
            detail: format!("local headroom cannot be counted: {e}"),
        })?;
        if !self.ceiling.covers(&next) {
            return Err(BudgetError::Unavailable {
                detail: "local headroom does not cover the reservation".to_string(),
            });
        }
        *consumed = next;
        Ok(())
    }

    /// Settle a completed attempt: the hold is replaced by ACTUAL usage (which may
    /// exceed it — the ledger records the fact; never clamped).
    pub async fn settle(&self, reserved: &BudgetDimensions, usage: &BudgetDimensions) {
        let mut consumed = self.consumed.lock().await;
        let released = consumed.subtract(reserved).unwrap_or_default();
        // Usage is what the provider reported, recorded in full. One too large to
        // count leaves the headroom spent BEYOND COUNTING, never wrapped small
        // (`SIGNOFF-REPAIR.4.5.2`). Every dimension at `u64::MAX` rather than at
        // the ceiling, so a later settlement's subtraction cannot reopen room the
        // uncountable usage took.
        *consumed = released.add(usage).unwrap_or(BudgetDimensions {
            calls: Some(u64::MAX),
            input_tokens: Some(u64::MAX),
            output_tokens: Some(u64::MAX),
            wall_clock_seconds: Some(u64::MAX),
        });
    }

    /// Release a hold that was never consumed (pre-dispatch refusals).
    pub async fn release(&self, reserved: &BudgetDimensions) {
        let mut consumed = self.consumed.lock().await;
        if let Ok(next) = consumed.subtract(reserved) {
            *consumed = next;
        }
    }

    /// The currently consumed dimensions (a test/inspection surface).
    pub async fn consumed(&self) -> BudgetDimensions {
        *self.consumed.lock().await
    }
}

/// What one supervised attempt produced.
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionReport {
    pub attempt_id: String,
    /// The attempt's terminal journal state. On [`SupervisorError::OutcomeUnknown`] the
    /// attempt is `outcome_unknown` in the journal — bounded and visible, not terminal.
    pub final_state: ProviderAttemptState,
    /// The streamed output chunks, verbatim (untrusted content, never parsed).
    pub chunks: Vec<String>,
    /// The normalized usage, when the adapter reported a receipt.
    pub usage: Option<NormalizedUsage>,
    /// The reservation this dispatch ran under (the WP6 correlation key).
    pub reservation_id: String,
    /// The result event journaled with the completion, when the caller asked for
    /// one ([`execute_attempt_emitting`]): pending in the journal from the same
    /// transaction that completed the attempt, and the caller's to deliver.
    pub result_event: Option<ResultEvent>,
    /// The wall-clock seconds the attempt took from its dispatch record to its
    /// end, rounded up (`SIGNOFF-REPAIR.4.4.6.1`): charged to the local ledger and
    /// reported to the control plane with the tokens. `None` for an attempt that
    /// never crossed the dispatch boundary, which spent nothing.
    pub wall_clock_seconds: Option<u64>,
}

/// The seconds since `dispatched_at`, rounded UP and never below one: an attempt
/// that crossed the dispatch boundary costs at least a second, and never less
/// than it took (`SIGNOFF-REPAIR.4.4.6.1`).
fn seconds_since(dispatched_at: tokio::time::Instant) -> u64 {
    let millis = u64::try_from(dispatched_at.elapsed().as_millis()).unwrap_or(u64::MAX);
    millis.div_ceil(1000).max(1)
}

/// Builds the outgoing result event from a completed attempt's report.
pub type ResultEventBuilder<'a> = &'a (dyn Fn(&ExecutionReport) -> ResultEvent + Send + Sync);

/// A typed supervision error.
#[derive(Debug)]
pub enum SupervisorError {
    Journal(JournalError),
    /// The attempt's result is indeterminate: the dispatch was acknowledged but no
    /// result arrived AND the adapter cannot prove one by status lookup. The journal
    /// holds `outcome_unknown`. This is a fact, not a retry recommendation: what to do
    /// with it is the caller's (and ultimately policy's) decision.
    OutcomeUnknown {
        attempt_id: String,
        detail: String,
    },
}

impl std::fmt::Display for SupervisorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupervisorError::Journal(e) => write!(f, "supervisor journal error: {e}"),
            SupervisorError::OutcomeUnknown { attempt_id, detail } => {
                write!(
                    f,
                    "provider attempt `{attempt_id}` is outcome_unknown: {detail} — \
                     it stays bounded and visible for adjudication"
                )
            }
        }
    }
}

impl std::error::Error for SupervisorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SupervisorError::Journal(e) => Some(e),
            _ => None,
        }
    }
}

impl From<JournalError> for SupervisorError {
    fn from(e: JournalError) -> Self {
        SupervisorError::Journal(e)
    }
}

/// Supervise one operation through a provider attempt: journal every boundary, invoke
/// the adapter behind it, and land the attempt on its honest terminal state.
pub async fn execute_attempt(
    journal: &Journal,
    adapter: &impl Adapter,
    operation_id: &str,
    request: &RunRequest,
    reservation: &ReservationReference,
    local: &LocalBudget,
) -> Result<ExecutionReport, SupervisorError> {
    supervise(
        journal,
        adapter,
        operation_id,
        request,
        reservation,
        local,
        None,
    )
    .await
}

/// [`execute_attempt`] for work whose result must reach the control plane
/// (`SIGNOFF-REPAIR.4.4.4.1`). When the attempt completes, by the runtime answer or
/// by a status-lookup proof, `result_event` builds the outgoing event from the
/// report and the journal writes it in the SAME transaction as the completion, so
/// no instant exists in which the attempt is complete and its result is not
/// pending. The event comes back in [`ExecutionReport::result_event`].
pub async fn execute_attempt_emitting(
    journal: &Journal,
    adapter: &impl Adapter,
    operation_id: &str,
    request: &RunRequest,
    reservation: &ReservationReference,
    local: &LocalBudget,
    result_event: ResultEventBuilder<'_>,
) -> Result<ExecutionReport, SupervisorError> {
    supervise(
        journal,
        adapter,
        operation_id,
        request,
        reservation,
        local,
        Some(result_event),
    )
    .await
}

/// Land a completed attempt: its terminal record and, when the caller asked for
/// one, its result event, in one journal transaction. A status-lookup proof lands
/// through the same `Complete` transition as a runtime answer (from
/// `outcome_unknown` instead of `dispatched`; the state machine checks which), so
/// one writer serves both paths.
async fn land_completed(
    journal: &Journal,
    report: &mut ExecutionReport,
    evidence: Option<&serde_json::Value>,
    result_event: Option<ResultEventBuilder<'_>>,
    now: chrono::DateTime<Utc>,
) -> Result<(), JournalError> {
    let attempt_id = report.attempt_id.clone();
    match result_event {
        Some(build) => {
            let event = build(report);
            journal
                .record_completed_with_event(&attempt_id, evidence, &event, now)
                .await?;
            report.result_event = Some(event);
        }
        None => journal.record_completed(&attempt_id, evidence, now).await?,
    }
    Ok(())
}

async fn supervise(
    journal: &Journal,
    adapter: &impl Adapter,
    operation_id: &str,
    request: &RunRequest,
    reservation: &ReservationReference,
    local: &LocalBudget,
    result_event: Option<ResultEventBuilder<'_>>,
) -> Result<ExecutionReport, SupervisorError> {
    let now = Utc::now();
    let attempt_id = ProviderAttemptId::new().to_string();

    // 0. THE budget gate: no dispatch without an applicable reservation (§14.3 step 4,
    // §14.6) — verified at BOTH boundaries: the reference itself, and the node's
    // local headroom. A refusal is journaled as `failed_before_dispatch` (audited)
    // and the adapter is never invoked.
    //
    // ⛔ `applicable_at(now)` rather than a window-blind check
    // (`SIGNOFF-REPAIR.11.24.1.1.2.2`): the issuing ledger stops holding the
    // reservation at `expires_at`, so a work item delivered after that instant
    // carries a proof of an allowance the ceiling has already re-lent. §14.4 —
    // *surface partial result and missing work instead of consuming an
    // unauthorized overrun* — is why the answer is this refusal and not a
    // delivery-time re-reservation.
    let attempt_id_ref = &attempt_id;
    let refused = |reason: String| async move {
        journal
            .record_failed_before_dispatch(attempt_id_ref, Some(&reason), now)
            .await?;
        Ok(ExecutionReport {
            attempt_id: attempt_id_ref.clone(),
            final_state: ProviderAttemptState::FailedBeforeDispatch,
            chunks: Vec::new(),
            usage: None,
            reservation_id: reservation.reservation_id.clone(),
            result_event: None,
            wall_clock_seconds: None,
        })
    };
    if let Err(e) = reservation.applicable_at(now) {
        journal
            .prepare_attempt(&attempt_id, operation_id, now)
            .await?;
        return refused(format!("no applicable reservation: {e}")).await;
    }
    if let Err(e) = local.try_reserve(&reservation.dimensions).await {
        journal
            .prepare_attempt(&attempt_id, operation_id, now)
            .await?;
        return refused(format!("local budget refused the dispatch: {e}")).await;
    }

    // 1 + 2. The attempt exists, and the dispatch boundary record is DURABLE before
    // the adapter is ever invoked (§17.4; the .3.1 boundary rule).
    journal
        .prepare_attempt(&attempt_id, operation_id, now)
        .await?;
    journal.record_dispatch(&attempt_id, None, now).await?;
    // The attempt's clock starts at its dispatch record (`SIGNOFF-REPAIR.4.4.6.1`).
    let dispatched_at = tokio::time::Instant::now();

    // The deadline, as an instant on the runtime's clock (a deadline already
    // past waits for nothing), and as the text the evidence names.
    let deadline = request.deadline.map(|at| {
        tokio::time::Instant::now() + (at - Utc::now()).to_std().unwrap_or(Duration::ZERO)
    });
    let deadline_label = request
        .deadline
        .map(|at| at.to_rfc3339())
        .unwrap_or_default();

    // 3. Invoke: a pre-boundary refusal, or an acknowledgement + streaming handle.
    // The boundary is already recorded, so a deadline passing HERE is step 5 too.
    let Some(invoked) = within(deadline, adapter.invoke(request, operation_id)).await else {
        let cancellation = cancel_within_grace(adapter, operation_id).await;
        return settle_unknown(
            UnknownOutcome {
                journal,
                adapter,
                operation_id,
                reservation,
                local,
                result_event,
                now,
                dispatched_at,
            },
            attempt_id,
            Vec::new(),
            &format!(
                "the deadline {deadline_label} passed before the provider acknowledged \
                 the dispatch; cancellation: {cancellation}"
            ),
        )
        .await;
    };
    match invoked {
        InvokeOutcome::FailedBeforeDispatch { reason, .. } => {
            journal
                .record_failed_before_dispatch(&attempt_id, Some(&reason), now)
                .await?;
            // Nothing was consumed: return the hold to the local pool.
            local.release(&reservation.dimensions).await;
            Ok(ExecutionReport {
                attempt_id,
                final_state: ProviderAttemptState::FailedBeforeDispatch,
                chunks: Vec::new(),
                usage: None,
                reservation_id: reservation.reservation_id.clone(),
                result_event: None,
                wall_clock_seconds: None,
            })
        }
        InvokeOutcome::Accepted(ack, mut handle) => {
            // The acknowledgement is distinct from completion: record the proof handle
            // it carries, then stream the attempt's events.
            if let Some(provider_request_id) = &ack.provider_request_id {
                journal
                    .attach_provider_request_id(&attempt_id, provider_request_id)
                    .await?;
            }

            let mut chunks = Vec::new();
            let mut collected = 0usize;
            let terminal = loop {
                let Some(next) = within(deadline, handle.next()).await else {
                    let cancellation = cancel_within_grace(adapter, operation_id).await;
                    break Terminal::Lost(format!(
                        "the deadline {deadline_label} passed with no result; cancellation: \
                         {cancellation}"
                    ));
                };
                match next {
                    // The stream ended with no terminal event: the response was
                    // lost after dispatch (or the attempt was cancelled).
                    None => break Terminal::Lost("response lost after dispatch".to_string()),
                    Some(AttemptEvent::ProviderRequestId { request_id }) => {
                        // Some providers only reveal their request handle AFTER
                        // dispatch (Codex's thread.started): attach it the same way
                        // an ack-carried id is attached.
                        journal
                            .attach_provider_request_id(&attempt_id, &request_id)
                            .await?;
                    }
                    Some(AttemptEvent::OutputChunk { chunk }) => {
                        collected = collected.saturating_add(chunk.len());
                        if collected > MAX_OUTPUT_BYTES {
                            let cancellation = cancel_within_grace(adapter, operation_id).await;
                            break Terminal::FailedKnown(format!(
                                "the output exceeded the {MAX_OUTPUT_BYTES}-byte bound; \
                                 cancellation: {cancellation}"
                            ));
                        }
                        chunks.push(chunk);
                    }
                    // A terminal event ENDS the attempt: never keep pulling the
                    // stream past a result.
                    Some(AttemptEvent::Completed { usage }) => break Terminal::Completed(usage),
                    Some(AttemptEvent::FailedKnown { reason }) => {
                        break Terminal::FailedKnown(reason)
                    }
                }
            };

            match terminal {
                Terminal::Completed(usage) => {
                    let normalized = usage.as_ref().map(|u| adapter.normalize_usage(u));
                    // Settle ACTUAL usage against the hold (overruns land in the
                    // local ledger as-is — recorded, never clamped).
                    let wall_clock_seconds = seconds_since(dispatched_at);
                    let actual = BudgetDimensions::attempt_usage(
                        normalized
                            .as_ref()
                            .and_then(|u| u.input_tokens.map(|v| v as u64)),
                        normalized
                            .as_ref()
                            .and_then(|u| u.output_tokens.map(|v| v as u64)),
                        Some(wall_clock_seconds),
                    );
                    local.settle(&reservation.dimensions, &actual).await;
                    let mut report = ExecutionReport {
                        attempt_id,
                        final_state: ProviderAttemptState::Completed,
                        chunks,
                        usage: normalized,
                        reservation_id: reservation.reservation_id.clone(),
                        result_event: None,
                        wall_clock_seconds: Some(wall_clock_seconds),
                    };
                    land_completed(journal, &mut report, usage.as_ref(), result_event, now).await?;
                    Ok(report)
                }
                Terminal::FailedKnown(reason) => {
                    journal
                        .record_failed_known(&attempt_id, Some(&json!({ "reason": reason })), now)
                        .await?;
                    let wall_clock_seconds = seconds_since(dispatched_at);
                    let actual =
                        BudgetDimensions::attempt_usage(None, None, Some(wall_clock_seconds));
                    local.settle(&reservation.dimensions, &actual).await;
                    Ok(ExecutionReport {
                        attempt_id,
                        final_state: ProviderAttemptState::FailedKnown,
                        chunks,
                        usage: None,
                        reservation_id: reservation.reservation_id.clone(),
                        result_event: None,
                        wall_clock_seconds: Some(wall_clock_seconds),
                    })
                }
                Terminal::Lost(reason) => {
                    settle_unknown(
                        UnknownOutcome {
                            journal,
                            adapter,
                            operation_id,
                            reservation,
                            local,
                            result_event,
                            now,
                            dispatched_at,
                        },
                        attempt_id,
                        chunks,
                        &reason,
                    )
                    .await
                }
            }
        }
    }
}

enum Terminal {
    Completed(Option<serde_json::Value>),
    FailedKnown(String),
    /// No result, and none can be claimed: the stream ended without one, or the
    /// deadline passed. The reason goes into the `outcome_unknown` evidence.
    Lost(String),
}

/// What [`settle_unknown`] needs of the attempt it lands.
struct UnknownOutcome<'a, A: Adapter> {
    journal: &'a Journal,
    adapter: &'a A,
    operation_id: &'a str,
    reservation: &'a ReservationReference,
    local: &'a LocalBudget,
    result_event: Option<ResultEventBuilder<'a>>,
    now: chrono::DateTime<Utc>,
    dispatched_at: tokio::time::Instant,
}

/// Step 5: the attempt has no result the supervisor can vouch for. Journal the
/// honest `outcome_unknown` with `reason`, then let ONLY a proof move it: a
/// provider status lookup that proves the result lands it; without one, the
/// caller receives [`SupervisorError::OutcomeUnknown`].
async fn settle_unknown<A: Adapter>(
    at: UnknownOutcome<'_, A>,
    attempt_id: String,
    chunks: Vec<String>,
    reason: &str,
) -> Result<ExecutionReport, SupervisorError> {
    let UnknownOutcome {
        journal,
        adapter,
        operation_id,
        reservation,
        local,
        result_event,
        now,
        dispatched_at,
    } = at;
    journal
        .record_outcome_unknown(&attempt_id, Some(reason), now)
        .await?;
    match adapter.query_status(operation_id).await {
        StatusLookupOutcome::Supported(AttemptResult::Completed { usage }) => {
            let normalized = usage.as_ref().map(|u| adapter.normalize_usage(u));
            let wall_clock_seconds = seconds_since(dispatched_at);
            let mut report = ExecutionReport {
                attempt_id,
                final_state: ProviderAttemptState::Completed,
                chunks,
                usage: normalized,
                reservation_id: reservation.reservation_id.clone(),
                result_event: None,
                wall_clock_seconds: Some(wall_clock_seconds),
            };
            land_completed(journal, &mut report, usage.as_ref(), result_event, now).await?;
            let actual = BudgetDimensions::attempt_usage(
                report
                    .usage
                    .as_ref()
                    .and_then(|u| u.input_tokens.map(|v| v as u64)),
                report
                    .usage
                    .as_ref()
                    .and_then(|u| u.output_tokens.map(|v| v as u64)),
                Some(wall_clock_seconds),
            );
            local.settle(&reservation.dimensions, &actual).await;
            Ok(report)
        }
        StatusLookupOutcome::Supported(AttemptResult::FailedKnown { reason }) => {
            journal
                .prove_result(
                    &attempt_id,
                    ProvenStatus::FailedKnown,
                    None,
                    Some(&json!({ "reason": reason })),
                    now,
                )
                .await?;
            let wall_clock_seconds = seconds_since(dispatched_at);
            let actual = BudgetDimensions::attempt_usage(None, None, Some(wall_clock_seconds));
            local.settle(&reservation.dimensions, &actual).await;
            Ok(ExecutionReport {
                attempt_id,
                final_state: ProviderAttemptState::FailedKnown,
                chunks,
                usage: None,
                reservation_id: reservation.reservation_id.clone(),
                result_event: None,
                wall_clock_seconds: Some(wall_clock_seconds),
            })
        }
        StatusLookupOutcome::Unsupported => {
            // The hold stays in place: an ambiguous attempt MAY have consumed
            // (§14.6 — release only amounts not potentially consumed).
            // Settlement/adjudication owns the release.
            Err(SupervisorError::OutcomeUnknown {
                attempt_id,
                detail: "status lookup unsupported by the adapter".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod local_budget_tests {
    use super::*;

    fn dims(calls: u64, tokens: u64) -> BudgetDimensions {
        BudgetDimensions {
            calls: Some(calls),
            input_tokens: Some(tokens),
            output_tokens: Some(tokens),
            wall_clock_seconds: Some(calls),
        }
    }

    /// `SIGNOFF-REPAIR.4.5.2` — a settlement whose reported usage cannot be
    /// counted leaves the local headroom spent, and a LATER settlement that
    /// frees its own unused hold does not reopen it.
    #[tokio::test]
    async fn an_uncountable_settlement_spends_the_headroom_for_good() {
        let local = LocalBudget::new(dims(100, 100_000));
        let hold = dims(1, 1_000);
        local.try_reserve(&hold).await.expect("first hold");
        local.try_reserve(&hold).await.expect("second hold");
        // The first attempt reports more tokens than can be summed with the
        // second's hold.
        local.settle(&hold, &dims(1, u64::MAX)).await;
        // The second used no call and almost no tokens, freeing nearly all of
        // its hold: room for a small request, IF that freed room were real.
        local.settle(&hold, &dims(0, 1)).await;
        assert!(
            local.try_reserve(&dims(1, 1)).await.is_err(),
            "the headroom stays spent"
        );
    }

    /// A reservation whose sum with the consumed amount cannot be counted is
    /// refused, not panicked on.
    #[tokio::test]
    async fn an_uncountable_reservation_is_refused() {
        let local = LocalBudget::new(dims(100, u64::MAX));
        local
            .try_reserve(&dims(1, u64::MAX))
            .await
            .expect("the whole ceiling");
        match local.try_reserve(&dims(1, 1)).await {
            Err(BudgetError::Unavailable { detail }) => assert!(
                detail.contains("cannot be counted"),
                "the refusal names the overflow: {detail}"
            ),
            other => panic!("expected the overflow refusal, got {other:?}"),
        }
    }
}
