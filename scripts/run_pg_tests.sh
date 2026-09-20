#!/usr/bin/env bash
# Repository-local environment and supervised disposable PostgreSQL runner.
# Usage: bash scripts/run_pg_tests.sh [--list] [authority command_api ...] [--demo]
# PG_BIN selects installed PostgreSQL 16 tools. No suites selects the broad run;
# RB_DEMO=0 omits its demonstration. Caller DATABASE_URL is never used.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# SIGNOFF-REPAIR.11.34 — the collection needs the workspace BINARIES, not just
# the test binaries `cargo test` builds. `profiles`' two R2 joins abort on an
# absent `target/debug/reasonbraid-extract` (deliberately: a control closing a
# coverage gap must not be able to report neither way), and the `pg-tests` CI job
# is a separate runner that never inherits the workspace job's build — so those
# two had never executed there. It lives HERE rather than in the workflow so the
# local run and the remote one prepare the same environment, and rather than in
# `run_pg_tests.py` so the runner stays the unit `scripts/tests/test_pg_runner.py`
# mocks. `--list` prints the suite names and needs nothing built.
if [ "${1:-}" != "--list" ]; then
  python3 -B "$ROOT/scripts/project_env.py" cargo build --workspace --bins --locked
fi

exec python3 -B "$ROOT/scripts/project_env.py" python3 -B scripts/run_pg_tests.py "$@"
