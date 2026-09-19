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
//! Cancellation is the caller's tool: [`execute_attempt`] drives the stream to a
//! terminal; a caller that observes a hang calls [`Adapter::cancel`] (the fake's
//! `HangForever` resolves deterministically on cancellation) and the stream then ends
//! — step 5 applies. Deadline enforcement is likewise caller-side (the request carries
//! the deadline; WP5 types the budgets that police it).

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

use crate::journal::{Journal, JournalError, ProvenStatus};

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
        let next = consumed.add(dims);
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
        *consumed = released.add(usage);
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
}

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

    // 3. Invoke: a pre-boundary refusal, or an acknowledgement + streaming handle.
    match adapter.invoke(request, operation_id).await {
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
            let mut terminal = None;
            while let Some(event) = handle.next().await {
                match event {
                    AttemptEvent::ProviderRequestId { request_id } => {
                        // Some providers only reveal their request handle AFTER
                        // dispatch (Codex's thread.started): attach it the same way
                        // an ack-carried id is attached.
                        journal
                            .attach_provider_request_id(&attempt_id, &request_id)
                            .await?;
                    }
                    AttemptEvent::OutputChunk { chunk } => chunks.push(chunk),
                    AttemptEvent::Completed { usage } => {
                        // A terminal event ENDS the attempt: never keep pulling the
                        // stream past a result.
                        terminal = Some(Terminal::Completed(usage));
                        break;
                    }
                    AttemptEvent::FailedKnown { reason } => {
                        terminal = Some(Terminal::FailedKnown(reason));
                        break;
                    }
                }
            }

            match terminal {
                Some(Terminal::Completed(usage)) => {
                    let normalized = usage.as_ref().map(|u| adapter.normalize_usage(u));
                    // Settle ACTUAL usage against the hold (overruns land in the
                    // local ledger as-is — recorded, never clamped).
                    let actual = BudgetDimensions::attempt_usage(
                        normalized
                            .as_ref()
                            .and_then(|u| u.input_tokens.map(|v| v as u64)),
                        normalized
                            .as_ref()
                            .and_then(|u| u.output_tokens.map(|v| v as u64)),
                    );
                    local.settle(&reservation.dimensions, &actual).await;
                    journal
                        .record_completed(&attempt_id, usage.as_ref(), now)
                        .await?;
                    Ok(ExecutionReport {
                        attempt_id,
                        final_state: ProviderAttemptState::Completed,
                        chunks,
                        usage: normalized,
                        reservation_id: reservation.reservation_id.clone(),
                    })
                }
                Some(Terminal::FailedKnown(reason)) => {
                    journal
                        .record_failed_known(&attempt_id, Some(&json!({ "reason": reason })), now)
                        .await?;
                    let actual = BudgetDimensions::attempt_usage(None, None);
                    local.settle(&reservation.dimensions, &actual).await;
                    Ok(ExecutionReport {
                        attempt_id,
                        final_state: ProviderAttemptState::FailedKnown,
                        chunks,
                        usage: None,
                        reservation_id: reservation.reservation_id.clone(),
                    })
                }
                None => {
                    // The stream ended with no terminal event: the response was lost
                    // after dispatch (or the attempt was cancelled). Journal the honest
                    // fact, then let ONLY a proof move it.
                    journal
                        .record_outcome_unknown(
                            &attempt_id,
                            Some("response lost after dispatch"),
                            now,
                        )
                        .await?;
                    match adapter.query_status(operation_id).await {
                        StatusLookupOutcome::Supported(AttemptResult::Completed { usage }) => {
                            let normalized = usage.as_ref().map(|u| adapter.normalize_usage(u));
                            journal
                                .prove_result(
                                    &attempt_id,
                                    ProvenStatus::Completed,
                                    None,
                                    usage.as_ref(),
                                    now,
                                )
                                .await?;
                            let actual = BudgetDimensions::attempt_usage(
                                normalized
                                    .as_ref()
                                    .and_then(|u| u.input_tokens.map(|v| v as u64)),
                                normalized
                                    .as_ref()
                                    .and_then(|u| u.output_tokens.map(|v| v as u64)),
                            );
                            local.settle(&reservation.dimensions, &actual).await;
                            Ok(ExecutionReport {
                                attempt_id,
                                final_state: ProviderAttemptState::Completed,
                                chunks,
                                usage: normalized,
                                reservation_id: reservation.reservation_id.clone(),
                            })
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
                            let actual = BudgetDimensions::attempt_usage(None, None);
                            local.settle(&reservation.dimensions, &actual).await;
                            Ok(ExecutionReport {
                                attempt_id,
                                final_state: ProviderAttemptState::FailedKnown,
                                chunks,
                                usage: None,
                                reservation_id: reservation.reservation_id.clone(),
                            })
                        }
                        StatusLookupOutcome::Unsupported => {
                            // The hold stays in place: an ambiguous attempt MAY have
                            // consumed (§14.6 — release only amounts not potentially
                            // consumed). Settlement/adjudication owns the release.
                            Err(SupervisorError::OutcomeUnknown {
                                attempt_id: attempt_id.clone(),
                                detail: "status lookup unsupported by the adapter".to_string(),
                            })
                        }
                    }
                }
            }
        }
    }
}

enum Terminal {
    Completed(Option<serde_json::Value>),
    FailedKnown(String),
}
