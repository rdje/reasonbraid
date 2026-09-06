#!/usr/bin/env bash
# scripts/run_pg_tests.sh — run the PostgreSQL-backed integration tests (.2.1) against an
# EPHEMERAL server: initdb into a temp dir, start on a throwaway port, drop everything on
# exit. No background service is left running (see docs/ci.md and the handoff doctrine).
#
# Usage:   bash scripts/run_pg_tests.sh
# Env:     PG_BIN  (default: $(brew --prefix postgresql@16)/bin)
#          PG_PORT (default: 55432)
set -euo pipefail

PG_PREFIX="${PG_PREFIX:-$(brew --prefix postgresql@16 2>/dev/null || true)}"
PG_BIN="${PG_BIN:-$PG_PREFIX/bin}"
PORT="${PG_PORT:-55432}"

for tool in initdb pg_ctl createdb; do
    if [ ! -x "$PG_BIN/$tool" ]; then
        echo "error: $PG_BIN/$tool not found — install postgresql@16 (brew install postgresql@16)" >&2
        exit 1
    fi
done

TMP="$(mktemp -d "${TMPDIR:-/tmp}/reasonbraid-pg.XXXXXX")"
SOCK="$TMP/sock"; mkdir -p "$SOCK"

cleanup() {
    "$PG_BIN/pg_ctl" -D "$TMP/data" -m immediate stop >/dev/null 2>&1 || true
    rm -rf "$TMP"
}
trap cleanup EXIT

"$PG_BIN/initdb" -D "$TMP/data" -A trust -U postgres >/dev/null 2>&1
"$PG_BIN/pg_ctl" -D "$TMP/data" -o "-p $PORT -k $SOCK -c listen_addresses=127.0.0.1" -w start >/dev/null 2>&1
"$PG_BIN/createdb" -h 127.0.0.1 -p "$PORT" -U postgres reasonbraid_test

export DATABASE_URL="postgres://postgres@127.0.0.1:$PORT/reasonbraid_test?sslmode=disable"
echo "== running reasonbraid-server atomic-transaction tests against 127.0.0.1:$PORT/reasonbraid_test =="
cargo test -p reasonbraid-server --test atomic_transaction -- --nocapture
