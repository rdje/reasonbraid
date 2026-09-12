#!/usr/bin/env bash
# scripts/check_lockstep_claim.sh — LOCKSTEP-CLAIM doctrine.
#
# A ticked LOCKSTEP box names the documents a leaf carried into its commit. It
# is a CLAIM about the commit, and nothing checked it: commit 6bf0c40 carried a
# ticked box naming MEMORY.md and LIVE_STATUS.md and staged neither. The cause
# chain was ordinary — a scripted multi-edit hit a failed assertion, the script
# exited non-zero, the compound command ran git add -A and git commit anyway,
# and -A staged whatever partial state existed.
#
# TASK-ACCEPTANCE cannot catch this. That gate proves the box is TICKED and
# CITES something re-runnable; it cannot prove the cited edit landed. This is
# the complementary half.
#
# The rule is keyed on the AUTHOR'S OWN CLAIM, not on a blanket requirement,
# and the census is why. Over the 25 commits before it was written, 10 closed a
# leaf, and of those CHANGELOG.md was staged 10/10 but MEMORY.md only 6/10,
# LIVE_STATUS.md 7/10 and DEV_NOTES.md 7/10. A blanket "closing a leaf must
# stage MEMORY" would have asserted a rule the project does not follow, flagged
# four pre-existing commits, and still missed 6bf0c40, which did stage
# CHANGELOG.md. Of the 6 commits in that window adding a bold LOCKSTEP bullet
# naming a core live document, exactly one — 6bf0c40 — named one it did not
# stage. Zero false positives across the rest.
#
# The honest escape, following the GAP-CLAIM-CENSUS precedent rather than
# parsing intent out of prose: a box that names a document in order to say it is
# UNCHANGED discloses that in the same heading section, as
#     lockstep: <doc> unchanged (<why>)
#
# Modes:
#   scripts/check_lockstep_claim.sh              the staged scope (the gate)
#   scripts/check_lockstep_claim.sh --self-test  the predicate, both directions
#   scripts/check_lockstep_claim.sh --against SHA  re-run over a past commit
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

HEADLINE="LOCKSTEP-CLAIM"

# The core live documents COMMIT.md lists under "Tracked files to keep in
# lockstep". Only root-level single documents belong here: docs/decisions/ and
# docs/book/ are directories a box names by area, not by file, and
# KNOWLEDGE_MAP.md is derived and has its own gate.
LIVE_DOCS=(README.md MEMORY.md LIVE_STATUS.md CHANGELOG.md DEV_NOTES.md)

# The claim shape: a BOLD LOCKSTEP bullet. Narrative prose that merely mentions
# the word ("its LOCKSTEP box overstated") is a report about a past commit, not
# a claim about this one, and is deliberately not matched.
CLAIM_RE='\*\*LOCKSTEP\*\*'

# Every document named on a claim line, as a whole filename: the leading guard
# keeps a path such as docs/archive/MEMORY.md from being read as the root one.
named_docs() {
  local line="$1" doc
  for doc in "${LIVE_DOCS[@]}"; do
    if printf '%s' "$line" | grep -qE "(^|[^A-Za-z0-9_/.])${doc//./\\.}([^A-Za-z0-9_]|$)"; then
      printf '%s\n' "$doc"
    fi
  done
}

# The added claim lines of one file, with the line numbers they land on, so the
# escape can be looked for in the claim's own heading section.
# Emits: <newlineno><TAB><text>
added_claim_lines() {
  local diff_cmd=("$@")
  "${diff_cmd[@]}" | awk '
    /^@@/ {
      # @@ -a,b +c,d @@ — c is the first new line number of this hunk.
      match($0, /\+[0-9]+/); n = substr($0, RSTART + 1, RLENGTH - 1) + 0; next
    }
    /^\+\+\+/ { next }
    /^\+/ { print n "\t" substr($0, 2); n++; next }
    /^-/  { next }
    { n++ }
  '
}

# Which heading section a line falls in, and whether that section discloses the
# document as deliberately unchanged.
escape_sections() {
  local file="$1" doc="$2"
  awk -v doc="$doc" '
    { line[FNR] = $0; low[FNR] = tolower($0) }
    END {
      for (i = 1; i <= FNR; i++) if (line[i] ~ /^#{1,6} /) cur = i; else sec[i] = cur
      for (i = 1; i <= FNR; i++) if (line[i] ~ /^#{1,6} /) sec[i] = i
      pattern = "lockstep: *" tolower(doc) " +unchanged"
      for (i = 1; i <= FNR; i++) if (low[i] ~ pattern) print sec[i]
    }
  ' "$file"
}

section_of() {
  awk -v target="$2" '
    { if ($0 ~ /^#{1,6} /) cur = FNR }
    FNR == target { print cur + 0; exit }
  ' "$1"
}

# ── the self-test: the predicate, both directions ───────────────────────────
if [ "${1:-}" = "--self-test" ]; then
  fails=0
  claims=(
    '- [x] **LOCKSTEP** — task tree, frontier, `MEMORY.md` and `CHANGELOG.md` carry the same scope.'
    '- [x] **LOCKSTEP** — the book and `LIVE_STATUS.md`.'
  )
  for probe in "${claims[@]}"; do
    printf '%s\n' "$probe" | grep -qE "$CLAIM_RE" || {
      echo "SELF-TEST: the claim matcher missed: $probe" >&2; fails=$((fails+1)); }
  done
  for probe in \
    '- Correction, same leaf: its LOCKSTEP box overstated. Two scripted edits failed.' \
    '- The lockstep rule is enforced by a gate.'; do
    if printf '%s\n' "$probe" | grep -qE "$CLAIM_RE"; then
      echo "SELF-TEST: the claim matcher over-matched narrative: $probe" >&2; fails=$((fails+1))
    fi
  done
  got="$(named_docs '- [x] **LOCKSTEP** — `MEMORY.md`, `CHANGELOG.md` and the book.' | tr '\n' ' ')"
  [ "$got" = "MEMORY.md CHANGELOG.md " ] || {
    echo "SELF-TEST: named_docs returned '$got'" >&2; fails=$((fails+1)); }
  got="$(named_docs '- [x] **LOCKSTEP** — docs/archive/MEMORY.md only.' | tr '\n' ' ')"
  [ -z "$got" ] || {
    echo "SELF-TEST: a nested path was read as the root document: '$got'" >&2; fails=$((fails+1)); }
  got="$(named_docs '- [x] **LOCKSTEP** — `MEMORY_ARCHITECTURE.md` is a different file.' | tr '\n' ' ')"
  [ -z "$got" ] || {
    echo "SELF-TEST: a longer filename matched a shorter one: '$got'" >&2; fails=$((fails+1)); }

  # The list must still be what COMMIT.md says it is.
  for doc in "${LIVE_DOCS[@]}"; do
    grep -q -- "$doc" COMMIT.md || {
      echo "SELF-TEST: $doc is not named in COMMIT.md — the list has drifted" >&2; fails=$((fails+1)); }
  done

  # The escape, both ways, on a real file.
  work="target/doctrine_scratch/lockstep_claim"; mkdir -p "$work"
  probe="$work/selftest.md"
  cat > "$probe" <<'PROBE'
## A leaf

- [x] **LOCKSTEP** — `MEMORY.md` carries the same scope.
- lockstep: MEMORY.md unchanged (this leaf changes no resume pointer)

## Another leaf

- [x] **LOCKSTEP** — `CHANGELOG.md`.
PROBE
  [ "$(escape_sections "$probe" MEMORY.md)" = "1" ] || {
    echo "SELF-TEST: the escape was not found in its own section" >&2; fails=$((fails+1)); }
  [ -z "$(escape_sections "$probe" CHANGELOG.md)" ] || {
    echo "SELF-TEST: an escape was found for a document that has none" >&2; fails=$((fails+1)); }
  [ "$(section_of "$probe" 4)" = "1" ] || {
    echo "SELF-TEST: section_of mis-attributed line 4" >&2; fails=$((fails+1)); }
  [ "$(section_of "$probe" 8)" = "6" ] || {
    echo "SELF-TEST: section_of mis-attributed line 8 ($(section_of "$probe" 8))" >&2; fails=$((fails+1)); }
  rm -f "$probe"

  [ "$fails" -eq 0 ] || exit 1
  echo "$HEADLINE self-test: 2 claim shapes matched, 2 narrative lines ignored, 3 naming cases, the escape both ways, ${#LIVE_DOCS[@]} documents confirmed in COMMIT.md"
  exit 0
fi

# ── scope ───────────────────────────────────────────────────────────────────
if [ "${1:-}" = "--against" ]; then
  sha="${2:?--against needs a commit}"
  changed="$(git show --name-only --format= "$sha")"
  diff_for() { git diff "$sha^" "$sha" -- "$1"; }
  content_of() { git show "$sha:$1"; }
  scope="$(printf '%s\n' "$changed" | grep -E '^docs/tasks/.*\.md$' || true)"
else
  changed="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null || true)"
  diff_for() { git diff --cached -U0 -- "$1"; }
  content_of() { cat "$1"; }
  scope="$(printf '%s\n' "$changed" | grep -E '^docs/tasks/.*\.md$' || true)"
fi
[ -n "$scope" ] || exit 0

work="target/doctrine_scratch/lockstep_claim"; mkdir -p "$work"
errs=0
while IFS= read -r file; do
  [ -n "$file" ] || continue
  snapshot="$work/$(printf '%s' "$file" | tr '/' '_')"
  content_of "$file" > "$snapshot" 2>/dev/null || continue
  while IFS=$'\t' read -r lineno text; do
    [ -n "${text:-}" ] || continue
    printf '%s' "$text" | grep -qE "$CLAIM_RE" || continue
    section="$(section_of "$snapshot" "$lineno")"
    while IFS= read -r doc; do
      [ -n "$doc" ] || continue
      printf '%s\n' "$changed" | grep -qxF "$doc" && continue
      # The disclosure that this document is deliberately untouched.
      if printf '%s\n' "$(escape_sections "$snapshot" "$doc")" | grep -qxF "${section:-0}"; then
        continue
      fi
      if [ "$errs" -eq 0 ]; then
        echo "$HEADLINE: a LOCKSTEP box claims a document this commit does not touch." >&2
      fi
      echo "    $file:$lineno names $doc, which is not in the changed set" >&2
      errs=$((errs+1))
    done < <(named_docs "$text")
  done < <(added_claim_lines diff_for "$file")
done < <(printf '%s\n' "$scope")

if [ "$errs" -ne 0 ]; then
  cat >&2 <<'EXPLAIN'

  A ticked LOCKSTEP box is a claim about THIS commit, and a box naming a
  document the commit does not carry is a false checklist entry — the exact
  shape 6bf0c40 shipped, where a partially-failed scripted edit was staged with
  `git add -A` and the box said the edits had landed.

  Discharge it either way:
    1. make the edit and stage the document, or
    2. disclose the intent in the claim's own heading section:
         lockstep: <doc> unchanged (<why>)

  ⛔ Do not remove the document's name to silence this. The name is the claim;
     removing it hides what the leaf decided instead of recording it.
EXPLAIN
  exit 1
fi
exit 0
