//! The node lifecycle facade (`PHASE-0.3.2`): journal + outbound channel + the
//! reconciliation gate.
//!
//! A [`Node`] is NOT schedulable until reconciliation completes (`KICKOFF.md` WP3,
//! `ROADMAP.md` §17.4 step 7). `reconcile` walks the reconnect protocol end to end:
//!
//! 1. classify crashed attempts (`dispatched → outcome_unknown`) in the journal;
//! 2. report the node's durable resume facts — last acknowledged cursor, pending local
//!    operation ids, ambiguous attempts;
//! 3. journal the replayed commands (deduplicated by command id — a redelivered
//!    command never creates a second local operation);
//! 4. apply the server's reconciliation directives (adjudicated → `reconciled`;
//!    `needs_adjudication` → the attempt stays visibly `outcome_unknown`);
//! 5. re-emit pending events with their ORIGINAL ids (§17.4 step 5), and mark events
//!    the server reports holding as acknowledged;
//! 6. acknowledge the cursor server-side and record it locally;
//! 7. only then become `Schedulable`.
//!
//! Any failure along the way leaves the node NOT schedulable (Offline): the next
//! `reconcile` retries the whole protocol, which is idempotent by construction.

use std::fmt;
use std::path::Path;

use chrono::Utc;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::channel::{
    AmbiguousAttempt, ChannelError, Directive, HandshakeRequest, NodeChannel, CHANNEL_VERSION,
};
use crate::journal::{CommandInput, Journal, JournalError};

/// Where the node stands with respect to the control plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    /// No usable connection (fresh node, failed reconcile, or the channel died).
    Offline,
    /// Connected and reconciling: the handshake is in flight or not fully applied.
    Reconciling,
    /// Reconciliation completed: the node may process and emit new work.
    Schedulable,
}

/// A typed node error.
#[derive(Debug)]
pub enum NodeError {
    Journal(JournalError),
    Channel(ChannelError),
    /// The journal holds a malformed stored payload (should be unreachable — the
    /// journal only stores values it serialized itself).
    MalformedJournal(String),
    /// An operation that requires a schedulable node was attempted earlier.
    NotSchedulable,
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeError::Journal(e) => write!(f, "node journal error: {e}"),
            NodeError::Channel(e) => write!(f, "node channel error: {e}"),
            NodeError::MalformedJournal(detail) => {
                write!(f, "node journal holds a malformed payload: {detail}")
            }
            NodeError::NotSchedulable => {
                write!(f, "the node is not schedulable yet — reconcile first")
            }
        }
    }
}

impl std::error::Error for NodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NodeError::Journal(e) => Some(e),
            NodeError::Channel(e) => Some(e),
            _ => None,
        }
    }
}

impl From<JournalError> for NodeError {
    fn from(e: JournalError) -> Self {
        NodeError::Journal(e)
    }
}

impl From<ChannelError> for NodeError {
    fn from(e: ChannelError) -> Self {
        NodeError::Channel(e)
    }
}

/// A ReasonBraid node: the journal owns its durable local facts, the channel owns its
/// outbound connection, and the state machine owns when it may do work.
pub struct Node {
    node_id: String,
    journal: Journal,
    channel: NodeChannel,
    state: RwLock<NodeState>,
}

impl Node {
    /// Open (or create) the journal at `journal_path` and build the outbound channel
    /// toward `base_url`. The node starts `Offline`; call [`Node::reconcile`].
    pub async fn open(
        journal_path: impl AsRef<Path>,
        base_url: impl Into<String>,
        node_id: String,
    ) -> Result<Self, NodeError> {
        Ok(Self {
            node_id: node_id.clone(),
            journal: Journal::open(journal_path).await?,
            channel: NodeChannel::new(base_url, node_id),
            state: RwLock::new(NodeState::Offline),
        })
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// The node's journal (the durable local facts; also the operator's inspection target).
    pub fn journal(&self) -> &Journal {
        &self.journal
    }

    /// The current lifecycle state.
    pub async fn state(&self) -> NodeState {
        *self.state.read().await
    }

    pub async fn is_schedulable(&self) -> bool {
        self.state().await == NodeState::Schedulable
    }

    /// Run the reconnect protocol end to end; on success the node is [`NodeState::Schedulable`].
    /// On ANY failure the node returns to [`NodeState::Offline`] (nothing half-applied is
    /// claimed as schedulable); the protocol is idempotent, so retrying is always safe.
    pub async fn reconcile(&self) -> Result<(), NodeError> {
        if let Err(e) = self.reconcile_inner().await {
            *self.state.write().await = NodeState::Offline;
            return Err(e);
        }
        *self.state.write().await = NodeState::Schedulable;
        Ok(())
    }

    async fn reconcile_inner(&self) -> Result<(), NodeError> {
        *self.state.write().await = NodeState::Reconciling;
        let now = Utc::now();

        // 1. Classify crashed attempts BEFORE reporting (dispatched → outcome_unknown).
        self.journal.recover(now).await?;

        // 2. The node's authoritative resume facts (§17.4 step 2).
        let last_acked_cursor = self.journal.last_acked_cursor().await?;
        let pending_operations = self.journal.pending_operations().await?;
        let ambiguous_attempts = self
            .journal
            .ambiguous_attempts()
            .await?
            .into_iter()
            .map(|a| AmbiguousAttempt {
                attempt_id: a.attempt_id,
                operation_id: a.operation_id,
            })
            .collect();

        // 3. The handshake exchange.
        let response = self
            .channel
            .handshake(&HandshakeRequest {
                channel_version: CHANNEL_VERSION,
                node_id: self.node_id.clone(),
                last_acked_cursor,
                pending_operations,
                ambiguous_attempts,
            })
            .await?;

        // 4. Journal the replay, deduplicated by command id (a duplicated command never
        //    creates a second local operation).
        let mut max_cursor = last_acked_cursor;
        for cmd in &response.replay {
            self.journal
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
            self.journal.ensure_operation(&cmd.command_id, now).await?;
            max_cursor = max_cursor.max(cmd.cursor);
        }

        // 5. Apply the server's reconciliation directives.
        for directive in &response.directives {
            match directive {
                Directive::Adjudicated { attempt_id, .. } => {
                    self.journal.reconcile(attempt_id, now).await?;
                }
                Directive::NeedsAdjudication { .. } => {
                    // The attempt stays `outcome_unknown` — bounded and visible, never
                    // silently retried.
                }
            }
        }

        // 6. Re-emit pending events with their ORIGINAL ids (§17.4 step 5) — except the
        //    ones the server reports holding (its `known_events`, built from the pending
        //    operation ids this handshake exchanged): those are provably delivered, so
        //    they are marked acknowledged locally instead of being re-sent. A transport
        //    failure aborts the whole reconcile: the events stay pending and the next
        //    reconcile re-sends them.
        let known: std::collections::HashSet<(String, String)> = response
            .known_events
            .iter()
            .map(|k| (k.operation_id.clone(), k.event_id.clone()))
            .collect();
        for event in self.journal.pending_events().await? {
            if known.contains(&(event.operation_id.clone(), event.event_id.clone())) {
                self.journal
                    .acknowledge_known_event(&event.operation_id, &event.event_id, max_cursor, now)
                    .await?;
                continue;
            }
            let payload: Value = serde_json::from_str(&event.payload)
                .map_err(|e| NodeError::MalformedJournal(e.to_string()))?;
            self.channel
                .send_event(&event.event_id, &event.operation_id, &payload)
                .await?;
            self.journal
                .acknowledge_event(&event.event_id, &max_cursor.to_string(), now)
                .await?;
        }

        // 7. Acknowledge the cursor server-side and record it locally.
        self.channel.acknowledge(max_cursor).await?;
        self.journal.set_last_acked_cursor(max_cursor).await?;

        Ok(())
    }

    /// Emit one event for an operation — schedulable nodes only. Journal FIRST
    /// (§17.4 persist-before-boundary), then send, then mark acknowledged with the
    /// current cursor. On transport failure the event stays pending in the journal
    /// (same id, same payload) and is re-emitted by the next `reconcile`.
    pub async fn emit_event(
        &self,
        operation_id: &str,
        event_id: &str,
        payload: &Value,
    ) -> Result<(), NodeError> {
        if !self.is_schedulable().await {
            return Err(NodeError::NotSchedulable);
        }
        let now = Utc::now();
        self.journal
            .record_outgoing_event(event_id, operation_id, payload, now)
            .await?;
        if let Err(e) = self
            .channel
            .send_event(event_id, operation_id, payload)
            .await
        {
            // Stays pending in the journal; the next reconcile re-emits it (original id).
            return Err(NodeError::Channel(e));
        }
        let cursor = self.journal.last_acked_cursor().await?;
        self.journal
            .acknowledge_event(event_id, &cursor.to_string(), now)
            .await?;
        Ok(())
    }
}
