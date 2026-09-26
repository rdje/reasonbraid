#!/usr/bin/env bash
# scripts/load_harness.sh — the capacity harness (`PHASE-7.4.1`, the `.3`
# criteria's feeder): a scripted CONCURRENT driver over the command API that
# records the per-request ingress→commit latency + the throughput at the
# target concurrency. The measured run is the reproducible record the
# extraction criteria judge against (the CLAIM_VERIFICATION shape: the
# numbers re-derive from the latencies file).
#
# Usage: bash scripts/load_harness.sh --database-url <url> [--commands N]
#        [--concurrency C] [--bin-root target/release] [--port P] [--timeout S]
# Exit 0 only when EXACTLY N commands ran, every one committed (200), and every
# worker exited cleanly; the summary prints either way.
#
# SIGNOFF-REPAIR.11.3.4 — what a published measurement owes, and did not have:
#   · exactly N commands: the count used to be rounded UP per worker (10 at 8
#     workers ran 16), and a zero or negative N passed having sent nothing;
#   · every request is bounded (--timeout, default 30 s) and every worker's exit
#     status counts — `wait $PIDS` kept only the last one;
#   · p50/p95 are over the COMMITTED requests, the failures reported beside them
#     (`scripts/load_summary.py`); they used to include the failures;
#   · the run is its own: a free port unless --port is given, a per-run output
#     directory under target/load/, a readiness probe that proves it reached ITS
#     server, the server reaped on exit, the binary root resolved from the
#     repository, and the database URL off the server's command line.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DATABASE_URL="${DATABASE_URL:-}"
COMMANDS=200
CONCURRENCY=8
BIN_ROOT="target/release"
PORT=""
TIMEOUT=30
while [ $# -gt 0 ]; do
    case "$1" in
        --database-url) DATABASE_URL="$2"; shift 2 ;;
        --commands) COMMANDS="$2"; shift 2 ;;
        --concurrency) CONCURRENCY="$2"; shift 2 ;;
        --bin-root) BIN_ROOT="$2"; shift 2 ;;
        --port) PORT="$2"; shift 2 ;;
        --timeout) TIMEOUT="$2"; shift 2 ;;
        *) echo "error: unknown argument: $1" >&2; exit 2 ;;
    esac
done
[ -n "$DATABASE_URL" ] || { echo "error: --database-url (or \$DATABASE_URL) is required" >&2; exit 2; }
for value in "$COMMANDS" "$CONCURRENCY" "$TIMEOUT"; do
    [[ "$value" =~ ^[1-9][0-9]*$ ]] || { echo "error: --commands, --concurrency and --timeout are positive integers (got '$value')" >&2; exit 2; }
done
if [ "$CONCURRENCY" -gt "$COMMANDS" ]; then CONCURRENCY="$COMMANDS"; fi
case "$BIN_ROOT" in /*) ;; *) BIN_ROOT="$ROOT/$BIN_ROOT" ;; esac
command -v jq >/dev/null || { echo "error: jq is required" >&2; exit 2; }
command -v uuidgen >/dev/null || { echo "error: uuidgen is required" >&2; exit 2; }
command -v python3 >/dev/null || { echo "error: python3 is required" >&2; exit 2; }
if [ -z "$PORT" ]; then
    PORT="$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
fi
export DATABASE_URL

SERVER_BASE="http://127.0.0.1:$PORT"
mkdir -p "$ROOT/target/load"
LOG_DIR="$(mktemp -d "$ROOT/target/load/run.XXXXXX")"
LATENCIES="$LOG_DIR/latencies.txt"
echo "load harness: run directory $LOG_DIR"

# The server boots against the caller's database and is reaped on the way out.
"$BIN_ROOT/rb-server" --port "$PORT" --host 127.0.0.1 >"$LOG_DIR/server.log" 2>&1 &
SERVER_PID=$!
trap 'kill "$SERVER_PID" 2>/dev/null || true; wait "$SERVER_PID" 2>/dev/null || true' EXIT

# Readiness of OUR server: alive, bound (rb-server prints this line only after
# binding), and answering — never merely "something answers on the port".
ready=""
for _ in $(seq 1 80); do
    if kill -0 "$SERVER_PID" 2>/dev/null \
        && grep -q "^rb-server listening on http://127.0.0.1:$PORT " "$LOG_DIR/server.log" \
        && curl -s -o /dev/null "$SERVER_BASE/v1/threads"; then
        ready=1; break
    fi
    kill -0 "$SERVER_PID" 2>/dev/null || break
    sleep 0.25
done
[ -n "$ready" ] || { echo "error: the server never listened (see $LOG_DIR/server.log)" >&2; exit 2; }

request_id() { printf 'req_%s' "$(uuidgen | tr '[:upper:]' '[:lower:]')"; }

envelope() { # operation, key, body-json -> the forged client envelope
    jq -nc --arg op "$1" --arg key "$2" --argjson body "$3" --arg rid "$(request_id)" \
        '{protocol_version:"reasonbraid/0.4",operation:$op,request_id:$rid,
          idempotency_key:$key,expected_aggregate_version:null,body:$body,
          authority_context:null,client_context:{correlation_id:null,causation_id:null}}'
}

# post_json <path> <principal-or-empty> <body> — the body on stdout, and the
# request REFUSED unless it answered 200: a setup step that failed is not setup.
post_json() {
    local answer status
    local headers=(-H 'content-type: application/json')
    if [ -n "$2" ]; then headers+=(-H "x-reasonbraid-principal: $2"); fi
    answer="$(curl -s --connect-timeout 5 --max-time "$TIMEOUT" -w '\n%{http_code}' -X POST \
        "${headers[@]}" --data "$3" "$SERVER_BASE$1")" || return 1
    status="${answer##*$'\n'}"
    printf '%s' "${answer%$'\n'*}"
    [ "$status" = "200" ]
}

# 1. The bootstrap human (the enroll needs no principal header).
HUMAN_JSON="$(post_json /v1/enrollments "" '{"kind":"human","name":"load-human"}')" \
    || { echo "error: the enroll failed: $HUMAN_JSON" >&2; exit 2; }
TENANT="$(printf '%s' "$HUMAN_JSON" | jq -er .tenant_id)" \
    && HUMAN="$(printf '%s' "$HUMAN_JSON" | jq -er .principal_id)" \
    || { echo "error: the enroll answered without a tenant and principal: $HUMAN_JSON" >&2; exit 2; }

# 2. The load thread (the contribute path rides its aggregate row).
CREATE_BODY="$(jq -nc --arg tenant "$TENANT" \
    '{tenant_id:$tenant,subject:"the load thread",objective:"measure the ingress→commit",budget:{calls:9999}}')"
CREATED="$(post_json /v1/threads "$HUMAN" "$(envelope thread.create load-create "$CREATE_BODY")")" \
    || { echo "error: the create failed: $CREATED" >&2; exit 2; }
THREAD="$(printf '%s' "$CREATED" | jq -er .thread_id)" \
    || { echo "error: the create answered without a thread id: $CREATED" >&2; exit 2; }

# 3. The timed concurrent loop: EXACTLY N contributes over C workers (the first
#    N mod C workers take one more), each request a FRESH idempotency key +
#    request id (the full claim → authorize → validate → apply path = the
#    ingress→commit measurement), each bounded by --timeout. An unanswered
#    request is written as status 000 and counts as a failure.
BASE=$(( COMMANDS / CONCURRENCY ))
EXTRA=$(( COMMANDS % CONCURRENCY ))
: > "$LATENCIES"
START="$(python3 -c 'import time; print(time.time())')"
WORKER_PIDS=()
for w in $(seq 1 "$CONCURRENCY"); do
    count=$BASE
    if [ "$w" -le "$EXTRA" ]; then count=$(( BASE + 1 )); fi
    (
        set -e
        for j in $(seq 1 "$count"); do
            KEY="load-$w-$j-$(uuidgen | tr -d - | cut -c1-8)"
            ENV="$(envelope thread.contribute "$KEY" \
                "$(jq -nc --arg tenant "$TENANT" --arg content "the load claim $w-$j" \
                    '{tenant_id:$tenant,content:$content,kind:"claim"}')")"
            curl -s --connect-timeout 5 --max-time "$TIMEOUT" -o /dev/null \
                -w '%{http_code} %{time_total}\n' \
                -H "x-reasonbraid-principal: $HUMAN" -H 'content-type: application/json' \
                --data "$ENV" "$SERVER_BASE/v1/threads/$THREAD/commands" || true
        done
    ) >> "$LATENCIES" &
    WORKER_PIDS+=("$!")
done
# The WORKERS only, each waited for by pid so every exit status counts — the
# server is a long-running background job that must NOT join the wait.
WORKERS_FAILED=0
for pid in "${WORKER_PIDS[@]}"; do
    wait "$pid" || WORKERS_FAILED=$(( WORKERS_FAILED + 1 ))
done
END="$(python3 -c 'import time; print(time.time())')"
WALL="$(python3 -c "print(max($END - $START, 1e-9))")"

# 4. The measured summary, re-derivable from the latencies file.
echo "  latencies: $LATENCIES (one 'status seconds' line per request)"
python3 -B "$ROOT/scripts/load_summary.py" "$LATENCIES" "$COMMANDS" "$WORKERS_FAILED" "$WALL"
