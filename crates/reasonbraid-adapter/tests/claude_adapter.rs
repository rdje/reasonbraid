//! Offline tests for the Claude CLI adapter (`PHASE-1.4.1`): the supervision mechanics
//! are exercised against a STUB binary that mimics `claude -p --json`'s event stream
//! — no provider spend, fully deterministic. The real dispatch is the env-gated live
//! test in `crates/reasonbraid-node/tests/claude_live.rs`.
//!
//! The stub is a POSIX shell script written under the build dir, so the tests cover
//! the REAL subprocess boundary (spawn, JSONL parsing, exit-status verdicts, kill) —
//! only the provider behind the binary is fake. The cross-adapter invariants (the
//! capability manifest, the unsupported-operation honesty, the dispatch boundary, the
//! lost-response honesty, the cancellation ceiling, the usage-accounting floor) run in
//! the shared conformance harness (`tests/conformance/`); this file owns the
//! CLAUDE-specific mechanics.

#[path = "conformance/stubs.rs"]
#[allow(dead_code)]
// this binary uses only ITS provider's stub; the sibling serves the other adapter's tests
mod stubs;

use std::time::Duration;

use reasonbraid_adapter::{
    Adapter, AttemptEvent, CancellationOutcome, ClaudeCliAdapter, InvokeOutcome, RunRequest,
    UsageConfidence,
};
use serde_json::json;

fn adapter_with_stub(name: &str) -> ClaudeCliAdapter {
    ClaudeCliAdapter::with_binary(stubs::claude_binary(name))
}

fn request_with(prompt: &str) -> RunRequest {
    RunRequest {
        payload: json!({ "prompt": prompt }),
        deadline: None,
        budget_hint: None,
    }
}

/// The happy path: init surfaces the session id, each TEXT block streams as a chunk
/// (the thinking block is skipped), and the result completes with usage + money cost.
#[tokio::test]
async fn complete_path_streams_events_and_normalizes_usage() {
    let adapter = adapter_with_stub("complete");
    let InvokeOutcome::Accepted(ack, mut handle) =
        adapter.invoke(&request_with("ok"), "op_ok").await
    else {
        panic!("expected an accepted dispatch");
    };
    assert!(
        ack.provider_request_id.is_none(),
        "claude reveals its session id in the stream"
    );

    let mut events = Vec::new();
    while let Some(event) = handle.next().await {
        events.push(event);
    }
    assert_eq!(events.len(), 4);
    assert_eq!(
        events[0],
        AttemptEvent::ProviderRequestId {
            request_id: "stub_ok".to_string()
        }
    );
    assert_eq!(
        events[1],
        AttemptEvent::OutputChunk {
            chunk: "hello ok".to_string()
        }
    );
    assert_eq!(
        events[2],
        AttemptEvent::OutputChunk {
            chunk: "world".to_string()
        }
    );
    let AttemptEvent::Completed { usage } = &events[3] else {
        panic!("expected completion, got {:?}", events[3]);
    };
    let usage = adapter.normalize_usage(usage.as_ref().unwrap());
    assert_eq!(usage.input_tokens, Some(10));
    assert_eq!(usage.output_tokens, Some(3));
    assert_eq!(usage.confidence, UsageConfidence::Exact);
    // Claude reports MONEY: the cost is known (unlike Codex, tokens only).
    assert_eq!(usage.cost, Some(json!(0.001234)));
}

/// The run payload's `prompt` travels as the USER prompt (an argument), verbatim.
#[tokio::test]
async fn prompt_travels_as_user_content() {
    let adapter = adapter_with_stub("prompt");
    let InvokeOutcome::Accepted(_, mut handle) =
        adapter.invoke(&request_with("echo me"), "op_echo").await
    else {
        panic!("expected an accepted dispatch");
    };
    let mut chunks = Vec::new();
    while let Some(event) = handle.next().await {
        if let AttemptEvent::OutputChunk { chunk } = event {
            chunks.push(chunk);
        }
    }
    assert_eq!(
        chunks.first().map(String::as_str),
        Some("hello echo me"),
        "the prompt is user content, passed verbatim as the first chunk"
    );
}

/// A non-zero exit is a definitive, PROVEN failure — never a guess.
#[tokio::test]
async fn nonzero_exit_produces_failed_known_with_the_stderr_tail() {
    let adapter = adapter_with_stub("fail");
    let InvokeOutcome::Accepted(_, mut handle) = adapter
        .invoke(&request_with("please fail"), "op_fail")
        .await
    else {
        panic!("expected an accepted dispatch");
    };
    let mut terminal = None;
    while let Some(event) = handle.next().await {
        if let AttemptEvent::FailedKnown { reason } = event {
            terminal = Some(reason);
        }
    }
    let reason = terminal.expect("a non-zero exit must produce a terminal failure");
    assert!(reason.contains("simulated provider error"), "got: {reason}");
}

/// A `result` event with `is_error:true` is the provider TELLING us it failed — a
/// definitive failure carrying the provider's own message, even before the exit status.
#[tokio::test]
async fn is_error_result_is_a_definitive_failure_with_the_provider_message() {
    let adapter = adapter_with_stub("refuse");
    let InvokeOutcome::Accepted(_, mut handle) = adapter
        .invoke(&request_with("please refuse"), "op_refuse")
        .await
    else {
        panic!("expected an accepted dispatch");
    };
    let mut terminal = None;
    while let Some(event) = handle.next().await {
        if let AttemptEvent::FailedKnown { reason } = event {
            terminal = Some(reason);
        }
    }
    let reason = terminal.expect("an is_error result must produce a terminal failure");
    assert!(
        reason.contains("simulated provider refusal"),
        "got: {reason}"
    );
}

/// EOF with a clean exit but NO result event is a lost response: the stream ends
/// without a terminal event, and only a status lookup could prove the result.
#[tokio::test]
async fn lost_response_ends_the_stream_without_a_terminal_event() {
    let adapter = adapter_with_stub("lose");
    let InvokeOutcome::Accepted(_, mut handle) =
        adapter.invoke(&request_with("lose it"), "op_lose").await
    else {
        panic!("expected an accepted dispatch");
    };
    let first = handle.next().await;
    assert!(matches!(
        first,
        Some(AttemptEvent::ProviderRequestId { .. })
    ));
    assert_eq!(
        handle.next().await,
        None,
        "no terminal event after the lost response"
    );
}

/// A missing binary is a deterministic pre-dispatch refusal.
#[tokio::test]
async fn missing_binary_fails_before_dispatch() {
    let adapter = ClaudeCliAdapter::with_binary("/nonexistent/claude-binary");
    match adapter.invoke(&request_with("ok"), "op_missing").await {
        InvokeOutcome::FailedBeforeDispatch { reason, .. } => {
            assert!(reason.contains("failed to spawn"), "got: {reason}");
        }
        other => panic!("expected a pre-dispatch refusal, got {other:?}"),
    }
}

/// Cancellation kills the child and is reported `BestEffort` — the provider may or
/// may not stop; the boundary cannot claim `Confirmed`.
#[tokio::test]
async fn cancel_kills_the_child_and_reports_best_effort() {
    let adapter = adapter_with_stub("cancel");
    let InvokeOutcome::Accepted(_, mut handle) = adapter
        .invoke(&request_with("sleep please"), "op_sleep")
        .await
    else {
        panic!("expected an accepted dispatch");
    };
    assert!(matches!(
        handle.next().await,
        Some(AttemptEvent::ProviderRequestId { .. })
    ));

    assert_eq!(
        adapter.cancel("op_sleep").await,
        CancellationOutcome::BestEffort
    );

    // The killed child yields a definitive failure (signal exit), then the stream ends.
    let terminal = tokio::time::timeout(Duration::from_secs(5), async {
        let mut last = None;
        while let Some(event) = handle.next().await {
            last = Some(event);
        }
        last
    })
    .await
    .expect("the killed child must terminate promptly");
    assert!(matches!(terminal, Some(AttemptEvent::FailedKnown { .. })));
}

/// The real honesty leg: status lookup is Unsupported on this boundary.

#[tokio::test]
async fn normalize_usage_maps_claude_receipt_shapes() {
    let adapter = adapter_with_stub("usage");

    let exact = adapter.normalize_usage(&json!({
        "type": "result", "subtype": "success", "is_error": false,
        "usage": {
            "input_tokens": 100, "cache_read_input_tokens": 50,
            "cache_creation_input_tokens": 20, "output_tokens": 10,
            "output_tokens_details": { "thinking_tokens": 3 }
        },
        "total_cost_usd": 0.0017, "duration_ms": 250, "num_turns": 1
    }));
    assert_eq!(
        exact.input_tokens,
        Some(100),
        "Anthropic's input_tokens already includes cache reads — no folding"
    );
    assert_eq!(
        exact.output_tokens,
        Some(10),
        "Anthropic's output_tokens already includes thinking tokens — no folding"
    );
    assert_eq!(exact.cost, Some(json!(0.0017)));
    assert_eq!(exact.confidence, UsageConfidence::Exact);

    // A receipt with no money reports cost None (unknown), never zero.
    let no_cost = adapter.normalize_usage(&json!({
        "usage": { "input_tokens": 5, "output_tokens": 6 }
    }));
    assert_eq!(no_cost.cost, None);
    assert_eq!(no_cost.confidence, UsageConfidence::Exact);

    let unknown = adapter.normalize_usage(&json!({ "note": "no numbers" }));
    assert_eq!(unknown.input_tokens, None);
    assert_eq!(unknown.output_tokens, None);
    assert_eq!(unknown.cost, None);
    assert_eq!(unknown.confidence, UsageConfidence::Unknown);
}
