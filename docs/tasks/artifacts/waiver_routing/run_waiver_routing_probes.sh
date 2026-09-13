#!/usr/bin/env bash
# docs/tasks/artifacts/waiver_routing/run_waiver_routing_probes.sh
# REASONBRAID-MAINTENANCE.2.2 — RED / GREEN / CONTROL probes for scripts/check_waiver_routing.sh.
#
# The doctrine: a task leaf claiming a gate DOES NOT APPLY must name the leaf that owns fixing
# the gate. A waiver is a bug report about the gate — the highest-signal one it can receive —
# so it must be routed, never left inert.
#
# ⭐⭐ THE ARM THAT MATTERS IS CTRL-1. This check was adopted from a real project, and the
# version there carries a latent FAIL-OPEN that was fixed on the way in rather than inherited:
#   `printf '%s\n' "$added" | grep -qE "$RE" || continue`
# Under `set -o pipefail`, `grep -q` exits at its FIRST match and closes the pipe; once the
# producer is large enough the upstream `printf` takes SIGPIPE (141), and pipefail promotes 141
# to the pipeline's status. The `|| continue` then SKIPS the file — so a large task leaf
# containing an unrouted waiver passes silently. CTRL-1 builds exactly that input and runs BOTH
# implementations: the unfixed form must MISS it, the shipped form must CATCH it. Without that
# arm, "we fixed it" is a claim.
#
# ⚠️ THE THRESHOLD IS NOT A FLAT 64 KiB, and the first fixture was built on that assumption and
# FAILED TO REPRODUCE — the probe caught the over-claim, not review. Measured on this platform:
#   65,606 B -> PIPESTATUS=(0 0)   (no SIGPIPE: writer fits in the pipe + grep's read-ahead)
#  131,139 B -> PIPESTATUS=(141 0) (SIGPIPE, fail-open)
# i.e. the effective bound is the pipe capacity PLUS whatever the consumer buffers before
# exiting. The fixture is therefore built well past the measured band, not just past 64 KiB.
#
# Each probe builds a throwaway git repository, because the check reads the STAGED diff.
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"
GUARD="$ROOT/scripts/check_waiver_routing.sh"
[ -f "$GUARD" ] || { echo "probe: REFUSED — $GUARD not found" >&2; exit 2; }

pass=0; fail=0
# ⛔ §13: scratch is REPOSITORY-derived, never ambient (SIGNOFF-REPAIR.11.2.2).
# This probe CLONES throwaway git repositories into $WORK, so a bare `mktemp -d`
# put whole repositories on another volume — measured at device 16777232 against
# the checkout's 16777244. `mktemp` still names it by exclusive creation.
mkdir -p "$ROOT/target/doctrine_scratch"
WORK="$(mktemp -d "$ROOT/target/doctrine_scratch/waiver-routing-probe.XXXXXX")"; trap 'rm -rf "$WORK"' EXIT

# $1 = name, $2 = guard to install -> prints the repo dir
mkrepo() {
  local d="$WORK/$1" guard="$2"
  rm -rf "$d"; mkdir -p "$d/scripts" "$d/docs/tasks"
  cp "$guard" "$d/scripts/check_waiver_routing.sh"
  git -C "$d" init -q .
  git -C "$d" config user.email probe@example.invalid
  git -C "$d" config user.name  probe
  printf '# seed\n' > "$d/docs/tasks/SEED.md"
  git -C "$d" add -A >/dev/null 2>&1
  git -C "$d" commit -qm "SEED-0001: seed" >/dev/null 2>&1
  printf '%s' "$d"
}

run() { ( cd "$1" && bash scripts/check_waiver_routing.sh 2>&1 ); }

probe() { # label · repo · expected exit (or nonzero) · expected substring
  local label="$1" d="$2" want="$3" want_txt="${4:-}" out rc ok=1
  out="$(cd "$d" && bash scripts/check_waiver_routing.sh 2>&1)"; rc=$?
  case "$want" in nonzero) [ "$rc" -ne 0 ] || ok=0 ;; *) [ "$rc" -eq "$want" ] || ok=0 ;; esac
  [ -n "$want_txt" ] && { printf '%s' "$out" | grep -qF "$want_txt" || ok=0; }
  if [ "$ok" -eq 1 ]; then
    printf '  ✓ %-10s exit=%s  %s\n' "$label" "$rc" "${want_txt:-<any>}"; pass=$((pass+1))
  else
    printf '  ✗ %-10s exit=%s (wanted %s / %s)\n' "$label" "$rc" "$want" "${want_txt:-<any>}"
    printf '%s\n' "$out" | sed 's/^/        /'; fail=$((fail+1))
  fi
}

echo "== WAIVER-ROUTING probes =="

# ------------------------------------------------------------------ GREEN-1: ordinary leaf
d="$(mkrepo green1 "$GUARD")"
printf '# TREE\n\n- ordinary work, no waiver claim here.\n' > "$d/docs/tasks/TREE.md"
git -C "$d" add -A >/dev/null
probe GREEN-1 "$d" 0 "waiver-routing: OK"

# ------------------------------------------------------------------ RED-1: unrouted waiver
d="$(mkrepo red1 "$GUARD")"
printf '# TREE\n\n- [x] ROOT CAUSE — the diagnosis-toolbox signatures do not apply to this defect class.\n' \
  > "$d/docs/tasks/TREE.md"
git -C "$d" add -A >/dev/null
probe RED-1 "$d" nonzero "without naming the leaf that owns fixing it"

# ------------------------------------------------------------------ GREEN-2: honesty is legal
# The same waiver, discharged by naming an owner. The doctrine must NOT punish the note.
d="$(mkrepo green2 "$GUARD")"
printf '# TREE\n\n- [x] ROOT CAUSE — the diagnosis-toolbox signatures do not apply to this defect\n  class (gate gap owned by TREE-NAME.5).\n' \
  > "$d/docs/tasks/TREE.md"
git -C "$d" add -A >/dev/null
probe GREEN-2 "$d" 0 "waiver-routing: OK"

# ------------------------------------------------------------------ ⭐⭐ CTRL-1: the fail-open
# Build the UNFIXED variant (the pipe form) and a >64 KiB staged addition whose waiver phrase
# appears EARLY, so grep -q exits immediately and the producer takes SIGPIPE.
unfixed="$WORK/unfixed_guard.sh"
python3 - "$GUARD" "$unfixed" <<'PY'
import sys, pathlib, re
src, dst = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
s = src.read_text()
# Splice on the EXACT three-line fixed block. Matching a bare `rm -f "$added_file"` finds the
# FIRST occurrence, which sits INSIDE the `if ... ; continue; fi` line, and leaves a dangling
# `; continue; fi` -- a syntax error, which the probe then scores as "exit 2" rather than as
# the fail-open it is meant to demonstrate. Caught by CTRL-1 refusing to pass.
block = [ln for ln in s.split("\n") if 'added_file=' in ln or 'grep -qE "$WAIVER_RE" "$added_file"' in ln
          or ln.strip() == 'rm -f "$added_file"']
assert len(block) == 3, block
i0 = s.index(block[0]); i1 = s.index(block[2], i0) + len(block[2])
s = s[:i0] + '''  printf '%s\\n' "$added" | grep -qE "$WAIVER_RE" || continue''' + s[i1:]
dst.write_text(s)
PY
chmod +x "$unfixed"

mkbig() { # $1 = repo dir — a leaf whose ADDED text far exceeds the pipe buffer
  { printf '# TREE\n\n- [x] ROOT CAUSE — the diagnosis-toolbox signatures do not apply here.\n\n'
    i=0; while [ "$i" -lt 4200 ]; do
      printf -- '- filler line %s: %s\n' "$i" "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      i=$((i+1))
    done
  } > "$1/docs/tasks/TREE.md"
}

d_fix="$(mkrepo ctrl1_fixed "$GUARD")";   mkbig "$d_fix";   git -C "$d_fix" add -A >/dev/null
d_unf="$(mkrepo ctrl1_unfixed "$unfixed")"; mkbig "$d_unf"; git -C "$d_unf" add -A >/dev/null
added_bytes=$(git -C "$d_fix" diff --cached -U0 -- docs/tasks/TREE.md | grep '^+' | grep -v '^+++' | wc -c | tr -d ' ')
printf '  (CTRL-1 fixture: staged addition = %s bytes, well past the measured 65,606-131,139 B band)\n' "$added_bytes"

out_fix="$(cd "$d_fix" && bash scripts/check_waiver_routing.sh 2>&1)"; rc_fix=$?
out_unf="$(cd "$d_unf" && bash scripts/check_waiver_routing.sh 2>&1)"; rc_unf=$?
if [ "$rc_fix" -ne 0 ] && [ "$rc_unf" -eq 0 ]; then
  printf '  ✓ %-10s shipped form CATCHES (exit=%s); unfixed pipe form MISSES (exit=%s) ⇒ the port closed a real FAIL-OPEN\n' \
    "CTRL-1" "$rc_fix" "$rc_unf"; pass=$((pass+1))
else
  printf '  ✗ %-10s fixed=%s unfixed=%s — expected fixed!=0 and unfixed==0\n' "CTRL-1" "$rc_fix" "$rc_unf"
  fail=$((fail+1))
fi

# ------------------------------------------------------------------ CTRL-2: scope, not capability
# An honest SCOPE statement ("pure-docs slice, the code gate does not apply") is correct and is
# NOT a bug report about the gate. It must not be bound.
d="$(mkrepo ctrl2 "$GUARD")"
printf '# TREE\n\n- this slice is pure-docs, so the code-change gate is not applicable.\n' \
  > "$d/docs/tasks/TREE.md"
git -C "$d" add -A >/dev/null
probe CTRL-2 "$d" 0 "waiver-routing: OK"

echo
printf 'probes: %d pass / %d fail\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
