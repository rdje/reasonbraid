#!/usr/bin/env bash
# Repository-local environment and supervised disposable PostgreSQL runner.
# Usage: bash scripts/run_pg_tests.sh [--list] [authority command_api ...] [--demo]
# PG_BIN selects installed PostgreSQL 16 tools. No suites selects the broad run;
# RB_DEMO=0 omits its demonstration. Caller DATABASE_URL is never used.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 -B "$ROOT/scripts/project_env.py" python3 -B scripts/run_pg_tests.py "$@"
