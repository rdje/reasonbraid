answers: my test passes in CI and fails when I run it directly — where do I look first; how do I stop a pinned tool version from being silently substituted; why does a control that was qualified against one runtime start failing; is a skipped test the same as a passing test; how do I make a skip visible in an ordinary cargo test run; my test harness discovers its own dependency — is that a problem?

# A pin a second path can bypass is not a pin

- **Type:** `knowledge`
- **Date:** `2026-09-15`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.4.7.2.4`; the pin itself was established by `SIGNOFF-REPAIR.11.4.3.1.2.5` and `docs/decisions/2026-09-10_browser-checkpoint-timing.md`, three days and forty commits earlier

## The question

A control fails on your machine and passes in CI. The code is identical, the
commit is identical, the test is not flaky — two runs on a quiet machine and a
loaded one give the same failure. What is the first thing to look at?

## The answer

**Which build of the external dependency each run actually selected — and
whether the test harness CHOSE it or DISCOVERED it.** A harness that falls back
to "whatever is installed" when its selector is unset is not running the pinned
dependency; it is running a different program with the same name, and every
verdict it produces is about that program instead.

The shape is mechanical, and it is worth stating as a rule:

```rust
// ⛔ discovery: the pin holds only when someone remembered to set the variable
pub fn runtime() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("PINNED_BIN") { return Some(p.into()); }
    ["/Applications/Some Browser.app/Contents/MacOS/Some Browser", "/usr/bin/other"]
        .into_iter().map(PathBuf::from).find(|p| p.is_file())   // <- the bypass
}

// ✅ selection: the harness names the runtime, or the control does not run
pub fn runtime() -> Option<PathBuf> {
    std::env::var_os("PINNED_BIN").map(PathBuf::from)
}
```

⭐ **The prose warning is not the mechanism.** This project's own book already
said, in the chapter that documents the launcher: *"Direct Cargo commands bypass
this setup and do not establish pinned-runtime coverage by themselves."* That
sentence was true, it was correct when written, and it had no teeth — the
fallback sat in the test support making a direct run look exactly like coverage.
A pin enforced by a sentence is a suggestion; a pin enforced by the absence of a
second path is a pin.

## The second half: a skip must not read like a pass

Removing the fallback moves the failure into a skip, and a skip is a weaker
guarantee that must never LOOK like a stronger one. In Rust this is a real trap,
because `libtest` captures `println!`/`eprintln!` and replays them only for a
FAILING test — so the skip notice is invisible in exactly the run where it
matters.

The mechanism, measured with a one-test `rustc --test` probe rather than assumed:
a write through the `std::io::stderr()` HANDLE is not captured, while the macros
are.

```text
$ ./probe                     # no --nocapture
PROBE-direct-stderr-handle
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Only the handle write survived. `println!` and `eprintln!` from the same passing
test did not appear.

## What this cost, measured

One control — a browser render deadline — read RED for five commits and was
reasoned about twice as a possible product defect, complete with a written
argument about which of two designs was wrong. The two-site contrast that
settled it changed one variable:

| runtime | verdict | worker elapsed | escaped helper survives past the worker |
| --- | --- | --- | --- |
| the ambient desktop browser | cleanup unconfirmed, failed 4 of 4 | 41.07 s | +6.24 s |
| the pinned testing build | cleanup confirmed, passed 2 of 2 | 31.13 s | 0, inside one sample |

The ten-second difference is not noise: it is the worker's whole cleanup budget
being spent waiting for an end-of-file that a process outside its own process
group was holding open.

⚠️ **And the part that is easy to file wrongly.** The pinned runtime leaks the
same escaped helper — `lsof` shows it at `PPID 1`, in a process group the worker
never owned, holding the other end of the worker's stderr pipe. Only its EXIT
LATENCY differs. So the green is a property of how fast somebody else's process
exits, not a containment guarantee, and saying so is the difference between a
closed question and a hidden one.

## How to apply

- When a control's verdict depends on an external program, the harness NAMES
  that program or the control does not run. Delete the discovery fallback.
- Say which runtime a green verdict was earned against, in the leaf and in the
  control's own comment. "The suite passes" is not a claim until it names the
  dependency versions it passed with.
- Announce a skip through a channel the test harness does not capture, and name
  both what went unchecked and the command that would check it.
- When two runtimes differ, ask whether the passing one differs STRUCTURALLY or
  only in timing. A latency-shaped green is still owed a containment owner —
  see [[proving-a-race-is-closed]] and [[an-aborted-run-is-not-a-partial-pass]].
- Related: [[a-self-test-cannot-be-tidier-than-the-real-input]] (the fixture is
  the real artifact), [[a-claim-of-sameness-is-worth-its-call-graph]] (what a
  control actually reaches).
