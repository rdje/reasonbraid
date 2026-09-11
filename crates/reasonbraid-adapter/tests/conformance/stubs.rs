//! Subprocess stubs shared by the real-adapter conformance scenarios AND each
//! adapter's own test file: a POSIX shell script standing in for the provider CLI,
//! written under the build dir, so the tests exercise the REAL subprocess boundary
//! (spawn, JSONL parsing, exit-status verdicts, kill) — only the provider is fake.

use std::path::PathBuf;
use std::sync::OnceLock;

/// One stub per adapter kind per process, written ONCE before any scenario can
/// spawn.
///
/// The scripts branch on the prompt, so a per-scenario copy never carried any
/// information — and copying them was actively harmful. On Linux, `execve`
/// returns `ETXTBSY` ("Text file busy") for a file that is still open for
/// writing anywhere: with several conformance tests in parallel threads, one
/// thread writing a stub while another forks to spawn gives that child the
/// open write descriptor, and the exec then fails. Remote CI reported exactly
/// that — `failed to spawn …/conformance-stubs/lose-14317-1/claude: Text file
/// busy (os error 26)` — while macOS does not enforce it, which is why this
/// passed locally for the project's whole life.
///
/// A `OnceLock` publishes the path only after the write and the chmod have
/// finished, so there is exactly one write per process and no window at all.
/// The process id in the directory keeps concurrent test BINARIES apart. This
/// removes the race rather than retrying around it.
fn stub_once(slot: &'static OnceLock<PathBuf>, file: &str, script: &str) -> PathBuf {
    slot.get_or_init(|| {
        let base = std::env::var_os("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
        let dir = base
            .join("conformance-stubs")
            .join(format!("{file}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(file);
        std::fs::write(&path, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        path
    })
    .clone()
}

/// The `codex` stub: branches on the prompt (the LAST argument).
pub fn codex_binary(_name: &str) -> PathBuf {
    static CODEX: OnceLock<PathBuf> = OnceLock::new();
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
    stub_once(&CODEX, "codex", script)
}

/// The `claude` stub: branches on the prompt (the LAST argument).
pub fn claude_binary(_name: &str) -> PathBuf {
    static CLAUDE: OnceLock<PathBuf> = OnceLock::new();
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
    stub_once(&CLAUDE, "claude", script)
}
