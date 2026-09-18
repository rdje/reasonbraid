#!/usr/bin/env bash
# TASK-ACCEPTANCE — a staged CODE change must be owned by a task-tree leaf that carries a ticked
# acceptance checklist, and each hard-gated box must be backed by EVIDENCE INSIDE ITS OWN BULLET.
#
# ⭐ THE DISCIPLINE, stated with no project's nouns: a change lands with (a) the CAUSE located,
# (b) the EFFECT measured, and (c) a statement that nothing regressed — each backed by output
# from a tool that was actually run, not by prose. "I fixed it" is a claim; a pasted verdict is
# an artifact someone else can re-run.
#
# ⭐⭐ WHY SCOPING IS THE SOUNDNESS PROPERTY, not a stylistic nicety. THREE leakage holes have
# been MEASURED on the project this was distilled from, each making the check weaker than it reads:
#   (1) cross-FILE leakage — the greps ran over ALL staged task files, so a co-staged, unrelated
#       tree file could supply the signature for a leaf that carried none. That is exactly how
#       one leaf passed: on tokens belonging to a different tree.
#   (2) incidental-PROSE leakage — a whole-file grep matched a token mentioned anywhere in the
#       leaf rather than inside the ticked box it was supposed to back.
#   (3) 🔴 cross-LEAF leakage — the extractor stopped at the FIRST box matching each label and
#       exited. A project that keeps one tree file per work unit has many leaves in one file:
#       measured at 194 `ROOT CAUSE` boxes, of which the gate read exactly ONE, at line 388, from
#       a leaf closed long before. EVERY code commit was validated against that same historical
#       bullet. Unlike (1) and (2) this was total, not occasional (`SIGNOFF-REPAIR.11.2.6`).
#   ⇒ the checklist must belong to THE LEAF THIS COMMIT CLOSES. Anything looser is a check that
#     reports green on evidence it never tied to the change in front of it.
#
# ⭐⭐ AND AN INERT CONTROL DOES NOT HOLD A LINE — IT HIDES THAT THE LINE MOVED. Because (3) made
# this gate vacuous for ~200 commits, its VOCABULARY drifted out of the corpus unnoticed. It
# hard-gated `ROOT CAUSE` (13 uses) and `ADDRESSED` (14) while the project wrote
# `REPRODUCE / ISSUE` (142), `FIX / LOCKSTEP` (192) and `NO REGRESSION` (203). Two defects, one
# cause. The label families are a project seam now (`.doctrine/acceptance_labels.txt`), derived
# from a census of the real corpus rather than chosen — the same discipline this gate demands of
# the changes it admits.
#
# ⚠️ WHAT "OWNS" MEANS HERE, and why it is derived from the DIFF rather than the commit message:
# a pre-commit hook runs BEFORE the message exists, so the message cannot be read. The owner is
# the leaf whose `Status:` line this commit turns to `done` — a fact carried by the staged diff
# itself. A commit closes 2.75 leaves on average and up to 5, because parent lanes close with
# their child and legitimately carry no checklist of their own, so the DEEPEST closed leaf is the
# one asked. Measured over 300 commits: scoping to every closed leaf refuses 57.7 %; to the
# deepest, with the corpus-derived vocabulary, 17.5 % — and that residual is real, not noise.
#
# ⚠️ A COMMIT THAT CLOSES NO LEAF IS `NOT CHECKED`, reported rather than silently passed. An
# acceptance checklist is the artifact of a leaf CLOSING; a code change mid-leaf is legitimately
# incomplete. Ownership of that change is a different doctrine's job, and it has one.
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
#   .doctrine/acceptance_labels.txt  <NAME><TAB><regex> per line — the hard-gated questions in
#                                  YOUR project's spellings. REPLACES the defaults below.
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

# ── self-test ────────────────────────────────────────────────────────────────────────────────
# Two-sided, and the NEGATIVE arm is the extractor this gate REPLACED: a first-box scan over the
# whole file. It must FAIL the cross-leaf fixture that the real extractor passes, or arm (1) is
# passing for an unrelated reason (`a-control-that-passes-for-an-unrelated-reason`). Pure text in,
# verdict out — no git index, so it costs milliseconds and runs on every commit.
if [ "${1:-}" = "--self-test" ]; then
  fails=0
  scratch="$ROOT/target/doctrine_scratch"; mkdir -p "$scratch"
  st="$(mktemp -d "$scratch/task-acceptance-selftest.XXXXXX")"; trap 'rm -rf "$st"' EXIT

  cat > "$st/tree.md" <<'FIXTURE'
# A tree

### PROJ.1 — a lane that closed long ago
- [x] **ROOT CAUSE** — cargo test rc=0
- [x] **ADDRESSED** — test result: ok. 3 passed
- [x] **NO REGRESSION** — make -n rc=0
- Status: `done`.

### PROJ.2 — the lane closing now
- Opened: `pending`.
- Status: `done`.

#### PROJ.2.1 — the leaf closing now
- REPRODUCED: the probe returned rc=1 before the change.
- THE REPAIR: bind the identifier; test result: ok. 9 passed
- NO REGRESSION: the neighbouring suite is unchanged, rc=0.
- Status: `done`.

#### PROJ.3.1 — a leaf that TALKS about the labels without answering them
- **The finding.** The gate wants ROOT CAUSE, ADDRESSED and NO REGRESSION in every leaf.
- **The census.** `NO REGRESSION` appears 203 times; rc=0.
- Status: `pending`.
FIXTURE

  # The owner extractor, given "every line was added".
  owners_of() {
    awk '
      /^#+[ \t]/ {
        cur = ""
        if (match($0, /[A-Z][A-Z0-9-]+(\.[0-9]+)+/)) cur = substr($0, RSTART, RLENGTH)
        next
      }
      cur != "" && tolower($0) ~ /^[[:space:]]*-[[:space:]]*status:[[:space:]]*`?done`?/ {
        if (!(cur in seen)) { seen[cur] = 1; d = split(cur, p, ".") - 1; print d "\t" cur }
      }
    ' "$1"
  }
  section_of() {
    awk -v want="$2" '
      /^#+[ \t]/ { cur = ""; if (match($0, /[A-Z][A-Z0-9-]+(\.[0-9]+)+/)) cur = substr($0, RSTART, RLENGTH); next }
      cur == want { print }
    ' "$1"
  }

  # (1) all three leaves are seen as closed, and the DEEPEST is PROJ.2.1 — not PROJ.1,
  #     whose complete checklist is the bullet the old extractor would have read.
  got="$(owners_of "$st/tree.md" | sort -n | tail -1 | cut -f2)"
  [ "$got" = "PROJ.2.1" ] || { echo "SELF-TEST: deepest closed leaf was '$got', expected PROJ.2.1" >&2; fails=$((fails+1)); }
  [ "$(owners_of "$st/tree.md" | wc -l | tr -d ' ')" = "3" ] || {
    echo "SELF-TEST: expected 3 closed leaves in the fixture" >&2; fails=$((fails+1)); }

  # (2) a parent NEVER inherits its child's evidence: PROJ.2's own section is two bullets.
  section_of "$st/tree.md" "PROJ.2" > "$st/parent.txt"
  if grep -qiE "no.?regress" "$st/parent.txt"; then
    echo "SELF-TEST: the parent lane inherited its child's NO REGRESSION bullet" >&2; fails=$((fails+1))
  fi

  # (3) the closing leaf's own section answers all three families, in the project's spellings.
  section_of "$st/tree.md" "PROJ.2.1" > "$st/leaf.txt"
  for kw in 'reproduce' 'the repair' 'no.?regress'; do
    grep -qiE "$kw" "$st/leaf.txt" || { echo "SELF-TEST: PROJ.2.1 should answer '$kw'" >&2; fails=$((fails+1)); }
  done

  # (4) 🔴 THE NEGATIVE ARM — the extractor this gate replaced. It reads the FIRST box matching
  #     each label and stops, so it validates PROJ.1 and reports green while PROJ.2.1, the leaf
  #     actually closing, carries no box at all. If this degenerate scan does NOT pick PROJ.1,
  #     the fixture no longer reproduces the defect and arm (1) proves nothing.
  first_box="$(awk '
    /^#+[ \t]/ { cur = ""; if (match($0, /[A-Z][A-Z0-9-]+(\.[0-9]+)+/)) cur = substr($0, RSTART, RLENGTH); next }
    /^[[:space:]]*-[[:space:]]*\[[xX]\]/ { if (match(tolower($0), /root.?cause/)) { print cur; exit } }
  ' "$st/tree.md")"
  [ "$first_box" = "PROJ.1" ] || {
    echo "SELF-TEST: the degenerate first-box scan chose '$first_box', so the fixture does not reproduce the cross-leaf hole" >&2
    fails=$((fails+1)); }

  # (4b) 🔴 INCIDENTAL PROSE ANSWERS NOTHING — the defect this gate's own in-situ falsification
  #      found. PROJ.3.1 MENTIONS all three labels and answers none; its bullets lead with "The
  #      finding" and "The census". A family matched anywhere in the bullet passes it, which is
  #      how a leaf ABOUT the vocabulary satisfies the vocabulary. The lead-in anchor is the
  #      difference, so both directions are asserted here.
  lead='^[[:space:]]*-[[:space:]]*(\[[xX]\][[:space:]]*)?[^[:alpha:]]*'
  section_of "$st/tree.md" "PROJ.3.1" > "$st/prose.txt"
  if grep -qiE "${lead}(no.?regress)" "$st/prose.txt"; then
    echo "SELF-TEST: incidental prose answered NO REGRESSION" >&2; fails=$((fails+1))
  fi
  if ! grep -qiE "no.?regress" "$st/prose.txt"; then
    echo "SELF-TEST: the prose fixture no longer MENTIONS the label, so arm (4b) proves nothing" >&2
    fails=$((fails+1))
  fi
  # ...and the real answers still read, so the anchor is not simply refusing everything.
  section_of "$st/tree.md" "PROJ.2.1" > "$st/leaf2.txt"
  for kw in 'reproduce' 'the repair' 'no.?regress'; do
    grep -qiE "${lead}($kw)" "$st/leaf2.txt" || {
      echo "SELF-TEST: the lead-in anchor rejected a real answer to '$kw'" >&2; fails=$((fails+1)); }
  done

  # (5) an UNTICKED box does not answer its question.
  printf -- '- [ ] **NO REGRESSION** — not run yet\n' > "$st/unticked.txt"
  if grep -E '^[[:space:]]*-[[:space:]]' "$st/unticked.txt" \
     | grep -vE '^[[:space:]]*-[[:space:]]*\[[[:space:]]\]' | grep -qiE "no.?regress"; then
    echo "SELF-TEST: an unticked box satisfied its label" >&2; fails=$((fails+1))
  fi

  # (6) the hunk parser reads POST-image line numbers, never line TEXT. Matching by text is the
  #     measured mistake: a `Status: done` line is byte-identical for every leaf.
  printf '%s\n' '@@ -1,0 +7,2 @@' '+alpha' '+beta' '@@ -20,1 +30,1 @@' '-old' '+gamma' > "$st/diff.txt"
  got="$(awk '
    /^@@/ { split($3, p, ","); n = substr(p[1], 2) + 0; next }
    /^\+\+\+/ { next }
    /^\+/ { print n; n++; next }
    /^ / { n++; next }
  ' "$st/diff.txt" | tr '\n' ' ')"
  [ "$got" = "7 8 30 " ] || { echo "SELF-TEST: hunk parser returned '$got', expected '7 8 30 '" >&2; fails=$((fails+1)); }

  if [ "$fails" -ne 0 ]; then
    echo "SELF-TEST FAILED: $fails arm(s)" >&2; exit 1
  fi
  echo "SELF-TEST: deepest-closed-leaf chosen over a complete sibling checklist, parent does not inherit its child's evidence, all three families answered in the project's spellings, incidental prose about a label answers nothing, the replaced first-box scan picks the WRONG leaf, an unticked box answers nothing, hunk parser reads post-image line numbers"
  exit 0
fi

# grep a FILE, never `printf "$var" | grep -q`: under pipefail, grep -q exits at the first match
# and the producer takes SIGPIPE, so the pipeline reports FAILURE ON SUCCESS once the input is
# large. That failure mode is silent and, at a `|| continue`, fails OPEN.
# ⛔ §13: scratch is REPOSITORY-derived, never ambient. A bare `mktemp -d` follows
# TMPDIR, measured on a different volume from this checkout, so a gate that enforces
# doctrine would itself be writing project data off the repository's storage.
# `mktemp` still names the directory by EXCLUSIVE CREATION — the half of
# STORAGE-LOCALITY that a clock-proposed name gets wrong (SIGNOFF-REPAIR.11.2.2).
scratch="$ROOT/target/doctrine_scratch"; mkdir -p "$scratch"
tmp="$(mktemp -d "$scratch/task-acceptance.XXXXXX")"; trap 'rm -rf "$tmp"' EXIT
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

# ── the hard-gated questions, a project seam ─────────────────────────────────────────────────
# Portable defaults. Generic engineering words only — never one project's nouns. A project whose
# corpus says it differently declares that in .doctrine/acceptance_labels.txt, which REPLACES
# these; see that file for why the seam exists and how its regexes were derived.
LABEL_FILE=".doctrine/acceptance_labels.txt"
if [ -f "$LABEL_FILE" ]; then
  grep -vE '^[[:space:]]*(#|$)' "$LABEL_FILE" > "$tmp/labels.txt"
else
  printf '%s\t%s\n' \
    "ROOT CAUSE"    'root.?cause|reproduce|reproduced|\bissue\b|measured|the census|the defect' \
    "ADDRESSED"     'addressed|\bfix\b|decided|the decision|the repair|the rule|the result' \
    "NO REGRESSION" 'no.?regress' > "$tmp/labels.txt"
fi
[ -s "$tmp/labels.txt" ] || { echo "TASK-ACCEPTANCE: $LABEL_FILE declares no label families." >&2; exit 1; }

# ⛔ ONE definition of "the family is this bullet's LEAD-IN", used by the gate AND by `--debt`.
# Two copies of one rule drift — this repository has repaired that four times — and a debt
# census counting differently from the gate that produces the debt is exactly that bug.
# `[^[:alpha:]]*` steps over the checkbox, any emoji and the bold markers, stopping at the first
# letter, which must begin the label.
LEAD='^[[:space:]]*-[[:space:]]*(\[[xX]\][[:space:]]*)?[^[:alpha:]]*'

# ── the debt this gate did not ask about while it was inert ──────────────────────────────────
# ⛔ PRINTED, NEVER RESTATED IN PROSE. The gate was vacuous for ~200 commits, so leaves closed
# without being asked these questions. That population is real and it is NOT cleared by repairing
# the gate: the check is staged-scope-aware and only ever examines the leaf closing NOW, so
# history neither blocks a commit nor gets fixed by one.
#
# ⛔ AND IT MUST NOT BE BACKFILLED. Writing "NO REGRESSION: ..." into a leaf closed weeks ago
# would be manufacturing evidence after the fact — asserting a suite was run when no one ran it.
# That is the one thing this gate exists to prevent, so doing it to satisfy the gate would be
# self-defeating. The decision is recorded in docs/decisions/; this flag keeps the number honest
# and countable instead of letting it become a sentence nobody can re-derive.
#
#     scripts/check_task_acceptance.sh --debt
if [ "${1:-}" = "--debt" ]; then
  total=0; owing=0
  for tree in $(git ls-files 'docs/tasks/*.md' | grep -vE '(^|/)TEMPLATE\.md$'); do
    awk '
      /^#+[ \t]/ { cur = ""; if (match($0, /[A-Z][A-Z0-9-]+(\.[0-9]+)+/)) cur = substr($0, RSTART, RLENGTH); next }
      cur != "" { print cur "\t" $0 }
    ' "$tree" > "$tmp/all.txt"
    # Leaves that are CLOSED: their section carries a `Status: done` line.
    cut -f1 "$tmp/all.txt" | sort -u | while IFS= read -r id; do
      [ -n "$id" ] || continue
      awk -F'\t' -v want="$id" '$1 == want { sub(/^[^\t]*\t/, ""); print }' "$tmp/all.txt" > "$tmp/sec.txt"
      grep -qiE '^[[:space:]]*-[[:space:]]*status:[[:space:]]*`?done`?' "$tmp/sec.txt" || continue
      grep -E '^[[:space:]]*-[[:space:]]' "$tmp/sec.txt" \
        | grep -vE '^[[:space:]]*-[[:space:]]*\[[[:space:]]\]' > "$tmp/b.txt" || true
      missing=""
      while IFS="$(printf '\t')" read -r label kw; do
        [ -n "$label" ] || continue
        grep -qiE "$LEAD($kw)" "$tmp/b.txt" || missing="$missing $label;"
      done < "$tmp/labels.txt"
      if [ -n "$missing" ]; then printf '%s\t%s\t%s\n' "$tree" "$id" "$missing"; fi
    done
  done > "$tmp/debt.txt"
  total="$(git ls-files 'docs/tasks/*.md' | grep -vE '(^|/)TEMPLATE\.md$' | xargs grep -ciE '^[[:space:]]*-[[:space:]]*status:[[:space:]]*`?done`?' 2>/dev/null | awk -F: '{s+=$2} END{print s+0}')"
  owing="$(grep -c . "$tmp/debt.txt" || true)"
  echo "TASK-ACCEPTANCE debt: $owing of $total closed leaves answer fewer than every question."
  echo "  Closed while this gate was inert (SIGNOFF-REPAIR.11.2.6). NOT backfilled — see"
  echo "  docs/decisions/ for why manufacturing the evidence afterwards would defeat the gate."
  sed 's/^/    /' "$tmp/debt.txt"
  exit 0
fi


staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null || true)"
[ -n "$staged" ] || exit 0

printf '%s\n' "$staged" > "$tmp/staged.txt"

grep -E "$code_re" "$tmp/staged.txt" > "$tmp/code.txt" 2>/dev/null || true
[ -s "$tmp/code.txt" ] || exit 0          # pure-docs change: this doctrine does not govern it

# ⛔ TEMPLATE.md is the blank form authors COPY — its boxes are deliberately unticked, so
# treating it as a leaf makes the doctrine block every commit that edits the template. Found by
# this check refusing its own commit: a FALSE POSITIVE, unlike the two refusals before it, which
# were correct. The same exclusion exists in the layer-C check in this repo (INDEX/TEMPLATE).
# A tree is docs/tasks/<TREE-ID>.md (TASK_TREE_README.md). Nested evidence is
# neither another owning tree nor a substitute for staging the real owner.
grep -E '^docs/tasks/[^/]+\.md$' "$tmp/staged.txt" | grep -vE '(^|/)TEMPLATE\.md$' \
  > "$tmp/leaves.txt" 2>/dev/null || true
if [ ! -s "$tmp/leaves.txt" ]; then
  {
    echo "TASK-ACCEPTANCE: a CODE change is staged but NO owning task-tree leaf (docs/tasks/*.md) is."
    echo "  staged code:"; sed 's/^/    /' "$tmp/code.txt"
    echo "  Stage the docs/tasks/<TREE>.md that owns this change, carrying the acceptance checklist."
  } >&2
  exit 1
fi

# ── who OWNS this commit: the leaf whose Status turns to `done` in the staged diff ───────────
# POST-image line numbers of the added lines. `-U0` so a hunk is exactly its additions; the
# hunk header's third field is `+<start>[,<count>]`, which is the only sound way to say WHERE a
# line landed. ⛔ Matching added lines by their TEXT is not sound and was tried: `- Status:
# `done`` is byte-identical for every leaf, so one commit appeared to close dozens.
added_lines() {
  git diff --cached -U0 -- "$1" | awk '
    /^@@/ { split($3, p, ","); n = substr(p[1], 2) + 0; next }
    /^\+\+\+/ { next }
    /^\+/ { print n; n++; next }
    /^ / { n++; next }
  '
}

# ⚠️ The line numbers above index the STAGED blob, so the content read beside them
# must be the staged blob too. Reading the working tree would silently misalign
# every attribution whenever a file is edited after `git add` — a pre-commit hook
# is exactly where that happens.
staged_blob() {
  git show ":$1" 2>/dev/null
}

# Every line belongs to the NEAREST PRECEDING heading that names a leaf; a heading that names
# none clears the owner, so a table or a prose section cannot be attributed to the leaf above it.
# Emits "<depth>\t<leaf>" for each leaf closed by this commit.
closed_leaves() {
  local file="$1" blob="$2"
  added_lines "$file" > "$tmp/added.txt"
  awk -v FS='\n' '
    FNR == NR { add[$1 + 0] = 1; next }
    /^#+[ \t]/ {
      cur = ""
      if (match($0, /[A-Z][A-Z0-9-]+(\.[0-9]+)+/)) cur = substr($0, RSTART, RLENGTH)
      next
    }
    cur != "" && add[FNR] && tolower($0) ~ /^[[:space:]]*-[[:space:]]*status:[[:space:]]*`?done`?/ {
      if (!(cur in seen)) {
        seen[cur] = 1
        d = split(cur, parts, ".") - 1
        print d "\t" cur
      }
    }
  ' "$tmp/added.txt" "$blob"
}

: > "$tmp/owners.txt"
n=0
while IFS= read -r leaf; do
  n=$((n + 1))
  blob="$tmp/staged-$n.md"
  staged_blob "$leaf" > "$blob" || continue
  [ -s "$blob" ] || continue
  printf '%s\t%s\n' "$leaf" "$blob" >> "$tmp/blobs.txt"
  closed_leaves "$leaf" "$blob" | while IFS="$(printf '\t')" read -r depth id; do
    printf '%s\t%s\t%s\t%s\n' "$depth" "$leaf" "$id" "$blob" >> "$tmp/owners.txt"
  done
done < "$tmp/leaves.txt"

if [ ! -s "$tmp/owners.txt" ]; then
  echo "task-acceptance: NOT CHECKED — this commit closes no task-tree leaf."
  echo "  A code change mid-leaf is legitimately incomplete; the acceptance checklist is the"
  echo "  artifact of a leaf CLOSING. Ownership of the change is TASK-TREE-OWNERSHIP's question."
  exit 0
fi

# The DEEPEST closed leaf is the one asked: a parent lane closes with its child and carries no
# checklist of its own. Ties (two equally deep leaves) are all asked.
deepest="$(cut -f1 "$tmp/owners.txt" | sort -n | tail -1)"

# ── the checklist of the leaf that owns this commit ──────────────────────────────────────────
fail=0
while IFS="$(printf '\t')" read -r depth file id blob; do
  [ "$depth" = "$deepest" ] || continue

  # The leaf's own section: every line whose nearest preceding leaf heading is this leaf. A
  # child's heading ends it, so a parent never inherits its child's evidence.
  awk -v want="$id" '
    /^#+[ \t]/ {
      cur = ""
      if (match($0, /[A-Z][A-Z0-9-]+(\.[0-9]+)+/)) cur = substr($0, RSTART, RLENGTH)
      next
    }
    cur == want { print }
  ' "$blob" > "$tmp/section.txt"

  if [ ! -s "$tmp/section.txt" ]; then
    echo "TASK-ACCEPTANCE: $file — leaf $id closes here but has no section to read." >&2
    fail=1; continue
  fi
  # A bullet answers a question unless it is an UNTICKED box: `- [ ] ROOT CAUSE`
  # is the shape of a checklist not yet done, and it must never satisfy the gate.
  grep -E '^[[:space:]]*-[[:space:]]' "$tmp/section.txt" \
    | grep -vE '^[[:space:]]*-[[:space:]]*\[[[:space:]]\]' > "$tmp/bullets.txt" || true

  # ⛔ THE FAMILY MUST BE THE BULLET'S LEAD-IN, never a word anywhere inside it. Found by
  # falsifying this gate in situ against its own leaf: that leaf DISCUSSES the label
  # `NO REGRESSION`, so five prose bullets mentioning it satisfied the question and deleting
  # the real checklist bullet changed nothing. That is leakage hole (2) — incidental prose —
  # walking back in through the widened vocabulary, and a self-referential leaf is exactly where
  # it bites. `[^[:alpha:]]*` steps over the checkbox, the emoji and the bold markers and stops
  # at the first letter, which must begin the label. Priced: 17.5 % -> 20.6 % refused over 300
  # commits, and the extra refusals are prose that was never a checklist answer.
  while IFS="$(printf '\t')" read -r label kw; do
    [ -n "$label" ] || continue
    if ! grep -qiE "$LEAD($kw)" "$tmp/bullets.txt"; then
      {
        echo "TASK-ACCEPTANCE: $file — leaf $id closes in this commit but no bullet answers '$label'."
        echo "  The checklist belongs to the leaf THIS commit closes, not to the file's first leaf."
        echo "  Add the bullet, or declare your project's spelling in $LABEL_FILE."
      } >&2
      fail=1
    fi
  done < "$tmp/labels.txt"

  # A tick is a claim; the leaf must cite something re-runnable. Scoped to this leaf's own
  # section — which is what holes (1) and (3) were both about.
  if ! grep -qE "$SIG" "$tmp/section.txt"; then
    {
      echo "TASK-ACCEPTANCE: $file — leaf $id answers every question but cites no tool output."
      echo "  A claim is not an artifact. Paste the invocation and its real result into the leaf,"
      echo "  or declare your project's signatures in .doctrine/evidence_tokens.txt."
    } >&2
    fail=1
  fi
done < "$tmp/owners.txt"

[ "$fail" -eq 0 ] || exit 1
echo "task-acceptance: OK (the leaf this commit closes answers every question, with tool output)"
exit 0
