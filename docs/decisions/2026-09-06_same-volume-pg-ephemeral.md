# 2026-09-06_same-volume-pg-ephemeral.md

## Context

`PHASE-1-MAINT-1` (a defect leaf opened during `.1.1.1`): the §13 same-volume
data-locality policy was adopted after `scripts/run_pg_tests.sh` was written, and
the script kept defaulting its ephemeral PostgreSQL cluster to
`${TMPDIR:-/tmp}/reasonbraid-pg.XXXXXX` — off the repo's volume on this host
(repo on `/Volumes/SSD/...`, `/tmp` on the system volume).

## Decision

- **Project-owned ephemeral data derives from the repo root at runtime.** The
  script now computes `ROOT` from its own location
  (`cd "$(dirname "${BASH_SOURCE[0]}")/.."`) and places the cluster at
  `$ROOT/target/pg-ephemeral.XXXXXX` — same volume as the repo, gitignored via
  `/target`, still unique per run. No project tool may default its data to
  `/tmp`, a user-home cache, or any other off-volume location (§13).
- **The mechanics survive the move.** `mktemp`'s per-run uniqueness and the
  cleanup trap (`trap cleanup EXIT` → `rm -rf "$TMP"`) are untouched — only the
  parent directory moved, so a crashed run leaves nothing but a gitignored
  directory on the repo volume.

## Consequences

- The full live verification re-runs green from the new location (twelve live
  server suites + CLI e2e + two-host demo, `rc=0`); a mid-run probe shows the
  cluster on the repo volume and a residue census shows no `/tmp` usage during
  or after the run.
- Future scripts that create project-owned temp data follow the same pattern:
  derive `$ROOT` at runtime, stay on the repo volume, gitignore it, clean it up.

answers:

- **A locality policy does not reach backwards into pre-existing tools.**
  Adoption is forward-looking: every tool that creates project data must be
  re-derived from the repo root after the fact, and the defect leaf made that
  re-check itself the work item — a pre-adoption default stays off-volume until
  a reader re-checks it.
- **The repo root is the runtime authority for paths, not the caller's CWD.**
  `ROOT` comes from the script's own location, so the suite behaves identically
  from any directory; persisted paths are repo-root-relative by construction
  (§12) and absolute only at runtime (§13).
