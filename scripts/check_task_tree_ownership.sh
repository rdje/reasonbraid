#!/usr/bin/env bash
# scripts/check_task_tree_ownership.sh — TASK-TREE-OWNERSHIP doctrine.
#
# Binding rule: no code change lands unless a task-tree leaf owns it. This check is
# a heuristic backstop, not a proof: when a commit stages a code change, it requires
# that the SAME commit also stages a task tree, docs/tasks/<TREE>.md (evidence the
# change was routed through a leaf). The real ownership is enforced by COMMIT.md
# discipline + review; this catches the obvious "code with no tree" slip.
#
# ⛔ THREE BLIND SPOTS, measured and closed (`SIGNOFF-REPAIR.11.40`). A commit that
# only changed a migration or a git hook, only deleted code, or only renamed it
# passed with no tree staged: the code list was a private glob naming neither
# migrations/ nor .githooks/, and `--diff-filter=ACM` dropped D and R. That glob
# was a second copy of the definition TASK-ACCEPTANCE reads, and the two disagreed
# (scripts/ was code to one gate and not to the other). Now:
#   - what counts as code is asked of TASK-ACCEPTANCE (`--code-paths`), which reads
#     the project seam .doctrine/code_paths.txt: ONE definition for both gates;
#   - every staged status counts, and a rename counts as its old AND new path;
#   - a tree is a staged, not deleted, docs/tasks/<TREE>.md, as TASK-ACCEPTANCE
#     already requires: evidence nested under docs/tasks/ is not a leaf.
# ⛔ And there is no bypass. SPINE_ALLOW_UNOWNED=1 skipped this gate and left no
# trace in the commit; nothing in this repository ever set it. A change nothing
# owns is what the rule forbids, so the answer to one is a leaf.
#
# CONTRACT: exit code is the verdict; explains on stderr; deterministic;
# read-only; staged-scope-aware; path-agnostic.
#
# Self-test: scripts/check_task_tree_ownership.sh --self-test
set -uo pipefail
SCRIPTS="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

# The staged change set: every status, a rename split into its two paths.
staged_paths() { git diff --cached --name-only --no-renames; }
# The staged trees: the change set without its deletions.
staged_trees() {
  git diff --cached --name-only --no-renames --diff-filter=d \
    | grep -E '^docs/tasks/[^/]+\.md$' | grep -vE '(^|/)TEMPLATE\.md$'
}

# Judges the index of the repository at the current directory. Prints the
# unowned code paths and returns 1 when code is staged and no tree is.
judge() {
  local code_re code
  code_re="$(bash "$SCRIPTS/check_task_acceptance.sh" --code-paths)" || return 2
  code="$(staged_paths | grep -E "$code_re")"
  [ -n "$code" ] || return 0
  [ -z "$(staged_trees)" ] || return 0
  printf '%s\n' "$code"
  return 1
}

# ── self-test ─────────────────────────────────────────────────────────────────
# Each arm is a real index in a throwaway repository, judged end to end: the
# lister, the shared definition (the built-in default, since the throwaway has no
# seam) and the verdict. One arm per blind spot, and the admitted shapes beside
# them, so a gate that refuses everything fails too.
if [ "${1:-}" = "--self-test" ]; then
  scratch="$ROOT/target/doctrine_scratch"; mkdir -p "$scratch"
  st="$(mktemp -d "$scratch/ownership-selftest.XXXXXX")"; trap 'rm -rf "$st"' EXIT
  # ⛔ Under the pre-commit hook git exports the REAL index to its children. Left
  # set, every `git` below would stage into the commit being judged.
  unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE GIT_OBJECT_DIRECTORY GIT_COMMON_DIR \
        GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_PREFIX
  g() { git -c user.name=selftest -c user.email=selftest@invalid -c core.hooksPath=/dev/null \
            -c commit.gpgsign=false "$@" >/dev/null 2>&1; }
  cd "$st" && g init -q . && mkdir -p crates/a migrations .githooks docs/tasks scripts
  echo 'fn a() {}' > crates/a/lib.rs; echo '-- m' > migrations/0001_m.sql
  echo 'exit 0' > .githooks/pre-commit; echo 'x' > scripts/tool.py; echo '# T' > docs/tasks/T.md
  g add -A && g commit -q -m base || { echo "TASK-TREE-OWNERSHIP self-test: no scratch repo" >&2; exit 1; }
  fails=0
  arm() { # expected-rc, label, then the shell that stages the change
    local want="$1" label="$2" got; shift 2
    g reset -q --hard HEAD; g clean -fdq
    eval "$*"
    judge >/dev/null; got=$?
    if [ "$got" != "$want" ]; then
      echo "TASK-TREE-OWNERSHIP self-test: $label — rc $got, expected $want" >&2; fails=$((fails+1))
    fi
  }
  arm 1 "a migration alone is code"        "echo '-- n' >> migrations/0001_m.sql; g add -A"
  arm 1 "a git hook alone is code"         "echo '# h' >> .githooks/pre-commit; g add -A"
  arm 1 "a script alone is code"           "echo 'y' >> scripts/tool.py; g add -A"
  arm 1 "a deletion is code"               "g rm -q crates/a/lib.rs"
  arm 1 "a rename out of code is code"     "g mv crates/a/lib.rs docs/a.md"
  arm 1 "an artifact is not a tree"        "echo 'z' >> crates/a/lib.rs; mkdir -p docs/tasks/artifacts; echo l > docs/tasks/artifacts/l.log; g add -A"
  arm 1 "a deleted tree is not an owner"   "echo 'z' >> crates/a/lib.rs; g rm -q docs/tasks/T.md; g add -A"
  arm 1 "an unstaged seam cannot narrow"   "mkdir -p .doctrine; echo '^nothing$' > .doctrine/code_paths.txt; echo '-- n' >> migrations/0001_m.sql; g add migrations"
  arm 0 "code with its tree"               "echo 'z' >> crates/a/lib.rs; echo '- leaf' >> docs/tasks/T.md; g add -A"
  arm 0 "a deletion with its tree"         "g rm -q crates/a/lib.rs; echo '- leaf' >> docs/tasks/T.md; g add -A"
  arm 0 "docs alone"                       "mkdir -p docs/book; echo p > docs/book/p.md; g add -A"
  arm 0 "nothing staged"                   ":"
  # TASK-ACCEPTANCE lists the change set the same way, so with no leaf staged it
  # must refuse the two listing blind spots, and a tree the commit deletes.
  for shape in "g rm -q crates/a/lib.rs" "g mv crates/a/lib.rs docs/a.md" \
               "echo z >> crates/a/lib.rs; g add -A; g rm -q docs/tasks/T.md"; do
    g reset -q --hard HEAD; g clean -fdq; eval "$shape"
    if bash "$SCRIPTS/check_task_acceptance.sh" >/dev/null 2>&1; then
      echo "TASK-TREE-OWNERSHIP self-test: TASK-ACCEPTANCE admitted \`$shape\` with no leaf" >&2
      fails=$((fails+1))
    fi
  done
  cd "$ROOT" || exit 1
  # And THIS repository's seam must hold the same line as the built-in default.
  seam_re="$(bash "$SCRIPTS/check_task_acceptance.sh" --code-paths)"
  for p in migrations/0001_m.sql .githooks/pre-commit scripts/tool.py crates/a/src/lib.rs Cargo.lock; do
    printf '%s\n' "$p" | grep -qE "$seam_re" \
      || { echo "TASK-TREE-OWNERSHIP self-test: this repository's seam misses $p" >&2; fails=$((fails+1)); }
  done
  [ "$fails" = 0 ] || exit 1
  echo "TASK-TREE-OWNERSHIP self-test: OK (8 refused, 4 admitted; TASK-ACCEPTANCE refuses 3 unowned shapes; the seam covers 5 code kinds)"
  exit 0
fi

unowned="$(judge)"; rc=$?
if [ "$rc" = 2 ]; then
  echo "TASK-TREE-OWNERSHIP: could not read the code definition (check_task_acceptance.sh --code-paths)." >&2
  exit 1
fi
if [ "$rc" = 1 ]; then
  echo "TASK-TREE-OWNERSHIP: code is staged but no task tree (docs/tasks/<TREE>.md) is:" >&2
  printf '%s\n' "$unowned" | sed 's/^/    /' >&2
  echo "  Stage the task-tree leaf that owns this change (docs/TASK_TREE.md)." >&2
  exit 1
fi
exit 0
