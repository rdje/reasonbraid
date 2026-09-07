#!/usr/bin/env bash
# scripts/demo_two_host.sh — the WP6 two-host crash/reconnect demonstration
# (`PHASE-0.6.2`; ROADMAP §26.1 Demonstration A, KICKOFF WP6).
#
# Orchestrates the control plane (rb-server + PostgreSQL) and one or two node
# workers (rb-node + SQLite journals + the deterministic fake adapter) through the
# whole acceptance scenario, with REAL kill points, and writes a reproducible
# evidence bundle under target/demo/<run-id>/:
#
#   1. enroll a human and two agent roles; create a thread with a budget;
#   1b. issue one-time enrollment tokens and ENROLL the two nodes (`.1.2.1`);
#       the `.1.2.2` authenticated handshake (HMAC key-proof), the lease/fencing
#       token, and the heartbeats ride the enrolled dev secrets from here on;
#   2. invite agent A — a PENDING invitation (no work yet); A ACCEPTS
#      (`.1.3.1` explicit participants) — the accept transaction lands the work
#      item in A's inbox WITH a reservation;
#   3. A contributes through the node channel; the result folds into the thread;
#      A's presence is observable ONLINE through the channel API;
#   4. duplicate transport: the SAME event is re-POSTed verbatim (with A's live
#      fencing token) — one domain effect;
#   5. the SERVER is killed and restarted — every accepted command survives; the
#      durable lease + presence survive with them;
#   6. the human challenges A's contribution — revise work reaches A's inbox;
#      the human contributes a position carrying an EVIDENCE reference (`.1.5.1`);
#   7. A's revise attempt hangs after dispatch; the node is KILLED (SIGKILL);
#      on restart the attempt recovers `outcome_unknown` — bounded and visible,
#      NEVER silently retried; the challenge stays unresolved;
#   8. a second thread with calls:1 exhausts its budget: the second dispatch is
#      DENIED at the server (denial row) and refused by the node's budget gate
#      (`failed_before_dispatch`, no provider contact);
#   9. the human closes thread A — closure preserves the contribution AND the
#      unresolved challenge; the audit view reconstructs the whole story
#      (commands, participants, authority, costs, stop reasons) through the
#      supported read surfaces only (`.1.8.1`).
#
# Exit status: 0 only when EVERY acceptance point above is verified by grep-able
# evidence; nonzero otherwise. No human copies messages: the agent content comes
# from the fake adapter's scripted chunks, never from the operator's keyboard.
#
# Two-host support: nodes run locally by default; pass `--node-host <host>` AND
# `--remote-workdir <dir>` (a caller-authorized remote scratch dir on that host,
# e.g. `~/rb-demo`) to drive the node processes over ssh instead — the binaries
# are copied there and the journals live there (node-local state is device-local
# by design, not project-owned data). env.txt records which mode ran.
#
# Usage:
#   bash scripts/demo_two_host.sh --database-url postgres://...     # local nodes
#   bash scripts/demo_two_host.sh --database-url ... --release      # release-built
#       binaries (PHASE-1.7.2 — the packaging proof; requires `make release`)
#   bash scripts/demo_two_host.sh --database-url ... --node-host h2 \
#        --remote-workdir '~/rb-demo'                               # real two hosts
# Env: RB_DEMO_KEEP=1 keeps the run dir on failure for inspection.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DATABASE_URL="${DATABASE_URL:-}"
SERVER_HOST="127.0.0.1"
SERVER_PORT="4311"
NODE_HOST=""            # empty = run nodes locally
REMOTE_WORKDIR=""       # required when NODE_HOST is set
RUN_ID="$(date +%Y%m%d-%H%M%S)"
KEEP="${RB_DEMO_KEEP:-0}"
RELEASE="${RELEASE:-0}"

while [ $# -gt 0 ]; do
    case "$1" in
        --database-url) DATABASE_URL="$2"; shift 2 ;;
        --server-host) SERVER_HOST="$2"; shift 2 ;;
        --server-port) SERVER_PORT="$2"; shift 2 ;;
        --node-host) NODE_HOST="$2"; shift 2 ;;
        --remote-workdir) REMOTE_WORKDIR="$2"; shift 2 ;;
        --run-id) RUN_ID="$2"; shift 2 ;;
        --release) RELEASE=1; shift ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

if [ -z "$DATABASE_URL" ]; then
    echo "error: --database-url (or \$DATABASE_URL) is required" >&2
    exit 2
fi
if [ -n "$NODE_HOST" ] && [ -z "$REMOTE_WORKDIR" ]; then
    echo "error: --remote-workdir is required when --node-host is set" >&2
    exit 2
fi
command -v jq >/dev/null || { echo "error: jq is required (duplicate-delivery reconstruction)" >&2; exit 2; }
command -v psql >/dev/null || { echo "error: psql is required (the .1.2.2 lease/fencing evidence)" >&2; exit 2; }

WORK="$ROOT/target/demo/$RUN_ID"        # repo-root-relative, same volume (§13)
EVIDENCE="$WORK/evidence"
mkdir -p "$WORK" "$EVIDENCE"

BIN_ROOT="$ROOT/target/debug"
if [ "$RELEASE" = "1" ]; then BIN_ROOT="$ROOT/target/release"; fi
BIN_SERVER="$BIN_ROOT/rb-server"
BIN_NODE="$BIN_ROOT/rb-node"
BIN_JOURNAL="$BIN_ROOT/rb-journal"
BIN_CLI="$BIN_ROOT/rb"

SERVER_BASE="http://$SERVER_HOST:$SERVER_PORT"
SERVER_PID=""
FAILURES=0

# ── helpers ─────────────────────────────────────────────────────────────────────

log() { printf '[%s] %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$EVIDENCE/timeline.txt"; }

fail() {
    log "FAIL: $*"
    FAILURES=$((FAILURES + 1))
}

check() { # check <label> <probe...>
    local label="$1"; shift
    if "$@" >/dev/null 2>&1; then
        log "PASS: $label"
    else
        fail "$label"
    fi
}

wait_for() { # wait_for <label> <timeout-s> <probe...>
    local label="$1" timeout="$2"; shift 2
    local deadline=$((SECONDS + timeout))
    while [ $SECONDS -lt $deadline ]; do
        if "$@" >/dev/null 2>&1; then return 0; fi
        sleep 0.3
    done
    # Never abort under `set -e`: the failure is already recorded; the summary
    # exit status decides.
    fail "$label (timed out after ${timeout}s)"
    return 0
}

cli() { "$BIN_CLI" --server "$SERVER_BASE" "$@"; }

# The bash -c probes run `cli`/`node_journal` as exported functions; their
# captured variables must be exported with them.
export BIN_CLI BIN_JOURNAL NODE_HOST SERVER_BASE WORK
export -f cli

# The node's work dir on ITS host (local: under the repo's target/; remote: the
# caller-authorized scratch dir).
node_dir() {
    if [ -z "$NODE_HOST" ]; then echo "$WORK/nodes/$1"; else echo "$REMOTE_WORKDIR/$1"; fi
}

# node_exec <dir> <rb-node args...>  — starts a node, its pid written to <dir>/node.pid.
# The journal path is passed by the caller as an ABSOLUTE local path; remote mode
# remaps the local node dir prefix to the caller-authorized remote scratch dir.
node_exec() {
    local dir="$1"; shift
    if [ -z "$NODE_HOST" ]; then
        mkdir -p "$dir"
        "$BIN_NODE" "$@" &
        echo $! > "$dir/node.pid"
    else
        local remote_args=""
        local a
        for a in "$@"; do
            remote_args="$remote_args $(printf '%s' "$a" | sed "s|$WORK/nodes/|$REMOTE_WORKDIR/|")"
        done
        ssh "$NODE_HOST" "mkdir -p '$dir' && cd '$dir' && nohup ./rb-node $remote_args > node.log 2>&1 & echo \$! > '$dir/node.pid'"
    fi
}

node_kill() { # node_kill <dir>  — SIGKILL the node recorded in <dir>/node.pid.
    local dir="$1"
    if [ -z "$NODE_HOST" ]; then
        [ -f "$dir/node.pid" ] && kill -9 "$(cat "$dir/node.pid")" >/dev/null 2>&1 || true
    else
        ssh "$NODE_HOST" "[ -f '$dir/node.pid' ] && kill -9 \$(cat '$dir/node.pid')" >/dev/null 2>&1 || true
    fi
}

node_wait() { # node_wait <dir>  — reap the local job so bash prints no Killed banner.
    local dir="$1"
    [ -z "$NODE_HOST" ] && [ -f "$dir/node.pid" ] && wait "$(cat "$dir/node.pid")" 2>/dev/null || true
}

# node_journal <dir> <rb-journal args...>
node_journal() {
    local dir="$1"; shift
    if [ -z "$NODE_HOST" ]; then
        ( cd "$dir" && exec "$BIN_JOURNAL" "$@" )
    else
        ssh "$NODE_HOST" "cd '$dir' && exec ./rb-journal $*"
    fi
}
export -f node_journal

cleanup() {
    node_kill "$(node_dir a)" || true
    node_kill "$(node_dir b)" || true
    if [ -n "$SERVER_PID" ]; then kill -9 "$SERVER_PID" >/dev/null 2>&1 || true; fi
    wait >/dev/null 2>&1 || true
}
trap cleanup EXIT

# ── environment record ──────────────────────────────────────────────────────────

{
    echo "run_id: $RUN_ID"
    echo "git_rev: $(git rev-parse HEAD)"
    echo "server: $SERVER_BASE"
    echo "bin_root: $BIN_ROOT ($([ "$RELEASE" = "1" ] && echo release || echo debug))"
    echo "node_host: ${NODE_HOST:-<local>}"
    echo "remote_workdir: ${REMOTE_WORKDIR:-<local: $WORK/nodes>}"
    echo "database_url: $DATABASE_URL"
    echo "date_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "channel: .1.2.2 authenticated (channel_version 2) — key-proof handshake,"
    echo "  lease/fencing token, heartbeats, observable presence"
    echo "limitation: fake adapter (deterministic) — the REAL-harness leg is the"
    echo "  RB_LIVE_CODEX=1 codex_live suite, out of scope for a no-token demo (WP6)"
    echo "limitation: dev trust store — the node secrets (channel-auth.txt) are the"
    echo "  .6.1 dev-profile stance, NOT production workload identity (ADR-006/ADR-007, Phase 2)"
} > "$EVIDENCE/env.txt"

# ── build ───────────────────────────────────────────────────────────────────────

if [ "$RELEASE" = "1" ]; then
    log "building the four release binaries (cargo build --release --bins)"
    cargo build --release --bins -q
else
    log "building the four binaries (cargo build --bins)"
    cargo build --bins -q
fi

if [ -n "$NODE_HOST" ]; then
    log "deploying rb-node + rb-journal to $NODE_HOST:$REMOTE_WORKDIR"
    ssh "$NODE_HOST" "mkdir -p '$REMOTE_WORKDIR'"
    scp -q "$BIN_NODE" "$BIN_JOURNAL" "$NODE_HOST:$REMOTE_WORKDIR/"
fi

# ── 1. control plane ────────────────────────────────────────────────────────────

log "starting rb-server on $SERVER_BASE"
"$BIN_SERVER" --host "$SERVER_HOST" --port "$SERVER_PORT" --database-url "$DATABASE_URL" \
    >"$WORK/server.log" 2>&1 &
SERVER_PID=$!
wait_for "server listens" 20 curl -s -o /dev/null "$SERVER_BASE/v1/threads"

export REASONBRAID_CLI_STATE="$WORK/.cli"   # repo-local CLI state dir (§13)

log "enrolling the human organizer (bootstraps the tenant)"
ENROLL_HUMAN="$(cli enroll human organizer --json)"
TENANT="$(printf '%s' "$ENROLL_HUMAN" | jq -r .tenant_id)"
HUMAN="$(printf '%s' "$ENROLL_HUMAN" | jq -r .principal_id)"
[ -n "$TENANT" ] && [ -n "$HUMAN" ] || { fail "enroll human returned ids"; exit 1; }

log "enrolling agent roles agent-a and agent-b"
ROLE_A="$(cli enroll role agent-a --tenant "$TENANT" --json | jq -r .principal_id)"
ROLE_B="$(cli enroll role agent-b --tenant "$TENANT" --json | jq -r .principal_id)"
[ -n "$ROLE_A" ] && [ -n "$ROLE_B" ] || { fail "enroll roles returned ids"; exit 1; }

# The `.1.2.1` enrollment bootstrap: the dev wiring's node id IS the role wire
# id it serves. One token per node, bound to the dev host claim; the node
# consumes it with its dev secret, and every later handshake proves the secret.
log "issuing one-time enrollment tokens and node dev secrets (`.1.2.1`)"
ISSUE_A="$(cli node issue-token --node "$ROLE_A" --host-claim dev-host --as organizer --tenant "$TENANT" --json)"
TOKEN_A="$(printf '%s' "$ISSUE_A" | jq -r .token_id)"
NONCE_A="$(printf '%s' "$ISSUE_A" | jq -r .nonce)"
ISSUE_B="$(cli node issue-token --node "$ROLE_B" --host-claim dev-host --as organizer --tenant "$TENANT" --json)"
TOKEN_B="$(printf '%s' "$ISSUE_B" | jq -r .token_id)"
NONCE_B="$(printf '%s' "$ISSUE_B" | jq -r .nonce)"
SECRET_A="dev-secret-a-$RUN_ID"
SECRET_B="dev-secret-b-$RUN_ID"
[ -n "$TOKEN_A" ] && [ -n "$TOKEN_B" ] || { fail "node tokens issued"; exit 1; }
{
    echo "# the .1.2.2 authenticated-channel facts of this run"
    echo "node_a: $ROLE_A"
    echo "node_b: $ROLE_B"
    echo "secret_a: $SECRET_A"
    echo "secret_b: $SECRET_B"
    echo "token_a: $TOKEN_A"
    echo "token_b: $TOKEN_B"
    echo "# dev trust store (.6.1): the secrets are the handshake keys, NOT"
    echo "# production workload identity (ADR-006/ADR-007, Phase 2)"
} > "$EVIDENCE/channel-auth.txt"

# The authenticated-channel probes (bash -c / wait_for) need these.
export ROLE_A ROLE_B DATABASE_URL
probe_poll() {
    local tok
    # The fencing token is the node's channel CREDENTIAL. The demo reads it via
    # psql ONLY to drive the authenticated probes as the node itself — no
    # supported surface should expose a live credential (least privilege), and
    # every STATE assertion elsewhere runs through the CLI/API/rb-journal
    # (the `.1.8` census note — this read is an oracle, not an inspection).
    tok="$(psql "$DATABASE_URL" -Atc "SELECT fencing_token FROM node_leases WHERE node_id = '$ROLE_A'")"
    [ -n "$tok" ] || return 1
    curl -s -o /dev/null -X POST -H 'content-type: application/json' \
        -d "{\"channel_version\":2,\"node_id\":\"$ROLE_A\",\"after_cursor\":0,\"fencing_token\":\"$tok\"}" \
        "$SERVER_BASE/v1/nodes/poll"
}
export -f probe_poll

log "creating thread A (default budget)"
THREAD_A="$(cli thread create --subject "is the claim justified?" \
    --objective "produce an independent answer and survive a crash mid-revision" \
    --as organizer --json | jq -r .thread_id)"
[ -n "$THREAD_A" ] || { fail "thread A created"; exit 1; }
log "thread A: $THREAD_A"

log "inviting agent-a — a PENDING invitation (`.1.3.1`: no work yet)"
cli thread invite --thread "$THREAD_A" --agent agent-a --as organizer >/dev/null
log "agent-a ACCEPTS — the accept dispatches the work WITH a reservation"
cli thread accept --thread "$THREAD_A" --as agent-a >/dev/null

# ── 3. node A contributes ───────────────────────────────────────────────────────

NODE_A_DIR="$(node_dir a)"
log "starting node A (fake adapter: the scripted independent answer) — enrolling first"
node_exec "$NODE_A_DIR" \
    --journal "$NODE_A_DIR/node.db" \
    --server "$SERVER_BASE" \
    --node-id "$ROLE_A" \
    --fake-script '[{"step":"emit_chunk","chunk":"AGENT-A: the claim holds only for"},{"step":"emit_chunk","chunk":" x<1; for x>=1 the bound fails."},{"step":"complete"}]' \
    --poll-ms 200 \
    --enroll-token "$TOKEN_A" --enroll-nonce "$NONCE_A" --node-secret "$SECRET_A" \
    >"$WORK/node-a.log" 2>&1

wait_for "node A contributes" 60 bash -c \
    "cli inspect thread '$THREAD_A' --as organizer --tenant '$TENANT' --json | grep -q contribution_submitted"
CONTRIBUTION_ID="$(cli inspect thread "$THREAD_A" --as organizer --tenant "$TENANT" --json \
    | jq -r '.events.events[] | select(.event_type == "thread.contribution_submitted") | .event_id' | head -1)"
[ -n "$CONTRIBUTION_ID" ] || { fail "contribution event id extracted"; exit 1; }
log "node A's contribution landed (event $CONTRIBUTION_ID)"

# no human copies messages: the content came from the adapter script, not the CLI.
check "the agent content is the adapter's scripted chunks (no human relay)" bash -c \
    "cli inspect thread '$THREAD_A' --as organizer --tenant '$TENANT' --json | grep -q 'AGENT-A: the claim holds only for'"

# The `.1.5.2` rounds surface: the human advances the round — the thread's round
# fact moves (server-assigned), and the contribution above carries round 1.
ADVANCE_OUT="$(cli thread advance-round --thread "$THREAD_A" --as organizer --tenant "$TENANT" --json)"
echo "$ADVANCE_OUT" | grep -q thread.round_advanced \
    || { fail "advance-round emitted thread.round_advanced"; exit 1; }
ROUND_NOW="$(cli inspect thread "$THREAD_A" --as organizer --tenant "$TENANT" --json \
    | jq -r '.thread.state.current_round')"
check "the thread advanced to round 2 (projection fact)" bash -c "[ '$ROUND_NOW' -eq 2 ]"
CONTRIB_ROUND="$(cli inspect thread "$THREAD_A" --as organizer --tenant "$TENANT" --json \
    | jq -r '[.events.events[] | select(.event_type == "thread.contribution_submitted")][0].body.round')"
check "the contribution carries its round (round 1)" bash -c "[ '$CONTRIB_ROUND' -eq 1 ]"

# The `.1.2.2` presence surface: the enrolled + handshaked node is observably
# ONLINE through the channel API (a derived fact of its live lease).
curl -s "$SERVER_BASE/v1/nodes/presence?node_id=$ROLE_A" > "$EVIDENCE/presence-a-online.json"
check "node A's presence is observable ONLINE through the channel API" \
    grep -q '"online":true' "$EVIDENCE/presence-a-online.json"

# ── 4. duplicate transport → one domain effect ─────────────────────────────────

# The duplicate rides the node's LIVE fencing token (the lease the handshake
# issued): it re-sends the exact authenticated event, verbatim. The token is a
# CREDENTIAL — psql is the demo's oracle for it (no supported surface should
# expose a live credential); every STATE assertion runs through the CLI/API/
# rb-journal (the `.1.8` census note).
FENCE_A="$(psql "$DATABASE_URL" -Atc "SELECT fencing_token FROM node_leases WHERE node_id = '$ROLE_A'")"
[ -n "$FENCE_A" ] || { fail "node A holds a live lease (fencing token present)"; exit 1; }
EVENTS_JSON="$(node_journal "$NODE_A_DIR" events node.db --json)"
DUP_BODY="$(printf '%s' "$EVENTS_JSON" | jq -c --arg n "$ROLE_A" --arg f "$FENCE_A" '
    { channel_version: 2,
      node_id: $n,
      event_id: .events[0].event_id,
      operation_id: .events[0].operation_id,
      payload: (.events[0].payload | fromjson),
      fencing_token: $f }')"
log "duplicating the delivery: re-POSTing $(printf '%s' "$DUP_BODY" | jq -r .event_id) verbatim"
curl -s -X POST "$SERVER_BASE/v1/nodes/events" \
    -H 'content-type: application/json' \
    -d "$DUP_BODY" > "$EVIDENCE/duplicate-delivery.json"
check "the duplicate transport is refused (accepted:false)" \
    grep -q '"accepted":false' "$EVIDENCE/duplicate-delivery.json"
CONTRIBUTIONS="$(cli inspect thread "$THREAD_A" --as organizer --tenant "$TENANT" --json \
    | jq '[.events.events[] | select(.event_type == "thread.contribution_submitted")] | length')"
check "exactly ONE contribution despite the duplicate" \
    bash -c "[ '$CONTRIBUTIONS' -eq 1 ]"

# ── 5. server restart loses nothing ─────────────────────────────────────────────

log "killing the control plane (SIGKILL) and restarting it on the same store"
kill -9 "$SERVER_PID" >/dev/null 2>&1 || true
wait "$SERVER_PID" >/dev/null 2>&1 || true
"$BIN_SERVER" --host "$SERVER_HOST" --port "$SERVER_PORT" --database-url "$DATABASE_URL" \
    >>"$WORK/server.log" 2>&1 &
SERVER_PID=$!
wait_for "server is back" 20 curl -s -o /dev/null "$SERVER_BASE/v1/threads"
# probe_poll re-reads the CURRENT fencing token every attempt: node A
# re-handshakes after the restart (rotating its token), so a stale capture
# would race — the probe fetches the live lease each time instead.
wait_for "server answers authenticated node polls after the restart" 30 probe_poll
check "every accepted command survived the restart" bash -c \
    "cli inspect thread '$THREAD_A' --as organizer --tenant '$TENANT' --json | grep -q 'thread.created' && cli inspect thread '$THREAD_A' --as organizer --tenant '$TENANT' --json | grep -q 'participant_invited'"
curl -s "$SERVER_BASE/v1/nodes/presence?node_id=$ROLE_A" > "$EVIDENCE/presence-a-after-restart.json"
check "node A's durable lease + presence survived the server restart" \
    grep -q '"online":true' "$EVIDENCE/presence-a-after-restart.json"

# ── 6. challenge → revise work reaches the node ────────────────────────────────

# Kill node A FIRST so the revise work waits in its inbox — the hang script must
# own the revise attempt (the crash-after-dispatch leg).
log "stopping node A so the revise work queues in its inbox"
node_kill "$NODE_A_DIR"
node_wait "$NODE_A_DIR"

log "the human challenges A's contribution"
cli thread challenge --thread "$THREAD_A" --target "$CONTRIBUTION_ID" \
    --text "your bound check only covers x<1 — justify x>=1" --as organizer >/dev/null

# The `.1.5.1` evidence-reference surface: the human contributes a position that
# CITES a source (references only — no acquisition, §3.7). The reconstruction
# beat (section 11) asserts the reference rides the event.
log "the human contributes a position with an evidence reference (.1.5.1)"
EVID_CONTRIB_OUT="$(cli thread contribute --thread "$THREAD_A" --kind position \
    --text "the bound is a modeling assumption, not a theorem — see the cited derivation" \
    --evidence-uri "https://example.org/bound-derivation" \
    --as organizer --tenant "$TENANT" --json)"
echo "$EVID_CONTRIB_OUT" | grep -q thread.contribution_submitted \
    || { fail "the evidence-referenced contribution"; exit 1; }

# ── 7. kill AFTER dispatch → honest ambiguity, never a silent retry ─────────────

log "starting node A with a HANG script (the queued revise attempt will dispatch, then hang)"
node_exec "$NODE_A_DIR" \
    --journal "$NODE_A_DIR/node.db" \
    --server "$SERVER_BASE" \
    --node-id "$ROLE_A" \
    --fake-script '[{"step":"emit_chunk","chunk":"AGENT-A: revising…"},{"step":"hang_forever"}]' \
    --poll-ms 200 \
    --node-secret "$SECRET_A" \
    >"$WORK/node-a.log" 2>&1

wait_for "the revise attempt is DISPATCHED (durable boundary record)" 30 bash -c \
    "node_journal '$NODE_A_DIR' pending node.db --json | grep -q dispatched"
log "killing node A AFTER dispatch (SIGKILL — the crash)"
node_kill "$NODE_A_DIR"
node_wait "$NODE_A_DIR"

log "restarting node A on the same journal with the normal script"
node_exec "$NODE_A_DIR" \
    --journal "$NODE_A_DIR/node.db" \
    --server "$SERVER_BASE" \
    --node-id "$ROLE_A" \
    --fake-script '[{"step":"emit_chunk","chunk":"AGENT-A: a normal answer"},{"step":"complete"}]' \
    --poll-ms 200 \
    --node-secret "$SECRET_A" \
    >"$WORK/node-a.log" 2>&1

wait_for "recovery classifies the attempt outcome_unknown" 30 bash -c \
    "node_journal '$NODE_A_DIR' ambiguous node.db --json | grep -q outcome_unknown"
node_journal "$NODE_A_DIR" ambiguous node.db --json > "$EVIDENCE/journal-a-ambiguous.json"
check "the ambiguous attempt is visible with its boundary history" \
    grep -q 'dispatched' "$EVIDENCE/journal-a-ambiguous.json"
check "the revise attempt was NOT silently retried (exactly one ambiguous attempt)" \
    grep -q 'outcome_unknown=1' <(node_journal "$NODE_A_DIR" inspect node.db)
sleep 1
check "no revision entered the thread (the ambiguous attempt produced no effect)" bash -c \
    "! cli inspect thread '$THREAD_A' --as organizer --tenant '$TENANT' --json | grep -q revision_submitted"

# ── 8. budget exhaustion on thread B ────────────────────────────────────────────

log "creating thread B with calls:1 — exactly ONE provider call is budgeted"
THREAD_B="$(cli thread create --subject "tight budget" \
    --objective "one call, then the ceiling refuses" \
    --budget-calls 1 --as organizer --json | jq -r .thread_id)"
[ -n "$THREAD_B" ] || { fail "thread B created"; exit 1; }

cli thread invite --thread "$THREAD_B" --agent agent-b --as organizer >/dev/null
cli thread accept --thread "$THREAD_B" --as agent-b >/dev/null

NODE_B_DIR="$(node_dir b)"
log "starting node B (fake adapter) — enrolling first"
node_exec "$NODE_B_DIR" \
    --journal "$NODE_B_DIR/node.db" \
    --server "$SERVER_BASE" \
    --node-id "$ROLE_B" \
    --fake-script '[{"step":"emit_chunk","chunk":"AGENT-B: the single budgeted answer"},{"step":"complete"}]' \
    --poll-ms 200 \
    --enroll-token "$TOKEN_B" --enroll-nonce "$NONCE_B" --node-secret "$SECRET_B" \
    >"$WORK/node-b.log" 2>&1

wait_for "node B contributes within its budget" 60 bash -c \
    "cli inspect thread '$THREAD_B' --as organizer --tenant '$TENANT' --json | grep -q contribution_submitted"
B_CONTRIBUTION_ID="$(cli inspect thread "$THREAD_B" --as organizer --tenant "$TENANT" --json \
    | jq -r '.events.events[] | select(.event_type == "thread.contribution_submitted") | .event_id' | head -1)"

log "challenging B's contribution — the revise dispatch must be DENIED (calls:1 exhausted)"
cli thread challenge --thread "$THREAD_B" --target "$B_CONTRIBUTION_ID" \
    --text "prove it" --as organizer >/dev/null

wait_for "node B's budget gate refuses the unreserved dispatch" 30 bash -c \
    "node_journal '$NODE_B_DIR' inspect node.db | grep -q 'failed_before_dispatch=1'"
check "the denial is journaled BEFORE any provider contact (failed_before_dispatch)" \
    grep -q 'failed_before_dispatch=1' <(node_journal "$NODE_B_DIR" inspect node.db)
check "no revision entered thread B" bash -c \
    "! cli inspect thread '$THREAD_B' --as organizer --tenant '$TENANT' --json | grep -q revision_submitted"

# The `.1.5.3` honest outcome: thread B is genuinely inconclusive — the budget
# gate blocked the revision and the challenge stands. Close it INCONCLUSIVELY
# with the unresolved item named; the register rides the close event.
cli thread close --thread "$THREAD_B" --reason "budget exhausted before the revision" \
    --outcome inconclusive --unresolved "the challenge against the contribution stands" \
    --as organizer >/dev/null
check "thread B closes INCONCLUSIVELY (the honest terminal)" bash -c \
    "cli inspect thread '$THREAD_B' --as organizer --tenant '$TENANT' --json | grep -Eq '\"state\": *\"inconclusive\"'"
check "the unresolved register rides the close event" bash -c \
    "cli inspect thread '$THREAD_B' --as organizer --tenant '$TENANT' --json | grep -q 'the challenge against the contribution stands'"

# ── 9. closure preserves contributions and unresolved objections ───────────────

log "closing thread A"
cli thread close --thread "$THREAD_A" --reason "demonstration complete" --as organizer >/dev/null
check "thread A is closed" bash -c \
    "cli inspect thread '$THREAD_A' --as organizer --tenant '$TENANT' --json | grep -Eq '\"state\": *\"closed\"'"
check "closure preserved the contribution" bash -c \
    "cli inspect thread '$THREAD_A' --as organizer --tenant '$TENANT' --json | grep -q contribution_submitted"
check "closure preserved the unresolved challenge" bash -c \
    "cli inspect thread '$THREAD_A' --as organizer --tenant '$TENANT' --json | grep -Eq '\"open_challenges\": *1'"

# ── 10. the `.1.6` inspection console (`.1.6.3`: the embedded static shell) ─────

# The page is served BY the server the demo already runs — the same binary that
# owns the API. curl is the browser stand-in: the page's rendering is JS, so the
# beat asserts the SHELL and the exact data the page fetches (no browser needed).
curl -s "$SERVER_BASE/" > "$EVIDENCE/console-index.html"
check "the inspection console is served at / (the embedded shell)" bash -c \
    "grep -q 'inspection console' '$EVIDENCE/console-index.html'"
check "the shell loads its assets from the same origin" bash -c \
    "grep -q '/app.js' '$EVIDENCE/console-index.html' && grep -q '/style.css' '$EVIDENCE/console-index.html'"

curl -s "$SERVER_BASE/app.js" > "$EVIDENCE/console-app.js"
check "the page consumes ONLY the documented read surfaces" bash -c \
    "for p in '/v1/threads?' '/v1/threads/' '/events?' '/audit?' '/budget?' '/v1/nodes/presence?node_id=' '/v1/nodes/inbox?node='; do grep -qF \"\$p\" '$EVIDENCE/console-app.js' || exit 1; done"
check "the page is read-only (no write verb)" bash -c \
    "! grep -q 'POST' '$EVIDENCE/console-app.js'"

# One live same-origin fetch with the dev header — the exact data the page
# renders when the user opens THREAD_A (the header the identity form sends).
curl -s -H "x-reasonbraid-principal: $HUMAN" \
    "$SERVER_BASE/v1/threads/$THREAD_A?tenant_id=$TENANT" > "$EVIDENCE/console-thread-a.json"
check "the page's data source returns the demo's thread (the live fetch)" bash -c \
    "grep -q 'is the claim justified?' '$EVIDENCE/console-thread-a.json'"
curl -s -H "x-reasonbraid-principal: $HUMAN" \
    "$SERVER_BASE/v1/threads/$THREAD_A/budget?tenant_id=$TENANT" > "$EVIDENCE/console-budget-a.json"
check "the page's budget view returns the ledger facts (.1.6.1)" bash -c \
    "grep -q '\"ceiling\"' '$EVIDENCE/console-budget-a.json'"

# ── 11. the audit view reconstructs the story (`.1.8.1` — the §26.1 step-7 claim) ──

# The supported read surfaces — the same GETs the console page makes — rebuild the
# whole demonstration without database surgery: the commands from the event
# timeline, the authority from the audit records, the costs from the budget ledger.
# The thread-scoped audit view starts at the invite: `thread.create` is
# authorized against the TENANT scope (the thread does not exist yet — its
# audit row is tenant-scoped, the `.6.1` shape the command_api suite asserts);
# the create command is still visible in the timeline check above.
curl -s -H "x-reasonbraid-principal: $HUMAN" \
    "$SERVER_BASE/v1/threads/$THREAD_A/audit?tenant_id=$TENANT" > "$EVIDENCE/audit-a.json"
curl -s -H "x-reasonbraid-principal: $HUMAN" \
    "$SERVER_BASE/v1/threads/$THREAD_A/events?tenant_id=$TENANT" > "$EVIDENCE/events-a.json"
curl -s -H "x-reasonbraid-principal: $HUMAN" \
    "$SERVER_BASE/v1/threads/$THREAD_B/audit?tenant_id=$TENANT" > "$EVIDENCE/audit-b.json"
curl -s -H "x-reasonbraid-principal: $HUMAN" \
    "$SERVER_BASE/v1/threads/$THREAD_B/budget?tenant_id=$TENANT" > "$EVIDENCE/budget-b.json"

check "A's audit records reconstruct the thread-scoped authority (invite → accept → contribute → close)" bash -c \
    "jq -e '[.records[].action] | contains([\"thread_invite\",\"thread_invitation_respond\",\"thread_contribute\",\"thread_close\"])' '$EVIDENCE/audit-a.json' >/dev/null"
check "every A audit record carries its proof (allowed/denied + a 64-hex policy digest)" bash -c \
    "jq -e 'all(.records[]; ((.policy_digest | length) == 64) and (.decision == \"allowed\" or .decision == \"denied\"))' '$EVIDENCE/audit-a.json' >/dev/null"
check "the event timeline reconstructs A's story in order (create → accept → contribute → close)" bash -c \
    "jq -e '(([.events[].event_type] | index(\"thread.created\")) < ([.events[].event_type] | index(\"thread.invitation_accepted\"))) and (([.events[].event_type] | index(\"thread.invitation_accepted\")) < ([.events[].event_type] | index(\"thread.contribution_submitted\"))) and (([.events[].event_type] | index(\"thread.contribution_submitted\")) < ([.events[].event_type] | index(\"thread.closed\")))' '$EVIDENCE/events-a.json' >/dev/null"
check "the evidence reference rides the human contribution's event (.1.5.1)" bash -c \
    "jq -er '.events[] | select(.event_type == \"thread.contribution_submitted\") | .body.evidence_refs? | select(. != null) | .[0].uri' '$EVIDENCE/events-a.json' | grep -q '^https://example.org/bound-derivation$'"
check "B's budget ledger reconstructs the denial (a denied reservation row with the engine's reason)" bash -c \
    "jq -e '[.reservations[] | select(.status == \"denied\") | .reason] | any(startswith(\"the ceiling\"))' '$EVIDENCE/budget-b.json' >/dev/null"
check "B's audit records the close authority (the stop reason rides the thread state)" bash -c \
    "jq -e '[.records[].action] | index(\"thread_close\") != null' '$EVIDENCE/audit-b.json' >/dev/null"

# ── evidence bundle ─────────────────────────────────────────────────────────────

cli inspect thread "$THREAD_A" --as organizer --tenant "$TENANT" --json > "$EVIDENCE/thread-a.json"
cli inspect thread "$THREAD_B" --as organizer --tenant "$TENANT" --json > "$EVIDENCE/thread-b.json"
node_journal "$NODE_A_DIR" inspect node.db > "$EVIDENCE/journal-a-inspect.txt"
node_journal "$NODE_B_DIR" inspect node.db > "$EVIDENCE/journal-b-inspect.txt"
{
    echo "# WP6 two-host demonstration — acceptance evidence ($RUN_ID)"
    echo
    echo "See timeline.txt for the step log and the assertions above for PASS/FAIL."
    echo "env.txt records the hosts, revision, and the fake-adapter limitation."
    echo
    echo "| Acceptance (KICKOFF WP6 / ROADMAP §26.1) | Evidence |"
    echo "| --- | --- |"
    echo "| no human copies messages | thread-a.json: the contribution content is the adapter's scripted chunk |"
    echo "| authenticated channel (.1.2.2) | presence-a-online.json + presence-a-after-restart.json: observable ONLINE presence; duplicate-delivery.json rode the live fencing token |"
    echo "| accepted commands survive restart | timeline: server SIGKILL + restart, thread-a.json intact |"
    echo "| duplicate transport → one domain effect | duplicate-delivery.json (accepted:false); thread-a.json has one contribution |"
    echo "| no silent retry of indeterminate calls | journal-a-ambiguous.json: dispatched → outcome_unknown, one attempt, no revision |"
    echo "| budget denial prevents a new dispatch | journal-b-inspect.txt: failed_before_dispatch=1, no revision in thread-b.json |"
    echo "| closure preserves contributions + objections | thread-a.json: closed, contribution + open_challenges=1 |"
    echo "| honest inconclusive outcome | thread-b.json: state inconclusive, the unresolved item rides the close event |"
    echo "| inspection console (.1.6.3) | console-index.html (the embedded shell served at /) + console-app.js (the documented surfaces only, no write verb) + console-thread-a.json / console-budget-a.json (the live same-origin fetches) |"
    echo "| evidence reference rides the contribution (.1.5.1) | events-a.json: the human contribution's event body carries the cited uri |"
    echo "| the audit view reconstructs the story (.1.8.1) | audit-a.json (invite→accept→contribute→close authority rows, each with a 64-hex policy digest — the create's authority is tenant-scoped) + events-a.json (the ordered timeline) + audit-b.json (the close authority) + budget-b.json (the denied reservation row with the engine's reason) |"
    echo "| reproducible evidence bundle | this directory — rerun with the commands in timeline.txt |"
} > "$EVIDENCE/summary.md"

if [ "$FAILURES" -gt 0 ]; then
    log "$FAILURES acceptance check(s) FAILED — see timeline.txt"
    if [ "$KEEP" = "1" ]; then log "kept the run dir for inspection: $WORK"; fi
    exit 1
fi

log "ALL acceptance checks passed — evidence bundle in $EVIDENCE"
echo "$WORK"
exit 0
