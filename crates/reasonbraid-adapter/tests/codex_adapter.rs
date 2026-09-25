//! Offline tests for the Codex CLI adapter (`PHASE-0.4.2`): the supervision mechanics
//! are exercised against a STUB binary that mimics `codex exec --json`'s event stream
//! — no provider spend, fully deterministic. The real dispatch is the env-gated live
//! test in `crates/reasonbraid-node/tests/codex_live.rs`.
//!
//! The stub is a POSIX shell script written under the build dir, so the tests cover
//! the REAL subprocess boundary (spawn, JSONL parsing, exit-status verdicts, kill) —
//! only the provider behind the binary is fake. The cross-adapter invariants (the
//! capability manifest, the unsupported-operation honesty, the dispatch boundary, the
//! lost-response honesty, the cancellation ceiling, the usage-accounting floor) run in
//! the shared conformance harness (`tests/conformance/`); this file owns the
//! CODEX-specific mechanics.

#[path = "conformance/stubs.rs"]
#[allow(dead_code)]
// this binary uses only ITS provider's stub; the sibling serves the other adapter's tests
mod stubs;

use std::time::Duration;

use reasonbraid_adapter::{
    Adapter, AttemptEvent, CancellationOutcome, CodexCliAdapter, InvokeOutcome, RunRequest,
    UsageConfidence,
};
use serde_json::json;

fn adapter_with_stub(name: &str) -> CodexCliAdapter {
    CodexCliAdapter::with_binary(stubs::codex_binary(name))
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

/// `SIGNOFF-REPAIR.10.1.1`: a prompt is never read as a command-line option.
///
/// The prompt is untrusted participant content (`§16.6`). Passed as a bare
/// argument, a prompt beginning with `-` lands in the provider CLI's option
/// parser AFTER `--sandbox read-only`, the flag `ACTION-BOUNDARY` pins as what
/// keeps model output from reaching an action. The stub replies with the argv
/// it RECEIVED, so this asserts the argv itself: `--` immediately before the
/// prompt, the prompt last and whole.
#[tokio::test]
async fn a_prompt_is_never_read_as_an_option() {
    let adapter = adapter_with_stub("argv");
    let prompt = "--argv-probe=--sandbox=danger-full-access";
    let InvokeOutcome::Accepted(_, mut handle) =
        adapter.invoke(&request_with(prompt), "op_argv").await
    else {
        panic!("expected an accepted dispatch");
    };
    let mut argv = None;
    while let Some(event) = handle.next().await {
        if let AttemptEvent::OutputChunk { chunk } = event {
            argv = Some(chunk);
        }
    }
    let argv = argv.expect("the stub replies with its argv");
    let args: Vec<&str> = argv.split('|').collect();
    assert_eq!(
        args.last(),
        Some(&prompt),
        "the prompt is the last argument, whole: {args:?}"
    );
    assert_eq!(
        args.get(args.len() - 2),
        Some(&"--"),
        "`--` ends the options before the prompt: {args:?}"
    );
    // Exactly ONE `--`: a second one earlier would turn every flag after it
    // into positional text, the sandbox included, while the prompt stayed safe.
    assert_eq!(
        args.iter().filter(|a| **a == "--").count(),
        1,
        "`--` appears once, just before the prompt: {args:?}"
    );
    let sandbox = args
        .iter()
        .position(|a| *a == "--sandbox")
        .expect("the sandbox flag");
    assert_eq!(args[sandbox + 1], "read-only", "{args:?}");
}

/// Run one stubbed attempt to its end within `bound`, collecting its events.
/// `None` means the stream did not END in time: the stall this leaf's third
/// control exists to catch.
async fn run_to_end(prompt: &str, bound: std::time::Duration) -> Option<Vec<AttemptEvent>> {
    let adapter = adapter_with_stub("bounds");
    let InvokeOutcome::Accepted(_, mut handle) = adapter
        .invoke(&request_with(prompt), &format!("op_{prompt}"))
        .await
    else {
        panic!("expected an accepted dispatch");
    };
    tokio::time::timeout(bound, async {
        let mut events = Vec::new();
        while let Some(event) = handle.next().await {
            events.push(event);
        }
        events
    })
    .await
    .ok()
}

/// `SIGNOFF-REPAIR.10.1.2`: a stderr tail that ends mid-character is not a
/// panic. The old tail was `buf[buf.len() - 1024..]`, a BYTE slice.
#[tokio::test]
async fn a_multibyte_stderr_tail_is_a_failure_not_a_panic() {
    let events = run_to_end("stderr-utf8", std::time::Duration::from_secs(20))
        .await
        .expect("the attempt ends");
    let Some(AttemptEvent::FailedKnown { reason }) = events.last() else {
        panic!("a failing exit is a known failure: {events:?}");
    };
    assert!(
        reason.contains('é'),
        "the tail is carried, whole characters only: {reason}"
    );
}

/// `SIGNOFF-REPAIR.10.1.2`: an event line longer than the adapter's bound is a
/// known failure naming the bound, as the supervisor treats output over ITS
/// bound. The old reader allocated the whole line and then skipped it.
#[tokio::test]
async fn an_over_long_stdout_line_is_refused_by_name() {
    let events = run_to_end("long-line", std::time::Duration::from_secs(30))
        .await
        .expect("the attempt ends");
    let Some(AttemptEvent::FailedKnown { reason }) = events.last() else {
        panic!("an over-long line is a known failure: {events:?}");
    };
    assert!(
        reason.contains("byte bound"),
        "the refusal names the bound: {reason}"
    );
}

/// `SIGNOFF-REPAIR.10.1.2`: the stderr kept is the END of the stream, where the
/// error is. The old buffer kept the first 8 KiB, so after one long line the
/// line that said what went wrong was never stored.
#[tokio::test]
async fn the_stderr_tail_is_the_end_of_the_stream() {
    let events = run_to_end("stderr-tail", std::time::Duration::from_secs(30))
        .await
        .expect("the attempt ends");
    let Some(AttemptEvent::FailedKnown { reason }) = events.last() else {
        panic!("a failing exit is a known failure: {events:?}");
    };
    assert!(
        reason.contains("TAILMARK"),
        "the last stderr line is in the reason: {reason}"
    );
}

/// `SIGNOFF-REPAIR.10.1.2`: stderr that is not UTF-8 does not stop the drain.
/// The old drain was `while let Ok(Some(line)) = lines.next_line()`, which ENDS
/// at the first invalid line and drops its reader, CLOSING the pipe. ⚠️ The
/// census first called that a stall; a first version of this control passed
/// on the old code and refuted it. The real consequence is that the child's
/// next stderr write fails, and a process that does not ignore SIGPIPE is
/// killed by it, so the provider's answer never arrives.
#[tokio::test]
async fn stderr_that_is_not_utf8_does_not_cut_the_child_off() {
    let events = run_to_end("stderr-invalid", std::time::Duration::from_secs(20))
        .await
        .expect("the attempt ends");
    assert!(
        matches!(events.last(), Some(AttemptEvent::Completed { .. })),
        "the provider's completion arrives: {events:?}"
    );
}

/// `SIGNOFF-REPAIR.10.1.2`: a stdout line that is not UTF-8 is not an event,
/// and is skipped like any other non-event line. The old `read_line` returned
/// an I/O error for it and the stream ended, so the completion after it was
/// lost.
#[tokio::test]
async fn a_stdout_line_that_is_not_utf8_is_skipped() {
    let events = run_to_end("stdout-invalid", std::time::Duration::from_secs(20))
        .await
        .expect("the attempt ends");
    assert!(
        matches!(events.last(), Some(AttemptEvent::Completed { .. })),
        "the completion after the invalid line arrives: {events:?}"
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
