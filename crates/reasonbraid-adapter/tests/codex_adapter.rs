//! Offline tests for the Codex CLI adapter (`PHASE-0.4.2`): the supervision mechanics
//! are exercised against a STUB binary that mimics `codex exec --json`'s event stream
//! — no provider spend, fully deterministic. The real dispatch is the env-gated live
//! test in `crates/reasonbraid-node/tests/codex_live.rs`.
//!
//! The stub is a POSIX shell script written under the build dir, so the tests cover
//! the REAL subprocess boundary (spawn, JSONL parsing, exit-status verdicts, kill) —
//! only the provider behind the binary is fake.

use std::path::PathBuf;
use std::time::Duration;

use reasonbraid_adapter::{
    Adapter, AttemptEvent, CancellationOutcome, CodexCliAdapter, InvokeOutcome, RunRequest,
    StatusLookupOutcome, UsageConfidence,
};
use serde_json::json;

/// Write the stub `codex` script (branches on the prompt argument) and return its path.
fn stub_binary(name: &str) -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = base.join("codex-stubs").join(format!("{name}-{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("codex");
    let script = r#"#!/bin/sh
# The adapter invokes: <binary> exec --json ... <prompt> — the prompt is the LAST arg.
for last in "$@"; do :; done
case "$last" in
  *fail*)
    echo '{"type":"thread.started","thread_id":"stub_fail"}'
    echo "boom: simulated provider error" >&2
    exit 2
    ;;
  *lose*)
    echo '{"type":"thread.started","thread_id":"stub_lose"}'
    exit 0
    ;;
  *sleep*)
    echo '{"type":"thread.started","thread_id":"stub_sleep"}'
    # Busy-loop with NO child process: the killed script IS the pipe holder, so
    # start_kill ends the stream immediately (a `sleep` child would inherit the
    # stdout pipe and delay EOF past the test's bound).
    while :; do :; done
    ;;
  *)
    echo '{"type":"thread.started","thread_id":"stub_ok"}'
    echo '{"type":"item.completed","item":{"type":"agent_message","text":"hello '"$last"'"}}'
    echo '{"type":"turn.completed","usage":{"input_tokens":10,"output_tokens":2}}'
    exit 0
    ;;
esac
"#;
    std::fs::write(&path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}

fn adapter_with_stub(name: &str) -> CodexCliAdapter {
    CodexCliAdapter::with_binary(stub_binary(name))
}

fn request_with(prompt: &str) -> RunRequest {
    RunRequest {
        payload: json!({ "prompt": prompt }),
        deadline: None,
        budget_hint: None,
    }
}

/// The happy path: the stream surfaces the provider handle, the chunk, and the
/// completion with an exact usage receipt.
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
        "codex reveals its id in the stream"
    );

    let mut events = Vec::new();
    while let Some(event) = handle.next().await {
        events.push(event);
    }
    assert_eq!(events.len(), 3);
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
    let AttemptEvent::Completed { usage } = &events[2] else {
        panic!("expected completion, got {:?}", events[2]);
    };
    let usage = adapter.normalize_usage(usage.as_ref().unwrap());
    assert_eq!(usage.input_tokens, Some(10));
    assert_eq!(usage.output_tokens, Some(2));
    assert_eq!(usage.confidence, UsageConfidence::Exact);
    assert_eq!(
        usage.cost, None,
        "tokens are reported, cost is not — unknown, never zero"
    );
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
    let mut saw_chunk = false;
    while let Some(event) = handle.next().await {
        if let AttemptEvent::OutputChunk { chunk } = event {
            assert_eq!(
                chunk, "hello echo me",
                "the prompt is user content, passed verbatim"
            );
            saw_chunk = true;
        }
    }
    assert!(saw_chunk);
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

/// EOF with a clean exit but NO turn.completed event is a lost response: the stream
/// ends without a terminal event, and only a status lookup could prove the result.
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
    let adapter = CodexCliAdapter::with_binary("/nonexistent/codex-binary");
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
async fn query_status_is_unsupported() {
    let adapter = adapter_with_stub("lookup");
    assert!(matches!(
        adapter.query_status("op_whatever").await,
        StatusLookupOutcome::Unsupported
    ));
}

/// The Codex receipt shape: reasoning tokens fold into output; unknown shapes stay
/// unknown rather than zero.
#[tokio::test]
async fn normalize_usage_maps_codex_receipt_shapes() {
    let adapter = adapter_with_stub("usage");

    let exact = adapter.normalize_usage(&json!({
        "input_tokens": 100, "cached_input_tokens": 40, "output_tokens": 10,
        "reasoning_output_tokens": 5
    }));
    assert_eq!(exact.input_tokens, Some(100));
    assert_eq!(
        exact.output_tokens,
        Some(15),
        "reasoning tokens are output tokens"
    );
    assert_eq!(exact.confidence, UsageConfidence::Exact);

    let unknown = adapter.normalize_usage(&json!({ "note": "no numbers" }));
    assert_eq!(unknown.input_tokens, None);
    assert_eq!(unknown.output_tokens, None);
    assert_eq!(unknown.confidence, UsageConfidence::Unknown);
}

/// The declared capabilities match the verified boundary: streaming yes, lookup no,
/// idempotency no, cancellation best-effort.
#[tokio::test]
async fn capabilities_match_the_verified_boundary() {
    let caps = adapter_with_stub("caps").capabilities();
    assert!(caps.streaming);
    assert!(!caps.status_lookup);
    assert!(!caps.provider_idempotency);
    assert_eq!(
        caps.cancellation,
        reasonbraid_adapter::CancellationStrength::BestEffort
    );
}
