//! The adapter conformance harness (`PHASE-2.6.1`): ONE mechanical suite that every
//! adapter — the deterministic fake and every real CLI adapter — passes against the
//! [`reasonbraid_adapter::Adapter`] contract (§19.4's checkable items).
//!
//! Each adapter family registers CONFORMANCE SCENARIOS — an adapter instance wired to a
//! trigger (a refusal, a lost response, a hang, a completion) plus the capabilities it
//! DECLARES. The harness then asserts the §19.4 invariants that hold for every adapter:
//!
//! 1. **Capability-manifest agreement** — the declared [`AdapterCapabilities`] match
//!    the scenario's verified boundary (a non-streaming declaration refuses the
//!    streaming path, an unsupported status lookup is declared as such).
//! 2. **Unsupported-operation honesty** — a lookup for a never-dispatched operation is
//!    `Unsupported`, never a fabricated terminal and never a retry recommendation.
//! 3. **The dispatch boundary** — a refusal returns `FailedBeforeDispatch` BEFORE any
//!    provider contact; no attempt handle exists to leak past the boundary.
//! 4. **Ambiguous-outcome honesty** — a lost response ends the stream WITHOUT a
//!    terminal event; the adapter never invents a completion or a failure.
//! 5. **Cancellation matches the declared strength** — the `cancel` outcome never
//!    exceeds the declared `CancellationStrength` (a `BestEffort` adapter cannot
//!    claim `Confirmed`).
//! 6. **Usage accounting never lies** — `normalize_usage` on an empty receipt reports
//!    `Unknown` confidence with every dimension `None`: unknown is unknown, never
//!    zero (§14.5).
//!
//! The trigger-specific mechanics (the exact stream shapes, the provider-specific
//! receipt mapping, the subprocess kill semantics) stay in each adapter's own test
//! file; this module owns the CROSS-ADAPTER invariants.

use reasonbraid_adapter::{
    Adapter, AdapterCapabilities, AttemptEvent, CancellationOutcome, CancellationStrength,
    InvokeOutcome, RunRequest, StatusLookupOutcome, UsageConfidence,
};
use serde_json::json;

/// What a scenario's adapter is wired to do when invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// `invoke` refuses before any dispatch (the fake's `fail_before_dispatch` fixture).
    Refuse,
    /// The scenario's adapter is constructed against a MISSING binary: the spawn
    /// refusal is the deterministic pre-dispatch refusal of a subprocess adapter.
    RefuseMissingBinary,
    /// The dispatch is acknowledged, then the response is lost (no terminal event).
    Lose,
    /// The attempt hangs until cancelled (the hang is a cancellation Notify, never a
    /// real sleep — this harness never waits on a wall clock).
    Hang,
    /// The attempt completes with a terminal `Completed` event.
    Complete,
}

/// One conformance scenario: a named adapter instance wired to a trigger, with the
/// capabilities it declares and the request that trips the trigger.
pub struct ConformanceScenario<A> {
    pub name: String,
    pub adapter: A,
    pub trigger: Trigger,
    pub expected: AdapterCapabilities,
    pub request: RunRequest,
}

/// The cancelled-attempt outcome a declaration may at most produce.
fn ceiling(declared: CancellationStrength) -> CancellationOutcome {
    match declared {
        CancellationStrength::None => CancellationOutcome::Ignored,
        CancellationStrength::BestEffort => CancellationOutcome::BestEffort,
        CancellationStrength::Confirmed => CancellationOutcome::Confirmed,
    }
}

/// Drain an accepted attempt's handle, returning every event it produced.
async fn drain(handle: &mut reasonbraid_adapter::AttemptHandle) -> Vec<AttemptEvent> {
    let mut events = Vec::new();
    while let Some(event) = handle.next().await {
        events.push(event);
    }
    events
}

/// Run every cross-adapter §19.4 invariant against one scenario.
pub async fn run<A: Adapter>(scenario: ConformanceScenario<A>) {
    let ConformanceScenario {
        name,
        adapter,
        trigger,
        expected,
        request,
    } = scenario;
    let op = format!("{name}-op");

    // 1. The capability manifest agrees with the scenario's verified boundary.
    assert_eq!(
        adapter.capabilities(),
        expected,
        "{name}: the declared capabilities drifted from the verified boundary"
    );

    // 2. Unsupported-operation honesty: a never-dispatched operation is Unsupported —
    // never a fabricated terminal, never a retry recommendation.
    match adapter
        .query_status(&format!("never-dispatched-{name}"))
        .await
    {
        StatusLookupOutcome::Unsupported => {}
        other => panic!("{name}: a never-dispatched lookup fabricated {other:?}"),
    }

    match trigger {
        // 3. The dispatch boundary: refusal happens BEFORE any provider contact and no
        // handle exists.
        Trigger::Refuse | Trigger::RefuseMissingBinary => {
            match adapter.invoke(&request, &op).await {
                InvokeOutcome::FailedBeforeDispatch { .. } => {}
                InvokeOutcome::Accepted(..) => {
                    panic!("{name}: a refusal trigger crossed the dispatch boundary")
                }
            }
        }

        // 4. Ambiguous-outcome honesty: the lost response ends the stream WITHOUT a
        // terminal event — the adapter never invents a completion or a failure.
        Trigger::Lose => {
            let InvokeOutcome::Accepted(_, mut handle) = adapter.invoke(&request, &op).await else {
                panic!("{name}: the lose trigger refused instead of dispatching");
            };
            let events = drain(&mut handle).await;
            assert!(
                events.iter().all(|e| !matches!(
                    e,
                    AttemptEvent::Completed { .. } | AttemptEvent::FailedKnown { .. }
                )),
                "{name}: the lost response fabricated a terminal event: {events:?}"
            );
        }

        // 5. Cancellation matches the declared strength: the outcome never exceeds the
        // declaration.
        Trigger::Hang => {
            let InvokeOutcome::Accepted(_, _handle) = adapter.invoke(&request, &op).await else {
                panic!("{name}: the hang trigger refused instead of dispatching");
            };
            let outcome = adapter.cancel(&op).await;
            assert!(
                strength_of(outcome) <= strength_of(ceiling(expected.cancellation)),
                "{name}: cancel reported {outcome:?}, exceeding the declared {:?}",
                expected.cancellation
            );
        }

        // 6. The completion path: a terminal Completed event exists, it is the LAST
        // event, and usage accounting never lies (empty receipt → Unknown, None dims).
        Trigger::Complete => {
            let InvokeOutcome::Accepted(_, mut handle) = adapter.invoke(&request, &op).await else {
                panic!("{name}: the complete trigger refused instead of dispatching");
            };
            let events = drain(&mut handle).await;
            let terminal_at = events.iter().position(|e| {
                matches!(
                    e,
                    AttemptEvent::Completed { .. } | AttemptEvent::FailedKnown { .. }
                )
            });
            let terminal_at = terminal_at.unwrap_or_else(|| {
                panic!("{name}: the complete trigger produced no terminal event: {events:?}")
            });
            assert_eq!(
                terminal_at,
                events.len() - 1,
                "{name}: events arrived after the terminal event: {events:?}"
            );
            let normalized = adapter.normalize_usage(&json!({}));
            assert_eq!(
                normalized.confidence,
                UsageConfidence::Unknown,
                "{name}: an empty receipt must be Unknown confidence, not zero"
            );
            assert!(
                normalized.input_tokens.is_none()
                    && normalized.output_tokens.is_none()
                    && normalized.cost.is_none(),
                "{name}: an empty receipt fabricated dimensions: {normalized:?}"
            );
        }
    }
}

fn strength_of(outcome: CancellationOutcome) -> u8 {
    match outcome {
        CancellationOutcome::Ignored => 0,
        CancellationOutcome::BestEffort => 1,
        CancellationOutcome::Confirmed => 2,
    }
}
/// The shared subprocess stubs (see [`stubs`]).
pub mod stubs;
