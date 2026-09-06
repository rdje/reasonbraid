//! The WP7 benchmark harness (`PHASE-0.7`; KICKOFF WP7, `ROADMAP.md` §13.7,
//! hypotheses H1/H6): a small versioned corpus run through four deliberation
//! workflows — single agent, blind-independent answers + deterministic
//! adjudication, critique/revise, moderator/synthesis with an explicit
//! unresolved register — with deterministic grading, per-case confidence,
//! cost accounting, and a spread-bearing report.
//!
//! Two agents serve the harness:
//!
//! - [`scripted::ScriptedAgent`] (default, CI-safe): plays the corpus's scripted
//!   outputs through the real [`crate::Adapter`] contract — the self-test
//!   (`tests/bench_harness.rs`) asserts the computed scores equal the corpus's
//!   expected ones, proving the measurement pipeline before any tokens are spent.
//! - the real [`crate::CodexCliAdapter`] (`RB_LIVE_CODEX=1`, never default):
//!   the actual numbers the Phase 1 routing decision rests on.
//!
//! # Measurement hygiene
//!
//! No LLM judge (a judge would share the measured models' correlated errors);
//! no independence score (agreement is descriptive); no average-only summaries
//! (every aggregate carries n/min/max); no invented calibration (Brier is
//! factual-only); unmeasured stays unmeasured.

pub mod corpus;
pub mod grader;
pub mod report;
pub mod scripted;
pub mod workflows;

pub use corpus::{Case, Corpus, CorpusError, Prompts};
pub use grader::{brier, grade, normalize, Graded, Trap};
pub use report::{build, write, Report};
pub use scripted::{Role, ScriptedAgent};
pub use workflows::{run_workflow, BenchError, Workflow, WorkflowResult};
