#!/usr/bin/env bash
# scripts/bootstrap.sh — first-time setup for a project scaffolded from bedrock.
#
#   scripts/bootstrap.sh [project-name]
#
# With a project-name it DE-TEMPLATES this copy into a fresh project: removes bedrock's own
# maintainer files (MAINTAINING.md + the BEDROCK-MAINTENANCE tree + the provenance record)
# and resets the layer-A/C seeds, then installs hooks, sets the name, generates the Knowledge
# Map, and verifies the enforcer. Without a name it just installs hooks + regenerates the map
# (safe to run in the bedrock source repo itself — it will NOT remove the maintainer files).
#
# Idempotent. It does NOT invent a task-tree from your roadmap — that judgment is left to you.
set -euo pipefail
# ⛔ Act on the repository THIS SCRIPT LIVES IN, never on the caller's working directory
# (BEDROCK-MAINTENANCE.2.7): `git rev-parse --show-toplevel` from the caller's cwd would fail
# outside a repository and — worse — de-template the PARENT repository when a user runs
# `<name>/scripts/bootstrap.sh <name>` from the directory they cloned into.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"
[ -d .git ] || { echo "bootstrap: $ROOT is not the root of a git clone" >&2; exit 2; }
name="${1:-}"

# 0) de-template (only when a project name is given AND this is still a pristine bedrock copy)
if [ -n "$name" ] && [ -f MAINTAINING.md ]; then
  echo "→ de-templating this bedrock copy into project '$name'…"
  rm -f MAINTAINING.md docs/tasks/BEDROCK-MAINTENANCE.md docs/decisions/reference_bedrock_provenance.md

  cat > MEMORY.md <<SEED
# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see \`MEMORY_ARCHITECTURE.md\`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read \`README.md\`, \`MEMORY_ARCHITECTURE.md\`, \`TOOLBOX.md\`, \`DOCTRINE_ENFORCEMENT.md\`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.

## Current state

- **Project:** $name — fresh from the bedrock template.
- **Active tree:** _none yet_
- **Next action:** replace \`ROADMAP.md\`; create your first task-tree
  (\`cp docs/tasks/TEMPLATE.md docs/tasks/<TREE-ID>.md\`), register it in \`docs/TASK_TREE.md\`.
- **Latest commit:** _none yet_
- **In-flight uncommitted work:** none.
SEED

  cat > docs/decisions/INDEX.md <<'SEED'
# Decision & Fact Records — Index (memory layer C)

Durable, cross-cutting facts and decisions live here, one record per file (ADR-style). Every
record must be listed below (the MEMORY-ARCH doctrine check enforces it). New record: copy
`TEMPLATE.md` → `<type>_<short-kebab-slug>.md`, fill it in, and add its row.

| Record | Type | One-line hook |
| --- | --- | --- |
| _none yet_ | | |
SEED

  # reset the Active Task Trees section (the workflow doc above it is preserved)
  sed -i '/^## Active Task Trees/,$d' docs/TASK_TREE.md
  cat >> docs/TASK_TREE.md <<'SEED'
## Active Task Trees

| Tree | Status | Frontier (next leaf) | Owner |
| --- | --- | --- | --- |
| _none yet — seed your first tree from `ROADMAP.md`_ | | | |
SEED

  # strip the maintainer-only notes (bounded by BEDROCK-MAINTAINER-NOTE markers)
  for f in CLAUDE.md ROADMAP.md; do
    [ -f "$f" ] && sed -i '/BEDROCK-MAINTAINER-NOTE:START/,/BEDROCK-MAINTAINER-NOTE:END/d' "$f"
  done
  echo "✓ de-templated (maintainer files removed; layer-A/C + tree index reset)"
fi

# 1) activate the git hooks (E3 enforcement)
git config core.hooksPath .githooks
echo "✓ git hooks activated (core.hooksPath=.githooks)"

# 2) make the spine scripts executable
chmod +x scripts/*.sh knowledge-map/scripts/*.sh .githooks/pre-commit .githooks/commit-msg 2>/dev/null || true
echo "✓ scripts marked executable"

# 3) set the project name (crate + roadmap title)
crate_before="$(grep -c '^name = "app"' crates/app/Cargo.toml 2>/dev/null || true)"; crate_before="${crate_before:-0}"
if [ -n "$name" ]; then
  [ -f crates/app/Cargo.toml ] && sed -i "s/^name = \"app\"/name = \"$name\"/" crates/app/Cargo.toml || true
  [ -f ROADMAP.md ] && sed -i "s/# ROADMAP — _(PROJECT NAME)_/# ROADMAP — $name/" ROADMAP.md || true
  echo "✓ project name set to '$name'"
fi

# 3b) seed the leaf that OWNS the bootstrap itself (only on a fresh de-template).
#     ⛔ WHY (BEDROCK-MAINTENANCE.2.7, measured on a trial clone): the crate rename above is a CODE
#     change, so the user's very first commit — the one that records this bootstrap — was refused by
#     TASK-TREE-OWNERSHIP and TASK-ACCEPTANCE with no owning leaf. The discipline is right; the
#     template must therefore ship the leaf, carrying the evidence this run just produced.
#     The enforcer lines are placeholders here and are filled in by step 5, after the Knowledge Map
#     exists — running the enforcer before the map is regenerated fails on KNOWLEDGE-MAP (measured).
if [ -n "$name" ] && [ ! -f docs/tasks/BOOTSTRAP.md ] && [ -f docs/tasks/TEMPLATE.md ]; then
  gate_head="__GATE_HEAD__"; gate_tail="__GATE_TAIL__"; gate_rc="__GATE_RC__"   # filled by step 5
  crate_after="$(grep -c '^name = "app"' crates/app/Cargo.toml 2>/dev/null || true)"; crate_after="${crate_after:-0}"
  upper="$(printf '%s' "$name" | tr '[:lower:]-' '[:upper:]_' | tr -cd 'A-Z0-9_')"
  today="$(date +%F)"
  cat > docs/tasks/BOOTSTRAP.md <<LEAF
# BOOTSTRAP: this project's bootstrap from the bedrock template

## Metadata

- Tree ID: \`BOOTSTRAP\`
- Status: \`done\`
- Roadmap lane: project setup
- Created: \`$today\`
- Owner: repo-local workflow

## Goal

Record the one-time de-templating of this copy of bedrock ($(cat DOCTRINE_VERSION 2>/dev/null || echo 'bedrock-scaffold')) into
project \`$name\`, performed by \`scripts/bootstrap.sh $name\`, with the evidence that run produced —
so the first commit of this project passes the same gates every later commit will.

## Non-Goals

- This tree does not describe the project's roadmap. Seed that tree from \`ROADMAP.md\`.

## Task Tree

- ID: \`BOOTSTRAP.1\`
  Status: \`done\`
  Goal: de-template, rename the crate, install the hooks, regenerate the Knowledge Map, verify the enforcer.

  ### Acceptance Checklist (enforced by \`TASK-ACCEPTANCE\`)

  - [x] **ROOT CAUSE (WHY + WHERE)** — a copy of bedrock carries the template's crate name and
    maintainer files: \`grep -c '^name = "app"' crates/app/Cargo.toml\` → $crate_before before the run,
    $crate_after after (\`rc=0\`); \`MAINTAINING.md\` and the maintainer tree are removed by the de-template step.
  - [x] **ADDRESSED (verified)** — crate renamed to \`$name\`; hooks installed: \`git config core.hooksPath\`
    → \`$(git config core.hooksPath)\` (\`rc=0\`); the Knowledge Map regenerated; the enforcer run inside this
    bootstrap: \`$gate_head\` … \`$gate_tail\` (\`rc=$gate_rc\`).
  - [x] **NO REGRESSION** — the same enforcer is the pre-commit hook: \`scripts/check_doctrines.sh\` →
    \`$gate_tail\`, \`rc=$gate_rc\`; \`make gate\` is that command.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | \`BOOTSTRAP.1\` | \`done\` | the bootstrap itself; nothing further belongs here |

## Commit Log

- \`$today\` — \`BOOTSTRAP.1\` — \`${upper}-BOOTSTRAP-0001\`: bootstrapped from bedrock.
LEAF
  # register it: the placeholder row becomes the BOOTSTRAP row; the seeding hint stays as a note
  sed -i "s/^| _none yet — seed your first tree from \`ROADMAP.md\`_ | | | |\$/| [\`BOOTSTRAP\`](tasks\/BOOTSTRAP.md) | \`done\` | \`.1\` — bootstrapped from bedrock; seed your first real tree from \`ROADMAP.md\` | repo-local |/" docs/TASK_TREE.md
  sed -i "s/^- \*\*Active tree:\*\* _none yet_\$/- **Active tree:** \`BOOTSTRAP\` (done) — seed your first real tree from \`ROADMAP.md\`/" MEMORY.md
  sed -i "s/^- \*\*Latest commit:\*\* _none yet_\$/- **Latest commit:** _none yet — commit the bootstrap first (bootstrap.sh printed the command)_/" MEMORY.md
  echo "✓ docs/tasks/BOOTSTRAP.md seeded with this run's evidence (owns the crate rename for the first commit)"
fi

# 4) generate the derived Knowledge Map (after de-templating, so it reflects the reset)
if [ -x knowledge-map/scripts/gen_knowledge_map.sh ]; then
  knowledge-map/scripts/gen_knowledge_map.sh > "$(knowledge-map/scripts/gen_knowledge_map.sh --print-map-path)"
  echo "✓ KNOWLEDGE_MAP.md generated"
fi

# 5) sanity: run the enforcer — and hand its verdict to the bootstrap leaf as evidence
echo "→ running the doctrine enforcer…"
gate_out="$(scripts/check_doctrines.sh 2>&1)" && gate_rc=0 || gate_rc=$?
printf '%s\n' "$gate_out"
if [ -f docs/tasks/BOOTSTRAP.md ] && grep -q '__GATE_HEAD__' docs/tasks/BOOTSTRAP.md; then
  gate_head="$(printf '%s\n' "$gate_out" | grep -E '^=== doctrine enforcement' | head -1 || true)"
  gate_tail="$(printf '%s\n' "$gate_out" | grep -E '^=== all doctrines green ===|FAILED' | tail -1 || true)"
  sed -i "s|__GATE_HEAD__|${gate_head:-(no summary line)}|g; s|__GATE_TAIL__|${gate_tail:-(no verdict line)}|g; s|__GATE_RC__|$gate_rc|g" docs/tasks/BOOTSTRAP.md
fi
[ "$gate_rc" = 0 ] || { echo "enforcer reported a breach — fix it before your first commit"; exit 1; }

cat <<'EOF'

bedrock is ready.

Next:
  0) Commit the bootstrap itself (its leaf docs/tasks/BOOTSTRAP.md carries the evidence):
       git add -A
       printf '%s\n' '<NAME>-BOOTSTRAP-0001 (leaf BOOTSTRAP.1): bootstrapped from bedrock' > git_message_brief.txt
       git commit -F git_message_brief.txt && : > git_message_brief.txt
     (the hooks run the enforcer; <NAME> = your project name in CAPITALS)
  1) Replace ROADMAP.md with your project's real roadmap.
  2) Create your first task-tree:
       cp docs/tasks/TEMPLATE.md docs/tasks/<TREE-ID>.md    # then fill it in
       (register it in docs/TASK_TREE.md's Active Task Trees table)
  3) Work its first leaf, then commit via COMMIT.md.

Re-run the spine check anytime with:  make gate
EOF
