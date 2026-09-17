#!/usr/bin/env bash
# scripts/check_self_tests.sh — SELF-TEST doctrine.
#
# Every registered check and census instrument in this repository carries a
# two-sided `--self-test`, and until this gate existed NOTHING RAN THEM. They
# were author-run: fired once while the change was being written, and never
# again.
#
# That is not hypothetical either. `scripts/census_pg_test_clusters.py`'s
# citation guard asserted that a probe name appears in ZERO tracked files — and
# wrote that probe as a literal in its own source, which is a tracked file. It
# passed exactly once, in the author's working tree before the file was added,
# and failed from the instant it was committed (`SIGNOFF-REPAIR.11.4.3.1.7.1`).
# Nobody noticed for thirty-odd commits, because nobody ran it.
#
# Measured before this gate was proposed (`SIGNOFF-REPAIR.11.4.3.1.7.2`):
# 17 scripts carried a `--self-test`, they cost 1.01 s in total, and the enforcer
# itself cost about 3.15 s. One second on a three-second gate is affordable;
# `SIGNOFF-REPAIR.11.5`'s constraint is that a gate people route around is a
# gate that lies, and nobody routes around one second.
#
# ⛔ This gate catches NOTHING today: every discovered self-test passes. Its value
# is preventing the class above — a control that silently stops working — not
# finding a present defect. Said plainly so a green run is not mistaken for
# evidence of one.
#
# ⚠️ Some check/census scripts have no `--self-test` at all. Running the
# discovered ones does not reach them, and this gate does not claim to.
#
# ⛔ NEITHER POPULATION IS WRITTEN HERE, and that is the repair rather than an
# omission. The two counts above used to be restated as live facts — "all 17
# pass", "11 of the 28" — and `SIGNOFF-REPAIR.11.16` measured them at 29 and
# 9-of-38 with nothing in the repository deriving either. The `17` survives
# because it is ANCHORED to the leaf that measured it; a number with no moment
# attached is a mirror, and a mirror nothing derives drifts. Ask for them:
#
#     scripts/check_self_tests.sh --census
#
# ⚠️ THE SELF-REFERENCE TRAP — and I walked into a variant of it while building
# this, which is why the defence is now belt AND braces rather than a careful
# exclusion list.
#
# Discovery greps tracked scripts for the self-test flag. Excluding this file was
# the obvious guard, and it was not enough: REGISTERING this check put the
# flag's literal text into `check_doctrines.sh`'s description, so discovery found
# the ENFORCER, ran it with the flag, and the enforcer — which ignores unknown
# arguments — ran every check including this one. Unbounded recursion, on the
# first `make gate` after registration. The gate exists because a control
# searched for a string it contained; building it, I made a discovery match a
# string it had just written.
#
# Two defences now, the first of which makes the failure impossible rather than
# merely unlikely:
#   1. RB_SELF_TEST_GATE is exported before any probe. A nested invocation sees
#      it and exits immediately, so no discovery mistake can recurse.
#   2. The enforcer is excluded by path: it is the RUNNER, not a check, and has
#      no self-test arm of its own.
#
# ⚠️ Residual, stated rather than hidden: a script that merely MENTIONS the flag
# without implementing it is run with an argument it ignores — i.e. run
# normally. Every script in the discovered population is a read-only check or
# census, so that is harmless today; it would not be for a mutating script, and
# a future one must be excluded here.
#
# Self-test: scripts/check_self_tests.sh --self-test
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"
SELF="scripts/check_self_tests.sh"
ENFORCER="scripts/check_doctrines.sh"
FLAG="--self-test"

# Defence 1: a nested invocation cannot run. Set before ANY probe, so a
# discovery mistake costs one immediate exit instead of unbounded recursion.
if [ -n "${RB_SELF_TEST_GATE:-}" ]; then
  exit 0
fi
export RB_SELF_TEST_GATE=1

# Every tracked script offering the flag, except this one and the enforcer that
# runs it (defence 2 — see the trap note).
discover() {
  git grep -l -- "$FLAG" -- scripts knowledge-map .githooks 2>/dev/null \
    | grep -vFx "$SELF" | grep -vFx "$ENFORCER" | sort
}

# The two populations this gate is about, printed rather than restated in prose.
# ⛔ DISCOVERED ⊄ INSTRUMENTS: discovery also reaches `knowledge-map/` and
# `.githooks/`, so a script may be run here and not be a `scripts/check_*` or
# `scripts/census_*` file. The third number is the gap WITHIN the instruments,
# which is the one the registry row is about.
if [ "${1:-}" = "--census" ]; then
  discovered="$(discover | grep -c . || true)"
  instruments="$(git ls-files 'scripts/check_*' 'scripts/census_*' | sort)"
  total="$(printf '%s\n' "$instruments" | grep -c . || true)"
  without="$(comm -23 <(printf '%s\n' "$instruments") \
                      <(git grep -l -- "$FLAG" -- scripts | sort) | grep -c . || true)"
  echo "SELF-TEST census: $discovered scripts carry a --self-test and are run by this gate;"
  echo "  of $total tracked check/census instruments, $without carry none:"
  comm -23 <(printf '%s\n' "$instruments") <(git grep -l -- "$FLAG" -- scripts | sort) \
    | sed 's/^/    /'
  exit 0
fi

# Run one script's self-test; print "path" on failure.
probe() {
  local script="$1" out
  case "$script" in
    *.py) out="$(python3 -B "$script" "$FLAG" 2>&1)" ;;
    *)    out="$(bash "$script" "$FLAG" 2>&1)" ;;
  esac
  if [ $? -ne 0 ]; then
    printf '%s\n' "$script"
    printf '%s\n' "$out" | tail -3 | sed 's/^/        /' >&2
  fi
}

if [ "${1:-}" = "$FLAG" ]; then
  fails=0
  # ⛔ §13: was `"${TMPDIR:-/tmp}"` — explicitly off-volume, in the gate that runs
  # every other gate's self-test (SIGNOFF-REPAIR.11.2.2).
  mkdir -p "$ROOT/target/doctrine_scratch"
  scratch="$(mktemp -d "$ROOT/target/doctrine_scratch/selftest-gate.XXXXXX")"
  trap 'rm -rf "$scratch"' EXIT

  # A script whose self-test FAILS must be reported...
  cat > "$scratch/bad.sh" <<'BAD'
#!/usr/bin/env bash
[ "${1:-}" = "--self-test" ] && { echo "SELF-TEST FAILED: deliberate" >&2; exit 1; }
exit 0
BAD
  # ...and one whose self-test PASSES must not be.
  cat > "$scratch/good.sh" <<'GOOD'
#!/usr/bin/env bash
[ "${1:-}" = "--self-test" ] && { echo "fine"; exit 0; }
exit 0
GOOD
  chmod +x "$scratch/bad.sh" "$scratch/good.sh"

  if [ -z "$(probe "$scratch/bad.sh" 2>/dev/null)" ]; then
    echo "SELF-TEST: a failing self-test was not reported" >&2; fails=$((fails+1))
  fi
  if [ -n "$(probe "$scratch/good.sh" 2>/dev/null)" ]; then
    echo "SELF-TEST: a passing self-test was wrongly reported" >&2; fails=$((fails+1))
  fi
  # Discovery must never return this script, nor the enforcer that runs it —
  # the trap this gate exists beside, and the one it actually fell into.
  if discover | grep -qFx "$SELF"; then
    echo "SELF-TEST: discovery returned this script, which would recurse" >&2; fails=$((fails+1))
  fi
  if discover | grep -qFx "$ENFORCER"; then
    echo "SELF-TEST: discovery returned the enforcer, which would recurse" >&2; fails=$((fails+1))
  fi
  # And the guard itself: a nested invocation must exit 0 without probing.
  if ! RB_SELF_TEST_GATE=1 bash "$SELF" >/dev/null 2>&1; then
    echo "SELF-TEST: the recursion guard did not short-circuit a nested run" >&2
    fails=$((fails+1))
  fi
  # And it must actually find the real population, or the gate is vacuous.
  found="$(discover | grep -c .)"
  if [ "$found" -lt 10 ]; then
    echo "SELF-TEST: discovery found only $found scripts — expected the real population" >&2
    fails=$((fails+1))
  fi

  [ "$fails" -eq 0 ] || exit 1
  echo "SELF-TEST self-test: a failing arm caught, a passing one ignored, neither this script nor the enforcer discovered, the recursion guard short-circuits, $found scripts discovered"
  exit 0
fi

mapfile -t scripts < <(discover)
[ "${#scripts[@]}" -gt 0 ] || exit 0

broken=()
for script in "${scripts[@]}"; do
  hit="$(probe "$script")"
  [ -z "$hit" ] || broken+=("$hit")
done
[ "${#broken[@]}" -eq 0 ] && exit 0

echo "SELF-TEST: a script's own --self-test does not pass." >&2
for script in "${broken[@]}"; do
  printf '    %s\n' "$script" >&2
done
cat >&2 <<'GUIDE'

  A control nobody runs is a control that can stop working silently. One in this
  repository passed exactly once — in the working tree before its own file was
  tracked — and failed from the moment it was committed, unnoticed for dozens of
  commits (SIGNOFF-REPAIR.11.4.3.1.7.1).

  Fix the self-test, or fix what it caught. Do NOT delete the arm to make this
  green: a guard never seen refuse is not known to work.
GUIDE
exit 1
