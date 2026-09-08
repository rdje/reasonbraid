//! # reasonbraid-adapter
//!
//! The harness adapter boundary (`KICKOFF.md` §3): the narrow, vendor-neutral contract
//! the node supervises, the deterministic fake adapter (the conformance oracle), and
//! the sanitized outcome fixture corpus.
//!
//! `PHASE-0.4.1` lands:
//!
//! - [`contract`] — the [`contract::Adapter`] trait and its types: capabilities are
//!   declared, a dispatch acknowledgement is distinct from completion, an unsupported
//!   status lookup is an honest fact (never a retry recommendation), and no credential
//!   field exists anywhere in the contract.
//! - [`fake`] — the deterministic scripted [`fake::FakeAdapter`]: stream, fail, hang
//!   (cancellation-Notify, no sleeps), report usage, ignore cancellation, and lose a
//!   response after dispatch — the `ROADMAP.md` §11.6 conformance oracle.
//! - [`fixtures`] — the sanitized outcome corpus (`fixtures/*.json`), mechanically
//!   credential-scanned and coverage-checked.
//!
//! `.4.2` qualifies the first REAL harness behind this same contract, and
//! `PHASE-1.4.1` the second: [`ClaudeCliAdapter`], the `.4.2` mirror over
//! `claude -p --output-format stream-json`.
//!
//! `PHASE-0.7` lands [`bench`] — the WP7 deliberation/routing benchmark: a versioned
//! eight-case corpus run through four workflows (single / blind-independent /
//! critique-revise / moderator-synthesis) with deterministic graders, per-case
//! confidence, cost accounting, and a spread-bearing report; the scripted agent
//! proves the harness, the `RB_LIVE_CODEX=1` real mode produces the numbers.
//!
//! See `docs/decisions/2026-09-06_fake-adapter.md` for the design record and
//! `docs/book/src/adapter-boundary.md` for the boundary documentation.

mod claude;
mod codex;
mod contract;
mod fake;
pub mod fixtures;
pub mod resolver;

pub mod bench;
pub mod certification;

pub use certification::{
    certify, dev_six_box_evidence, BoxStatus, CertificationReport, CheckResult,
    ConformanceScenario, SixBoxEvidence, SixBoxRecord, Trigger, Verdict,
};
pub use claude::ClaudeCliAdapter;
pub use codex::CodexCliAdapter;
pub use contract::{
    Adapter, AdapterCapabilities, AttemptEvent, AttemptHandle, AttemptResult, AttemptStream,
    CancellationOutcome, CancellationStrength, DispatchAck, InvokeOutcome, NormalizedUsage,
    PolicyInjectionMode, RunRequest, StatusLookupOutcome, UsageConfidence, SDK_VERSION,
};
pub use fake::{FakeAdapter, FixtureSpec, ScriptStep, StatusLookupSpec};
pub use resolver::{ResolverAdvertise, EGRESS_CLASSES, SANDBOX_LEVELS};
