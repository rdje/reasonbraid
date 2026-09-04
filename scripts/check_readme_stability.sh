#!/usr/bin/env bash
# scripts/check_readme_stability.sh — README-STABILITY (README_POLICY.md).
#
# Keeps README.md a stable LANDING PAGE instead of letting it grow into a changelog,
# roadmap, catalogue or documentation inventory. Structural, deterministic, NON-MUTATING,
# no network. Exits nonzero on any breach, with a routing hint naming the canonical home.
#
# ⭐ WHY BOTH A LINE CAP AND A BYTE CAP — this is the one design decision worth defending,
# because "add a line cap" is the obvious shape and it is NOT sufficient. Measured on a real
# project running this spine: its layer-A MEMORY.md sat at 60 lines — PASSING, exactly at its
# line cap — while carrying 138,403 BYTES. That is 2,306 bytes per line, with a single line of
# 18,816 bytes. A file the standard calls a "bounded resume pointer" was a 138 KB document and
# its guard was green the whole time. The same class appeared independently in that project's
# README, where ONE bullet measured 4,369 bytes.
#   ⇒ Line and byte checks are COMPLEMENTS, not redundancy: neither wrapped prose nor very
#     long lines can bypass the budget. That precedent is why this guard ships with both caps
#     from day one, and why scripts/check_memory_architecture.sh now carries both too.
#
# ⛔ NEVER raise a cap to land new content. Move the detail to its canonical home (below).
#   A cap increase requires an explicit reviewed decision recorded in your task-tree that the
#   landing-page contract itself expanded.
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

# ⚠️ TEMPLATE DEFAULTS, deliberately generous — they ship to a project whose README is not
# this one, so they are set to the policy's own published example rather than fitted to this
# repository's README. README_POLICY.md tells the adopting project to TIGHTEN them after its
# own review-and-trim, which is the point at which a cap becomes meaningful. Both are
# env-overridable so a project can ratchet without editing the spine.
LINE_CAP="${README_LINE_CAP:-300}"
BYTE_CAP="${README_BYTE_CAP:-16384}"
TARGET="README.md"
POLICY="README_POLICY.md"

fail=0
note(){ printf 'README-STABILITY: %s\n' "$1" >&2; fail=1; }

# ------------------------------------------------------------------ refuse rather than skip
# A skip is never a pass: with the landing page or its policy absent this check cannot judge
# anything, and returning 0 would report "the doctrine holds" over an absence.
if [ ! -f "$TARGET" ]; then
  printf 'README-STABILITY: REFUSED — %s is missing; it is the thing this doctrine governs.\n' \
    "$TARGET" >&2
  exit 2
fi
if [ ! -f "$POLICY" ]; then
  printf 'README-STABILITY: REFUSED — %s is missing; the caps above would be unreviewable numbers.\n' \
    "$POLICY" >&2
  exit 2
fi

lines=$(wc -l < "$TARGET" | tr -d ' ')
bytes=$(wc -c < "$TARGET" | tr -d ' ')

routing_hint() {
  cat >&2 <<'HINT'
                 Route the new detail to its canonical home instead of growing the landing page:
                   user-facing feature detail ....... the user guide / docs/book/
                   current work and priorities ...... docs/tasks/, docs/TASK_TREE.md, ROADMAP.md
                   release history .................. CHANGELOG.md, git history
                   design rationale ................. docs/decisions/
                   exhaustive inventories ........... a generated index or a dedicated reference
                   diagnostics and procedure ........ TOOLBOX.md, contributor docs
                 Full policy: README_POLICY.md
HINT
}

# ------------------------------------------------------------------ the line cap
if [ "$lines" -gt "$LINE_CAP" ]; then
  note "$TARGET is $lines lines (> cap $LINE_CAP)."
  routing_hint
fi

# ------------------------------------------------------------------ the byte cap
# Complements the line cap: wrapped prose cannot bypass the byte budget, and a single very
# long line cannot bypass the line budget. Neither cap is redundant with the other.
if [ "$bytes" -gt "$BYTE_CAP" ]; then
  note "$TARGET is $bytes bytes (> cap $BYTE_CAP)."
  routing_hint
fi

# ------------------------------------------------------------------ changelog leakage
# ⚠️ HONEST BOUND, stated rather than implied: this is NOT a general "is this changelog
# content?" oracle. It detects exactly ONE leakage class — the dated historical annotation
# ("... changed on 2026-01-31"), which is a release-history row living on a landing page.
# A date on a landing page is history; the escape is to move it, not to weaken this check.
dated=$(grep -cE '20[0-9]{2}-[0-9]{2}-[0-9]{2}' "$TARGET" || true)
if [ "${dated:-0}" -gt 0 ]; then
  note "$TARGET carries $dated date-stamped line(s) — release history belongs in CHANGELOG.md."
  grep -nE '20[0-9]{2}-[0-9]{2}-[0-9]{2}' "$TARGET" | head -5 | sed 's/^/                   /' >&2
fi

# ------------------------------------------------------------------ the policy stays reachable
# The caps are only defensible if a reader can find the reviewed decision behind them. If the
# README stops naming the policy, the numbers above become folklore.
if ! grep -q "$POLICY" "$TARGET"; then
  note "$TARGET no longer links $POLICY — the caps must stay traceable to the decision that set them."
fi

[ "$fail" -eq 0 ] || exit 1
printf 'README-STABILITY: OK — %s is %s/%s lines, %s/%s bytes.\n' \
  "$TARGET" "$lines" "$LINE_CAP" "$bytes" "$BYTE_CAP"
exit 0
