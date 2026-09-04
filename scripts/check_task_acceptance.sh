#!/usr/bin/env bash
# TASK-ACCEPTANCE — a staged CODE change must be owned by a task-tree leaf that carries a ticked
# acceptance checklist, and each hard-gated box must be backed by EVIDENCE INSIDE ITS OWN BULLET.
#
# ⭐ THE DISCIPLINE, stated with no project's nouns: a change lands with (a) the CAUSE located,
# (b) the EFFECT measured, and (c) a statement that nothing regressed — each backed by output
# from a tool that was actually run, not by prose. "I fixed it" is a claim; a pasted verdict is
# an artifact someone else can re-run.
#
# ⭐⭐ WHY BOX-SCOPING IS THE SOUNDNESS PROPERTY, not a stylistic nicety. Two leakage holes were
# MEASURED on the project this was distilled from, and both made the check weaker than it reads:
#   (1) cross-FILE leakage — the greps ran over ALL staged task files, so a co-staged, unrelated
#       tree file could supply the signature for a leaf that carried none. That is exactly how
#       one leaf passed: on tokens belonging to a different tree.
#   (2) incidental-PROSE leakage — a whole-file grep matched a token mentioned anywhere in the
#       leaf rather than inside the ticked box it was supposed to back.
#   ⇒ the signature must sit in the SAME BULLET as the box it backs. Anything looser is a check
#     that reports green on evidence it never actually tied to a claim.
#
# ⚠️ HONEST LIMIT, stated rather than hidden: this verifies a box was TICKED and that
# tool-shaped output sits inside it. It cannot verify the output is true. A ticked box is
# leg 1 (presence); the un-fakeable leg is re-running the cited command in CI. Do not describe
# this check as proving correctness — it proves the author cited something re-runnable.
#
# ── PROJECT SEAMS (this is what keeps the check neutral) ─────────────────────────────────────
#   .doctrine/code_paths.txt       one glob per line — what counts as a CODE change here.
#                                  Absent -> the built-in default below (Rust workspace shape).
#   .doctrine/evidence_tokens.txt  one regular expression per line — YOUR tools' output signatures,
#                                  ADDED to the universal defaults. Absent -> defaults only.
# ⛔ Never hardcode one project's tool names in this file. That is the difference between a
#   portable standard and a fork of somebody else's workflow.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on stderr;
# deterministic; read-only; staged-scope-aware; path-agnostic.
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

# ── what counts as a code change ─────────────────────────────────────────────────────────────
default_code_re='(^|/)(crates|src|scripts)/|\.(rs|sh)$|(^|/)Makefile$'
if [ -f .doctrine/code_paths.txt ]; then
  code_re="$(grep -vE '^\s*(#|$)' .doctrine/code_paths.txt | paste -sd'|' -)"
  [ -n "$code_re" ] || code_re="$default_code_re"
else
  code_re="$default_code_re"
fi

staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null || true)"
[ -n "$staged" ] || exit 0

# grep a FILE, never `printf "$var" | grep -q`: under pipefail, grep -q exits at the first match
# and the producer takes SIGPIPE, so the pipeline reports FAILURE ON SUCCESS once the input is
# large. That failure mode is silent and, at a `|| continue`, fails OPEN.
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
printf '%s\n' "$staged" > "$tmp/staged.txt"

grep -E "$code_re" "$tmp/staged.txt" > "$tmp/code.txt" 2>/dev/null || true
[ -s "$tmp/code.txt" ] || exit 0          # pure-docs change: this doctrine does not govern it

# ⛔ TEMPLATE.md is the blank form authors COPY — its boxes are deliberately unticked, so
# treating it as a leaf makes the doctrine block every commit that edits the template. Found by
# this check refusing its own commit: a FALSE POSITIVE, unlike the two refusals before it, which
# were correct. The same exclusion exists in the layer-C check in this repo (INDEX/TEMPLATE).
grep -E '^docs/tasks/.*\.md$' "$tmp/staged.txt" | grep -vE '(^|/)TEMPLATE\.md$' \
  > "$tmp/leaves.txt" 2>/dev/null || true
if [ ! -s "$tmp/leaves.txt" ]; then
  {
    echo "TASK-ACCEPTANCE: a CODE change is staged but NO owning task-tree leaf (docs/tasks/*.md) is."
    echo "  staged code:"; sed 's/^/    /' "$tmp/code.txt"
    echo "  Stage the docs/tasks/<TREE>.md that owns this change, carrying the acceptance checklist."
  } >&2
  exit 1
fi

# ── evidence signatures ──────────────────────────────────────────────────────────────────────
# Universal defaults. Every entry is either standard Rust/Cargo tooling (this template is a Rust
# scaffold, so these apply to ANY consumer) or plain build-flow forensics available in ANY
# project. ⛔ No entry may name a specific project's tool.
#
# ⚠️ PRICED AGAINST A REAL CORPUS, and the first cut was TOO NARROW — measured, not guessed.
# The first version of this list rejected a leaf whose boxes cited `awk version 20200816`,
# `probes: 3 pass / 6 fail` and `exit=0`: all genuinely tool-emitted, none matched. That is the
# failure mode where *a signature family that does not fit the real corpus becomes a gate authors
# learn to waive*. Generic result shapes (`exit=N`, `rc=N`, `N pass / N fail`, version banners)
# were added because they are what tools actually print — not to make the gate easier.
DEFAULT_SIG='error\[E[0-9]{4}\]|could not compile|clippy::[a-z_]{3,}|panicked at|assertion (failed|`)|test result: (ok|FAILED)|running [0-9]+ tests?|cargo (test|build|bench|flamegraph)|flamegraph|self-time|call-graph|/usr/bin/sample|\bspindump\b|\bperf (record|stat)\b|\bvalgrind\b|git (ls-files|log -S|log --all -S|rev-list|fsck|reflog|diff-tree|merge-base|cat-file|show )|\bshellcheck\b|bash -n |sh -n |make -n |make --dry-run|\bE2BIG\b|\bENOSPC\b|\bEACCES\b|\bARG_MAX\b|exit(ed)?[ =:](code )?[0-9]+|\brc=[0-9]+|PIPESTATUS|[0-9]+ (pass|passed|ok)[ ,/]+[0-9]+ (fail|failed)|version [0-9]{4,}|[0-9]+\.[0-9]+\.[0-9]+'
SIG="$DEFAULT_SIG"
if [ -f .doctrine/evidence_tokens.txt ]; then
  extra="$(grep -vE '^\s*(#|$)' .doctrine/evidence_tokens.txt | paste -sd'|' -)"
  [ -n "$extra" ] && SIG="$SIG|$extra"
fi

# The three hard-gated boxes. FIX / REPRODUCE / LOCKSTEP are good practice but not blocked, so
# an honest author is never forced to invent evidence for a box that does not apply.
fail=0
while IFS= read -r leaf; do
  [ -r "$leaf" ] || continue
  for spec in 'ROOT CAUSE:root.?cause' 'ADDRESSED:addressed' 'NO REGRESSION:no.?regress'; do
    label="${spec%%:*}"; kw="${spec#*:}"
    # A box's BULLET = the "- [x] ..." line plus its indented continuation lines, so wrapped
    # markdown still counts while a token elsewhere in the file does not.
    # ⛔ POSIX awk only — no `IGNORECASE`, which is a gawk extension that BSD awk (the default
    # on several platforms) silently IGNORES, so the box would never match and every leaf would
    # be reported as having no checklist at all. Case-folding is done with tolower(), which is
    # POSIX. A template must run on whatever awk the consumer has.
    awk -v kw="$kw" '
      BEGIN{ inbox=0 }
      {
        line = $0
        isbox = (line ~ /^[[:space:]]*-[[:space:]]*\[[xX ]\]/)
        if (isbox) {
          if (inbox) exit
          if (match(tolower(line), kw)) { inbox=1; print; next }
          next
        }
        if (inbox) {
          # A bullet continues through indented and blank lines; anything flush-left ends it.
          if (line ~ /^[[:space:]]+/ || line ~ /^[[:space:]]*$/) { print; next }
          exit
        }
      }
    ' "$leaf" > "$tmp/box.txt"

    if [ ! -s "$tmp/box.txt" ]; then
      echo "TASK-ACCEPTANCE: $leaf has no '$label' box in its acceptance checklist." >&2
      fail=1; continue
    fi
    if ! head -1 "$tmp/box.txt" | grep -qE '\[[xX]\]'; then
      echo "TASK-ACCEPTANCE: $leaf — the '$label' box is present but NOT ticked." >&2
      fail=1; continue
    fi
    if ! grep -qE "$SIG" "$tmp/box.txt"; then
      {
        echo "TASK-ACCEPTANCE: $leaf — the '$label' box is ticked but carries no tool-output evidence"
        echo "  INSIDE ITS OWN BULLET. A tick is a claim; the box asks for output from a command you ran."
        echo "  Add the invocation and its real output to that bullet, or declare your project's own"
        echo "  signatures in .doctrine/evidence_tokens.txt (one extended regular expression per line)."
      } >&2
      fail=1
    fi
  done
done < "$tmp/leaves.txt"

[ "$fail" -eq 0 ] || exit 1
echo "task-acceptance: OK (every staged code-change leaf carries a ticked, evidence-backed checklist)"
exit 0
