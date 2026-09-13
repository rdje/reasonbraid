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
use std::sync::Arc;

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
#[derive(Clone)]
pub struct Node {
    node_id: String,
    journal: Journal,
    channel: NodeChannel,
    state: Arc<RwLock<NodeState>>,
}

impl Node {
    /// Open (or create) the journal at `journal_path` and build the outbound channel
    /// toward `base_url`, proving the node's identity with its workload certificate
    /// (the `.1.2.1` enrollment leaf or a `.1.2.2` rotation). The node starts
    /// `Offline`; call [`Node::reconcile`].
    pub async fn open(
        journal_path: impl AsRef<Path>,
        base_url: impl Into<String>,
        node_id: String,
        cert_der: Vec<u8>,
        key: rcgen::KeyPair,
    ) -> Result<Self, NodeError> {
        // `SIGNOFF-REPAIR.4.2.9`: a rotation must survive a restart. The
        // channel rotates on its own when the leaf nears expiry, and until this
        // sink existed the fresh identity lived only in memory — so a restart
        // loaded the superseded certificate, and a node that stayed down past
        // that certificate's expiry could not rotate its way out (rotation
        // requires a usable certificate) and needed re-enrolling by an operator.
        //
        // The files are the ones `rb-node` already loads at start: `cert.der`
        // and `key.der` beside the journal (ADR-007, §13 — node-local state on
        // the journal's own volume).
        let dir = journal_path.as_ref().parent().map(|d| d.to_path_buf());
        let channel = NodeChannel::new(base_url, node_id.clone(), cert_der, key);
        let channel = match dir {
            Some(dir) => channel.persisting_identity_with(Arc::new(
                move |cert: &[u8], key: &rcgen::KeyPair| {
                    // A failure here is reported and NOT fatal: the node is already
                    // holding a working identity, and refusing to continue would
                    // turn a degraded-durability condition into an outage. The next
                    // start reports the stale certificate loudly by rotating again.
                    if let Err(e) = std::fs::write(dir.join("cert.der"), cert) {
                        eprintln!("node: could not persist the rotated certificate: {e}");
                    }
                    if let Err(e) = std::fs::write(dir.join("key.der"), key.serialize_der()) {
                        eprintln!("node: could not persist the rotated key: {e}");
                    }
                },
            )),
            None => channel,
        };
        Ok(Self {
            node_id: node_id.clone(),
            journal: Journal::open(journal_path).await?,
            channel,
            state: Arc::new(RwLock::new(NodeState::Offline)),
        })
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// The node's journal (the durable local facts; also the operator's inspection target).
    pub fn journal(&self) -> &Journal {
        &self.journal
    }

    /// The node's outbound channel (the worker polls the delivery tail with it).
    pub fn channel(&self) -> &NodeChannel {
        &self.channel
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

        // 3. The handshake exchange. The certificate + proof are the CHANNEL.s
        //    job (it owns the workload identity): it fills both from the
        //    installed leaf, so the proof always covers what is sent.
        // `sent`/`received` bracket the exchange so the clock offset at 3.6 is
        // measured against their midpoint (`SIGNOFF-REPAIR.3.4.3.1.2`).
        let sent = Utc::now();
        let response = self
            .channel
            .handshake(&HandshakeRequest {
                channel_version: CHANNEL_VERSION,
                node_id: self.node_id.clone(),
                last_acked_cursor,
                pending_operations,
                ambiguous_attempts,
                cert_der: String::new(),
                proof_signature: String::new(),
                // Like the two above: the channel fills it, because it owns the
                // workload identity and the proof it signs (`SIGNOFF-REPAIR.4.2.2`).
                nonce: String::new(),
            })
            .await?;
        let received = Utc::now();

        // 3.5 The tenant's current revocation epoch (`.1.5.2`, ADR-008) — stored
        //     before the replay journals, so every command journaled here is
        //     evaluated against an epoch at least as fresh as its delivery.
        self.journal
            .set_revocation_epoch(response.revocation_epoch)
            .await?;
        // 3.6 The server's clock (`SIGNOFF-REPAIR.3.4.3.1.2`), measured against
        //     the instant this response was taken in. Stored beside the epoch so
        //     the dispatch gate can evaluate `decided_at` in the server's terms
        //     rather than treating the difference between two clocks as age.
        self.journal
            .record_server_time(response.server_time, sent, received)
            .await?;

        // 4. Journal the replay, deduplicated by command id (a duplicated command never
        //    creates a second local operation). The admission decision metadata rides
        //    the row (`.1.5.2`): the cached decision the dispatch boundary evaluates.
        let mut max_cursor = last_acked_cursor;
        for cmd in &response.replay {
            let decided_at = cmd.decided_at.as_ref().map(|d| d.to_rfc3339());
            self.journal
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

/// `SIGNOFF-REPAIR.4.2.9` — `Node::open` wires the channel's identity sink to
/// the SAME two files `rb-node` loads at start, so a rotation survives a
/// restart.
///
/// The pairing is the whole point: `rb-node`'s `load_workload_identity` reads
/// `cert.der` and `key.der` beside the journal, and until this leaf nothing ever
/// rewrote them after enrollment. A control that checked only "a sink fires"
/// would not have caught a sink writing to the wrong place.
#[cfg(test)]
mod rotated_identity_survives_restart {
    use super::*;

    fn key() -> rcgen::KeyPair {
        rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).expect("a test key")
    }

    /// A fixture directory on the repository's own volume (§13), named by
    /// exclusive creation rather than by a clock.
    fn fixture_dir(name: &str) -> std::path::PathBuf {
        let base = std::env::var_os("CARGO_TARGET_TMPDIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target")
            });
        let dir = base
            .join("identity-persistence")
            .join(format!("{name}-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(dir.parent().expect("the fixture parent")).expect("fixture parent");
        std::fs::DirBuilder::new()
            .create(&dir)
            .expect("the fixture directory is new");
        dir
    }

    #[tokio::test]
    async fn a_rotation_rewrites_the_files_the_next_start_reads() {
        let dir = fixture_dir("rotation");
        let journal = dir.join("node.db");
        let node = Node::open(
            &journal,
            "http://127.0.0.1:1",
            "nod_00000000-0000-7000-8000-000000000429".to_string(),
            vec![0x01, 0x02],
            key(),
        )
        .await
        .expect("open the node");

        // Nothing is written at open: the identity came FROM those files.
        assert!(
            !dir.join("cert.der").exists(),
            "opening a node writes no identity — it was handed one"
        );

        // The rotation the channel performs inside `handshake`, driven directly.
        let rotated_key = key();
        let rotated_key_der = rotated_key.serialize_der();
        node.channel()
            .install_identity(vec![0xAA, 0xBB, 0xCC], rotated_key);

        // THE INVARIANT: the two files the next start reads now hold the
        // ROTATED identity, byte for byte.
        assert_eq!(
            std::fs::read(dir.join("cert.der")).expect("cert.der was written"),
            vec![0xAA, 0xBB, 0xCC],
            "the rotated certificate is on disk, in the file `rb-node` loads"
        );
        assert_eq!(
            std::fs::read(dir.join("key.der")).expect("key.der was written"),
            rotated_key_der,
            "and so is its key — a certificate without its key restores nothing"
        );
    }
}
