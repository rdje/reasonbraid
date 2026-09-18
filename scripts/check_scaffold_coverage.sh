#!/usr/bin/env bash
# scripts/check_scaffold_coverage.sh — SCAFFOLD-COVERAGE doctrine.
#
# Every check the doctrine registry NAMES must be carried by the scaffold's
# NEUTRAL list, or a project that pulls the spine cannot commit.
#
# THE MEASURED DEFECT (`SIGNOFF-REPAIR.11.23`). `scripts/update_scaffold.sh`
# carried `scripts/check_doctrines.sh` — the registry of 18 doctrines — and NOT
# the 7 check scripts that registry names: HEADING-DEPTH, TASK-STATUS,
# LOCKSTEP-CLAIM, INDEX-FRONTIER, FRONTIER-STATUS, RUST-FORMATTING, SELF-TEST.
# Copying only the 29 NEUTRAL paths into a fresh `git init` repository and
# running the enforcer printed seven `?? (missing/not executable: …)` rows and
# `10 doctrine breach(es) — commit blocked`. The spine as shipped produced a
# project that could not make its first commit.
#
# ⭐ ROOT CAUSE, a shape this repository has now met four times: NEUTRAL is a
# SECOND COPY of a fact the registry owns, and nothing derived it. INDEX-FRONTIER
# is the index's copy of the frontier; FRONTIER-STATUS is a row's copy of a
# leaf's status; this is the scaffold's copy of the registry. Each drifted the
# moment the first copy grew — and this one drifted in the direction that hurts,
# because NEUTRAL is the only mechanism that moves an earned doctrine anywhere.
#
# ⚠️ ONE DIRECTION ONLY, deliberately. NEUTRAL legitimately carries files the
# registry never names — the hooks, the memory architecture, the templates — so
# this asks only that REGISTERED ⊆ NEUTRAL. The converse would condemn them.
#
# ⚠️ The two CONDITIONAL registrations are handled by name rather than by
# accident: KNOWLEDGE-MAP is registered only when its subsystem exists (its
# scripts are in NEUTRAL) and PROJECT-SPECIFIC is the per-project slot, which a
# template must NOT carry — a project's own doctrines are its own.
#
# CONTRACT: exit code is the verdict; explains on stderr; deterministic;
# read-only; fast; path-agnostic.
#
# Self-test: scripts/check_scaffold_coverage.sh --self-test
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

ENFORCER="scripts/check_doctrines.sh"
SCAFFOLD="scripts/update_scaffold.sh"
# The per-project slot: deliberately NOT carried by a template.
EXEMPT="scripts/check_doctrines.project.sh"

# Registered check paths: the third pipe-field of each registry entry.
#
# ⛔ BOTH REGISTRATION FORMS. The array literal and the CONDITIONAL
# `DOCTRINES+=("…")` lines are equally registrations, and an earlier version
# anchored on `^[[:space:]]*"` so it saw only the first — which made the
# `$EXEMPT` filter below dead code and would have missed any doctrine moved to
# the conditional form. Found by mutating the exemption away and watching
# nothing happen, not by reading (`SIGNOFF-REPAIR.11.23`).
# ⚠️ And ONLY those two forms. Dropping the anchor entirely also matched the
# registry's own explanatory comment — `Each entry: "ID|what it proves|
# relative/path/to/check.sh"` — and reported that placeholder as an uncarried
# check. A key too loose returns the wrong instance
# (`docs/knowledge/a-key-too-loose-returns-the-wrong-instance.md`); a comment
# line is not a registration, so the two shapes are matched explicitly.
registered() {
  sed -n -E \
    -e 's/^[[:space:]]*"[A-Z][A-Z0-9-]*\|[^|]*\|([^"]*)".*/\1/p' \
    -e 's/^[[:space:]]*DOCTRINES\+=\("[A-Z][A-Z0-9-]*\|[^|]*\|([^"]*)"\).*/\1/p' \
    "$1"
}

# The NEUTRAL array's entries, comments and blanks dropped.
neutral() {
  sed -n '/^NEUTRAL=(/,/^)/p' "$1" | sed -e '1d' -e '$d' -e 's/#.*//' \
    | tr -d '[:blank:]' | grep -v '^$'
}

if [ "${1:-}" = "--self-test" ]; then
  fails=0
  scratch="$ROOT/target/doctrine_scratch"; mkdir -p "$scratch"
  tmp="$(mktemp -d "$scratch/scaffold-coverage.XXXXXX")"
  trap 'rm -rf "$tmp"' EXIT

  cat > "$tmp/enf.sh" <<'ENF'
DOCTRINES=(
  "ALPHA|proves alpha|scripts/check_alpha.sh"
  "BETA|proves beta|scripts/check_beta.sh"
)
ENF
  cat > "$tmp/scaf_ok.sh" <<'OK'
NEUTRAL=(
  # a comment line is not a path
  scripts/check_alpha.sh
  scripts/check_beta.sh
  COMMIT.md
)
OK
  cat > "$tmp/scaf_bad.sh" <<'BAD'
NEUTRAL=(
  scripts/check_alpha.sh
  COMMIT.md
)
BAD
  # The parsers must actually parse — a silently empty set would pass anything.
  [ "$(registered "$tmp/enf.sh" | grep -c .)" = "2" ] || { echo "SELF-TEST: registered() parsed $(registered "$tmp/enf.sh" | grep -c .), expected 2" >&2; fails=$((fails+1)); }
  [ "$(neutral "$tmp/scaf_ok.sh" | grep -c .)" = "3" ] || { echo "SELF-TEST: neutral() parsed $(neutral "$tmp/scaf_ok.sh" | grep -c .), expected 3 (the comment must not count)" >&2; fails=$((fails+1)); }
  # Covered: silent. Uncovered: named.
  if [ -n "$(comm -23 <(registered "$tmp/enf.sh" | sort -u) <(neutral "$tmp/scaf_ok.sh" | sort -u))" ]; then
    echo "SELF-TEST: a fully covered scaffold was refused" >&2; fails=$((fails+1))
  fi
  gap="$(comm -23 <(registered "$tmp/enf.sh" | sort -u) <(neutral "$tmp/scaf_bad.sh" | sort -u))"
  if [ "$gap" != "scripts/check_beta.sh" ]; then
    echo "SELF-TEST: the uncovered check was not named (got '$gap')" >&2; fails=$((fails+1))
  fi
  # And the one-directional rule: extra NEUTRAL entries are legal.
  if [ -n "$(comm -23 <(registered "$tmp/enf.sh" | sort -u) <(neutral "$tmp/scaf_ok.sh" | sort -u))" ]; then
    echo "SELF-TEST: NEUTRAL carrying COMMIT.md was treated as a breach" >&2; fails=$((fails+1))
  fi
  # ── The CONDITIONAL registration form, and the exemption that depends on it.
  # Both arms exist because mutating the exemption away changed nothing: the
  # parser could not see `DOCTRINES+=(…)` at all, so the filter was dead code.
  cat > "$tmp/enf_cond.sh" <<'COND'
DOCTRINES=(
  "ALPHA|proves alpha|scripts/check_alpha.sh"
)
[ -x "knowledge-map/scripts/check_knowledge_map.sh" ] && \
  DOCTRINES+=("KNOWLEDGE-MAP|derived map in sync|knowledge-map/scripts/check_knowledge_map.sh")
[ -x "scripts/check_doctrines.project.sh" ] && \
  DOCTRINES+=("PROJECT-SPECIFIC|this project's own doctrine checks|scripts/check_doctrines.project.sh")
COND
  if [ "$(registered "$tmp/enf_cond.sh" | grep -c .)" != "3" ]; then
    echo "SELF-TEST: the conditional DOCTRINES+= form was not parsed (got $(registered "$tmp/enf_cond.sh" | grep -c .), expected 3)" >&2
    fails=$((fails+1))
  fi
  # The per-project slot must be EXEMPT — a template must not carry it...
  if ! registered "$tmp/enf_cond.sh" | grep -vFx "$EXEMPT" | grep -qv "doctrines.project"; then
    echo "SELF-TEST: the exemption removed everything" >&2; fails=$((fails+1))
  fi
  if registered "$tmp/enf_cond.sh" | grep -vFx "$EXEMPT" | grep -q "doctrines.project"; then
    echo "SELF-TEST: the per-project slot was NOT exempt, so a template would be asked to carry it" >&2
    fails=$((fails+1))
  fi
  # ...and it must be the ONLY exemption: a conditional check that is not the
  # project slot still has to be carried.
  if ! registered "$tmp/enf_cond.sh" | grep -vFx "$EXEMPT" | grep -q "check_knowledge_map"; then
    echo "SELF-TEST: a conditional non-project check was wrongly exempted" >&2; fails=$((fails+1))
  fi

  # A project with no scaffold script is NOT CHECKED, never a breach: run this
  # very script from a scratch tree that has an enforcer and no scaffold.
  mkdir -p "$tmp/bare/scripts"
  cp "$ENFORCER" "$tmp/bare/scripts/check_doctrines.sh"
  cp "$0" "$tmp/bare/scripts/check_scaffold_coverage.sh"
  if ! bash "$tmp/bare/scripts/check_scaffold_coverage.sh" >/dev/null 2>&1; then
    echo "SELF-TEST: a project with no scaffold script was reported as a breach" >&2
    fails=$((fails+1))
  fi

  # The real files must parse to something, or this gate is vacuous here.
  [ "$(registered "$ENFORCER" | grep -c .)" -ge 10 ] || { echo "SELF-TEST: the real registry parsed too few entries" >&2; fails=$((fails+1)); }
  [ "$(neutral "$SCAFFOLD" | grep -c .)" -ge 10 ] || { echo "SELF-TEST: the real NEUTRAL list parsed too few entries" >&2; fails=$((fails+1)); }

  [ "$fails" -eq 0 ] || exit 1
  echo "SCAFFOLD-COVERAGE self-test: both parsers verified against known inputs, both registration forms parsed, the per-project slot exempt while a conditional non-project check is not, a covered scaffold passes, an uncovered check is named, extra NEUTRAL entries stay legal, a project with no scaffold script is NOT CHECKED rather than refused, and both real files parse non-empty"
  exit 0
fi

[ -f "$ENFORCER" ] || { echo "SCAFFOLD-COVERAGE: $ENFORCER is missing" >&2; exit 1; }

# ⚠️ A PROJECT NEED NOT CARRY THE SCAFFOLD SCRIPT, and this gate must not
# invent a breach out of that. `scripts/update_scaffold.sh` is deliberately
# absent from its own NEUTRAL list — a sync script that overwrites itself
# mid-run is its own hazard — so a project created by `bootstrap.sh` has one
# and a project that only ever pulled NEUTRAL may not. With no scaffold there
# is nothing to cover, which is NOT CHECKED rather than a pass or a failure.
# ⛔ Found because this very gate failed in the fresh-repo reproduction it was
# written to validate (`SIGNOFF-REPAIR.11.23`).
if [ ! -f "$SCAFFOLD" ]; then
  echo "SCAFFOLD-COVERAGE: no $SCAFFOLD in this project — NOT CHECKED (this is not a pass)" >&2
  exit 0
fi

reg="$(registered "$ENFORCER" | grep -vFx "$EXEMPT" | sort -u)"
neu="$(neutral "$SCAFFOLD" | sort -u)"
[ -n "$reg" ] || { echo "SCAFFOLD-COVERAGE: parsed no registry entries — the parser is broken, not the registry" >&2; exit 1; }
[ -n "$neu" ] || { echo "SCAFFOLD-COVERAGE: parsed no NEUTRAL entries — the parser is broken, not the list" >&2; exit 1; }

gap="$(comm -23 <(printf '%s\n' "$reg") <(printf '%s\n' "$neu"))"
[ -z "$gap" ] && exit 0

echo "SCAFFOLD-COVERAGE: the registry names checks the scaffold does not carry:" >&2
printf '%s\n' "$gap" | sed 's/^/    /' >&2
cat >&2 <<'GUIDE'

  A project that pulls this spine gets an enforcer registering these, without
  the files — `?? (missing/not executable)` and a BLOCKED first commit. It was
  reproduced that way in a fresh repository (SIGNOFF-REPAIR.11.23).

  Add each path to NEUTRAL in scripts/update_scaffold.sh. If a check is
  deliberately project-only, it belongs in scripts/check_doctrines.project.sh
  rather than in the universal registry.
GUIDE
exit 1
