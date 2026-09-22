#!/usr/bin/env bash
# Hunt the pre-push script suite's rare stall (`SIGNOFF-REPAIR.11.26`).
#
# ⛔ WHY THIS IS AN INSTRUMENT AND NOT A COMMAND SOMEONE REMEMBERS. `.11.26`'s
#   acceptance is *catch ONE occurrence with the instrument now in place*, and
#   its own standing note says a reproduction attempt over an UNSTATED sample is
#   not a measurement. A hunt therefore has to report its run count, and the run
#   count has to survive the session that produced it.
# 🔴 THE LEG-3 BREACH THIS EXISTS TO STOP, MEASURED TWICE. `.11.26.2` found the
#   240-invocation distribution gone because `target/gate_child_timing.jsonl` is
#   untracked and had been replaced. The first hunt written for this leaf then
#   repeated it from the other side: it bounded disk by deleting all but the
#   last two runs' artefacts, so 179 runs produced 3 runs of evidence.
# ⭐ THE RULE THAT FOLLOWS IS NARROW: a PASSING run's artefacts are disposable
#   and a FAILING run's are the whole point, so retention is keyed on the
#   verdict rather than on age. The summary line is appended for every run.
# ⛔ NOT A GATE and never registered as one: it runs the suite N times, which
#   costs ~20-30 s per run. `COMMIT.md`'s pre-push block runs it ONCE; this is
#   what you reach for when you are hunting the failure deliberately.
# ⚠️ Every artefact it writes is under `target/` and therefore UNTRACKED. A
#   figure taken from a hunt is carried unless the leaf that publishes it says
#   so — which is `docs/CLAIM_VERIFICATION.md` leg 3, and is why this script is
#   tracked even though its output cannot be.
# CONTRACT: exit 0 when the requested runs completed with no failure, 1 when a
#   failure was caught (its artefacts retained and named), 2 on a usage error.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 2

OUT="target/stall_hunt"
HEADLINE="STALL-HUNT"
note() { printf '%s: %s\n' "$HEADLINE" "$1" >&2; }
ok()   { printf '%s: %s\n' "$HEADLINE" "$1"; }

# The suite `COMMIT.md` names as the pre-push condition, with the per-child
# timing log armed. ⛔ Its output is REDIRECTED, never piped through `tail`:
# the run that first caught this failure had its three error texts discarded by
# the way it was invoked (`SIGNOFF-REPAIR.11.26`).
run_suite() { # $1 = log path, $2 = timing jsonl path
  RB_CHILD_TIMING_LOG="$2" \
    python3 -B scripts/project_env.py python3 -B -m unittest discover \
    -s scripts/tests -p 'test_*.py' > "$1" 2>&1
}

summarise() { # reads the tsv on stdin
  awk -F'\t' '
    NF >= 2 { n++; if ($2 != 0) f++
              if ($3 != "") { s += $3; if ($3 > mx) mx = $3
                              if (mn == "" || $3 < mn) mn = $3 } }
    END { if (n == 0) { print "runs=0"; exit }
          printf "runs=%d failures=%d wall_s min=%.1f mean=%.1f max=%.1f\n",
                 n, f + 0, mn, s / n, mx }'
}

# ⛔ Defined ABOVE the self-test, because the self-test CALLS it. The first
# version put it below and arm 4 reported `command not found` — the arm
# caught its own script, which is the shape `SELF-TEST` exists for.
prune_passing() { # $1 = directory, $2 = run number whose artefacts are disposable
  rm -f "$1/run-$2.log" "$1/run-$2.jsonl"
}

if [ "${1:-}" = "--self-test" ]; then
  arms=0; passed=0
  arm() { arms=$((arms + 1)); if [ "$1" = ok ]; then passed=$((passed + 1)); ok "arm $arms $2"
          else note "arm $arms FAILED — $2"; fi; }
  work="target/stall_hunt_selftest"; rm -rf "$work"; mkdir -p "$work"
  # 1-2 the summary arithmetic, including the failure column
  got="$(printf '1\t0\t20.0\tT\n2\t0\t30.0\tT\n' | summarise)"
  [ "$got" = "runs=2 failures=0 wall_s min=20.0 mean=25.0 max=30.0" ] \
    && arm ok "the summary reports runs, failures and the wall-clock interval" \
    || arm bad "summary arithmetic gave '$got'"
  got="$(printf '1\t0\t20.0\tT\n2\t101\t60.0\tT\n' | summarise)"
  case "$got" in *"failures=1"*) arm ok "a non-zero rc is counted as a failure" ;;
    *) arm bad "the failure column is not counted: '$got'" ;; esac
  # 3 an empty ledger is reported, never divided by zero
  [ "$(printf '' | summarise)" = "runs=0" ] \
    && arm ok "an empty ledger reports runs=0 rather than dividing by zero" \
    || arm bad "the empty case is not handled"
  # 4-5 ⭐ THE RETENTION RULE, in BOTH directions — the defect this script was
  #     written after was deleting evidence, so the arm that matters is the one
  #     asserting a FAILING run's artefacts survive.
  mkdir -p "$work/runs"
  : > "$work/runs/run-1.log"; : > "$work/runs/run-1.jsonl"
  : > "$work/runs/run-2.log"; : > "$work/runs/run-2.jsonl"
  prune_passing "$work/runs" 1
  [ ! -e "$work/runs/run-1.log" ] \
    && arm ok "a PASSING run's artefacts are pruned" \
    || arm bad "a passing run's artefacts must be disposable"
  [ -e "$work/runs/run-2.log" ] \
    && arm ok "NEGATIVE — the run the prune was not asked about survives" \
    || arm bad "the prune must take only the run it is given"
  rm -rf "$work"
  ok "--self-test: arms=${passed}/${arms}"
  [ "$passed" = "$arms" ] || exit 1
  exit 0
fi

runs="${1:-30}"
case "$runs" in ''|*[!0-9]*) note "usage: $0 [runs] | --self-test"; exit 2 ;; esac
mkdir -p "$OUT"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
ledger="$OUT/hunt-$stamp.tsv"
: > "$ledger"
ok "hunting the pre-push script suite over $runs run(s); ledger $ledger"
for i in $(seq 1 "$runs"); do
  log="$OUT/run-$i.log"; jsonl="$OUT/run-$i.jsonl"
  start="$(python3 -c 'import time; print(time.time())')"
  run_suite "$log" "$jsonl"
  rc=$?
  end="$(python3 -c 'import time; print(time.time())')"
  dur="$(python3 -c "print(f'{$end-$start:.3f}')")"
  printf '%s\t%s\t%s\t%s\n' "$i" "$rc" "$dur" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$ledger"
  if [ "$rc" -ne 0 ]; then
    note "OCCURRENCE CAUGHT on run $i (rc=$rc, ${dur}s)"
    note "  the failure text is retained at $log"
    note "  the per-child timing is retained at $jsonl"
    note "  a phase carrying started_at and no elapsed_ms is the stall"
    ok "$(summarise < "$ledger")"
    exit 1
  fi
  prune_passing "$OUT" "$i"
done
ok "no occurrence: $(summarise < "$ledger")"
exit 0
