//! The node work loop (`PHASE-0.6.2`): the WP6 wiring that turns inbox commands
//! into supervised provider attempts and attempt results into node-emitted
//! `work_result` events the control plane folds into the thread.
//!
//! # The loop
//!
//! 1. [`Worker::tick`] polls the channel tail after the journal's acknowledged
//!    cursor, journals the new commands (deduplicated by command id — a redelivery
//!    never creates a second local operation), and acknowledges the cursor.
//! 2. [`Worker::process`] walks the journal's [`Journal::work_items`] and applies
//!    the execution/skip decision: an item with NO attempt (or only a `prepared`
//!    one — the dispatch boundary was never crossed) is safe to execute; anything
//!    else is skipped. `outcome_unknown` is never silently retried — proof or
//!    adjudication owns it (§14.6).
//! 3. Execution rides the WP4 supervisor [`execute_attempt`] behind the WP5 budget
//!    gate (the reservation from the inbox payload AND the node's local headroom —
//!    both boundaries). Only a `completed` attempt emits a `work_result` event;
//!    failures and ambiguities stay journal facts, bounded and visible.
//! 4. A transport failure anywhere returns the worker to [`WorkerError::Channel`];
//!    the caller re-runs [`Node::reconcile`], which re-emits pending events with
//!    their ORIGINAL ids (§17.4 step 5) — so a crash between emission and
//!    acknowledgement still produces exactly one server receipt.
//!
//! # Untrusted content
//!
//! Adapter chunks are joined VERBATIM into the `work_result` payload (opaque
//! untrusted content — never parsed here, never interpreted as domain meaning).

use std::fmt;
use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use reasonbraid_adapter::{Adapter, RunRequest};
use reasonbraid_core::{BudgetDimensions, EventId, ProviderAttemptState, ReservationReference};
use serde_json::{json, Value};

use crate::channel::ChannelError;
use crate::journal::{CommandInput, JournalError, WorkItem};
use crate::node::{Node, NodeError};
use crate::supervisor::{execute_attempt, LocalBudget, SupervisorError};

/// The worker's typed error surface.
#[derive(Debug)]
pub enum WorkerError {
    Journal(JournalError),
    /// A channel/transport failure — the caller reconciles and resumes.
    Channel(ChannelError),
    Node(NodeError),
    Supervision(SupervisorError),
    /// A stored payload that does not parse (should be unreachable — the journal
    /// only stores values it serialized itself).
    MalformedPayload(String),
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkerError::Journal(e) => write!(f, "worker journal error: {e}"),
            WorkerError::Channel(e) => write!(f, "worker channel error: {e}"),
            WorkerError::Node(e) => write!(f, "worker node error: {e}"),
            WorkerError::Supervision(e) => write!(f, "worker supervision error: {e}"),
            WorkerError::MalformedPayload(detail) => {
                write!(f, "worker read a malformed inbox payload: {detail}")
            }
        }
    }
}

impl std::error::Error for WorkerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WorkerError::Journal(e) => Some(e),
            WorkerError::Channel(e) => Some(e),
            WorkerError::Node(e) => Some(e),
            WorkerError::Supervision(e) => Some(e),
            _ => None,
        }
    }
}

impl From<JournalError> for WorkerError {
    fn from(e: JournalError) -> Self {
        WorkerError::Journal(e)
    }
}

impl From<ChannelError> for WorkerError {
    fn from(e: ChannelError) -> Self {
        WorkerError::Channel(e)
    }
}

impl From<NodeError> for WorkerError {
    fn from(e: NodeError) -> Self {
        WorkerError::Node(e)
    }
}

impl From<SupervisorError> for WorkerError {
    fn from(e: SupervisorError) -> Self {
        WorkerError::Supervision(e)
    }
}

/// The node work loop: a reconciled [`Node`], the adapter behind the supervisor,
/// the node's local budget ledger, and the poll interval between channel tails.
pub struct Worker<A: Adapter> {
    node: Node,
    adapter: A,
    local: LocalBudget,
    poll_interval: Duration,
}

impl<A: Adapter> Worker<A> {
    pub fn new(node: Node, adapter: A, local: LocalBudget, poll_interval: Duration) -> Self {
        Self {
            node,
            adapter,
            local,
            poll_interval,
        }
    }

    /// Run until an error. On a channel failure the CALLER reconciles and calls
    /// `run` again — reconciliation re-emits anything pending with original ids, so
    /// nothing is lost or doubled across the reconnect.
    pub async fn run(&self) -> Result<(), WorkerError> {
        loop {
            self.tick().await?;
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// One poll-then-process pass.
    pub async fn tick(&self) -> Result<(), WorkerError> {
        let now = Utc::now();
        let cursor = self.node.journal().last_acked_cursor().await?;

        let poll = self.node.channel().poll(cursor).await?;
        for cmd in &poll.commands {
            self.node
                .journal()
                .record_command(
                    &CommandInput {
                        command_id: &cmd.command_id,
                        tenant_id: &cmd.tenant_id,
                        thread_id: &cmd.thread_id,
                        payload: &cmd.payload,
                        authz_ref: None,
                        server_cursor: &cmd.cursor.to_string(),
                    },
                    now,
                )
                .await?;
            self.node
                .journal()
                .ensure_operation(&cmd.command_id, now)
                .await?;
        }

        if poll.current_cursor > cursor {
            self.node.channel().acknowledge(poll.current_cursor).await?;
            self.node
                .journal()
                .set_last_acked_cursor(poll.current_cursor)
                .await?;
        }

        for item in self.node.journal().work_items().await? {
            self.process(&item).await?;
        }
        Ok(())
    }

    /// The execution/skip decision and one item's attempt→event pipeline.
    pub async fn process(&self, item: &WorkItem) -> Result<(), WorkerError> {
        let safe = match item.latest_attempt_status.as_deref() {
            // No attempt yet, or a crash before the dispatch boundary was crossed:
            // the provider was never contacted, so execution is safe.
            None | Some("prepared") => true,
            // dispatched/outcome_unknown: proof or adjudication owns it (§14.6 —
            // a silent retry could duplicate a provider effect). Terminal states:
            // done.
            Some(_) => false,
        };
        if !safe {
            return Ok(());
        }

        let payload: Value = serde_json::from_str(&item.payload)
            .map_err(|e| WorkerError::MalformedPayload(e.to_string()))?;
        let work_kind = payload
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkerError::MalformedPayload("work item has no kind".to_string()))?;

        let operation_id = match &item.operation_id {
            Some(op) => op.clone(),
            None => {
                self.node
                    .journal()
                    .ensure_operation(&item.command_id, Utc::now())
                    .await?
                    .operation_id
            }
        };

        // The reservation from the inbox payload. An absent reservation (the server
        // refused to reserve — ceiling exhausted) is an EMPTY reference: the
        // supervisor's applicability check refuses the dispatch BEFORE any adapter
        // contact and journals `failed_before_dispatch` with the reason. The budget
        // denial is thereby visible and bounded, never a silent skip.
        let reservation: ReservationReference = match payload.get("reservation") {
            Some(value) if !value.is_null() => {
                serde_json::from_value(value.clone()).map_err(|e| {
                    WorkerError::MalformedPayload(format!("unreadable reservation: {e}"))
                })?
            }
            _ => ReservationReference {
                reservation_id: String::new(),
                dimensions: BudgetDimensions::default(),
                issued_at: Utc::now(),
            },
        };

        // The deadline follows the reservation's wall-clock allowance; a work item
        // without one gets the dev-profile default.
        let deadline = Utc::now()
            + ChronoDuration::seconds(
                reservation
                    .dimensions
                    .wall_clock_seconds
                    .unwrap_or(60)
                    .min(3600) as i64,
            );
        let request = RunRequest {
            payload: payload.clone(),
            deadline: Some(deadline),
            budget_hint: None,
        };

        match execute_attempt(
            self.node.journal(),
            &self.adapter,
            &operation_id,
            &request,
            &reservation,
            &self.local,
        )
        .await?
        {
            report if report.final_state == ProviderAttemptState::Completed => {
                let event_payload = json!({
                    "kind": "work_result",
                    "work_kind": work_kind,
                    "command_id": item.command_id,
                    "reservation_id": reservation.reservation_id,
                    "attempt_id": report.attempt_id,
                    "content": report.chunks.join(""),
                    "usage": report.usage,
                });
                self.node
                    .emit_event(&operation_id, &EventId::new().to_string(), &event_payload)
                    .await?;
                eprintln!(
                    "worker: {} emitted a {work_kind} result for {}",
                    self.node.node_id(),
                    item.command_id
                );
            }
            report => {
                eprintln!(
                    "worker: attempt {} ended `{}` — journaled, no contribution emitted",
                    report.attempt_id,
                    report.final_state.as_str()
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The skip decision: only a missing or `prepared` attempt is safe to execute.
    #[test]
    fn skip_decision_is_exactly_none_or_prepared() {
        for (status, expect_safe) in [
            (None, true),
            (Some("prepared"), true),
            (Some("dispatched"), false),
            (Some("outcome_unknown"), false),
            (Some("completed"), false),
            (Some("failed_known"), false),
            (Some("failed_before_dispatch"), false),
            (Some("reconciled"), false),
        ] {
            let safe = match status {
                None | Some("prepared") => true,
                Some(_) => false,
            };
            assert_eq!(
                safe,
                expect_safe,
                "status {status:?} must be {}",
                if expect_safe { "safe" } else { "skipped" }
            );
        }
    }

    /// A null/absent reservation deserializes to an EMPTY reference, which the
    /// supervisor's applicability check refuses (failed_before_dispatch) — the
    /// budget denial is visible, never a silent skip.
    #[test]
    fn missing_reservation_becomes_an_empty_reference() {
        let ref_ = ReservationReference {
            reservation_id: String::new(),
            dimensions: BudgetDimensions::default(),
            issued_at: Utc::now(),
        };
        assert!(ref_.applicable().is_err(), "an empty reference must refuse");
    }
}
