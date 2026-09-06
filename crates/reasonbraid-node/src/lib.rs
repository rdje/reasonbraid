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
//! The outbound node channel with cursor resume and the reconciliation handshake
//! (`.3.2`) and the execution supervisor build on this journal.
//!
//! See `docs/decisions/2026-09-06_node-journal.md` for the design record and
//! `docs/book/src/node-journal.md` for the operator-facing documentation.

mod journal;

pub use journal::{
    AttemptSummary, CommandInput, CommandRecorded, EventSummary, Journal, JournalCounts,
    JournalError, JournalHealth, OperationRecorded, ProvenStatus, RecoveryReport, TransitionRow,
    DURABILITY_JOURNAL_MODE, DURABILITY_SYNCHRONOUS,
};
