# A retained test fixture is retired by reduction, never by deletion

- Date: 2026-09-18
- Status: accepted
- Owner: `SIGNOFF-REPAIR.7.3.2.1`
- Related: `scripts/census_retained_fixtures.py` (the instrument),
  `scripts/census_pg_test_clusters.py` (`.11.4.3.1.7`'s wholesale disposition,
  now guarded against undoing this one),
  `crates/reasonbraid-browse/tests/support/mod.rs` and `scripts/run_pg_tests.py`
  (the two retainers), `docs/book/src/deployment.md`.

## The decision

A test suite that retains its workspace on failure needs a **retirement** rule, and
that rule is:

> **Keep the receipt, drop the reproducible payload.** Never delete the fixture.

Retention itself is correct and is not changed: the retained directory is what makes
a failed run diagnosable, and both suites go on retaining exactly as before.

## Why not deletion

Deletion was the obvious repair and it is wrong twice over. It destroys the record
that the failure happened at all, and it answers a question nobody asked — the
fixtures are not *too many*, they are *too large*. Measured across both populations
at 1,896,248,619 bytes in 37 fixtures: **99.93 % is reproducible payload** and
1.34 MB is the evidence. Dropping the payload recovers everything deletion would and
keeps every fact.

## Which bytes are evidence — measured, not chosen

This is the part that cannot be reasoned out from file names, and getting it wrong is
silent. In one browser fixture of 56,546,450 bytes, `profile/` holds 56,394,675 of
them — 36.7 MiB being a single `model.tflite` Chrome downloaded — and the evidence is
1,293 bytes in seven files.

⛔ **Two of those seven sit inside the payload directory.** The production worker
writes `owner.json` and `completion.json` — carrying `browser_group` and
`cleanup_confirmed` — into the workspace it is about to be judged on. Dropping
`.project-data` wholesale, the rule written first, would have destroyed precisely the
record `SIGNOFF-REPAIR.7.3.2` exists to preserve. `crashes/` is kept for the same
reason: it holds only a `settings.dat` today, but a real dump is evidence.

So payload is declared as **globs matching DIRECTORIES**, and anything unmatched is
kept:

| Population | Dropped | Kept |
| --- | --- | --- |
| `target/browser-lifetime-controls/case-*` | `.project-data/browser/*/profile`, `.../cache`, the fixture's XDG dirs | `worker.json`, `stdout.log`, `stderr.log`, `owner.json`, `completion.json`, `crashes/`, `Cargo.toml` |
| `target/pg-tests/run-*` | every DIRECTORY under `data/` | `runner.json`, `postgres.log`, `command-N.log`, `data/*.conf`, `PG_VERSION`, `postmaster.opts` |

For PGDATA this is a rule rather than an enumeration: every directory is cluster
storage, every plain file at its root says what the failing cluster ran with. A new
unrecognised file is therefore kept — the mistake errs toward retention.

## Two prohibitions the instrument is built around

**It never deletes a fixture.** Reduction leaves the directory in place and writes
`retired.json` naming each dropped tree, its file count and bytes, and that it is
reproducible by re-running the control. A reader who needs to know what is no longer
inspectable is told.

**It never signals a process.** `SIGNOFF-REPAIR.7.3.2` states the rule — *never signal
a historical numeric PID solely from a stale receipt* — so every liveness probe is
`signal 0`, an existence question. A recorded id may since have been recycled, so the
probe may only ever conclude *this id is in use*, never *this is that process*. An id
in use **keeps** the fixture. Being wrong therefore costs disk, never evidence.

## Consequence for the neighbouring instrument

`scripts/census_pg_test_clusters.py` removes a cluster outright. That disposition is
`SIGNOFF-REPAIR.11.4.3.1.7`'s and is deliberately left intact, but a reduced cluster is
receipts with the bulk already gone: removing it there would destroy what reduction
preserved and recover almost nothing. It now refuses any cluster carrying
`retired.json`, with its own self-test arm.

## Result

34 of 37 fixtures reduced, **1,824,989,121 bytes dropped, 0 deleted**. Verified a
second way with `du -sk`: `target/browser-lifetime-controls` 511,614 KiB → 384 KiB,
`target/pg-tests` 1,340,190 KiB → 70,464 KiB. `run-9_ueev0t` was kept whole by the
citation guard, three tracked files naming it — the guard firing in production rather
than only in its own self-test.
