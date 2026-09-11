//! Subprocess stubs shared by the real-adapter conformance scenarios AND each
//! adapter's own test file: a POSIX shell script standing in for the provider CLI,
//! written under the build dir, so the tests exercise the REAL subprocess boundary
//! (spawn, JSONL parsing, exit-status verdicts, kill) — only the provider is fake.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_STUB: AtomicU64 = AtomicU64::new(0);

fn stub_dir(name: &str) -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    // A scenario name plus a clock reading is not a unique directory: scenario
    // names repeat across adapters, this host returns byte-identical
    // `subsec_nanos` for consecutive calls, and `create_dir_all` succeeds on an
    // existing directory rather than refusing. Two stubs could therefore share a
    // directory while one is being written and the other executed. The counter
    // is unique within the process and the process id across processes — the
    // same shape as `.11.4.3.1.2.17`, `.7.3.3.1` and `.7.4.1`.
    let sequence = NEXT_STUB.fetch_add(1, Ordering::Relaxed);
    let dir = base
        .join("conformance-stubs")
        .join(format!("{name}-{}-{sequence}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The `codex` stub: branches on the prompt (the LAST argument).
pub fn codex_binary(name: &str) -> PathBuf {
    let path = stub_dir(name).join("codex");
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

/// The `claude` stub: branches on the prompt (the LAST argument).
pub fn claude_binary(name: &str) -> PathBuf {
    let path = stub_dir(name).join("claude");
    let script = r#"#!/bin/sh
# The adapter invokes: <binary> -p ... -- <prompt> — the prompt is the LAST arg.
for last in "$@"; do :; done
case "$last" in
  *fail*)
    echo '{"type":"system","subtype":"init","session_id":"stub_fail","model":"stub"}'
    echo "boom: simulated provider error" >&2
    exit 2
    ;;
  *refuse*)
    echo '{"type":"system","subtype":"init","session_id":"stub_refuse","model":"stub"}'
    echo '{"type":"result","subtype":"error_during_execution","is_error":true,"result":"simulated provider refusal","session_id":"stub_refuse"}'
    exit 1
    ;;
  *lose*)
    echo '{"type":"system","subtype":"init","session_id":"stub_lose","model":"stub"}'
    exit 0
    ;;
  *sleep*)
    echo '{"type":"system","subtype":"init","session_id":"stub_sleep","model":"stub"}'
    # Busy-loop with NO child process: the killed script IS the pipe holder, so
    # start_kill ends the stream immediately (a `sleep` child would inherit the
    # stdout pipe and delay EOF past the test's bound).
    while :; do :; done
    ;;
  *)
    echo '{"type":"system","subtype":"init","session_id":"stub_ok","model":"stub"}'
    echo '{"type":"assistant","session_id":"stub_ok","message":{"content":[{"type":"thinking","thinking":"internal reasoning is not the reply"},{"type":"text","text":"hello '"$last"'"},{"type":"text","text":"world"}]}}'
    echo '{"type":"result","subtype":"success","is_error":false,"result":"hello '"$last"' world","session_id":"stub_ok","usage":{"input_tokens":10,"output_tokens":3,"cache_read_input_tokens":0,"cache_creation_input_tokens":0},"total_cost_usd":0.001234,"duration_ms":42,"num_turns":1}'
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
