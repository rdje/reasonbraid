#!/usr/bin/env bash
# HANDOFF BACKGROUND-JOB CENSUS — maintainer standing rule (2026-08-30, ported by REASONBRAID-MAINTENANCE.2.5).
#
#   "at a handoff point, that is whenever I am ready to /exit, ensure there is
#    no background job running."
#
# WHY THIS EXISTS. In the originating project a task tree was opened on a checkout that the
# resume pointer said was clean and that was in fact being REWRITTEN under it: a chain script
# from a session that no longer existed had been running for 1 h 24 m, overwriting three
# TRACKED artifacts of an already-closed leaf at a different configuration, with its log gone
# with the session. A background job outlives the session that spawned it, and its output
# paths are tracked files.
#
# ⛔ HOW IT DETECTS — PATTERN-FREE, and that is the whole point.
# The first cut matched COMMAND LINES against a list of project tokens, and the maintainer
# refuted it in one question: `caffeinate -i -t 300`, spawned by this session with its cwd
# inside the repo, was invisible to it because "caffeinate" was not on the list. A census
# built from a list of things you thought of cannot see the thing you did not.
#
# So detection is now a property, not a vocabulary — ONE `lsof` call over this uid:
#   * OPEN HANDLE under the repo (any fd, including `txt`, the executing image) -> PROJECT WORK
#   * command line naming this checkout                                          -> PROJECT WORK
#   * cwd under the repo but ZERO handles and no project token                   -> ADVISORY
# ⭐ The cwd/handle split is the discriminator that stops it crying wolf, and it is principled
# rather than an allowlist: **cwd is INHERITED**, so every shell and helper this session spawns
# has one, while a process actually doing project work holds a FILE. Measured: `caffeinate`
# reports `fd=cwd` under the repo and zero handles.
#
# ⛔ SCOPE — a HANDOFF check, deliberately NOT a doctrine and NOT wired into .githooks/pre-commit.
# A commit may legitimately land while a job runs (commit completed work FIRST, then wait for the
# job). A commit-blocking form would punish exactly that behaviour.
#
# HONEST LIMIT, stated so a green verdict is not over-read: it censuses THIS uid's processes, so a
# project job running as another user is not seen; and a process that has finished its writes and
# closed every handle, with a command line naming no repo path and a cwd outside, is invisible to
# both arms. It excludes the caller's own ancestry, and this script plus the harness wrapper are
# also excluded by name because that ancestry walk can truncate when an intermediate shell exits.
#
# Usage:  bash scripts/check_no_background_jobs.sh [--all]
#   --all  also list ADVISORY rows (inherited-cwd session infrastructure); never changes the exit code
# Exit:   0 = no project work running · 1 = project work still running · 2 = usage error

set -uo pipefail

SHOW_ALL=""
case "${1:-}" in
  -h|--help) sed -n '2,40p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
  --all) SHOW_ALL=1 ;;
  "") : ;;
  *) printf 'usage: bash scripts/check_no_background_jobs.sh [--all]\n' >&2; exit 2 ;;
esac

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" || exit 2

# The caller's own ancestry: this script, the shell that ran it, the harness above that.
# ⚠️ This walk CAN TRUNCATE if an intermediate shell has already exited and the caller is
# reparented (observed once, not reliably reproducible) — hence the by-name arm further down.
ancestry() {
  local p="${1:-$$}" guard=0
  while [ -n "$p" ] && [ "$p" -gt 1 ] 2>/dev/null && [ "$guard" -lt 64 ]; do
    printf '%s\n' "$p"
    p="$(ps -o ppid= -p "$p" 2>/dev/null | tr -d '[:space:]')"
    guard=$((guard + 1))
  done
}
EXCLUDE=" $(ancestry "$$" | tr '\n' ' ') "

# ONE lsof call, pattern-free: which pids hold a repo FILE, and which merely inherited a repo cwd.
LSOF="$(lsof -u "$(id -u)" -Fpfn 2>/dev/null | awk -v R="$REPO_ROOT/" -v RE="$REPO_ROOT" '
  /^p/ { pid = substr($0, 2); next }
  /^f/ { fd  = substr($0, 2); next }
  /^n/ { n = substr($0, 2)
         if (index(n, R) == 1 || n == RE) { if (fd == "cwd") c[pid] = 1; else h[pid]++ } }
  END  { for (p in c) printf "%s cwd %d\n", p, h[p] + 0
         for (p in h) if (!(p in c)) printf "%s hnd %d\n", p, h[p] }')"

cwd_in_repo()  { printf '%s\n' "$LSOF" | grep -q "^$1 cwd "; }
handles_of()   { printf '%s\n' "$LSOF" | awk -v p="$1" '$1==p {print $3; exit}'; }

SNAP="$(ps -Ao pid=,etime=,command= 2>/dev/null)"
BLOCKING=""; ADVISORY=""

while IFS= read -r line; do
  line="${line#"${line%%[![:space:]]*}"}"     # ps PADS its pid column; without this trim the pid
  [ -n "$line" ] || continue                  # below is EMPTY and the exclusion silently never fires
  pid="${line%% *}"
  case "$pid" in ''|*[!0-9]*) continue ;; esac
  case "$EXCLUDE" in *" $pid "*) continue ;; esac
  cmd="${line#* }"; cmd="${cmd#* }"
  case "$cmd" in
    *check_no_background_jobs.sh*|*.claude/shell-snapshots*|*/.codex/*|*/.cursor/*) continue ;;
  esac

  h="$(handles_of "$pid")"; h="${h:-0}"
  incwd=0; cwd_in_repo "$pid" && incwd=1
  names_repo=0; case "$cmd" in *"$REPO_ROOT"*) names_repo=1 ;; esac

  if [ "$h" -gt 0 ] || [ "$names_repo" = 1 ]; then
    BLOCKING="${BLOCKING}${line}"$'\n'
  elif [ "$incwd" = 1 ]; then
    ADVISORY="${ADVISORY}[inherited cwd, 0 repo handles] ${line}"$'\n'
  fi
done < <(printf '%s\n' "$SNAP")

BLOCKING="$(printf '%s' "$BLOCKING" | sed '/^$/d')"
ADVISORY="$(printf '%s' "$ADVISORY" | sed '/^$/d')"

if [ -n "$BLOCKING" ]; then
  printf 'handoff: %s project-owned process(es) STILL RUNNING — not handoff-ready:\n\n' \
    "$(printf '%s\n' "$BLOCKING" | wc -l | tr -d ' ')"
  printf '%s\n' "$BLOCKING" | sed 's/^/  /'
  printf '\nKill them AND THEIR CHILDREN — a parent'"'"'s death does not propagate (measured: killing\n'
  printf 'a driver left two of its children running) — then re-run this until it is empty.\n'
  exit 1
fi

printf 'handoff: OK — no project-owned background job is running (repo %s)\n' "$REPO_ROOT"
if [ -n "$ADVISORY" ]; then
  n="$(printf '%s\n' "$ADVISORY" | wc -l | tr -d ' ')"
  if [ -n "$SHOW_ALL" ]; then
    printf '  advisory: %s session process(es) hold an inherited cwd but no repo handle:\n' "$n"
    printf '%s\n' "$ADVISORY" | sed 's/^/    /'
  else
    printf '  (advisory: %s session process(es) with an inherited repo cwd and 0 repo handles; --all to list)\n' "$n"
  fi
fi
exit 0
