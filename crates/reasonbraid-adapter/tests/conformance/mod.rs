//! The adapter conformance harness (`PHASE-2.6.1`, re-expressed through
//! the certification suite in `PHASE-8.4.3`): ONE mechanical run that
//! every adapter — the deterministic fake and every real CLI adapter —
//! passes against the [`reasonbraid_adapter::Adapter`] contract (§19.4's
//! checkable items).
//!
//! Each adapter family registers CONFORMANCE SCENARIOS — an adapter
//! instance wired to a trigger (a refusal, a lost response, a hang, a
//! completion) plus the capabilities it DECLARES. The shared run is the
//! certification's six cross-adapter invariants:
//!
//! 1. **Capability-manifest agreement** — the declared
//!    [`AdapterCapabilities`] match the scenario's verified boundary.
//! 2. **Unsupported-operation honesty** — a lookup for a never-dispatched
//!    operation is `Unsupported`, never a fabricated terminal and never a
//!    retry recommendation.
//! 3. **The dispatch boundary** — a refusal returns `FailedBeforeDispatch`
//!    BEFORE any provider contact; no attempt handle leaks past it.
//! 4. **Ambiguous-outcome honesty** — a lost response ends the stream
//!    WITHOUT a terminal event; the adapter never invents one.
//! 5. **Cancellation matches the declared strength** — the `cancel`
//!    outcome never exceeds the declared `CancellationStrength`.
//! 6. **Usage accounting never lies** — `normalize_usage` on an empty
//!    receipt reports `Unknown` confidence with every dimension `None`.
//!
//! The trigger-specific mechanics (the exact stream shapes, the
//! provider-specific receipt mapping, the subprocess kill semantics) stay
//! in each adapter's own test file; this module owns the CROSS-ADAPTER
//! run. The tests assert the certification's fail-closed verdict; the
//! report itself is the digest-pinned qualification record.

use reasonbraid_adapter::Adapter;

pub use reasonbraid_adapter::certification::{ConformanceScenario, Trigger};

/// Run the certification for one scenario: the verdict must pass (the
/// panic carries the report's first refusal — the test-time assertion
/// shape over the report-producing certification). The record's digest
/// re-derives and the SDK token matches before the verdict is trusted.
pub async fn run<A: Adapter>(
    scenario: ConformanceScenario<A>,
) -> reasonbraid_adapter::CertificationReport {
    let name = scenario.name.clone();
    let report =
        reasonbraid_adapter::certify(scenario, reasonbraid_adapter::dev_six_box_evidence()).await;
    assert!(
        report.digest_verifies(),
        "{name}: the qualification record's digest fails to re-derive"
    );
    assert!(
        report.sdk_version_matches(),
        "{name}: the adapter's contract version drifts from the SDK token"
    );
    match &report.verdict {
        reasonbraid_adapter::Verdict::Passed => {}
        reasonbraid_adapter::Verdict::Refused { first_refusal } => {
            panic!("{name}: the certification refused: {first_refusal}")
        }
    }
    report
}

/// The shared subprocess stubs (see [`stubs`]).
pub mod stubs;
