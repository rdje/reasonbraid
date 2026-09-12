#!/usr/bin/env bash
# scripts/check_doctrines.sh — THE GENERAL DOCTRINE ENFORCER (driver + registry).
#
# Runs every mechanizable doctrine check as one registry, reports per-doctrine
# PASS/FAIL, and exits NONZERO on any breach. Called by .githooks/pre-commit
# (fast local gate) and by CI (the backstop a local --no-verify cannot dodge).
#
# Enforcement layering (defense in depth):
#   E1 discovery  : the doctrine docs (README, MEMORY_ARCHITECTURE, TOOLBOX, docs/decisions/).
#   E2 self-check : THIS script + each registered scripts/check_*.sh (single source of truth).
#   E3 git hook   : .githooks/pre-commit calls this (activate via `git config core.hooksPath .githooks`).
#   E4 CI         : the same script runs in CI so a bypassed local hook still fails the build.
#
# To add a PROJECT-SPECIFIC doctrine, append a check to scripts/check_doctrines.project.sh
# (the pluggable slot) — never edit this driver's universal registry.
set -uo pipefail   # deliberately NOT -e: run ALL checks, collect every result, then report.

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

# Universal registry. Each entry: "ID|what it proves|relative/path/to/check.sh"
DOCTRINES=(
  "MEMORY-ARCH|durable 4-layer memory architecture invariants (MEMORY_ARCHITECTURE.md)|scripts/check_memory_architecture.sh"
  "DOCPATH|tracked .md files carry no checkout-specific absolute paths|scripts/check_docpaths.sh"
  "TASK-TREE-OWNERSHIP|every staged code change is owned by a task-tree leaf|scripts/check_task_tree_ownership.sh"
  "README-STABILITY|README.md stays a stable landing page — a line cap AND a byte cap (README_POLICY.md)|scripts/check_readme_stability.sh"
  "WAIVER-ROUTING|a task leaf saying a gate does not apply names the leaf that owns fixing it|scripts/check_waiver_routing.sh"
  "TASK-ACCEPTANCE|a staged code change is owned by a leaf whose ticked checklist carries tool output IN each box|scripts/check_task_acceptance.sh"
  "LIVE-DOC-CURRENCY|no tracked document reports its own currency (Last updated: …) — git carries it, a hand-kept date is false the day after|scripts/check_live_doc_currency.sh"
  "LESSON-PROMOTION|a new dated lesson in DEV_NOTES.md is PROMOTED to the retrievable layer (docs/knowledge or a decisions record gaining answers:) or EXPLICITLY DECLINED in its leaf — never silently dropped|scripts/check_lesson_promotion.sh"
  "ROUTING-EVIDENCE|a task leaf that routes a finding OUT to another tree records a ROUTING EVIDENCE section — what was measured, and whether the finding reproduces outside the family it is sent to|scripts/check_routing_evidence.sh"
  "GAP-CLAIM-CENSUS|a task leaf that ADDS a nothing-checks-X claim records the census it rests on, in the same section — such a sentence quantifies over the whole tree and is false the moment one reader exists|scripts/check_gap_claims.sh"
  "HEADING-DEPTH|no tracked Markdown carries an ATX heading deeper than level 6 — seven hashes is a PARAGRAPH in GFM, so a tree that encodes depth in heading level silently stops being structure and its deepest leaves are attributed to the heading above them|scripts/check_heading_depth.sh"
  "TASK-STATUS|a task-tree leaf carries exactly ONE Status line — a later one silently supersedes an earlier one, the frontier is written from these lines, and a leaf that closed while its first line still said pending sat on the frontier for seven commits|scripts/check_task_status.sh"
  "TABLE-ARITY-RATCHET|a staged markdown file may not RAISE the number of table rows whose cell count disagrees with their header — GFM silently drops the extra cells or pads the missing ones|scripts/check_table_arity.sh"
)
# Optional derived-artifact sync check (present only when the subsystem exists).
[ -x "knowledge-map/scripts/check_knowledge_map.sh" ] && \
  DOCTRINES+=("KNOWLEDGE-MAP|the derived Knowledge Map is in sync with its sources|knowledge-map/scripts/check_knowledge_map.sh")
# The pluggable project-specific slot (starts as a no-op stub).
[ -x "scripts/check_doctrines.project.sh" ] && \
  DOCTRINES+=("PROJECT-SPECIFIC|this project's own doctrine checks|scripts/check_doctrines.project.sh")

fails=0
printf '=== doctrine enforcement (%s checks) ===\n' "${#DOCTRINES[@]}"
for entry in "${DOCTRINES[@]}"; do
  id="${entry%%|*}"; rest="${entry#*|}"; proves="${rest%%|*}"; path="${rest##*|}"
  if [ ! -x "$path" ]; then
    printf '  ?? %-22s (missing/not executable: %s)\n' "$id" "$path"; fails=$((fails+1)); continue
  fi
  if out="$("$path" 2>&1)"; then
    printf '  ✅ %-22s %s\n' "$id" "$proves"
  else
    printf '  ❌ %-22s %s\n' "$id" "$proves"
    printf '%s\n' "$out" | sed 's/^/       /'
    fails=$((fails+1))
  fi
done

if [ "$fails" -ne 0 ]; then
  printf '=== %d doctrine breach(es) — commit blocked ===\n' "$fails" >&2
  exit 1
fi
printf '=== all doctrines green ===\n'
exit 0
