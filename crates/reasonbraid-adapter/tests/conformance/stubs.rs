//! Subprocess stubs shared by the real-adapter conformance scenarios AND each
//! adapter's own test file: a POSIX shell script standing in for the provider CLI,
//! written under the build dir, so the tests exercise the REAL subprocess boundary
//! (spawn, JSONL parsing, exit-status verdicts, kill) — only the provider is fake.

use std::path::PathBuf;
use std::sync::OnceLock;

/// EVERY stub, written before ANY scenario can spawn.
///
/// On Linux, `execve` returns `ETXTBSY` ("Text file busy") for a file that is
/// still open for writing ANYWHERE in the system. A `fork` copies the whole
/// descriptor table, so a thread that forks to spawn one stub inherits the
/// open write descriptor of a DIFFERENT stub that another thread happens to be
/// writing, and holds it until its own `exec` completes.
///
/// The superseded version gave each adapter kind its own `OnceLock` and its
/// comment claimed that left "no window at all". That was wrong, and remote CI
/// disproved it: `failed to spawn …/conformance-stubs/codex-14645/codex: Text
/// file busy (os error 26)`, with the write that leaked into the fork being
/// the CLAUDE stub's. One lock per kind serialises each stub against itself
/// and against nothing else.
///
/// One lock for ALL of them closes it. `get_or_init` blocks every other thread
/// until the initializer returns, so no scenario can hold a stub path — and
/// therefore cannot spawn — until every write and chmod has finished. The race
/// is removed rather than retried around. macOS does not enforce `ETXTBSY` at
/// all, which is why this passed locally for the project's whole life.
struct Stubs {
    codex: PathBuf,
    claude: PathBuf,
}

// ⛔ NO CLEANUP GUARD HERE, AND THE REASON IS STRUCTURAL RATHER THAN AN
// OVERSIGHT (`SIGNOFF-REPAIR.11.2.1.3.2.6`). Every other fixture producer in
// this workspace now returns a `reasonbraid_core::fixture::Fixture`, whose
// `Drop` removes the directory when the test passes and keeps it when the test
// fails. A `static` is NEVER DROPPED, so a guard stored in this `OnceLock`
// would never run and would be decoration.
//
// ⭐ And the design is right as it stands: these two stubs are written once per
// test BINARY precisely so that every test in it shares them, which means there
// is no moment at which *this test passed* is even a question about them.
// ⛔ AND THE OBVIOUS RETROFIT IS REFUSED, not merely unchosen: per-test stubs
// would let each hold a guard, and would put concurrent writes back beside
// concurrent spawns — reintroducing the ETXTBSY race documented above, which
// the remote runner has already caught once. Trading a defect a gate has
// caught for 272 KiB of disk is not a repair.
// Decided, with the trigger that would revisit it:
// docs/decisions/2026-09-21_one-fixture-family-keeps-no-cleanup-and-the-reason-is-a-race.md
static STUBS: OnceLock<Stubs> = OnceLock::new();

fn stubs() -> &'static Stubs {
    STUBS.get_or_init(|| {
        // The base follows the PROCESS, not the build (§12). `CARGO_TARGET_TMPDIR`
        // was measured UNSET in this project's runs, so the compile-time fallback
        // this replaces was the live path, baking one checkout's absolute path
        // into the binary (SIGNOFF-REPAIR.11.2.1.2.1).
        let base = reasonbraid_core::repository_root()
            .expect("the tests run inside the repository")
            .join("target");
        // The lock keeps this process's writes apart from every spawn. What
        // keeps this RUN apart from every other is exclusive creation, not the
        // name: a process id separates concurrent binaries, because two live
        // processes cannot share one, but it is REUSED across runs — and
        // `create_dir_all` turns that reuse from a refusal into an adoption of
        // whatever an earlier run left behind (SIGNOFF-REPAIR.11.2.1.1).
        let parent = base.join("conformance-stubs");
        std::fs::create_dir_all(&parent).unwrap();
        let dir = parent.join(uuid::Uuid::now_v7().to_string());
        std::fs::DirBuilder::new()
            .create(&dir)
            .expect("the stub directory is new");
        Stubs {
            codex: write_stub(&dir, "codex", CODEX_SCRIPT),
            claude: write_stub(&dir, "claude", CLAUDE_SCRIPT),
        }
    })
}

fn write_stub(dir: &std::path::Path, file: &str, script: &str) -> PathBuf {
    let path = dir.join(file);
    std::fs::write(&path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}

/// The `codex` provider script.
const CODEX_SCRIPT: &str = r#"#!/bin/sh
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

/// The `claude` provider script.
const CLAUDE_SCRIPT: &str = r#"#!/bin/sh
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

/// The `codex` stub: branches on the prompt (the LAST argument).
pub fn codex_binary(_name: &str) -> PathBuf {
    stubs().codex.clone()
}

/// The `claude` stub: branches on the prompt (the LAST argument).
pub fn claude_binary(_name: &str) -> PathBuf {
    stubs().claude.clone()
}
