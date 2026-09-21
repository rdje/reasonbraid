answers: why does target/conformance-stubs still accumulate when every other family was repaired; can the fixture cleanup guard be put behind a OnceLock; why are the conformance stubs written once per test binary; what is ETXTBSY and why does it only bite on Linux; when is leaving an accumulation the right call

# One fixture family keeps no cleanup, and the reason is a race that remote CI already found

- **Type:** `decision`
- **Date:** `2026-09-21`
- **Status:** accepted
- **Owner:** leaf `SIGNOFF-REPAIR.11.2.1.3.2.6`
- **Instrument:** `scripts/census_fixture_population.py`
- **Evidence:** `crates/reasonbraid-adapter/tests/conformance/stubs.rs`

## Context

`SIGNOFF-REPAIR.11.2.1.3.2` gave every generated fixture family a producer-owned
cleanup rule: `reasonbraid_core::fixture::Fixture`, whose `Drop` removes the
directory when the test passes and keeps it — with everything in it — when the
test fails. Across seven leaves that reached every family in tracked Rust that
accumulates, except one.

`target/conformance-stubs` holds **34 entries / 272 KiB** at `b724086`. Its
producer writes two executable shell stubs inside `static STUBS: OnceLock<Stubs>`.

⭐ And the figure is pinned because it MOVES, which is the point: re-measured at
`50a9712`, after one session's verification runs, it is **37 entries / 296 KiB**.
That is the accepted cost behaving exactly as this record says it will — one
directory per test binary per run — and it is why the revisit trigger below is a
threshold reported by an instrument rather than a number anybody remembers.

Two facts decide this record, and the second is the one that matters:

1. **A `static` is never dropped.** A `Fixture` stored in that `OnceLock` would
   never run its `Drop`. The guard would be decoration.
2. **The `OnceLock` is not a convenience — it closes a race that remote CI
   caught.** On Linux, `execve` returns `ETXTBSY` for a file still open for
   writing anywhere in the system, and `fork` copies the whole descriptor table.
   A thread forking to spawn one stub inherits another thread's open write
   descriptor for a *different* stub and holds it until its own `exec`
   completes. The superseded design gave each adapter kind its own `OnceLock`
   and its comment claimed that left "no window at all"; the runner disproved
   it — `failed to spawn …/conformance-stubs/codex-14645/codex: Text file busy
   (os error 26)`, where the leaked write was the *claude* stub's.

**One lock for all of them removes the race rather than retrying around it**, by
blocking every other thread until every write and chmod has finished. macOS does
not enforce `ETXTBSY` at all, which is why this passed locally for the project's
whole life and only the Linux runner ever saw it.

## Decision

**This family keeps no consumed cleanup, and the obvious retrofit is refused.**

Making the stubs per-test — the change that would let each hold a guard — puts
concurrent writes back beside concurrent spawns and **reintroduces the exact
defect the runner found**. Trading a defect a remote gate has already caught for
272 KiB of disk is not a repair.

⛔ Stated plainly so it is not mistaken for an oversight: this is the one family
the cleanup lane did not reach, it is named, and the reason lives beside the
`OnceLock` in the producer as well as here.

## What was considered and refused

- **Per-test stubs, each with a guard** — reintroduces the `ETXTBSY` race. Refused.
- **A process-exit hook (`atexit`)** — would work, needs a new dependency in a
  test module and runs arbitrary filesystem work at exit alongside live threads.
  Disproportionate to 272 KiB, and it buys a *new* mechanism nothing else uses.
- **A sweeper over the family** — the whole lane's finding is that a sweeper is
  a workaround for a producer that does not clean up. It would be one here too.

## Cost, bounded

One directory per test **binary** per run, holding two shell scripts. It grows
with runs of the adapter conformance suites, not with the number of tests, which
is why 34 entries occupy 272 KiB — about 8 KiB each.

⚠️ Some of those 34 are older than the rule that shaped the rest: `process-95655`
and `process-23697` sit beside uuid-named entries, residue from before
`SIGNOFF-REPAIR.11.2.1.1` replaced process-id naming with exclusive creation.
Nothing is swept; `.11.2.1.3.1` established that nothing cited is still present,
and removing a fixture was never this lane's act.

## What would change this decision

- The family stops being negligible — say it passes **10 MiB**, which
  `scripts/census_fixture_population.py` reports on every run, so the number is
  asked rather than remembered.
- A cleanup mechanism that does not need per-test writes arrives for another
  reason, making this a free ride rather than a new dependency.
- The stubs stop being written concurrently with spawns — for instance if the
  conformance suites became single-threaded for another reason — at which point
  the race that forces the `OnceLock` no longer exists.

## Related

- `docs/knowledge/a-control-that-passes-for-an-unrelated-reason.md` — the
  superseded per-kind locks passed on macOS for a reason unrelated to being
  correct.
- `SIGNOFF-REPAIR.11.2.1.3.2` — the lane this is the single exception to.
