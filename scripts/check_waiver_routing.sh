#!/usr/bin/env bash
# WAIVER-ROUTING — a task leaf that says a gate DOES NOT APPLY to it must name the leaf that owns
# fixing the gate. A waiver is a bug report about the gate; it must be routed, never inert.
#
# ⭐ THE DURABLE LAW THIS MECHANIZES: an author writing a waiver IS the gate reporting a missing
# capability. It is the single highest-signal defect report a gate can receive — it comes from
# someone who did the work, hit the boundary, and wrote down exactly where it was.
#
# Provenance (kept because the evidence is the argument, with the project's nouns removed): on a
# real project running this spine, an author wrote, by hand, INSIDE a ticked acceptance box, that
# the gate's diagnosis-signature families did not fit their defect class. They were RIGHT and
# precise — the gate modelled four families and none covered a build-flow defect — and NOTHING
# HAPPENED. The note sat unread for months until a later task re-derived the identical gap from
# scratch. A second leaf even wrote out its evidence under a "Diagnosis tool signatures:" heading
# and got no credit for it. The capability gap was reported, in writing, by the person best placed
# to see it, and the process had nowhere to put it.
#
# THE RULE
#   If a staged `docs/tasks/*.md` ADDS a line asserting a check/signature/gate does not apply,
#   cannot be satisfied, or is being waived, that file must ALSO cite an owning leaf id
#   (`TREE-NAME.4`, `TREE.4.2`, …) or a work-unit/slice id (`PREFIX-FAMILY-0001`). Otherwise:
#   block, and quote the line.
#
# ⛔ DESIGN CONSTRAINT, deliberate and load-bearing:
#   THIS MUST NOT PUNISH HONESTY. Forbidding waiver language outright would simply delete the
#   signal — authors would stop writing the note and the gap would become invisible again, which
#   is strictly worse than an unread note. So a waiver stays entirely LEGAL; it just has to name
#   an owner. One token, and the inert note becomes tracked work.
#
# ARCHETYPE: evidence (DOCTRINE_ENFORCEMENT.md §3).
#   HONEST LIMIT — this verifies an owner was NAMED, not that the owner is real or that the work
#   happens. Stated rather than hidden.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on stderr;
# deterministic; read-only; staged-scope-aware; path-agnostic; fast.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

# Staged task-tree files only. No staged set (e.g. a manual run) => nothing to judge.
staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null \
          | grep -E '^docs/tasks/.*\.md$' || true)"
[ -n "$staged" ] || exit 0

# Waiver / inapplicability language. Kept tight and phrase-anchored so it fires on a real claim
# ("the signatures do not apply") and not on incidental prose containing the words separately.
# ⚠️ SCOPE-vs-CAPABILITY, and the sweep is what forced the distinction. The first draft triggered
# on any "the gate does not apply", which fired on a dozen HONEST SCOPE statements — "this slice is
# pure-docs, so the code-change gate does not apply". Those are correct and are NOT bug reports:
# the gate is behaving exactly as designed. The signal worth routing is narrower and sharper — a
# claim that the gate DOES apply but its SIGNATURE SURFACE cannot express the author's evidence.
# That narrower claim is the only thing this doctrine binds.
WAIVER_RE='(^|[^-[:alnum:]])[Ww]aiver note|[A-Z][A-Z0-9_]*_WAIVER|(signature|signatures|diagnosis.toolbox|diagnosis tool|diagnosis-tool)s? (do|does) not apply|no (signature|diagnosis) (family|group) (fits|matches|models|exists)|cannot be (satisfied|expressed) by (the|any) (gate|check|signature)|exempt from (the|this) (gate|check)'

# An owning leaf id (TREE.4 / TREE.4.2 / TREE.10.4b) or a work-unit id (PREFIX-FAMILY-0001).
OWNER_RE='`?[A-Z][A-Z0-9-]+\.[0-9]+[0-9a-z.]*`?|[A-Z][A-Z0-9]+-[A-Z0-9-]+-[0-9]{4}'

fail=0
for file in $staged; do
  [ -r "$file" ] || continue

  # ADDED lines only — this binds NEW claims, never the historical record.
  added="$(git diff --cached -U0 -- "$file" 2>/dev/null | grep '^+' | grep -v '^+++' || true)"
  [ -n "$added" ] || continue

  # Bind only when this commit ADDS a waiver claim; the historical record is never retro-bound.
  # ⛔ Written to a FILE, not piped. `printf "$var" | grep -q` returns failure ON SUCCESS once the
  # producer exceeds the pipe buffer (~64 KiB) and the match is early: grep exits at the first
  # match, printf takes SIGPIPE (141), and `pipefail` promotes 141 to the pipeline status. This
  # site would then `continue` — i.e. SKIP the file and let an unrouted waiver through. It fails
  # OPEN, which is the worst direction. Measured on the originating project: PIPESTATUS=(141 0).
  added_file="$(mktemp)"; printf '%s\n' "$added" > "$added_file"
  if ! grep -qE "$WAIVER_RE" "$added_file"; then rm -f "$added_file"; continue; fi
  rm -f "$added_file"

  # A waiver is discharged when an owner is named in its immediate NEIGHBOURHOOD in the file
  # (the trigger line +/- WINDOW lines).
  #
  # ⚠️ WHY A WINDOW AND NOT THE SAME LINE — caught by USING this doctrine on its own first
  # customers. Markdown prose WRAPS, so "the diagnosis-toolbox signatures do not apply" and the
  # "(owned by TREE.4)" that discharges it routinely land on different physical lines. A strict
  # same-line rule is unsatisfiable for any wrapped paragraph and would push authors toward
  # deleting the waiver instead of owning it — the exact outcome this doctrine exists to prevent.
  # The window is kept SMALL so a leaf id elsewhere in the file cannot vacuously discharge a
  # waiver: the citation has to be in the same paragraph a human would read as one thought.
  WINDOW=6
  undischarged=""
  while IFS= read -r ln; do
    [ -n "$ln" ] || continue
    lo=$(( ln > WINDOW ? ln - WINDOW : 1 )); hi=$(( ln + WINDOW ))
    win="$(sed -n "${lo},${hi}p" "$file" 2>/dev/null)"
    win_file="$(mktemp)"; printf '%s\n' "$win" > "$win_file"
    if ! grep -qE "$OWNER_RE" "$win_file"; then
      rm -f "$win_file"
      undischarged="${undischarged}${file}:${ln}: $(sed -n "${ln}p" "$file")"$'\n'
    else
      rm -f "$win_file"
    fi
  done < <(grep -nE "$WAIVER_RE" "$file" 2>/dev/null | cut -d: -f1)
  [ -n "$undischarged" ] || continue

  fail=1
  {
    echo "WAIVER-ROUTING: $file states a gate does not apply, without naming the leaf that owns fixing it."
    printf '%s' "$undischarged" | sed 's/^/  offending: /'
    cat <<'MSG'
  A waiver is a BUG REPORT ABOUT THE GATE — the highest-signal one it can get, because it comes
  from someone who did the work and hit the boundary. It must be routed, not left inert.
  (A real waiver note of exactly this shape was correct and sat unread for months, until a later
  task re-derived the identical gap from scratch.)

  Discharge it by naming an owner ON THE SAME LINE — either is fine:
    - an existing leaf:  "... signatures do not apply (gate gap owned by TREE-NAME.5)"
    - a new leaf you open for it, or your work-unit id, e.g. PREFIX-<FAMILY>-<NNNN>.
  ⛔ Do NOT delete the waiver to pass this check. Saying it is correct; owning it is the point.
MSG
  } >&2
done

if [ "$fail" -ne 0 ]; then
  exit 1
fi
echo "waiver-routing: OK (no unrouted gate-waiver claim added in the staged task leaves)"
exit 0
