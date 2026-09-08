//! The third-party certification demonstration (`PHASE-8.4.3`): a
//! NON-BUILT-IN adapter runs the SAME certification the dev three pass —
//! the honest vendor adapter earns the digest-pinned qualification
//! record; the violating vendor adapter earns the TYPED refusal (the
//! fail-closed verdict — a partial trust never certifies).

use std::future::Future;
use std::pin::Pin;

use reasonbraid_adapter::{
    certify, dev_six_box_evidence, Adapter, AdapterCapabilities, AttemptEvent, AttemptHandle,
    AttemptStream, BoxStatus, CancellationOutcome, CancellationStrength, ConformanceScenario,
    DispatchAck, InvokeOutcome, NormalizedUsage, PolicyInjectionMode, RunRequest,
    StatusLookupOutcome, Trigger, UsageConfidence, Verdict,
};
use serde_json::json;

/// A one-shot attempt stream (the terminal event, then None).
struct OneShotStream {
    state: Option<AttemptEvent>,
}

impl AttemptStream for OneShotStream {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<AttemptEvent>> + Send + '_>> {
        Box::pin(async move { self.state.take() })
    }
}

fn caps() -> AdapterCapabilities {
    AdapterCapabilities {
        streaming: false,
        cancellation: CancellationStrength::BestEffort,
        provider_idempotency: false,
        status_lookup: false,
        tool_support: false,
        policy_injection: PolicyInjectionMode::None,
    }
}

/// The honest vendor adapter: the minimal contract implementation with the
/// truthful usage accounting.
struct VendorAdapter;

impl Adapter for VendorAdapter {
    fn capabilities(&self) -> AdapterCapabilities {
        caps()
    }

    async fn invoke(&self, _request: &RunRequest, _operation_id: &str) -> InvokeOutcome {
        InvokeOutcome::Accepted(
            DispatchAck {
                provider_request_id: None,
            },
            AttemptHandle::new(Box::new(OneShotStream {
                state: Some(AttemptEvent::Completed { usage: None }),
            })),
        )
    }

    async fn cancel(&self, _operation_id: &str) -> CancellationOutcome {
        CancellationOutcome::BestEffort
    }

    async fn query_status(&self, _operation_id: &str) -> StatusLookupOutcome {
        StatusLookupOutcome::Unsupported
    }

    fn normalize_usage(&self, _raw_receipt: &serde_json::Value) -> NormalizedUsage {
        NormalizedUsage {
            input_tokens: None,
            output_tokens: None,
            cost: None,
            confidence: UsageConfidence::Unknown,
        }
    }
}

/// The lying vendor adapter: the usage accounting fabricates the exact
/// zeroed dimensions from an empty receipt (§14.5: unknown is unknown,
/// never zero).
struct LyingAdapter;

impl Adapter for LyingAdapter {
    fn capabilities(&self) -> AdapterCapabilities {
        caps()
    }

    async fn invoke(&self, _request: &RunRequest, _operation_id: &str) -> InvokeOutcome {
        InvokeOutcome::Accepted(
            DispatchAck {
                provider_request_id: None,
            },
            AttemptHandle::new(Box::new(OneShotStream {
                state: Some(AttemptEvent::Completed { usage: None }),
            })),
        )
    }

    async fn cancel(&self, _operation_id: &str) -> CancellationOutcome {
        CancellationOutcome::BestEffort
    }

    async fn query_status(&self, _operation_id: &str) -> StatusLookupOutcome {
        StatusLookupOutcome::Unsupported
    }

    fn normalize_usage(&self, _raw_receipt: &serde_json::Value) -> NormalizedUsage {
        NormalizedUsage {
            input_tokens: Some(0),
            output_tokens: Some(0),
            cost: Some(json!(0)),
            confidence: UsageConfidence::Exact,
        }
    }
}

fn complete_scenario<A: Adapter>(name: &str, adapter: A) -> ConformanceScenario<A> {
    ConformanceScenario {
        name: name.to_string(),
        adapter,
        trigger: Trigger::Complete,
        expected: caps(),
        request: RunRequest {
            payload: json!({ "prompt": "certify me" }),
            deadline: None,
            budget_hint: None,
        },
    }
}

/// The honest third-party adapter certifies: the verdict passes, the
/// record's self-digest re-derives, the SDK token matches, and the
/// conformance box is the Qualified evidence.
#[tokio::test]
async fn the_third_party_adapter_certifies_with_the_digest_pinned_record() {
    let report = certify(
        complete_scenario("vendor:honest", VendorAdapter),
        dev_six_box_evidence(),
    )
    .await;
    assert_eq!(report.verdict, Verdict::Passed, "the vendor certifies");
    assert!(
        report.digest_verifies(),
        "the record's self-digest re-derives: {}",
        report.digest
    );
    assert!(report.sdk_version_matches(), "the SDK token matches");
    assert!(matches!(
        report.six_box.conformance,
        BoxStatus::Qualified { .. }
    ));
    // The record round-trips (the release tool's certify verb parses it).
    let wire = serde_json::to_string(&report).expect("the record serializes");
    let parsed: reasonbraid_adapter::CertificationReport =
        serde_json::from_str(&wire).expect("the record parses");
    assert_eq!(parsed, report, "the wire roundtrip preserves the record");
}

/// The violating third-party adapter earns the TYPED refusal: the
/// fail-closed verdict names the invariant (never a partial trust).
#[tokio::test]
async fn the_violating_adapter_gets_the_typed_refusal() {
    let report = certify(
        complete_scenario("vendor:lying", LyingAdapter),
        dev_six_box_evidence(),
    )
    .await;
    match &report.verdict {
        Verdict::Refused { first_refusal } => {
            assert!(
                first_refusal.contains("usage_accounting_honesty"),
                "the refusal names the invariant: {first_refusal}"
            );
        }
        Verdict::Passed => panic!("the lying adapter must not certify"),
    }
    assert!(matches!(
        report.six_box.conformance,
        BoxStatus::Untested { .. }
    ));
    assert!(
        report.digest_verifies(),
        "even a refused record's digest re-derives (the evidence stays)"
    );
}
