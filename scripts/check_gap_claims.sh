#!/usr/bin/env bash
# GAP-CLAIM-CENSUS — a task leaf that ADDS a "nothing checks X" claim must record the CENSUS it
# rests on. Exits NONZERO on breach. Called by scripts/check_doctrines.sh (hook + CI).
#
# ⭐⭐ WHY THIS EXISTS (ported by REASONBRAID-MAINTENANCE.2.6 from the originating project, where the
#   defect was a HABIT rather than a slip: three false "nothing checks X" claims in three days, two of
#   them reaching a commit and one REFUTED by the very next slice — the reader it said did not exist
#   existed, and had for weeks). An "X is checked by NOTHING" sentence is not a description — it is a
#   UNIVERSALLY QUANTIFIED CLAIM over the whole tree, i.e. a CENSUS claim, and it is false the moment
#   one reader exists.
# ⭐ AND THE CENSUS IS NOT MERELY DEFENSIVE — IT LOCATES THE DEFECT. In the third occurrence the false
#   claim would have sent a fix at a mechanism that did not exist; running the census instead produced
#   the exact predicate the fix then corrected. That is the argument for blocking rather than warning.
#
# ⛔⛔ STAGED-DIFF-SCOPED, NEVER OVER THE EXISTING TREE. Measured upstream over the closed tracked
#   population: 242 claim lines across 29 task files, 81 with no census in their own section. A blocker
#   firing on 81 pre-existing claims is unusable and teaches bypass. Scoped to what a commit ADDS, it
#   fires on nothing already in the tree. `--all` reports that backlog, advisory, never blocking.
# ⭐⭐ CALIBRATED BY REPLAYING REAL HISTORY, NOT AN IMAGINED PHRASING: over 150 upstream commits the
#   predicate would have blocked 16 added lines in 14 commits, 12 of them genuine unbacked claims
#   (75 % true-positive on the strict reading). Both founding sentences — the active
#   "no instrument in the tree crosses the two" and the passive "is checked by nothing" — are pinned
#   verbatim by `--self-test`.
# ⛔ THE GOVERNED POPULATION IS `docs/tasks/*.md` ONLY (measured: claims in the changelog and the
#   dev notes co-occur with a task-leaf claim in the same commit 76–90 % of the time, so governing
#   them would double-fire). The layer-A resume pointer is excluded STRUCTURALLY — it is byte-capped.
#
# ARCHETYPE: evidence. HONEST LIMIT — this verifies the census was RECORDED, not that it was RUN,
#   nor that its population was right, nor that its conclusion was correct.
# ⚠️ KNOWN FALSE-POSITIVE CLASS (measured upstream at 1 in 13): a line that QUOTES an earlier claim
#   fires it. Accepted: the discharge is one pasted command; it errs toward asking.
# CONTRACT: exit code is the verdict; explains on stderr; deterministic; read-only;
#   staged-scope-aware; path-agnostic; inert out of scope.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

HEADLINE="GAP-CLAIM-CENSUS"
WORK="target/doctrine_scratch/gap_claim_census"

note() { printf '%s: %s\n' "$HEADLINE" "$1" >&2; }
ok()   { printf '%s: %s\n' "$HEADLINE" "$1"; }

# ── the claim predicate ─────────────────────────────────────────────────────────────────────────
# ⛔ Keyed on `nothing` + a TRANSITIVE VERB, never on the bare word (`nothing else` / `nothing
# shipped` are not claims about the tree). The verb is what makes the sentence quantify over it.
# ⭐⭐ The PASSIVE VOICE is the same claim with the subject moved (`… is checked by nothing`), which
# the first cut upstream missed on its own founding sentence. The participle is what skips the
# idiom `and by nothing else`; irregulars that do not end in `ed` are listed explicitly.
CLAIM_RE='nothing (checks|checked|reads|compares|measures|measured|watches|enforces|gates|verifies|validates|asserts|consumes|notices|invokes|runs|ran|re-runs|re-derives|derives|looks|tracks|tracked|reports|counts|sees|knows)|nothing (in|inside) the (repo|repository|tree|codebase|doctrine|gate|registry)|[a-z]+ed by nothing|held by nothing|read by nothing|kept by nothing|built by nothing|met by nothing|no instrument'

# ── the discharge ───────────────────────────────────────────────────────────────────────────────
# A COMMAND-SHAPED token in the claim's own heading section — the thing that ENUMERATES a
# population — or an explicit `census:` disclosure. ⛔ The bare word "census" is deliberately NOT a
# discharge: it appears inside the claims themselves ("nothing re-runs this census"). ⛔ `grep -n`
# and `grep -w` are in the list because they are the commonest census spelling; a BARE `grep` is
# not, because "verified by grep" is a claim, not tool output. The FLAG is what makes it an
# invocation.
CENSUS_RE='git grep|grep -r|grep -c|grep -l|grep -o|grep -n|grep -w|git ls-files|git log -s|git log --grep|git log --oneline|rg -|--dump-|--report-|--self-test|--lint|scripts/check_|make |comm -|wc -l|sort -u|uniq -c|sed -n|git show|git diff|cargo |a search of|searched (the|all|every)|census:'

# Section boundaries are ANY ATX heading, deliberately: "nearest heading above" needs no leaf-id
# syntax and cannot mis-parse one.
CLASSIFY_AWK='
  FNR==NR { added[$1+0]=1; next }
  { line[FNR]=$0; low[FNR]=tolower($0) }
  END{
    for(i=1;i<=FNR;i++){ if(line[i] ~ /^#{1,6} /) cur=i; sec[i]=cur }
    for(i=1;i<=FNR;i++){ if(low[i] ~ CENSUS_RE) has[sec[i]]=1 }
    for(i=1;i<=FNR;i++){
      if(added[i] && low[i] ~ CLAIM_RE && !has[sec[i]])
        printf "BLOCKED\t%d\t%s\n", i, substr(line[i],1,160)
    }
  }'

added_line_numbers() {
  awk '
    /^@@/ { split($3,a,","); ln=a[1]+0; next }
    /^\+\+\+/ { next }
    /^\+/ { print ln; ln++; next }'
}

report_rows() {
  local file="$1" lineno text
  while IFS=$'\t' read -r _ lineno text; do
    [ -n "$lineno" ] || continue
    printf '  %s:%s\n      %s\n' "$file" "$lineno" "$text" >&2
  done
}

remedy() {
  {
    echo
    echo "  An \"X is checked by NOTHING\" sentence is a CENSUS claim over the whole tree, not a"
    echo "  description — it is false the moment ONE reader exists."
    echo
    echo "  Discharge it in the SAME heading section, either way:"
    echo "    1. paste the census you ran, e.g."
    echo "         git grep -n '<symbol>' -- src scripts | wc -l   # -> 0"
    echo "       Any enumerating command counts: git grep / grep -r / git ls-files / git log -S /"
    echo "       a scripts/check_*.sh, a --self-test or a --report-/--dump- instrument invocation."
    echo "    2. or disclose that you did not run one:"
    echo "         census: not run (<why>) — the claim is REASONED, not measured"
    echo "       An honest 'not run' is a legal answer and a useful signal; a silent claim is not."
    echo
    echo "  ⛔ Census the population that would REFUTE you, not the one that confirms you."
  } >&2
}

mode="${1:-}"

if [ "$mode" = "--self-test" ]; then
  mkdir -p "$WORK/selftest"; work="$WORK/selftest"; arms=0; passed=0
  classify() { awk -v CLAIM_RE="$CLAIM_RE" -v CENSUS_RE="$CENSUS_RE" "$CLASSIFY_AWK" "$1" "$2"; }
  arm() { arms=$((arms + 1)); if [ "$1" = ok ]; then passed=$((passed + 1)); ok "arm $arms $2"; else note "arm $arms FAILED — $2"; fi; }
  # 1 GREEN control: a claim WITH a census in its own section is not reported
  printf '### `.9` — a leaf that measures its own claim\n- **THE GAP** — nothing reads the `@sample` annotation.\n- **CENSUS** — `git grep -c '"'"'@sample'"'"' -- src scripts` returns 0 over 2 paths.\n' > "$work/green.md"; printf '2\n3\n' > "$work/green.nums"
  [ -z "$(classify "$work/green.nums" "$work/green.md")" ] && arm ok "GREEN control — a censused claim passes" || arm bad "a censused claim must not be reported"
  # 2 RED: the same claim with the census REMOVED
  printf '### `.9` — a leaf that measures its own claim\n- **THE GAP** — nothing reads the `@sample` annotation.\n' > "$work/red.md"; printf '2\n' > "$work/red.nums"
  classify "$work/red.nums" "$work/red.md" | grep -q '^BLOCKED' && arm ok "RED — an unbacked claim is reported" || arm bad "an unbacked claim must be reported"
  # 3 the founding ACTIVE sentence, verbatim
  printf '### `.8` — the denominator\n  so it cannot see a profile. => **no instrument in the tree crosses the two**, and the\n' > "$work/founding.md"; printf '2\n' > "$work/founding.nums"
  classify "$work/founding.nums" "$work/founding.md" | grep -q '^BLOCKED' && arm ok "the founding active sentence is caught" || arm bad "'no instrument in the tree crosses the two' must fire"
  # 3b the founding PASSIVE sentence, verbatim shape
  printf '### `.7` — the legacy profile\n- coherence is checked at 0 on every commit; **faithfulness — every reachable rule DERIVABLE from\n  the legacy standard — is checked by nothing**, which is why nine over-acceptances sat behind one profile.\n' > "$work/passive.md"; printf '2\n3\n' > "$work/passive.nums"
  classify "$work/passive.nums" "$work/passive.md" | grep -q '^BLOCKED' && arm ok "the founding PASSIVE sentence is caught" || arm bad "'is checked by nothing' must fire"
  # 3c the idiom the participle rule must NOT catch
  printf '### `.9` — an ordinary leaf\n- the re-derivation differed by the `-o` path spelling **and by nothing else at all**.\n' > "$work/idiom2.md"; printf '2\n' > "$work/idiom2.nums"
  [ -z "$(classify "$work/idiom2.nums" "$work/idiom2.md")" ] && arm ok "'and by nothing else' is the idiom, not a claim" || arm bad "the participle rule must skip 'and by nothing else'"
  # 4 a pre-existing claim (not in the added set) is INERT
  : > "$work/none.nums"
  [ -z "$(classify "$work/none.nums" "$work/red.md")" ] && arm ok "a pre-existing claim is INERT" || arm bad "the check must judge only what the commit ADDS"
  # 5 the house idiom is NOT a claim
  printf '### `.9` — an ordinary leaf\n- the totals delta *is* that rule and nothing else. Nothing shipped moved, and nothing technical\n  blocks it; there is nothing to do here.\n' > "$work/idiom.md"; printf '2\n3\n' > "$work/idiom.nums"
  [ -z "$(classify "$work/idiom.nums" "$work/idiom.md")" ] && arm ok "nothing else / shipped / technical are NOT claims" || arm bad "the bare word must not fire"
  # 6 the explicit disclosure discharges
  printf '### `.9` — a reasoned claim\n- **THE GAP** — nothing watches the self-rejection matrix.\n- census: not run (the population is the whole engine; priced at ~20 min and declined here)\n' > "$work/disclosed.md"; printf '2\n3\n' > "$work/disclosed.nums"
  [ -z "$(classify "$work/disclosed.nums" "$work/disclosed.md")" ] && arm ok "'census: not run (<why>)' is a legal discharge" || arm bad "an honest disclosure must discharge"
  # 7 a census in a DIFFERENT section does not discharge
  printf '### `.8` — an earlier leaf\n- **CENSUS** — `git grep -c '"'"'@sample'"'"' -- src` returns 0.\n\n### `.9` — the leaf making the claim\n- **THE GAP** — nothing reads the `@sample` annotation.\n' > "$work/other.md"; printf '5\n' > "$work/other.nums"
  classify "$work/other.nums" "$work/other.md" | grep -q '^BLOCKED' && arm ok "a census in another section does not discharge" || arm bad "the discharge must be section-scoped"
  # 8 hunk arithmetic: added lines resolve to NEW-file numbers
  parsed="$(printf '%s\n' '--- a/x.md' '+++ b/x.md' '@@ -1,0 +2,2 @@' '+alpha' '+beta' '@@ -9,1 +11,1 @@' '-old' '+gamma' | added_line_numbers | tr '\n' ' ')"
  [ "$parsed" = "2 3 11 " ] && arm ok "added-line numbers resolve to NEW-file positions (2 3 11)" || arm bad "hunk arithmetic yields '$parsed'"
  ok "--self-test: arms=${passed}/${arms}"
  [ "$passed" = "$arms" ] || exit 1
  exit 0
fi

if [ "$mode" = "--all" ]; then
  mkdir -p "$WORK"; total=0; unbacked=0; files=0
  for f in $(git ls-files 'docs/tasks/*.md'); do
    [ -r "$f" ] || continue
    seq 1 "$(wc -l < "$f")" > "$WORK/all.nums"
    rows="$(awk -v CLAIM_RE="$CLAIM_RE" -v CENSUS_RE="$CENSUS_RE" "$CLASSIFY_AWK" "$WORK/all.nums" "$f")"
    n_claims="$(awk -v CLAIM_RE="$CLAIM_RE" '{ if (tolower($0) ~ CLAIM_RE) c++ } END { print c+0 }' "$f")"
    n_unbacked="$(printf '%s' "$rows" | grep -c '^BLOCKED' || true)"
    total=$((total + n_claims)); unbacked=$((unbacked + n_unbacked))
    [ "$n_claims" -gt 0 ] && { files=$((files + 1)); printf '  %-52s claims=%-4s unbacked=%s\n' "$f" "$n_claims" "$n_unbacked"; }
  done
  ok "--all (ADVISORY, never blocks): ${total} claim line(s) across ${files} file(s), ${unbacked} with no census in their own section"
  exit 0
fi

mkdir -p "$WORK"
staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null | grep -E '^docs/tasks/[^/]*\.md$' || true)"
[ -n "$staged" ] || { ok "NOT EVALUATED — no staged docs/tasks file"; exit 0; }
fail=0; checked=0
for file in $staged; do
  case "$file" in docs/tasks/TEMPLATE.md) continue ;; esac
  # Judge the INDEX content, never the worktree: the commit ships what is staged.
  git show ":$file" > "$WORK/staged.md" 2>/dev/null || continue
  git diff --cached -U0 -- "$file" 2>/dev/null | added_line_numbers | sort -un > "$WORK/staged.nums"
  [ -s "$WORK/staged.nums" ] || continue
  checked=$((checked + 1))
  rows="$(awk -v CLAIM_RE="$CLAIM_RE" -v CENSUS_RE="$CENSUS_RE" "$CLASSIFY_AWK" "$WORK/staged.nums" "$WORK/staged.md")"
  [ -n "$rows" ] || continue
  [ "$fail" = 0 ] && note "a staged task leaf ADDS a \"nothing checks X\" claim with no census in its own section."
  fail=1
  printf '%s\n' "$rows" | report_rows "$file"
done
if [ "$fail" != 0 ]; then remedy; exit 1; fi
if [ "$checked" = 0 ]; then ok "NOT EVALUATED — no staged docs/tasks file carried added lines"; else ok "OK — ${checked} staged task file(s), no added claim quantifying over the tree without a census"; fi
exit 0
