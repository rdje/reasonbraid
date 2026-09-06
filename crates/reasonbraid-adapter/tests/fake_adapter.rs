//! Integration tests for the deterministic fake adapter (`PHASE-0.4.1`): the
//! conformance-oracle behaviors — stream, fail, hang, report usage, ignore
//! cancellation, lose a response after dispatch — driven directly against the
//! [`reasonbraid_adapter::Adapter`] contract, with no sleeps anywhere (the hang is a
//! cancellation Notify, so the only timeout here is the negative bound that PROVES the
//! hang).

use std::time::Duration;

use reasonbraid_adapter::fixtures::corpus;
use reasonbraid_adapter::{
    Adapter, AttemptEvent, AttemptResult, CancellationOutcome, FakeAdapter, InvokeOutcome,
    RunRequest, StatusLookupOutcome, UsageConfidence,
};
use serde_json::json;

fn request() -> RunRequest {
    RunRequest {
        payload: json!({ "operation": "contribute" }),
        deadline: None,
        budget_hint: None,
    }
}

fn fixture(name: &str) -> FakeAdapter {
    let spec = corpus()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("no fixture named {name}"));
    FakeAdapter::from_spec(spec)
}

/// The refusal path: `invoke` fails BEFORE any dispatch, and no attempt handle exists.
#[tokio::test]
async fn fail_before_dispatch_returns_a_refusal_without_a_handle() {
    let adapter = fixture("fail_before_dispatch");
    match adapter.invoke(&request(), "op_refuse").await {
        InvokeOutcome::FailedBeforeDispatch { reason, .. } => {
            assert!(reason.contains("not allowed"));
        }
        other => panic!("expected a pre-dispatch refusal, got {other:?}"),
    }
}

/// Determinism: the same script produces the identical event sequence every run.
#[tokio::test]
async fn script_replay_is_deterministic() {
    let adapter = fixture("complete_streaming");
    let drive = || async {
        let mut events = Vec::new();
        if let InvokeOutcome::Accepted(_, mut handle) = adapter.invoke(&request(), "op_det").await {
            while let Some(event) = handle.next().await {
                events.push(event);
            }
        }
        events
    };
    let first = drive().await;
    let second = drive().await;
    assert_eq!(first.len(), 3, "two chunks + a terminal event");
    assert_eq!(first, second, "identical script → identical event sequence");
}

/// The hang: no event until cancellation, and the cancellation is CONFIRMED — the
/// only timing in the whole fake is this negative bound proving the hang.
#[tokio::test]
async fn hang_forever_produces_nothing_until_a_confirmed_cancel() {
    let adapter = fixture("hang_forever");
    let InvokeOutcome::Accepted(_, mut handle) = adapter.invoke(&request(), "op_hang").await else {
        panic!("hang script must accept the dispatch");
    };

    // The stream hangs: nothing arrives within the bound.
    let bounded = tokio::time::timeout(Duration::from_millis(100), handle.next()).await;
    assert!(bounded.is_err(), "a hang must not emit events");

    // Cancellation is confirmed, and only then does the stream end (with NO terminal
    // event — the result is indeterminate, recoverable only via status lookup).
    assert_eq!(
        adapter.cancel("op_hang").await,
        CancellationOutcome::Confirmed
    );
    assert_eq!(
        handle.next().await,
        None,
        "cancelled stream ends without a result"
    );
}

/// Cancellation the provider IGNORES: reported honestly, and the run continues.
#[tokio::test]
async fn ignore_cancellation_is_reported_and_the_script_continues() {
    let adapter = fixture("ignore_cancellation");
    let InvokeOutcome::Accepted(_, mut handle) = adapter.invoke(&request(), "op_ignore").await
    else {
        panic!("expected an accepted dispatch");
    };
    assert_eq!(
        adapter.cancel("op_ignore").await,
        CancellationOutcome::Ignored
    );
    assert_eq!(
        handle.next().await,
        Some(AttemptEvent::Completed { usage: None })
    );
}

/// Best-effort cancellation on a plain script: reported as such, stream unaffected.
#[tokio::test]
async fn best_effort_cancel_leaves_a_normal_stream_running() {
    let adapter = fixture("complete_streaming");
    let InvokeOutcome::Accepted(_, mut handle) = adapter.invoke(&request(), "op_be").await else {
        panic!("expected an accepted dispatch");
    };
    assert_eq!(
        adapter.cancel("op_be").await,
        CancellationOutcome::BestEffort
    );
    assert!(matches!(
        handle.next().await,
        Some(AttemptEvent::OutputChunk { .. })
    ));
}

/// A lost response ends the stream with NO terminal event — the caller must look up.
#[tokio::test]
async fn lose_response_ends_the_stream_without_a_terminal_event() {
    let adapter = fixture("lose_response_no_lookup");
    let InvokeOutcome::Accepted(ack, mut handle) = adapter.invoke(&request(), "op_lost").await
    else {
        panic!("expected an accepted dispatch");
    };
    assert!(
        ack.provider_request_id.is_none(),
        "no lookup → no provider handle"
    );
    assert_eq!(
        handle.next().await,
        None,
        "stream ends with no terminal event"
    );
    assert_eq!(handle.next().await, None);
}

/// Status lookup is a capability fact: unsupported means the fate is indeterminate —
/// the contract carries no retry recommendation anywhere in it.
#[tokio::test]
async fn query_status_is_unsupported_when_the_provider_offers_no_lookup() {
    let adapter = fixture("lose_response_no_lookup");
    let InvokeOutcome::Accepted(_, mut handle) = adapter.invoke(&request(), "op_q").await else {
        panic!("expected an accepted dispatch");
    };
    assert_eq!(handle.next().await, None);
    assert!(matches!(
        adapter.query_status("op_q").await,
        StatusLookupOutcome::Unsupported
    ));
}

/// When the provider CAN prove the result, the lookup returns it — the §11.3 recovery
/// path — and the dispatch ack carried the provider request handle.
#[tokio::test]
async fn query_status_proves_the_result_when_the_provider_supports_it() {
    let adapter = fixture("lose_response_with_lookup");
    let InvokeOutcome::Accepted(ack, mut handle) = adapter.invoke(&request(), "op_proof").await
    else {
        panic!("expected an accepted dispatch");
    };
    assert_eq!(
        ack.provider_request_id.as_deref(),
        Some("prv_op_proof"),
        "lookup-capable providers expose a request handle in the ack"
    );
    assert_eq!(handle.next().await, None);
    match adapter.query_status("op_proof").await {
        StatusLookupOutcome::Supported(AttemptResult::Completed { usage }) => {
            assert!(usage.is_some(), "the proof carries the usage receipt");
        }
        other => panic!("expected a proven completion, got {other:?}"),
    }
}

/// Usage normalization maps receipt shapes deterministically; unknown dimensions stay
/// `None` — never zero, never guessed (`§14.1`).
#[tokio::test]
async fn normalize_usage_maps_receipt_shapes() {
    let adapter = fixture("complete_single");

    let exact = adapter.normalize_usage(&json!({
        "input_tokens": 120, "output_tokens": 40, "exact": true,
        "cost": { "currency": "usd", "amount": 0.01 }
    }));
    assert_eq!(exact.input_tokens, Some(120));
    assert_eq!(exact.output_tokens, Some(40));
    assert!(exact.cost.is_some());
    assert_eq!(exact.confidence, UsageConfidence::Exact);

    let estimated = adapter.normalize_usage(&json!({ "input_tokens": 100 }));
    assert_eq!(estimated.input_tokens, Some(100));
    assert_eq!(
        estimated.output_tokens, None,
        "unmetered is unknown, not zero"
    );
    assert_eq!(estimated.confidence, UsageConfidence::Estimated);

    let unknown = adapter.normalize_usage(&json!({}));
    assert_eq!(unknown.confidence, UsageConfidence::Unknown);
    assert_eq!(unknown.input_tokens, None);
}

/// Capabilities come from the spec — the caller branches on these, never on names.
#[tokio::test]
async fn capabilities_are_reported_from_the_spec() {
    let streaming = fixture("complete_streaming").capabilities();
    assert!(streaming.streaming);
    assert!(!streaming.status_lookup);

    let lookup = fixture("lose_response_with_lookup").capabilities();
    assert!(lookup.status_lookup);
    assert!(lookup.provider_idempotency);
}

/// Every script step variant parses from the corpus (a serde-shape regression guard).
#[test]
fn corpus_scripts_deserialize_every_step_shape() {
    let mut seen = std::collections::HashSet::new();
    for spec in corpus() {
        for step in &spec.script {
            seen.insert(step.tag());
        }
    }
    for tag in [
        "emit_chunk",
        "malformed_output",
        "complete",
        "fail_known",
        "fail_before_dispatch",
        "hang_forever",
        "ignore_cancellation",
        "lose_response",
    ] {
        assert!(seen.contains(tag), "step `{tag}` never deserialized");
    }
}

/// The malformed-output step passes garbage through VERBATIM as an opaque chunk —
/// nothing parses it into domain meaning at this boundary.
#[tokio::test]
async fn malformed_output_passes_through_verbatim() {
    let adapter = fixture("malformed_output");
    let InvokeOutcome::Accepted(_, mut handle) = adapter.invoke(&request(), "op_mal").await else {
        panic!("expected an accepted dispatch");
    };
    let mut chunks = Vec::new();
    while let Some(event) = handle.next().await {
        match event {
            AttemptEvent::OutputChunk { chunk } => chunks.push(chunk),
            AttemptEvent::Completed { .. } => {}
            AttemptEvent::FailedKnown { reason } => panic!("unexpected failure: {reason}"),
        }
    }
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0], "valid prefix");
    assert_eq!(chunks[1], "\u{1f}\u{0} garbage {{{ not-json");
}
