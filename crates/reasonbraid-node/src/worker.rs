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
use reasonbraid_core::{BudgetDimensions, EventId, ProviderAttemptId, ReservationReference};
use serde_json::{json, Value};

use crate::channel::ChannelError;
use crate::journal::{CommandInput, JournalError, ResultEvent, WorkItem};
use crate::node::{EventDelivery, Node, NodeError};
use crate::supervisor::{execute_attempt_emitting, ExecutionReport, LocalBudget, SupervisorError};

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

impl WorkerError {
    /// Whether the node's remedy is to reconcile and resume the work loop, rather
    /// than to stop. A channel failure is one, whether the worker's own call
    /// failed or the node's send of a result did (`SIGNOFF-REPAIR.4.4.5.2`): the
    /// reconcile re-handshakes and re-emits anything pending under its original
    /// id. A dispatch refused on a node that is not schedulable is the other
    /// (`SIGNOFF-REPAIR.4.4.4.2.2`): the reconcile is what makes it schedulable.
    /// `rb-node`'s loop asks this, so the decision is testable here rather than
    /// buried in a binary.
    pub fn calls_for_reconcile(&self) -> bool {
        matches!(
            self,
            WorkerError::Channel(_)
                | WorkerError::Node(NodeError::Channel(_))
                | WorkerError::Node(NodeError::NotSchedulable)
        )
    }
}

/// How long the node waits before its next reconcile attempt, after
/// `consecutive_failures` failed ones in a row (`SIGNOFF-REPAIR.4.4.5.3`): one
/// second, doubling, capped at a minute. The count restarts with each recovery,
/// so a node that reconciles is back to one second the next time it loses the
/// channel.
///
/// The retry used to be a fixed second for ever, so a control plane down for an
/// hour met 3,600 handshakes from every node, each paying for a certificate
/// proof. No jitter: the node crate carries no randomness source, and the dev
/// profile runs a handful of nodes. A fleet profile, where many nodes lose one
/// control plane together, is the trigger to add it.
pub fn reconcile_backoff(consecutive_failures: u32) -> Duration {
    const FIRST: Duration = Duration::from_secs(1);
    const CEILING: Duration = Duration::from_secs(60);
    FIRST
        .saturating_mul(2u32.saturating_pow(consecutive_failures))
        .min(CEILING)
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

        // `sent`/`received` bracket the request so the clock offset below is
        // measured against their midpoint rather than an instant a whole
        // request leg away (`SIGNOFF-REPAIR.3.4.3.1.2`).
        let sent = Utc::now();
        let poll = self.node.channel().poll(cursor).await?;
        let received = Utc::now();
        // Every held tenant's current epoch (`.1.5.2`, ADR-008; per tenant since
        // `SIGNOFF-REPAIR.5.3.5.3.2`) — the freshness references each cached
        // admission decision is evaluated against, by its command's tenant.
        // Stored BEFORE the commands journal, so a command journaled in this
        // tick is always evaluated against its tenant's epoch at least as fresh
        // as its delivery.
        self.node
            .journal()
            .set_revocation_epochs(&poll.revocation_epochs)
            .await?;
        // The server's clock rides the same response (`SIGNOFF-REPAIR.3.4.3.1.2`).
        self.node
            .journal()
            .record_server_time(poll.server_time, sent, received)
            .await?;
        for cmd in &poll.commands {
            let decided_at = cmd.decided_at.as_ref().map(|d| d.to_rfc3339());
            self.node
                .journal()
                .record_command(
                    &CommandInput {
                        command_id: &cmd.command_id,
                        tenant_id: &cmd.tenant_id,
                        thread_id: &cmd.thread_id,
                        payload: &cmd.payload,
                        authz_ref: cmd.authz_ref.as_deref(),
                        policy_digest: cmd.policy_digest.as_deref(),
                        decided_at: decided_at.as_deref(),
                        revocation_epoch: cmd.revocation_epoch,
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
            match self.process(&item).await {
                Ok(()) => {}
                // One ITEM's failure (`SIGNOFF-REPAIR.4.4.5.2`): its payload cannot
                // be read, so it can never dispatch, and the items after it have
                // nothing to do with it. It is dead-lettered (once: the report
                // dedups, so a stuck item is not re-reported every tick) and the
                // tick goes on. A channel, journal or schedulability failure is
                // the whole node's, and still ends the tick.
                Err(WorkerError::MalformedPayload(detail)) => {
                    self.report_dead_letter(&item, &format!("malformed work payload: {detail}"))
                        .await?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// The execution/retry decision and one item's attempt→event pipeline.
    pub async fn process(&self, item: &WorkItem) -> Result<(), WorkerError> {
        // The payload facts the retry gate reads (parsed BEFORE the decision —
        // the gate's inputs are the delivery's own fields).
        let payload: Value = serde_json::from_str(&item.payload)
            .map_err(|e| WorkerError::MalformedPayload(e.to_string()))?;
        let reservation_present = payload.get("reservation").is_some_and(|v| !v.is_null());
        let duplicate_authorized = payload
            .get("allow_possible_duplicate")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // The cached ADMISSION decision (`.1.5.2`) — read FIRST: the retry
        // count measures attempts made under the CURRENT decision (a `.2.4`
        // replay refreshes it, so pre-replay refusals stop counting — the
        // replayed admission starts a fresh bounded window).
        let cached = self
            .node
            .journal()
            .cached_decision(&item.command_id)
            .await?;
        // How far the SERVER's clock is ahead of this one, measured on the poll
        // that just ran (`SIGNOFF-REPAIR.3.4.3.1.2`). `0` before any handshake,
        // which is the same answer this node gave before the offset existed.
        let offset =
            ChronoDuration::milliseconds(self.node.journal().clock_offset_ms().await?.unwrap_or(0));
        // ⚠️ The filter below compares `decided_at` — the SERVER's database clock
        // — against `updated_at`, which this node wrote from its OWN clock. That
        // is a third cross-clock comparison, and `.3.4.3.1`'s census did not
        // find it because it looked for server instants compared against
        // `Utc::now()`, not against stored local ones. Its failure direction is
        // the dangerous one: with this node's clock BEHIND the server's, genuine
        // post-decision attempts read as older than the decision, are filtered
        // out, and the retry bound never trips. So the decision time is brought
        // into LOCAL terms before it meets a local timestamp.
        let decision_time = cached.as_ref().map(|d| d.decided_at - offset);
        let attempt_count = match &item.operation_id {
            Some(op) => self
                .node
                .journal()
                .attempts_for_operation(op)
                .await?
                .into_iter()
                .filter(|a| {
                    decision_time.is_none_or(|t| {
                        chrono::DateTime::parse_from_rfc3339(&a.updated_at)
                            .map(|u| u.with_timezone(&chrono::Utc) >= t)
                            .unwrap_or(true)
                    })
                })
                .count(),
            None => 0,
        };

        // THE retry gate (`.2.3`, §14.6): a pure decision over the previous
        // attempt's facts. No attempt / a prepared one (the boundary never
        // crossed) and a bounded, reserved pre-dispatch refusal re-dispatch;
        // an outcome_unknown retries only with the explicit possible-duplicate
        // authorization; a budget-denied item (no reservation) and every
        // terminal state are refused — the refusal is logged with the reason
        // (the §9.8 code) and reported to the server as the DEAD LETTER (`.2.4`:
        // the auto-quarantine rides the refusal, once).
        match reasonbraid_core::retry_decision(
            item.latest_attempt_status.as_deref(),
            attempt_count,
            reservation_present,
            duplicate_authorized,
        ) {
            reasonbraid_core::RetryVerdict::Retry => {}
            // The work is done and its result is journaled (and delivered, or
            // pending for the reconcile to deliver): nothing to do, and nothing
            // to report (`SIGNOFF-REPAIR.4.4.8`). Silent, because every tick
            // after a success passes through here.
            reasonbraid_core::RetryVerdict::Settled => return Ok(()),
            reasonbraid_core::RetryVerdict::Refuse { reason } => {
                eprintln!(
                    "worker: {} REFUSED the re-dispatch of {} (retry gate): {reason}",
                    self.node.node_id(),
                    item.command_id
                );
                self.report_dead_letter(item, reason).await?;
                return Ok(());
            }
        }

        // THE cached-decision gate (`.1.5.2`, ADR-008): the dispatch boundary
        // honors the ADMISSION decision the delivery carried — a fresh,
        // epoch-current cached allow dispatches; an expired or epoch-stale one
        // refuses the irreversible write (fail-closed), and a command with NO
        // cached decision is never dispatched (no admission = no dispatch).
        // The refusal is journaled as `failed_before_dispatch` — visible,
        // bounded, never a silent skip.
        // TWO instants, deliberately, and mixing them would corrupt the journal.
        // `now` is this node's own clock and is what gets WRITTEN — every journal
        // timestamp is local, and a server-corrected value stored among them
        // would be compared against local ones later. `server_now` is the same
        // instant expressed in the SERVER's terms, and is used only to EVALUATE
        // a server instant (`SIGNOFF-REPAIR.3.4.3.1.2`). Comparing `decided_at`
        // against the local clock made the difference between two clocks read as
        // age, which past `CACHED_ALLOW_TTL_SECONDS` of skew refused EVERY
        // dispatch — the outage this leaf closes.
        let now = Utc::now();
        let server_now = now + offset;
        match cached {
            None => {
                self.refuse_dispatch(
                    item,
                    "no cached admission decision (the delivery carried none — \
                     a pre-0013 row or plain channel traffic)",
                    now,
                )
                .await?;
                return Ok(());
            }
            Some(decision) => {
                // THE COMMAND'S OWN tenant's epoch (`SIGNOFF-REPAIR.5.3.5.3.2`):
                // a node may hold several tenants' work, and each admission was
                // decided under its own tenant's revocations.
                let current_epoch = self
                    .node
                    .journal()
                    .revocation_epoch_for(&item.tenant_id)
                    .await?;
                let Some(current_epoch) = current_epoch else {
                    self.refuse_dispatch(
                        item,
                        &format!(
                            "no revocation epoch reference for tenant `{}` (no \
                             handshake/poll has carried it yet)",
                            item.tenant_id
                        ),
                        now,
                    )
                    .await?;
                    return Ok(());
                };
                match decision.evaluate(server_now, current_epoch as u64) {
                    reasonbraid_core::CacheVerdict::Allow => {}
                    reasonbraid_core::CacheVerdict::Deny { reason } => {
                        self.refuse_dispatch(item, &reason, now).await?;
                        return Ok(());
                    }
                    reasonbraid_core::CacheVerdict::Stale => {
                        let reason = format!(
                            "cached admission decision is stale (decided {}, expires {}, \
                             server now {}, clock offset {} ms, recorded epoch {}, \
                             current epoch {})",
                            decision.decided_at,
                            decision.expires_at,
                            server_now,
                            offset.num_milliseconds(),
                            decision.revocation_epoch,
                            current_epoch
                        );
                        self.refuse_dispatch(item, &reason, now).await?;
                        return Ok(());
                    }
                }
            }
        }

        // THE schedulability gate (`SIGNOFF-REPAIR.4.4.4.2.2`): a node that has
        // not completed its reconcile spends NOTHING. It comes after the two
        // gates above, which decide from journaled facts alone, and before the
        // supervisor, whose first act toward a provider is the paid one. Nothing
        // is journaled: the item is untouched and dispatches after the reconcile
        // the caller runs on this error (`rb-node`'s loop does).
        if !self.node.is_schedulable().await {
            eprintln!(
                "worker: {} is not schedulable; the dispatch of {} waits for the reconcile",
                self.node.node_id(),
                item.command_id
            );
            return Err(WorkerError::Node(NodeError::NotSchedulable));
        }

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
            // The empty id is what refuses; the window is filled with the
            // current instant so the placeholder is not accidentally the one
            // reference that never lapses (`SIGNOFF-REPAIR.11.24.1.1.2.2`).
            _ => ReservationReference {
                reservation_id: String::new(),
                dimensions: BudgetDimensions::default(),
                issued_at: Utc::now(),
                expires_at: Utc::now(),
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

        // The result event is built INSIDE the supervisor and journaled in the
        // same transaction as the attempt's completion (`SIGNOFF-REPAIR.4.4.4.1`):
        // there is no instant at which the attempt is complete and its result
        // is not pending, so neither a crash nor an unschedulable node can lose it.
        let event_id = EventId::new().to_string();
        let build_result = |report: &ExecutionReport| ResultEvent {
            event_id: event_id.clone(),
            payload: json!({
                "kind": "work_result",
                "work_kind": work_kind,
                "command_id": item.command_id,
                "reservation_id": report.reservation_id,
                "attempt_id": report.attempt_id,
                "content": report.chunks.join(""),
                "usage": report.usage,
            }),
        };
        let report = match execute_attempt_emitting(
            self.node.journal(),
            &self.adapter,
            &operation_id,
            &request,
            &reservation,
            &self.local,
            &build_result,
        )
        .await
        {
            Ok(report) => report,
            // An unknown outcome is a FACT about this attempt, and the supervisor
            // has already journaled it `outcome_unknown` (`SIGNOFF-REPAIR.4.4.5.2`).
            // Nothing about the node needs recovering: the retry gate owns the
            // item from here, and retries it only with the explicit
            // possible-duplicate authorization. So it is reported, not raised;
            // raising it stopped the node process.
            Err(SupervisorError::OutcomeUnknown { attempt_id, detail }) => {
                eprintln!(
                    "worker: attempt {attempt_id} of {} ended `outcome_unknown` ({detail}) — \
                     journaled; the retry gate owns it",
                    item.command_id
                );
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };
        match &report.result_event {
            Some(event) => {
                match self
                    .node
                    .deliver_journaled_event(&operation_id, &event.event_id, &event.payload)
                    .await?
                {
                    EventDelivery::Delivered => eprintln!(
                        "worker: {} emitted a {work_kind} result for {}",
                        self.node.node_id(),
                        item.command_id
                    ),
                    EventDelivery::Deferred => eprintln!(
                        "worker: {} journaled a {work_kind} result for {}; the node is not \
                         schedulable, so the next reconcile delivers it",
                        self.node.node_id(),
                        item.command_id
                    ),
                }
            }
            None => {
                eprintln!(
                    "worker: attempt {} ended `{}` — journaled, no contribution emitted",
                    report.attempt_id,
                    report.final_state.as_str()
                );
            }
        }
        Ok(())
    }

    /// Journal a dispatch-boundary refusal (`.1.5.2`, ADR-008): the attempt is
    /// prepared and IMMEDIATELY failed before dispatch with the reason — the
    /// adapter is never contacted and the item's terminal status makes the next
    /// skip decision permanent (a refused dispatch is never silently retried).
    async fn refuse_dispatch(
        &self,
        item: &WorkItem,
        reason: &str,
        at: chrono::DateTime<Utc>,
    ) -> Result<(), WorkerError> {
        let operation_id = match &item.operation_id {
            Some(op) => op.clone(),
            None => {
                self.node
                    .journal()
                    .ensure_operation(&item.command_id, at)
                    .await?
                    .operation_id
            }
        };
        let attempt_id = ProviderAttemptId::new().to_string();
        self.node
            .journal()
            .prepare_attempt(&attempt_id, &operation_id, at)
            .await?;
        self.node
            .journal()
            .record_failed_before_dispatch(&attempt_id, Some(reason), at)
            .await?;
        eprintln!(
            "worker: {} REFUSED the dispatch of {} (fail-closed): {reason}",
            self.node.node_id(),
            item.command_id
        );
        Ok(())
    }

    /// Report the terminal refusal to the server (`.2.4`): the dead letter
    /// auto-quarantines the inbox row with the reason, ONCE per operation (the
    /// outgoing-events check dedupes). BEST-EFFORT: the event is journaled
    /// FIRST (durable — a reconcile re-emits it with the original id), and a
    /// failed send (offline channel) defers the report instead of failing the
    /// tick — the refusal itself is already a journal fact.
    async fn report_dead_letter(&self, item: &WorkItem, reason: &str) -> Result<(), WorkerError> {
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
        if self.node.journal().has_dead_letter(&operation_id).await? {
            return Ok(());
        }
        let payload = json!({
            "kind": "work_dead_lettered",
            "command_id": item.command_id,
            "reason": reason,
        });
        let event_id = EventId::new().to_string();
        self.node
            .journal()
            .record_outgoing_event(&event_id, &operation_id, &payload, Utc::now())
            .await?;
        if let Err(e) = self
            .node
            .channel()
            .send_event(&event_id, &operation_id, &payload)
            .await
        {
            eprintln!(
                "worker: {} deferred the dead-letter report for {} ({e}) — \
                 the next reconcile re-emits it",
                self.node.node_id(),
                item.command_id
            );
        } else {
            eprintln!(
                "worker: {} dead-lettered {} (reason: {reason})",
                self.node.node_id(),
                item.command_id
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reconcile backoff (`SIGNOFF-REPAIR.4.4.5.3`): doubling from one
    /// second, never past a minute, and never overflowing however long the
    /// control plane stays away.
    #[test]
    fn the_reconcile_backoff_doubles_and_is_bounded() {
        let waits: Vec<u64> = (0..9).map(|n| reconcile_backoff(n).as_secs()).collect();
        assert_eq!(waits, [1, 2, 4, 8, 16, 32, 60, 60, 60]);
        assert_eq!(reconcile_backoff(u32::MAX), Duration::from_secs(60));
    }

    /// The run loop's decision (`SIGNOFF-REPAIR.4.4.4.2.2`, `.4.4.5.2`): a channel
    /// failure, the node's failed send, and a dispatch refused on an unschedulable
    /// node all reconcile; a malformed payload or journal is not a channel's to
    /// fix.
    #[test]
    fn a_refused_dispatch_on_an_unschedulable_node_calls_for_reconcile() {
        assert!(WorkerError::Node(NodeError::NotSchedulable).calls_for_reconcile());
        assert!(WorkerError::Channel(ChannelError::NotEnrolled).calls_for_reconcile());
        assert!(
            WorkerError::Node(NodeError::Channel(ChannelError::NotAuthenticated))
                .calls_for_reconcile(),
            "a failed send of a result is a channel loss (`SIGNOFF-REPAIR.4.4.5.2`)"
        );
        assert!(!WorkerError::MalformedPayload("x".to_string()).calls_for_reconcile());
        assert!(
            !WorkerError::Node(NodeError::MalformedJournal("x".to_string())).calls_for_reconcile()
        );
    }

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
        let now = Utc::now();
        let ref_ = ReservationReference {
            reservation_id: String::new(),
            dimensions: BudgetDimensions::default(),
            issued_at: now,
            expires_at: now,
        };
        assert!(
            ref_.applicable_at(now).is_err(),
            "an empty reference must refuse"
        );
    }
}
