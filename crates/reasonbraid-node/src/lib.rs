//! # reasonbraid-node
//!
//! The Rust node crate (`KICKOFF.md` §3): outbound connection, SQLite journal, and
//! execution supervisor.
//!
//! `PHASE-0.3.1` lands the WP3 journal and its inspection surface:
//!
//! - [`journal::Journal`] — the node-local SQLite journal (WAL + `synchronous=FULL`,
//!   recorded in `journal_meta`), with boundary-before-boundary ordering: a provider
//!   dispatch is recorded durably *before* the adapter is invoked, so crash recovery
//!   can report the honest `outcome_unknown` instead of guessing, and
//!   [`journal::Journal::prove_result`] is the only path from ambiguity to a proven
//!   terminal result (§11.3 provider lookup).
//! - `rb-journal` — the read-only inspection CLI (the `src/bin/rb-journal.rs` binary):
//!   operators inspect journal health, pending work, and ambiguous attempts without
//!   opening SQLite by hand.
//!
//! `PHASE-0.3.2` lands the WP3 outbound channel and the reconciliation gate:
//!
//! - [`channel::NodeChannel`] — the outbound connection to the control plane
//!   (HTTP/1 JSON over the loopback dev profile): the cursor-reporting handshake,
//!   original-id event submission, cursor acknowledgement, and the live poll.
//! - [`node::Node`] — the lifecycle facade: [`node::Node::reconcile`] runs the
//!   reconnect protocol end to end (replay, directives, pending-event re-emission) and
//!   becomes [`node::NodeState::Schedulable`] only when it completes — the WP3
//!   "not schedulable until reconciliation completes" acceptance.
//!
//! `PHASE-2.1.2.2` proves the channel with the workload certificate: the handshake
//! signs the channel fields with the leaf.s key ([`channel::compute_cert_proof`]), the
//! server chains the leaf to its CA and answers with a lease + fencing token, and
//! events/ack/poll/heartbeat all ride that token; [`channel::NodeChannel::heartbeat`]
//! renews the lease.
//!
//! The execution supervisor and the adapter boundary are WP4's leaves.
//!
//! See `docs/decisions/2026-09-06_node-journal.md` for the journal design record and
//! `docs/book/src/node-journal.md` for the operator-facing documentation.

mod channel;
mod journal;
mod node;
mod supervisor;
mod worker;

pub use channel::{
    compute_cert_proof, AckResponse, AmbiguousAttempt, ChannelError, Directive, EventReceipt,
    HandshakeRequest, HandshakeResponse, HeartbeatResponse, KnownEvent, NodeChannel, PollResponse,
    ProofCoverage, ReplayCommand, RotateRequest, RotateResponse, CHANNEL_VERSION,
};
pub use journal::{
    AttemptSummary, CommandInput, CommandRecorded, EventSummary, Journal, JournalCounts,
    JournalError, JournalHealth, OperationRecorded, ProvenStatus, RecoveryReport, TransitionRow,
    WorkItem, DURABILITY_JOURNAL_MODE, DURABILITY_SYNCHRONOUS,
};
pub use node::{Node, NodeError, NodeState};
pub use supervisor::{execute_attempt, ExecutionReport, LocalBudget, SupervisorError};
pub use worker::{Worker, WorkerError};
