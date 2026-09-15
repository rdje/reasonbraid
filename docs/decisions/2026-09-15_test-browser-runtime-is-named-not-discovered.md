---
answers:
  - Which browser does the browse test suite run against?
  - Why do the real-browser controls skip when I run `cargo test` directly?
  - Why does the workspace suite use `--no-fail-fast`?
  - Was the browser deadline control's failure a product defect?
  - Does the pinned testing browser contain its own crash-reporter helpers?
---
# The test suite's browser runtime is NAMED by the harness, never discovered

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.4`
- **Date:** 2026-09-15
- **Supersedes:** nothing. It **adds** to `2026-09-10_browser-checkpoint-timing.md`, which
  established the pin and is left byte-unchanged.

## Context

`2026-09-10_browser-checkpoint-timing.md` pinned Chrome for Testing as the browse
suite's runtime, because desktop Chrome starts crash-reporting and update helpers
that leave the worker's process group while retaining its stderr. `scripts/ci_browser.py`
implements the pin: it downloads the archive, checks its SHA-256 against a recorded
value, verifies the executable's exact version, and exports `R3_BROWSER_BIN`.

The pin had a second path around it. `crates/reasonbraid-browse/tests/support/mod.rs`
carried its own discovery list, so any run that did not go through the launcher —
a bare `cargo test --all`, which is what the workspace re-derivations use — silently
selected `/Applications/Google Chrome.app/…` instead. The book already warned that
direct Cargo commands establish no pinned-runtime coverage; nothing enforced it.

## The fact / decision

1. **`browser_binary()` in the browse test support reads `R3_BROWSER_BIN` and nothing
   else.** There is no fallback. A run that does not name a runtime skips the six
   real-browser controls rather than qualifying them against an unpinned one.
2. **A skip is announced through a channel `libtest` does not capture** — a write to
   the `std::io::stderr()` handle — so it cannot read as a pass. The notice names what
   went unqualified and the command that qualifies it.
3. **The workspace test command uses `--no-fail-fast`**, in `make test` and in
   `.github/workflows/rust.yml`. Those are the two places that run the whole workspace;
   a crate-scoped development run keeps its early stop.
4. **The production worker's own discovery is unchanged.** `crates/reasonbraid-browse/src/main.rs`
   still falls back to an installed browser: a deployment runs what the host provides,
   and narrowing that is a deployment decision `SIGNOFF-REPAIR.7.3.2` owns, not a test one.

## Why

**Measured, one variable, same machine and the same competing load.** The control
`a_real_navigation_deadline_stops_the_browser_and_origin` failed 4 of 4 runs against
desktop Google Chrome 152.0.7977.83 and passed 2 of 2 against the pinned Chrome for
Testing 153.0.8010.36. The worker's elapsed time was 41.07 s against desktop Chrome
and 31.13 s against the pinned build; the ten-second difference is the worker's entire
10 s cleanup budget spent waiting for an end-of-file that could not arrive.

`lsof` names the holder rather than inferring it: the worker's fd 11 is one end of a
pipe whose other end is `chrome_crashpad_handler`'s fd 2, and that handler reports
`PPID 1` in a process group the worker never owned. It outlived the worker's own exit
by 6.24 s.

⚠️ **The pinned runtime escapes identically** — two handlers, `PPID 1`, process groups
outside the owned one, holding the same pipe. Only the exit latency differs. The green
is therefore a latency property of a third-party process, not a containment guarantee,
and `SIGNOFF-REPAIR.7.3.2` owns that gap.

The `--no-fail-fast` adoption rests on the census at `SIGNOFF-REPAIR.11.4.7.2.2`:
default fail-fast reached 11 of 94 test binaries and left four crates unknown, while
`--no-fail-fast` reached 94 — 815 tests passed, 1 failed — for 319.3 s of test
execution. A gate that answers "is the workspace green" cannot answer it from 11
binaries. The cost is that a red run now pays full price instead of stopping early.

🔴 **A completed workspace run under the pinned runtime does not yet exist.** The first
`make test` after this change reached 78 of ~94 test binaries with zero failures and was
cut off by `ci_browser.py`'s own `COMMAND_SECONDS = 3600` cap — the first timeout in 16
receipts. ⛔ That is not attributable to `--no-fail-fast` (fail-fast stops at a FAILURE, and
there were none), but the exoneration is reasoning rather than measurement, and
`SIGNOFF-REPAIR.11.4.8` owns both the completed run and the deadline that admits it.

## How to apply

- Run the browse controls through `make test` or `scripts/ci_browser.py`. A verdict
  about them names the browser build it was earned against.
- Never reintroduce a discovery fallback into test support. If a control needs an
  external program, the harness names it or the control does not run — see
  [[a-pin-a-second-path-can-bypass-is-not-a-pin]].
- A cleanup-unconfirmed refusal from the worker is the worker telling the truth. Do not
  repair it by loosening an assertion: `a_render_refusal_survives_an_unconfirmed_cleanup`
  pins that reading with an injected `setsid` writer, and
  [[one-field-cannot-carry-two-facts]] records why the two facts travel separately.
