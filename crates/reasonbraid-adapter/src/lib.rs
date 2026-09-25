//! # reasonbraid-adapter
//!
//! The harness adapter boundary (`KICKOFF.md` §3): the narrow, vendor-neutral contract
//! the node supervises, the deterministic fake adapter (the conformance oracle), and
//! the sanitized outcome fixture corpus.
//!
//! `PHASE-0.4.1` lands:
//!
//! - `contract` — the [`Adapter`] trait and its types: capabilities are
//!   declared, a dispatch acknowledgement is distinct from completion, an unsupported
//!   status lookup is an honest fact (never a retry recommendation), and no credential
//!   field exists anywhere in the contract.
//! - `fake` — the deterministic scripted [`FakeAdapter`]: stream, fail, hang
//!   (cancellation-Notify, no sleeps), report usage, ignore cancellation, and lose a
//!   response after dispatch — the `ROADMAP.md` §11.6 conformance oracle.
//! - [`fixtures`] — the sanitized outcome corpus (`fixtures/*.json`), mechanically
//!   credential-scanned and coverage-checked.
//!
//! `.4.2` qualifies the first REAL harness behind this same contract, and
//! `PHASE-1.4.1` the second: [`ClaudeCliAdapter`], the `.4.2` mirror over
//! `claude -p --output-format stream-json`.
//!
//! `PHASE-0.7` lands [`mod@bench`] — the WP7 deliberation/routing benchmark: a versioned
//! eight-case corpus run through four workflows (single / blind-independent /
//! critique-revise / moderator-synthesis) with deterministic graders, per-case
//! confidence, cost accounting, and a spread-bearing report; the scripted agent
//! proves the harness, the `RB_LIVE_CODEX=1` real mode produces the numbers.
//!
//! `PHASE-8.4.1` lands [`resolver`] — the §12.2 resolver advertise, the
//! declarative row a third-party resolver publishes for the server's capability
//! registry to filter and rank.
//!
//! ⛔ **The two boundaries in this crate are not the same shape, and the crate
//! root says so rather than leaving it to be inferred from which traits exist.**
//! A HARNESS is implemented in process, through [`Adapter`]. A
//! RESOLVER is not: it contributes [`resolver::ResolverAdvertise`] and an
//! out-of-process worker binary, and there is no acquisition trait for it —
//! `SIGNOFF-REPAIR.11.24.1.5`, and [`resolver`]'s own header carries the
//! argument. The asymmetry is ADR-018's: a resolver's advertised
//! `sandbox_level` states what its code provides and the server filters on it,
//! so a surface that let third-party code run in this process could only ever
//! honestly claim `none` while being able to claim anything.
//!
//! See `docs/decisions/2026-09-06_fake-adapter.md` for the design record and
//! `docs/book/src/adapter-boundary.md` for the boundary documentation.

mod claude;
mod codex;
mod contract;
mod fake;
// `SIGNOFF-REPAIR.10.1.2`: the bounded stream readers both provider-CLI adapters share.
pub mod fixtures;
pub mod resolver;
mod subprocess;

mod allowlist;
pub mod bench;
pub mod certification;

pub use allowlist::{capabilities_within, verify_ladder, AllowedCapabilities, RungRefusal};
pub use certification::{
    certify, dev_six_box_evidence, BoxStatus, CertificationReport, CheckResult,
    ConformanceScenario, SixBoxEvidence, SixBoxRecord, Trigger, Verdict,
};
pub use claude::ClaudeCliAdapter;
pub use codex::CodexCliAdapter;
// `SIGNOFF-REPAIR.10.1.2`: exported so the node's supervisor can pin its own
// output bound against it.
pub use contract::{
    Adapter, AdapterCapabilities, AttemptEvent, AttemptHandle, AttemptResult, AttemptStream,
    CancellationOutcome, CancellationStrength, DispatchAck, InvokeOutcome, NormalizedUsage,
    PolicyInjectionMode, RunRequest, StatusLookupOutcome, UsageConfidence, SDK_VERSION,
};
pub use fake::{FakeAdapter, FixtureSpec, ScriptStep, StatusLookupSpec};
pub use resolver::{ResolverAdvertise, EGRESS_CLASSES, SANDBOX_LEVELS};
pub use subprocess::MAX_LINE_BYTES;
