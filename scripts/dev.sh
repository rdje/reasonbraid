#!/usr/bin/env bash
# scripts/dev.sh — the one-command development environment (PHASE-1.7.1).
#
# Boots an EPHEMERAL on-volume PostgreSQL cluster (§13 same-volume locality —
# `$ROOT/target/dev-ephemeral.XXXXXX`, gitignored, per-run unique), starts
# `rb-server` (dev profile, loopback, migrations run on startup), and prints
# the console URL + the CLI hint. Ctrl-C stops the server and removes the
# cluster with a residue census. This is an interactive DEV LOOP, not the test
# harness (`scripts/run_pg_tests.sh` owns verification).
#
# Usage:   make dev                      (foreground dev loop)
#          bash scripts/dev.sh --check   (self-verification beat: boot → probe
#                                         → real CLI call → teardown → census)
# Flags:   --port N     control-plane port (default 4310)
#          --pg-port N  ephemeral PostgreSQL port (default 55432)
# Env:     PG_BIN       (default: $(brew --prefix postgresql@16)/bin)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

SERVER_PORT=4310
PG_PORT=55432
CHECK_MODE=0
while [ $# -gt 0 ]; do
    case "$1" in
        --port) SERVER_PORT="$2"; shift 2 ;;
        --pg-port) PG_PORT="$2"; shift 2 ;;
        --check) CHECK_MODE=1; shift ;;
        *) echo "dev.sh: unknown flag: $1" >&2; exit 2 ;;
    esac
done

PG_PREFIX="${PG_PREFIX:-$(brew --prefix postgresql@16 2>/dev/null || true)}"
PG_BIN="${PG_BIN:-$PG_PREFIX/bin}"
for tool in initdb pg_ctl createdb; do
    if [ ! -x "$PG_BIN/$tool" ]; then
        echo "error: $PG_BIN/$tool not found — install postgresql@16 (brew install postgresql@16)" >&2
        exit 1
    fi
done

# §13 same-volume locality (the PHASE-1-MAINT-1 shape): the cluster lives under
# the repo's target/ — never /tmp or any off-volume location.
mkdir -p "$ROOT/target"
TMP="$(mktemp -d "$ROOT/target/dev-ephemeral.XXXXXX")"
SOCK="$TMP/sock"; mkdir -p "$SOCK"

SERVER_PID=""
TORN_DOWN=""
cleanup() {
    if [ -n "$TORN_DOWN" ]; then return 0; fi
    if [ -n "$SERVER_PID" ]; then kill "$SERVER_PID" >/dev/null 2>&1 || true; wait "$SERVER_PID" 2>/dev/null || true; fi
    "$PG_BIN/pg_ctl" -D "$TMP/data" -m immediate stop >/dev/null 2>&1 || true
    rm -rf "$TMP"
    echo "dev: ephemeral PostgreSQL + server torn down"
}
trap cleanup EXIT

"$PG_BIN/initdb" -D "$TMP/data" -A trust -U postgres >/dev/null 2>&1
"$PG_BIN/pg_ctl" -D "$TMP/data" -o "-p $PG_PORT -k $SOCK -c listen_addresses=127.0.0.1" -w start >/dev/null 2>&1
"$PG_BIN/createdb" -h 127.0.0.1 -p "$PG_PORT" -U postgres reasonbraid_dev

DATABASE_URL="postgres://postgres@127.0.0.1:$PG_PORT/reasonbraid_dev?sslmode=disable"
export DATABASE_URL

cargo build --bins -q

SERVER_BIN="$ROOT/target/debug/rb-server"
CLI_BIN="$ROOT/target/debug/rb"
"$SERVER_BIN" --host 127.0.0.1 --port "$SERVER_PORT" --database-url "$DATABASE_URL" &
SERVER_PID=$!

# Wait for the listener (the console probe doubles as the API probe: same listener).
UP=0
for _ in $(seq 1 100); do
    if curl -s -o /dev/null "http://127.0.0.1:$SERVER_PORT/v1/threads"; then UP=1; break; fi
    sleep 0.1
done
if [ "$UP" != "1" ]; then
    echo "error: rb-server did not come up on port $SERVER_PORT" >&2
    exit 1
fi

if [ "$CHECK_MODE" = "1" ]; then
    fail() { echo "dev-check: FAIL — $1" >&2; exit 1; }

    # 1. The console serves from the SAME binary at / (the .1.6.2 shell marker).
    curl -s "http://127.0.0.1:$SERVER_PORT/" | grep -q 'inspection console' \
        || fail "console shell not served at /"
    echo "dev-check: PASS — console served at /"

    # 2. A REAL CLI call through the control API (enroll → state file → thread list).
    export REASONBRAID_CLI_STATE="$ROOT/target/dev-check-cli"   # repo-local (§13)
    mkdir -p "$REASONBRAID_CLI_STATE"
    ENROLLED="$("$CLI_BIN" --server "http://127.0.0.1:$SERVER_PORT" enroll human devcheck --json 2>/dev/null)" \
        || fail "rb enroll through the control API"
    printf '%s' "$ENROLLED" | grep -q '"tenant_id"' \
        || fail "enroll response carries no tenant_id"
    echo "dev-check: PASS — real CLI enroll through the control API"

    LISTED="$("$CLI_BIN" --server "http://127.0.0.1:$SERVER_PORT" inspect threads --as devcheck --json 2>/dev/null)" \
        || fail "rb inspect threads through the control API"
    echo "dev-check: PASS — thread listing ($(printf '%s' "$LISTED" | grep -c '"id"') thread(s) visible)"

    # 3. Graceful stop + residue census (explicit teardown — the EXIT trap's
    #    re-run is a no-op; the census must observe the cleaned state).
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
    SERVER_PID=""
    "$PG_BIN/pg_ctl" -D "$TMP/data" -m immediate stop >/dev/null 2>&1 || true
    rm -rf "$TMP"
    TORN_DOWN=1
    rm -rf "$REASONBRAID_CLI_STATE"
    RESIDUE="$(find "$ROOT/target" -maxdepth 1 -name 'dev-ephemeral.*' | wc -l | tr -d ' ')"
    [ "$RESIDUE" = "0" ] || fail "residue: $RESIDUE dev-ephemeral dir(s) left in target/"
    echo "dev-check: PASS — residue census clean ($RESIDUE dev-ephemeral dirs)"
    echo "dev-check: OK"
    exit 0
fi

echo
echo "== ReasonBraid dev environment (PHASE-1.7.1) =="
echo "  console:   http://127.0.0.1:$SERVER_PORT/"
echo "  api base:  http://127.0.0.1:$SERVER_PORT"
echo "  postgres:  $DATABASE_URL (ephemeral — dropped on exit)"
echo "  cli:       $CLI_BIN --server http://127.0.0.1:$SERVER_PORT enroll human you"
echo "             (or export REASONBRAID_SERVER=http://127.0.0.1:$SERVER_PORT)"
echo "Ctrl-C stops the server and removes the ephemeral cluster."
echo

wait "$SERVER_PID"
