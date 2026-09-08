#!/usr/bin/env bash
# scripts/load_harness.sh — the capacity harness (`PHASE-7.4.1`, the `.3`
# criteria's feeder): a scripted CONCURRENT driver over the command API that
# records the per-request ingress→commit latency + the throughput at the
# target concurrency. The measured run is the reproducible record the
# extraction criteria judge against (the CLAIM_VERIFICATION shape: the
# numbers re-derive from the latencies file).
#
# Usage: bash scripts/load_harness.sh --database-url <url> [--commands N]
#        [--concurrency C] [--bin-root target/release]
# Exit 0 when every command committed (the 200s) and the summary printed.

set -uo pipefail

DATABASE_URL="${DATABASE_URL:-}"
COMMANDS=200
CONCURRENCY=8
BIN_ROOT="target/release"
PORT=4391
while [ $# -gt 0 ]; do
    case "$1" in
        --database-url) DATABASE_URL="$2"; shift 2 ;;
        --commands) COMMANDS="$2"; shift 2 ;;
        --concurrency) CONCURRENCY="$2"; shift 2 ;;
        --bin-root) BIN_ROOT="$2"; shift 2 ;;
        *) echo "error: unknown argument: $1" >&2; exit 2 ;;
    esac
done
[ -n "$DATABASE_URL" ] || { echo "error: --database-url (or \$DATABASE_URL) is required" >&2; exit 2; }
command -v jq >/dev/null || { echo "error: jq is required" >&2; exit 2; }
command -v uuidgen >/dev/null || { echo "error: uuidgen is required" >&2; exit 2; }
command -v python3 >/dev/null || { echo "error: python3 is required" >&2; exit 2; }

SERVER_BASE="http://127.0.0.1:$PORT"
LOG_DIR="target/load"
mkdir -p "$LOG_DIR"
LATENCIES="$LOG_DIR/latencies.txt"

# The server boots against the caller's database; the harness kills it on
# the way out (the demo's pattern).
"$BIN_ROOT/rb-server" --database-url "$DATABASE_URL" --port "$PORT" --host 127.0.0.1 \
    >"$LOG_DIR/server.log" 2>&1 &
SERVER_PID=$!
trap 'kill "$SERVER_PID" 2>/dev/null || true' EXIT

ready=""
for _ in $(seq 1 40); do
    if curl -s -o /dev/null "$SERVER_BASE/v1/threads"; then ready=1; break; fi
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

# 1. The bootstrap human (the enroll needs no principal header).
HUMAN_JSON="$(curl -s -X POST "$SERVER_BASE/v1/enrollments" \
    -H 'content-type: application/json' --data '{"kind":"human","name":"load-human"}')"
TENANT="$(printf '%s' "$HUMAN_JSON" | jq -r .tenant_id)"
HUMAN="$(printf '%s' "$HUMAN_JSON" | jq -r .principal_id)"
[ -n "$TENANT" ] && [ "$TENANT" != "null" ] || { echo "error: the enroll failed: $HUMAN_JSON" >&2; exit 2; }

# 2. The load thread (the contribute path rides its aggregate row).
CREATE_BODY="$(jq -nc --arg tenant "$TENANT" \
    '{tenant_id:$tenant,subject:"the load thread",objective:"measure the ingress→commit",budget:{calls:9999}}')"
CREATED="$(curl -s -X POST "$SERVER_BASE/v1/threads" -H "x-reasonbraid-principal: $HUMAN" \
    -H 'content-type: application/json' --data "$(envelope thread.create load-create "$CREATE_BODY")")"
THREAD="$(printf '%s' "$CREATED" | jq -r .thread_id)"
[ -n "$THREAD" ] && [ "$THREAD" != "null" ] || { echo "error: the create failed: $CREATED" >&2; exit 2; }

# 3. The timed concurrent loop: N contributes at C workers, each request a
#    FRESH idempotency key + request id (the full claim → authorize →
#    validate → apply path = the ingress→commit measurement).
PER_WORKER=$(( (COMMANDS + CONCURRENCY - 1) / CONCURRENCY ))
: > "$LATENCIES"
START="$(python3 -c 'import time; print(time.time())')"
WORKER_PIDS=""
for w in $(seq 1 "$CONCURRENCY"); do
    (
        for j in $(seq 1 "$PER_WORKER"); do
            KEY="load-$w-$j-$(uuidgen | tr -d - | cut -c1-8)"
            ENV="$(envelope thread.contribute "$KEY" \
                "$(jq -nc --arg tenant "$TENANT" --arg content "the load claim $w-$j" \
                    '{tenant_id:$tenant,content:$content,kind:"claim"}')")"
            curl -s -o /dev/null -w '%{http_code} %{time_total}\n' \
                -H "x-reasonbraid-principal: $HUMAN" -H 'content-type: application/json' \
                --data "$ENV" "$SERVER_BASE/v1/threads/$THREAD/commands"
        done
    ) >> "$LATENCIES" &
    WORKER_PIDS="$WORKER_PIDS $!"
done
# The WORKERS only — the server is a long-running background job that must
# NOT join the wait (a bare `wait` would hang on it forever).
# shellcheck disable=SC2086
wait $WORKER_PIDS
END="$(python3 -c 'import time; print(time.time())')"

# 4. The measured summary (the p50/p95 re-derive from the latencies file).
TOTAL="$(wc -l < "$LATENCIES" | tr -d ' ')"
FAILED="$(awk '$1 != "200"' "$LATENCIES" | wc -l | tr -d ' ')"
WALL="$(python3 -c "print(max($END - $START, 1e-9))")"
if [ "$TOTAL" -gt 0 ]; then
    P50="$(awk '{print $2}' "$LATENCIES" | sort -n | sed -n "$(( (TOTAL * 50 + 99) / 100 ))p")"
    P95="$(awk '{print $2}' "$LATENCIES" | sort -n | sed -n "$(( (TOTAL * 95 + 99) / 100 ))p")"
else
    P50=0; P95=0
fi
THROUGHPUT="$(python3 -c "print(round($TOTAL / $WALL, 1))")"

echo "load harness: $TOTAL commands at $CONCURRENCY workers in ${WALL}s"
echo "  ingress→commit p50: ${P50}s  p95: ${P95}s"
echo "  throughput: ${THROUGHPUT} commands/s  failures: $FAILED"
echo "  latencies: $LATENCIES (one 'status seconds' line per request)"

[ "$TOTAL" -ge "$COMMANDS" ] || { echo "FAIL: $TOTAL of $COMMANDS commands ran" >&2; exit 1; }
[ "$FAILED" -eq 0 ] || { echo "FAIL: $FAILED commands did not commit with 200" >&2; exit 1; }
echo "PASS: every command committed (200) and the summary is recorded"
