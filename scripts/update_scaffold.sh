#!/usr/bin/env bash
# scripts/update_scaffold.sh — pull the latest project-NEUTRAL spine from the ReasonBraid
# template into THIS project, WITHOUT touching your roadmap, task-trees, decisions, or code.
#
#   scripts/update_scaffold.sh <reasonbraid-repo-url-or-local-path>
#
# Only the files listed in NEUTRAL below are re-synced. Everything project-owned
# (CLAUDE.md, README.md, ROADMAP.md, the live-docs, the project doctrine slot, the curated
# subsystems.md, and all of docs/tasks/ + docs/decisions/ records) is deliberately left
# alone. After syncing: review `git diff`, run `make gate`, and commit.
set -euo pipefail
URL="${1:-}"
[ -n "$URL" ] || { echo "usage: scripts/update_scaffold.sh <reasonbraid-repo-url-or-local-path>" >&2; exit 2; }
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"

# ⛔ §13: scratch is REPOSITORY-derived, never ambient (SIGNOFF-REPAIR.11.2.2).
# This one CLONES into its scratch, so an ambient TMPDIR put a whole repository
# on another volume — the largest of the five sites by bytes written.
scratch="$ROOT/target/doctrine_scratch"; mkdir -p "$scratch"
tmp="$(mktemp -d "$scratch/scaffold.XXXXXX")"; trap 'rm -rf "$tmp"' EXIT
if [ -d "$URL/.git" ]; then
  cp -R "$URL" "$tmp/reasonbraid"
else
  git clone --depth 1 "$URL" "$tmp/reasonbraid" >/dev/null 2>&1 || { echo "clone failed: $URL" >&2; exit 1; }
fi

# The project-NEUTRAL spine — safe to overwrite because it never carries project content.
NEUTRAL=(
  MEMORY_ARCHITECTURE.md
  DOCTRINE_ENFORCEMENT.md
  TOOLBOX.md
  README_POLICY.md
  COMMIT.md
  AGENTS.md
  docs/TASK_TREE.md
  docs/TASK_TREE_README.md
  docs/tasks/TEMPLATE.md
  docs/decisions/TEMPLATE.md
  .githooks/pre-commit
  .githooks/commit-msg
  scripts/check_doctrines.sh
  scripts/check_memory_architecture.sh
  scripts/check_live_doc_currency.sh
  scripts/check_no_background_jobs.sh
  scripts/check_lesson_promotion.sh
  scripts/check_routing_evidence.sh
  scripts/check_gap_claims.sh
  scripts/check_table_arity.sh
  scripts/check_readme_stability.sh
  scripts/check_waiver_routing.sh
  scripts/check_task_acceptance.sh
  .doctrine/README.md
  scripts/check_docpaths.sh
  scripts/check_task_tree_ownership.sh
  knowledge-map/scripts/gen_knowledge_map.sh
  knowledge-map/scripts/check_knowledge_map.sh
  DOCTRINE_VERSION
)

n=0
for f in "${NEUTRAL[@]}"; do
  if [ -f "$tmp/reasonbraid/$f" ]; then
    mkdir -p "$(dirname "$f")"
    cp "$tmp/reasonbraid/$f" "$f"
    echo "  synced $f"
    n=$((n+1))
  fi
done
chmod +x scripts/*.sh knowledge-map/scripts/*.sh .githooks/pre-commit .githooks/commit-msg 2>/dev/null || true

echo "✓ $n scaffold file(s) synced to $(cat DOCTRINE_VERSION 2>/dev/null || echo '?')."
echo "  Review 'git diff', run 'make gate', then commit."
