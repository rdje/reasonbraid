//! The certification suite (`PHASE-8.4.3`, ADR-027, §19.4): the SAME six
//! cross-adapter invariants the conformance harness asserts, run as the
//! REPORT-producing certification a third-party adapter goes through. The
//! verdict is FAIL-CLOSED — ANY refusal refuses the whole certification
//! (never a partial trust) — and the report is the digest-pinned
//! qualification record (`sha256:<hex>` over the canonical JSON); the
//! signature rides the release tool's `certify` command (the ADR-027
//! identity + the manifest pattern).

use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    Adapter, AdapterCapabilities, AttemptEvent, CancellationOutcome, CancellationStrength,
    InvokeOutcome, RunRequest, StatusLookupOutcome, UsageConfidence, SDK_VERSION,
};

/// What a certification scenario's adapter is wired to do when invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// `invoke` refuses before any dispatch.
    Refuse,
    /// The scenario's adapter is constructed against a MISSING binary: the
    /// spawn refusal is the deterministic pre-dispatch refusal.
    RefuseMissingBinary,
    /// The dispatch is acknowledged, then the response is lost (no terminal
    /// event).
    Lose,
    /// The attempt hangs until cancelled (the harness never waits on a wall
    /// clock — the hang is a cancellation Notify in the deterministic
    /// adapters).
    Hang,
    /// The attempt completes with a terminal `Completed` event.
    Complete,
}

/// One certification scenario: a named adapter instance wired to a trigger,
/// with the capabilities it declares and the request that trips the trigger.
pub struct ConformanceScenario<A> {
    pub name: String,
    pub adapter: A,
    pub trigger: Trigger,
    pub expected: AdapterCapabilities,
    pub request: RunRequest,
}

/// One invariant's result — a typed pass or a typed refusal (the refusal
/// vocabulary: the invariant's name + the reason, never a bare false).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CheckResult {
    Passed { check: String },
    Refused { check: String, reason: String },
}

impl CheckResult {
    fn passed(check: &str) -> Self {
        CheckResult::Passed {
            check: check.to_string(),
        }
    }

    fn refused(check: &str, reason: impl Into<String>) -> Self {
        CheckResult::Refused {
            check: check.to_string(),
            reason: reason.into(),
        }
    }
}

/// The certification verdict: the all-pass, or the typed first refusal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum Verdict {
    Passed,
    Refused { first_refusal: String },
}

/// The six-box gate's per-box status (the book's qualification checklist):
/// a box is the measured `Qualified` (with its evidence) or the explicit
/// `Untested` (with the reason — never blank, never inferred).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BoxStatus {
    Qualified { evidence: String },
    Untested { reason: String },
}

/// The six-box evidence the operator submits with a certification run (the
/// conformance box rides the certification verdict itself — the caller
/// cannot self-attest it).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SixBoxEvidence {
    /// The stub-mechanics pass (the scenario's deterministic fixtures).
    pub stub_mechanics: BoxStatus,
    /// The env-gated bounded live dispatch (the real-provider run).
    pub live_dispatch: BoxStatus,
    /// The credential containment (the contract's no-credential-field rule).
    pub credential_containment: BoxStatus,
    /// The release manifest entry (the ADR-027 manifest row).
    pub manifest_entry: BoxStatus,
    /// The dependency-ledger row (the deny gate + the committed lockfile).
    pub dependency_ledger: BoxStatus,
}

/// The certification report — the digest-pinned qualification record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CertificationReport {
    /// The scenario's name (the adapter's qualified identity in the record).
    pub scenario: String,
    /// The contract version the adapter reported.
    pub sdk_version: String,
    /// The six invariants' results.
    pub checks: Vec<CheckResult>,
    /// The fail-closed verdict.
    pub verdict: Verdict,
    /// The six-box gate's evaluation (the conformance box mirrors the
    /// verdict; the rest ride the submitted evidence).
    pub six_box: SixBoxRecord,
    /// The record's own digest (`sha256:<hex>` over the canonical JSON
    /// WITHOUT this field) — the ADR-027 re-derivation anchor.
    pub digest: String,
}

/// The six-box gate's evaluated record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SixBoxRecord {
    pub conformance: BoxStatus,
    pub stub_mechanics: BoxStatus,
    pub live_dispatch: BoxStatus,
    pub credential_containment: BoxStatus,
    pub manifest_entry: BoxStatus,
    pub dependency_ledger: BoxStatus,
}

fn strength_of(outcome: CancellationOutcome) -> u8 {
    match outcome {
        CancellationOutcome::Ignored => 0,
        CancellationOutcome::BestEffort => 1,
        CancellationOutcome::Confirmed => 2,
    }
}

fn ceiling(declared: CancellationStrength) -> CancellationOutcome {
    match declared {
        CancellationStrength::None => CancellationOutcome::Ignored,
        CancellationStrength::BestEffort => CancellationOutcome::BestEffort,
        CancellationStrength::Confirmed => CancellationOutcome::Confirmed,
    }
}

async fn drain(handle: &mut crate::AttemptHandle) -> Vec<AttemptEvent> {
    let mut events = Vec::new();
    while let Some(event) = handle.next().await {
        events.push(event);
    }
    events
}

/// Run the certification: the six §19.4 invariants against one scenario,
/// then the six-box gate over the verdict + the submitted evidence. The
/// report collects EVERY check's result (the evidence, not the first
/// failure) and the fail-closed verdict names the first refusal.
pub async fn certify<A: Adapter>(
    scenario: ConformanceScenario<A>,
    boxes: SixBoxEvidence,
) -> CertificationReport {
    let ConformanceScenario {
        name,
        adapter,
        trigger,
        expected,
        request,
    } = scenario;
    let op = format!("{name}-cert-op");
    let mut checks = Vec::new();

    // 1. The capability manifest agrees with the verified boundary.
    let caps = adapter.capabilities();
    checks.push(if caps == expected {
        CheckResult::passed("capability_manifest_agreement")
    } else {
        CheckResult::refused(
            "capability_manifest_agreement",
            format!("the declared {caps:?} drifted from the verified {expected:?}"),
        )
    });

    // 2. Unsupported-operation honesty: a never-dispatched lookup is
    //    Unsupported — never a fabricated terminal, never a retry
    //    recommendation.
    checks.push(
        match adapter
            .query_status(&format!("never-dispatched-{name}"))
            .await
        {
            StatusLookupOutcome::Unsupported => CheckResult::passed("unsupported_lookup_honesty"),
            other => CheckResult::refused(
                "unsupported_lookup_honesty",
                format!("a never-dispatched lookup fabricated {other:?}"),
            ),
        },
    );

    match trigger {
        // 3. The dispatch boundary: the refusal happens BEFORE any provider
        //    contact; no attempt handle leaks past the boundary.
        Trigger::Refuse | Trigger::RefuseMissingBinary => {
            checks.push(match adapter.invoke(&request, &op).await {
                InvokeOutcome::FailedBeforeDispatch { .. } => {
                    CheckResult::passed("dispatch_boundary")
                }
                InvokeOutcome::Accepted(..) => CheckResult::refused(
                    "dispatch_boundary",
                    "a refusal trigger crossed the dispatch boundary",
                ),
            });
        }

        // 4. Ambiguous-outcome honesty: the lost response ends the stream
        //    WITHOUT a terminal event — never an invented completion/failure.
        Trigger::Lose => {
            let result = match adapter.invoke(&request, &op).await {
                InvokeOutcome::Accepted(_, handle) => {
                    let mut handle = handle;
                    let events = drain(&mut handle).await;
                    if events.iter().all(|e| {
                        !matches!(
                            e,
                            AttemptEvent::Completed { .. } | AttemptEvent::FailedKnown { .. }
                        )
                    }) {
                        CheckResult::passed("ambiguous_outcome_honesty")
                    } else {
                        CheckResult::refused(
                            "ambiguous_outcome_honesty",
                            format!("the lost response fabricated a terminal event: {events:?}"),
                        )
                    }
                }
                InvokeOutcome::FailedBeforeDispatch { reason, .. } => CheckResult::refused(
                    "ambiguous_outcome_honesty",
                    format!("the lose trigger refused instead of dispatching: {reason}"),
                ),
            };
            checks.push(result);
        }

        // 5. Cancellation matches the declared strength: the outcome never
        //    exceeds the declaration.
        Trigger::Hang => {
            let result = match adapter.invoke(&request, &op).await {
                InvokeOutcome::Accepted(..) => {
                    let outcome = adapter.cancel(&op).await;
                    if strength_of(outcome) <= strength_of(ceiling(expected.cancellation)) {
                        CheckResult::passed("cancellation_matches_declaration")
                    } else {
                        CheckResult::refused(
                            "cancellation_matches_declaration",
                            format!(
                                "cancel reported {outcome:?}, exceeding the declared {:?}",
                                expected.cancellation
                            ),
                        )
                    }
                }
                InvokeOutcome::FailedBeforeDispatch { reason, .. } => CheckResult::refused(
                    "cancellation_matches_declaration",
                    format!("the hang trigger refused instead of dispatching: {reason}"),
                ),
            };
            checks.push(result);
        }

        // 6. The completion path: a terminal Completed event exists, it is
        //    the LAST event, and the usage accounting never lies (the empty
        //    receipt → Unknown confidence, every dimension None).
        Trigger::Complete => {
            let result = match adapter.invoke(&request, &op).await {
                InvokeOutcome::Accepted(_, handle) => {
                    let mut handle = handle;
                    let events = drain(&mut handle).await;
                    let terminal_at = events.iter().position(|e| {
                        matches!(
                            e,
                            AttemptEvent::Completed { .. } | AttemptEvent::FailedKnown { .. }
                        )
                    });
                    let Some(terminal_at) = terminal_at else {
                        checks.push(CheckResult::refused(
                            "terminal_completion",
                            format!("the complete trigger produced no terminal event: {events:?}"),
                        ));
                        return finish(name, adapter.sdk_version().to_string(), checks, boxes);
                    };
                    if terminal_at != events.len() - 1 {
                        checks.push(CheckResult::refused(
                            "terminal_completion",
                            format!("events arrived after the terminal event: {events:?}"),
                        ));
                        return finish(name, adapter.sdk_version().to_string(), checks, boxes);
                    }
                    let normalized = adapter.normalize_usage(&json!({}));
                    if normalized.confidence != UsageConfidence::Unknown
                        || normalized.input_tokens.is_some()
                        || normalized.output_tokens.is_some()
                        || normalized.cost.is_some()
                    {
                        checks.push(CheckResult::refused(
                            "usage_accounting_honesty",
                            format!("an empty receipt fabricated dimensions: {normalized:?}"),
                        ));
                        return finish(name, adapter.sdk_version().to_string(), checks, boxes);
                    }
                    CheckResult::passed("terminal_completion")
                }
                InvokeOutcome::FailedBeforeDispatch { reason, .. } => CheckResult::refused(
                    "terminal_completion",
                    format!("the complete trigger refused instead of dispatching: {reason}"),
                ),
            };
            checks.push(result);
        }
    }

    finish(name, adapter.sdk_version().to_string(), checks, boxes)
}

fn finish(
    name: String,
    sdk_version: String,
    checks: Vec<CheckResult>,
    boxes: SixBoxEvidence,
) -> CertificationReport {
    let first_refusal = checks.iter().find_map(|c| match c {
        CheckResult::Refused { check, reason } => Some(format!("{check}: {reason}")),
        CheckResult::Passed { .. } => None,
    });
    let verdict = match first_refusal {
        Some(first) => Verdict::Refused {
            first_refusal: first,
        },
        None => Verdict::Passed,
    };
    // The conformance box mirrors the verdict (the caller cannot self-attest it).
    let conformance = match &verdict {
        Verdict::Passed => BoxStatus::Qualified {
            evidence: "the six §19.4 invariants passed".to_string(),
        },
        Verdict::Refused { first_refusal } => BoxStatus::Untested {
            reason: format!("the certification refused: {first_refusal}"),
        },
    };
    let mut report = CertificationReport {
        scenario: name,
        sdk_version,
        checks,
        verdict,
        six_box: SixBoxRecord {
            conformance,
            stub_mechanics: boxes.stub_mechanics,
            live_dispatch: boxes.live_dispatch,
            credential_containment: boxes.credential_containment,
            manifest_entry: boxes.manifest_entry,
            dependency_ledger: boxes.dependency_ledger,
        },
        digest: String::new(),
    };
    report.digest = report.digest_hex();
    report
}

impl CertificationReport {
    /// The record's digest anchor: `sha256:<hex>` over the canonical JSON
    /// WITHOUT the digest field (the ADR-027 re-derivation — a record's
    /// digest is re-computable from the record alone).
    pub fn digest_hex(&self) -> String {
        let mut canonical = self.clone();
        canonical.digest = String::new();
        let bytes = serde_json::to_string(&canonical).expect("the report serializes");
        let hash = Sha256::digest(bytes.as_bytes());
        format!("sha256:{}", hex_lower(&hash))
    }

    /// Verify the record's self-digest (the re-derive check the verifier
    /// runs before the signature).
    pub fn digest_verifies(&self) -> bool {
        self.digest == self.digest_hex()
    }

    /// The adapter's reported contract version matches the SDK's token —
    /// the first axis of the compatibility matrix.
    pub fn sdk_version_matches(&self) -> bool {
        self.sdk_version == SDK_VERSION
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The dev profile's six-box evidence for the BUILT-IN adapters: the
/// measured boxes + the honest named-untested boxes (the env-gated live
/// dispatch + the release-manifest rows generate at `make release`).
pub fn dev_six_box_evidence() -> SixBoxEvidence {
    SixBoxEvidence {
        stub_mechanics: BoxStatus::Qualified {
            evidence: "the deterministic fixture corpus (`fixtures/MANIFEST.json`)".to_string(),
        },
        live_dispatch: BoxStatus::Untested {
            reason: "the env-gated `RB_LIVE_*` run produces no CI measurement".to_string(),
        },
        credential_containment: BoxStatus::Qualified {
            evidence: "the contract carries no credential field (contract.rs)".to_string(),
        },
        manifest_entry: BoxStatus::Untested {
            reason: "the release manifests generate at `make release`".to_string(),
        },
        dependency_ledger: BoxStatus::Qualified {
            evidence: "the deny gate + the committed Cargo.lock".to_string(),
        },
    }
}
