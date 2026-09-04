#!/usr/bin/env bash
# LESSON-PROMOTION — a durable lesson written to the development notes must be either PROMOTED
# into the retrievable layer or EXPLICITLY DECLINED, never silently dropped.
#
# Provenance: maintainer directive 2026-08-01 in the originating project, ported by
# BEDROCK-MAINTENANCE.2.6. THE MEASURED DEFECT this exists to stop: the notes file there carried
# 1 592 dated lesson entries across 62 191 lines and is NOT a Knowledge Map scan dir, so not one of
# them was reachable by question; the decisions dir IS a scan dir and 0 of its 142 records carried
# `answers:`. The promotion mechanism existed, was wired, and was skipped 1 592 times — silently,
# because no gate asked. KNOWLEDGE-MAP checks the map is in sync with its SOURCES; it never asks
# whether a lesson REACHED a source. A reminder in COMMIT.md had already lost 1 592 times; hence a
# doctrine check.
#
# THE RULE
#   If a commit stages a NEW dated lesson heading in DEV_NOTES.md (an `## …` line carrying a
#   YYYY-MM-DD date), it must ALSO stage EITHER
#     (a) a change under docs/knowledge/ (if the project has one) or a docs/decisions/ record
#         gaining `answers:`                                                    [PROMOTED]
#     (b) the token `promotion: declined (<reason>)` in a staged docs/tasks/*.md leaf  [DECLINED]
#
# ARCHETYPE: evidence. HONEST LIMIT — stated rather than hidden: this verifies a DECISION WAS
# RECORDED, not that the decision was correct. A lazy `promotion: declined (n/a)` passes. What it
# makes impossible is the SILENT omission, which is the measured failure.
#
# CONTRACT: exit code is the verdict; explains on stderr; deterministic; read-only;
# staged-scope-aware; path-agnostic; fast. Ground truth (pure verdict + controls) runs on every
# invocation and REFUSES (exit 2) if a control misses.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

NOTES="DEV_NOTES.md"
DECLINE_TOKEN="promotion: declined"

# --- the decision, as a pure function so it can carry ground truth ---
# $1 = count of newly added dated lesson headings; $2 = 1 if a promotion is staged; $3 = 1 if an
# explicit decline is staged. Echoes: ok | needs-decision
lesson_promotion_verdict() {
    local added="$1" promoted="$2" declined="$3"
    if [ "$added" -eq 0 ]; then printf 'ok\n'; return 0; fi
    if [ "$promoted" -eq 1 ] || [ "$declined" -eq 1 ]; then printf 'ok\n'; return 0; fi
    printf 'needs-decision\n'
}

lesson_promotion_self_check() {
    local spec want got misses=0
    for spec in "0:0:0:ok" "0:1:0:ok" "3:1:0:ok" "3:0:1:ok" "3:1:1:ok" \
                "1:0:0:needs-decision" "9:0:0:needs-decision"; do
        want="${spec##*:}"
        got="$(lesson_promotion_verdict "$(echo "$spec" | cut -d: -f1)" \
                                        "$(echo "$spec" | cut -d: -f2)" \
                                        "$(echo "$spec" | cut -d: -f3)")"
        if [ "$got" != "$want" ]; then
            printf 'lesson-promotion: CONTROL MISSED: %s expected=%s got=%s\n' "$spec" "$want" "$got" >&2
            misses=$((misses + 1))
        fi
    done
    # the heading predicate: a dated `## ` line is a lesson, an undated one is not
    local n
    n="$(printf '%s\n' '## _(2026-09-04)_ — a lesson' '## 2026-09-04 - a lesson' | grep -cE '^## .*[0-9]{4}-[0-9]{2}-[0-9]{2}' || true)"
    [ "$n" = 2 ] || { printf 'lesson-promotion: CONTROL MISSED: dated headings not matched (%s)\n' "$n" >&2; misses=$((misses + 1)); }
    n="$(printf '%s\n' '## _(YYYY-MM-DD)_ — bootstrap' '## Section' | grep -cE '^## .*[0-9]{4}-[0-9]{2}-[0-9]{2}' || true)"
    [ "$n" = 0 ] || { printf 'lesson-promotion: CONTROL MISSED: an undated heading matched (%s)\n' "$n" >&2; misses=$((misses + 1)); }
    if [ "$misses" -gt 0 ]; then
        printf 'lesson-promotion: the gate does not discriminate (%d control(s) missed); refusing\n' "$misses" >&2
        exit 2
    fi
}
lesson_promotion_self_check
[ "${1:-}" = "--self-test" ] && { echo "LESSON-PROMOTION --self-test: 9/9 controls"; exit 0; }

# No staged set (a manual run outside a commit) => nothing to judge.
staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null || true)"
[ -n "$staged" ] || { echo "LESSON-PROMOTION: ok (nothing staged)"; exit 0; }
printf '%s\n' "$staged" | grep -qx "$NOTES" || { echo "LESSON-PROMOTION: ok ($NOTES not staged)"; exit 0; }

# Newly ADDED dated lesson headings only — reflowing an existing entry is not a new lesson.
added_entries="$(git diff --cached -U0 -- "$NOTES" 2>/dev/null \
                 | grep -E '^\+## .*[0-9]{4}-[0-9]{2}-[0-9]{2}' | sed 's/^\+//' || true)"
added_count="$(printf '%s' "$added_entries" | grep -c . || true)"
[ "$added_count" -eq 0 ] && { echo "LESSON-PROMOTION: ok (no new dated lesson in $NOTES)"; exit 0; }

# (a) PROMOTED: a docs/knowledge/ change, or a docs/decisions/ record gaining `answers:`.
promoted=0
printf '%s\n' "$staged" | grep -qE '^docs/knowledge/.*\.md$' && promoted=1
if [ "$promoted" -eq 0 ]; then
    git diff --cached -U0 -- docs/decisions 2>/dev/null | grep -qE '^\+answers:' && promoted=1
fi

# (b) DECLINED: an explicit token in a staged task leaf, carrying a REAL reason.
# ⛔ A MENTION IS NOT A DECISION: the placeholder `(<reason>)` is rejected, the reason must be non-empty.
declined=0
for f in $(printf '%s\n' "$staged" | grep -E '^docs/tasks/.*\.md$' || true); do
    [ -r "$f" ] || continue
    case "$f" in docs/tasks/TEMPLATE.md) continue ;; esac
    if grep -E "${DECLINE_TOKEN} \(..*\)" "$f" | grep -vqF "${DECLINE_TOKEN} (<reason>)"; then
        declined=1; break
    fi
done

if [ "$(lesson_promotion_verdict "$added_count" "$promoted" "$declined")" = "ok" ]; then
    echo "LESSON-PROMOTION: ok ($added_count new lesson(s), decision recorded)"
    exit 0
fi

{
    printf 'LESSON-PROMOTION: %d new lesson entr%s in %s with NO promotion and NO explicit decline:\n' \
        "$added_count" "$([ "$added_count" -eq 1 ] && echo 'y' || echo 'ies')" "$NOTES"
    printf '%s\n' "$added_entries" | sed 's/^/    /'
    printf '\nA durable lesson that is written down but not retrievable is the measured defect this\n'
    printf 'gate exists to stop (1 592 entries accumulated upstream, none reachable by question).\n\n'
    printf 'Do ONE of:\n'
    printf '  PROMOTE  add docs/knowledge/<slug>.md (front matter at LINE 1) with `answers:`, or add\n'
    printf '           `answers:` to a docs/decisions/ record, then regenerate the Knowledge Map\n'
    printf '  DECLINE  write `%s (<reason>)` in the owning docs/tasks/*.md leaf.\n\n' "$DECLINE_TOKEN"
    printf 'DECLINING IS EXPECTED AND FINE — most lessons are per-slice history and belong exactly\n'
    printf 'where they are. Promote only what is DURABLE (still true after the slice lands), GENERAL\n'
    printf '(reusable beyond one component), RE-VERIFIABLE (a command re-proves it) and QUESTION-SHAPED.\n'
} >&2
exit 1
