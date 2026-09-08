#!/usr/bin/env bash
# scripts/run_pg_tests.sh — run the PostgreSQL-backed integration tests (.2.1 atomic
# transaction, .2.2 outbox worker, .3.2 node channel, .5.1 authority, .5.2 budget,
# transaction, .2.2 outbox worker, .3.2 node channel, .5.1 authority, .5.2 budget,
# .6.1 command API, .6.2 node wiring, 1.1.1 aggregate library, 1.1.2 identity store,
# 1.2.1 node enrollment + the real-binary CLI end-to-end suite)
# against an EPHEMERAL server:
# initdb into a temp dir under the repo root (target/pg-ephemeral.XXXXXX, gitignored —
# §13 same-volume data locality: never /tmp or any off-volume location), start on a
# throwaway port, drop everything on exit. No background service is left running
# (see docs/ci.md and the handoff doctrine).
#
# Usage:   bash scripts/run_pg_tests.sh
# Env:     PG_BIN  (default: $(brew --prefix postgresql@16)/bin)
#          PG_PORT (default: 55432)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

PG_PREFIX="${PG_PREFIX:-$(brew --prefix postgresql@16 2>/dev/null || true)}"
PG_BIN="${PG_BIN:-$PG_PREFIX/bin}"
PORT="${PG_PORT:-55432}"

for tool in initdb pg_ctl createdb; do
    if [ ! -x "$PG_BIN/$tool" ]; then
        echo "error: $PG_BIN/$tool not found — install postgresql@16 (brew install postgresql@16)" >&2
        exit 1
    fi
done

# §13 same-volume locality: the ephemeral cluster lives on the repo's volume
# (target/ is gitignored), so project-owned data never lands on /tmp or another
# filesystem. mktemp keeps the per-run uniqueness the cleanup trap relies on.
mkdir -p "$ROOT/target"
TMP="$(mktemp -d "$ROOT/target/pg-ephemeral.XXXXXX")"
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
echo "== running reasonbraid-server PostgreSQL integration tests against 127.0.0.1:$PORT/reasonbraid_test =="
cargo test -p reasonbraid-server --test atomic_transaction --test outbox_worker --test node_channel --test authority --test budget --test command_api --test node_work --test aggregate_library --test identity_store --test node_enrollment --test node_inbox --test invitations --test backup_restore --test migration_upgrade --test escalation --test node_replacement --test profiles --test evaluation --test routing --test policy --test rls --test quota --test quarantine --test classification --test federation --test cards -- --nocapture

echo "== running the reasonbraid-cli end-to-end suite (real rb binary, in-process control API) =="
cargo test -p reasonbraid-cli --test cli_end_to_end -- --nocapture

# The WP6 two-host demonstration (.6.2): the full crash/reconnect scenario with
# real kill points and a grep-verified acceptance bundle. RB_DEMO=0 skips it.
if [ "${RB_DEMO:-1}" != "0" ]; then
    echo "== running the two-host crash/reconnect demonstration (scripts/demo_two_host.sh) =="
    bash scripts/demo_two_host.sh --database-url "$DATABASE_URL"
fi
