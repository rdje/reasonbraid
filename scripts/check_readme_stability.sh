#!/usr/bin/env bash
# scripts/check_readme_stability.sh — README-STABILITY (README_POLICY.md, ReasonBraid revision).
#
# Keeps README.md a stable LANDING PAGE instead of letting it grow into a changelog,
# roadmap, catalogue or documentation inventory. Structural, deterministic, NON-MUTATING,
# no network. Exits nonzero on any breach, with a routing hint naming the canonical home.
# Runs UNCONDITIONALLY (no changed-path short-circuit): landing-page size and the health of
# its routed destinations are properties of the resulting tree, not of the staged diff.
#
# ⭐ WHY BOTH A LINE CAP AND A BYTE CAP — this is the one design decision worth defending,
# because "add a line cap" is the obvious shape and it is NOT sufficient. Measured on a real
# project running this spine: its layer-A MEMORY.md sat at 60 lines — PASSING, exactly at its
# line cap — while carrying 138,403 BYTES. That is 2,306 bytes per line, with a single line of
# 18,816 bytes. A file the standard calls a "bounded resume pointer" was a 138 KB document and
# its guard was green the whole time. The same class appeared independently in that project's
# README, where ONE bullet measured 4,369 bytes.
#   ⇒ Line and byte checks are COMPLEMENTS, not redundancy: neither wrapped prose nor very
#     long lines can bypass the budget.
#
# ⛔ NEVER raise a cap to land new content. Move the detail to its canonical home (below).
#   A cap increase requires an explicit reviewed decision recorded in your task-tree that the
#   landing-page contract itself expanded (README_POLICY.md, Mechanical growth guard).
#
# ReasonBraid revision (PHASE-0-MAINT-1, 2026-09-06): the caps below are DERIVED from the
# reviewed landing page (47 lines / 1,772 bytes) plus explicit bounded headroom — not template
# example values. The routing-pressure-closure leg inventories every destination the README,
# the policy, and this guard's own hint name (`.doctrine/readme_routes.txt`) and enforces the
# CHANGELOG append-only-history rotation threshold. Decision record:
# docs/decisions/2026-09-06_readme-policy-readoption.md.
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

# Derived caps — reviewed 2026-09-06 (README.md: 47 lines / 1,772 bytes) + explicit bounded
# headroom. Env overrides exist for FALSIFICATION runs only, never as a growth path.
LINE_CAP="${README_LINE_CAP:-60}"
BYTE_CAP="${README_BYTE_CAP:-2400}"
# CHANGELOG rotation threshold (append-only history): the measured 48,495-byte baseline is
# recorded as governed debt; rotation = git history. Not a growth authorization.
CHANGELOG_BYTE_CAP="${README_CHANGELOG_BYTE_CAP:-96000}"
TARGET="README.md"
POLICY="README_POLICY.md"
INVENTORY=".doctrine/readme_routes.txt"
CHANGELOG="CHANGELOG.md"

fail=0
note(){ printf 'README-STABILITY: %s\n' "$1" >&2; fail=1; }

# ── pure verdicts (stdin-driven extraction; the --self-test arm proves them) ────────────
extract_routes() { # raw markdown-ish text on stdin → candidate path tokens, filtered
  # ⛔ LC_ALL=C PINS THE COLLATION, and it is the instrument that is pinned
  # rather than the comparison that is loosened (`SIGNOFF-REPAIR.11.27`).
  # `sort` orders by the ambient locale: `en_US.UTF-8` returns `docs/book/`
  # first and `LC_ALL=C` returns it third, so this function's output depended
  # on the host. The gate's own verdict never did — its callers iterate the
  # tokens and count unrouted ones — but the SELF-TEST compares the output as a
  # string, so it passed here and failed on the runner, which is a control
  # pinning behaviour-on-this-host as ground truth. Loosening the comparison
  # would have hidden that and left the trap for the next arm that compares
  # this output. `tr`'s ranges are pinned by the same export.
  LC_ALL=C tr -c 'A-Za-z0-9_./-' '\n' \
    | LC_ALL=C sed 's#^\./##' \
    | LC_ALL=C grep -E '(\.md|\.sh|\.txt)$|/' \
    | LC_ALL=C grep -v -E '^https?://|^/' \
    | LC_ALL=C sort -u
}
closure_verdict() { # $1 inventory present (0/1); $2 unrouted count; $3 malformed count
  if [ "$1" -eq 0 ]; then printf 'needs-routing'; return 0; fi
  if [ "$2" -eq 0 ] && [ "$3" -eq 0 ]; then printf 'ok'; else printf 'needs-routing'; fi
}
cap_verdict() { # lines, bytes, line cap, byte cap — inclusive ceilings
  if [ "$1" -gt "$3" ] || [ "$2" -gt "$4" ]; then printf 'breach'; else printf 'ok'; fi
}

# ── self-test arm: prove the verdicts BEFORE they judge the tree ────────────────────────
if [ "${1:-}" = "--self-test" ]; then
  # extraction control: path-shaped tokens survive; commands, URLs, plain words, and
  # placeholder spans (`<reasonbraid-url>`) do not.
  got="$(printf 'see `docs/book/` and `ROADMAP.md` and `make check` and [p](README_POLICY.md) and `https://x/y` and `./scripts/update_scaffold.sh <reasonbraid-url>`\n' | extract_routes)"
  # The C-collation order, which is now the only order this function produces.
  want="$(printf 'README_POLICY.md\nROADMAP.md\ndocs/book/\nscripts/update_scaffold.sh')"
  if [ "$got" != "$want" ]; then
    printf 'self-test MISSED: extraction\n  got:  %s\n  want: %s\n' "$got" "$want" >&2
    exit 1
  fi
  # closure verdict: missing inventory is a fail; any unrouted/malformed row is a fail.
  for spec in "1:0:0:ok" "0:0:0:needs-routing" "1:1:0:needs-routing" "1:0:1:needs-routing" "0:1:1:needs-routing"; do
    want="${spec##*:}"; rest="${spec%:*}"
    got="$(closure_verdict "$(printf '%s' "$rest" | cut -d: -f1)" "$(printf '%s' "$rest" | cut -d: -f2)" "$(printf '%s' "$rest" | cut -d: -f3)")"
    if [ "$got" != "$want" ]; then
      printf 'self-test MISSED: closure %s expected=%s got=%s\n' "$spec" "$want" "$got" >&2
      exit 1
    fi
  done
  # cap verdict: inclusive ceilings — at-cap passes, one over fails, either dimension alone fails.
  for spec in "47:1772:60:2400:ok" "60:2400:60:2400:ok" "61:1772:60:2400:breach" "47:2401:60:2400:breach"; do
    want="${spec##*:}"; rest="${spec%:*}"
    got="$(cap_verdict "$(printf '%s' "$rest" | cut -d: -f1)" "$(printf '%s' "$rest" | cut -d: -f2)" "$(printf '%s' "$rest" | cut -d: -f3)" "$(printf '%s' "$rest" | cut -d: -f4)")"
    if [ "$got" != "$want" ]; then
      printf 'self-test MISSED: caps %s expected=%s got=%s\n' "$spec" "$want" "$got" >&2
      exit 1
    fi
  done
  printf 'README-STABILITY: self-test ok (extraction + closure + caps ground truth)\n'
  exit 0
fi

routing_hint() {
  cat >&2 <<'HINT'
                 Route the new detail to its canonical home instead of growing the landing page:
                   user-facing feature detail ....... the user guide / `docs/book/`
                   current work and priorities ...... `docs/tasks/`, `docs/TASK_TREE.md`, `ROADMAP.md`
                   release history .................. `CHANGELOG.md` (git history is the query path)
                   design rationale ................. `docs/decisions/`
                   exhaustive inventories ........... a generated index such as `KNOWLEDGE_MAP.md`
                   diagnostics and procedure ........ `TOOLBOX.md`, contributor docs
                 Full policy: `README_POLICY.md`
HINT
}

# ------------------------------------------------------------------ refuse rather than skip
# A skip is never a pass: with the landing page, its policy, or the route inventory absent
# this check cannot judge anything, and returning 0 would report "the doctrine holds" over an
# absence.
for required in "$TARGET" "$POLICY" "$INVENTORY" "$CHANGELOG"; do
  if [ ! -f "$required" ]; then
    printf 'README-STABILITY: REFUSED — %s is missing; the growth guard cannot judge the tree.\n' \
      "$required" >&2
    exit 2
  fi
done

lines=$(wc -l < "$TARGET" | tr -d ' ')
bytes=$(wc -c < "$TARGET" | tr -d ' ')

# ------------------------------------------------------------------ the line cap
if [ "$(cap_verdict "$lines" "$bytes" "$LINE_CAP" "$BYTE_CAP")" = "breach" ]; then
  [ "$lines" -gt "$LINE_CAP" ] && note "$TARGET is $lines lines (> cap $LINE_CAP)."
  [ "$bytes" -gt "$BYTE_CAP" ] && note "$TARGET is $bytes bytes (> cap $BYTE_CAP)."
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

# ------------------------------------------------------------------ routing pressure closure
# Every destination the README links or names, every destination this guard's own hint emits,
# and every path named inside a declared control must end at a governed terminal (a registry
# row whose path is a prefix of it). An unclassified destination, a malformed row, or a
# missing registry fails the adoption — overflow pressure may not be moved, only governed.
governed() { # $1 = token; success iff some row's path is a prefix of it
  local token="$1" found=0 p
  while IFS='|' read -r p _; do
    [ -n "$p" ] || continue
    case "$token" in
      "$p"*) found=1; break ;;
    esac
  done < "$INVENTORY"
  [ "$found" -eq 1 ]
}

routes_from_readme() {
  extract_routes < "$TARGET"
}
routes_from_hint() {
  sed -n '/^routing_hint()/,/^HINT$/p' "$0" | sed '1d;$d' | extract_routes
}
routes_from_controls() { # transitive leg: paths named inside declared controls
  grep -v '^#' "$INVENTORY" | cut -d'|' -f3 | extract_routes
}

unrouted=0
for token in $(routes_from_readme; routes_from_hint; routes_from_controls); do
  if ! governed "$token"; then
    note "unrouted destination: $token — add a governed row to $INVENTORY (class + pressure control) or remove the link."
    unrouted=$((unrouted + 1))
  fi
done

malformed=0
while IFS='|' read -r path class control owner; do
  case "$path" in ''|'#'*) continue ;; esac
  if [ -z "$class" ] || [ -z "$control" ] || [ -z "$owner" ]; then
    note "malformed route row in $INVENTORY: '$path' — path|class|control|owner are all required."
    malformed=$((malformed + 1))
  fi
done < "$INVENTORY"

if [ "$(closure_verdict 1 "$unrouted" "$malformed")" != "ok" ]; then
  note "routing pressure closure failed ($unrouted unrouted, $malformed malformed) — see README_POLICY.md 'Routing pressure closure'."
fi

# ------------------------------------------------------------------ append-only history threshold
# CHANGELOG.md is the release-history terminal: query-first (git log is the access path), with
# the rotation threshold enforced here. Rotation is git history; raising the threshold needs
# the same explicit review as the README cap.
cb=$(wc -c < "$CHANGELOG" | tr -d ' ')
if [ "$cb" -gt "$CHANGELOG_BYTE_CAP" ]; then
  note "$CHANGELOG is $cb bytes (> rotation threshold $CHANGELOG_BYTE_CAP) — rotate the oldest entries into git history, never raise the threshold without review."
fi

[ "$fail" -eq 0 ] || exit 1
printf 'README-STABILITY: OK — %s is %s/%s lines, %s/%s bytes; %s routes governed (%s bytes, threshold %s); closure ok.\n' \
  "$TARGET" "$lines" "$LINE_CAP" "$bytes" "$BYTE_CAP" \
  "$(grep -vcE '^($|#)' "$INVENTORY")" "$cb" "$CHANGELOG_BYTE_CAP"
exit 0
