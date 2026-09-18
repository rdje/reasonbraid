#!/usr/bin/env bash
# scripts/check_visibility_policy.sh — VISIBILITY-POLICY doctrine.
#
# The director's 2026-09-09 correction is durable: this repository is public and
# must remain public (docs/decisions/2026-09-09_public-repository-policy.md).
# The superseded private-repository instruction had already leaked past two
# hand-run censuses — into the CI guide (caught by .11.4.3.1.2.4) and then into
# the book's introduction and the governance charter. A census a human runs once
# cannot hold a policy; this check holds it on every commit.
#
# Every tracked Markdown sentence that states or instructs a PRIVATE visibility
# for the repository must be listed verbatim in .doctrine/visibility_exceptions.txt
# with its reason. Legitimate sentences exist — a prohibition, a negation, a
# quoted historical line inside preserved evidence — so the rule is not "never
# write the word": it is "every such sentence is reviewed, and a NEW or REWORDED
# one is reviewed again". A listed entry that no longer matches is also a breach,
# so the allowlist cannot rot into a blanket exclusion.
#
# Self-test: scripts/check_visibility_policy.sh --self-test
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

ALLOWLIST=".doctrine/visibility_exceptions.txt"

# Instruction- and premise-shaped statements about THIS repository's visibility.
# Deliberately NOT a bare "private" match: the corpus legitimately discusses
# private files, private channels, private overlays and private repositories in
# general, and an allowlist of those would teach bypass rather than review.
# ⚠️ THE NOUN IS ABBREVIATED TOO (`SIGNOFF-REPAIR.11.2.4`). Every clause was
# anchored on the full word `repositor(y|ies)`, so two live instances written
# `the repo is private` sat in the corpus invisibly — in a gate built for
# exactly that claim. `repo(s|sitory|sitories)?` covers all four spellings.
#
# ⛔ The widening is DERIVED, not invented. Censusing every tracked-Markdown line
# carrying a repo word beside `privat` showed the abbreviation is the only
# uncovered spelling that STATES a visibility; the many `private-repository
# instruction` lines put `private` BEFORE the noun and are corrections of the
# superseded policy, so no clause matches them and none should. Measured delta:
# 3 matches to 11, none lost.
PATTERN='(keep(ing|s)?|mak(e|es|ing)) +(a|the|this|its) +repo(s|sitory|sitories)? +(\*\*)?privat'
PATTERN="$PATTERN"'|repo(s|sitory|sitories)? +(is|are|was|were|stays?|remains?) +(\*\*)?privat'
PATTERN="$PATTERN"'|repo(s|sitory|sitories)? +(must|should|shall|will|may|can|could) +(not +)?(remain|be|stay|become)s? +(\*\*)?privat'
PATTERN="$PATTERN"'|privat[a-z]* +until +(ADR|that ADR|name|clearance)'

trim() {
  local s="$1"
  s="${s#"${s%%[![:space:]]*}"}"
  s="${s%"${s##*[![:space:]]}"}"
  printf '%s' "$s"
}

if [ "${1:-}" = "--self-test" ]; then
  fails=0
  # The matcher must FIRE on each shape the leak actually took.
  while IFS= read -r probe; do
    [ -n "$probe" ] || continue
    if ! printf '%s\n' "$probe" | grep -qiE "$PATTERN"; then
      echo "SELF-TEST: the matcher missed: $probe" >&2; fails=$((fails+1))
    fi
  done <<'PROBES'
Keep the repository private until ADR-001 records the naming decision.
One person fills both roles while the repository is private and pre-clearance.
The repository must remain private.
Keep this repository **private** until that ADR's clearance gate passes.
model has no external owners yet (the repo is private) —
channel: the repo is PRIVATE, the ADR-001 gate);
Keep the repo private until ADR-001 records the naming decision.
The repos must remain private.
PROBES
  # And it must NOT fire on the unrelated senses the corpus is full of.
  while IFS= read -r probe; do
    [ -n "$probe" ] || continue
    if printf '%s\n' "$probe" | grep -qiE "$PATTERN"; then
      echo "SELF-TEST: the matcher over-matched: $probe" >&2; fails=$((fails+1))
    fi
  done <<'PROBES'
The store writes a complete private state.json.next and synchronizes it.
Private repositories and provider credentials remain on their owning nodes.
Confidential reports require a separate private channel or workspace.
Keep the repository public, as explicitly directed.
the earlier private-repository instruction was wrong; README and ADR-001
the embargo is the honest private-repo shape (the fix ships with its leaf)
A broad private-repository match yields 30 tracked hits, mostly private files.
PROBES
  [ "$fails" -eq 0 ] || exit 1
  echo "VISIBILITY-POLICY self-test: matcher fires on 8 leak shapes including the four abbreviated spellings, silent on 7 unrelated senses including the corrections that put 'private' BEFORE the noun"
  exit 0
fi

if [ ! -f "$ALLOWLIST" ]; then
  echo "VISIBILITY-POLICY: missing $ALLOWLIST" >&2
  exit 1
fi

declare -a allowed=()
while IFS= read -r entry || [ -n "$entry" ]; do
  case "$entry" in ''|'#'*) continue;; esac
  allowed+=("$entry")
done < "$ALLOWLIST"

declare -a matched=()
errs=0
while IFS= read -r hit; do
  [ -n "$hit" ] || continue
  path="${hit%%:*}"; rest="${hit#*:}"
  line="${rest%%:*}"; text="$(trim "${rest#*:}")"
  key="$path	$text"
  found=0
  for entry in "${allowed[@]}"; do
    if [ "$entry" = "$key" ]; then found=1; matched+=("$entry"); break; fi
  done
  if [ "$found" -eq 0 ]; then
    if [ "$errs" -eq 0 ]; then
      echo "VISIBILITY-POLICY: this repository is public and must remain public" >&2
      echo "  (docs/decisions/2026-09-09_public-repository-policy.md)." >&2
      echo "  Unreviewed private-visibility statement(s):" >&2
    fi
    echo "    $path:$line: $text" >&2
    errs=$((errs+1))
  fi
done < <(git grep -n -I -iE "$PATTERN" -- '*.md' 2>/dev/null)

if [ "$errs" -gt 0 ]; then
  echo "  Correct the statement, or — if it is a prohibition, a negation or a" >&2
  echo "  preserved historical quotation — add its exact line to $ALLOWLIST." >&2
  exit 1
fi

# A stale exception is a breach too: the allowlist must describe what is there.
for entry in "${allowed[@]}"; do
  still=0
  for seen in ${matched[@]+"${matched[@]}"}; do
    if [ "$seen" = "$entry" ]; then still=1; break; fi
  done
  if [ "$still" -eq 0 ]; then
    echo "VISIBILITY-POLICY: stale exception in $ALLOWLIST (no longer present):" >&2
    echo "    ${entry%%	*}: ${entry#*	}" >&2
    errs=$((errs+1))
  fi
done

[ "$errs" -eq 0 ] || exit 1
exit 0
