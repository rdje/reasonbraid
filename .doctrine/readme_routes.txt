# ReasonBraid routed-destination registry (README_POLICY.md — Routing pressure closure).
# One row per destination named by README.md, README_POLICY.md, or the guard's
# routing hint. Format: path|class|pressure control|owner|assertions
# classes: reader_navigation | author_overflow | hot_live | generated_index |
#          append_only_history | frozen_legacy | external_service
#
# The FIFTH field is optional and machine-readable: space-separated kind=operand
# terms, evaluated every commit by ROUTE-CONTROL (census_route_controls.py).
# kinds: ceiling=<bytes> | doctrine=<ID> | growth=<shape> | index_entry=<index>
#        guard_required=<script> | build_target=<target> | identity=<revision>
#
# It sits BESIDE the human sentence and never replaces it. The sentence is for a
# reader; the terms are what a checker can refuse on. Nothing is extracted from
# the prose: a census of these twenty controls measured the obvious extractor at
# 5 wrong operands in 18 -- two task-tree names read as doctrine ids and three
# HISTORICAL byte figures read as ceilings -- with 6 of the 19 claims invisible
# to it (SIGNOFF-REPAIR.11.4.2.6.7.1). An operand a row does not declare is an
# operand nobody adjudicated.
#
# A row with no terms is adjudicated NARRATIVE: its control states something no
# machine can decide, and that is a positive classification rather than an
# omission. census_route_controls.py refuses when an expressible row drops its
# terms, so the field cannot be emptied to make the gate green.
# scripts/check_readme_stability.sh fails when a linked or emitted destination
# has no row here, when a row is malformed, or when a declared ceiling is
# exceeded. A row's path governs that path and everything under it (prefix
# closure). Raising any ceiling needs the same explicit review as the README cap.
docs/book/|reader_navigation|mdBook built by `make book` (commit workflow + CI); chapters are task-tree-leaf owned and locked to the code|repo-local|build_target=book
ROADMAP.md|reader_navigation|frozen at v0.4.1; changes only per the errata rule (factual or security corrections, Phase 0 blockers) or the v0.5.0 gate|director
KICKOFF.md|frozen_legacy|Phase 0 execution companion; content identity once the PHASE-0 tree closes (Phase 1 execution lives in `docs/tasks/PHASE-1.md`), anchored at 457d3a7 — the public-repository correction that landed three days AFTER that close, and the byte state every later commit must match|director|identity=457d3a7
CLAUDE.md|reader_navigation|one-line bootstrap pointer contract (E1); MEMORY-ARCH doctrine verifies the pointer|repo-local (scaffold)|doctrine=MEMORY-ARCH
AGENTS.md|reader_navigation|one-line bootstrap pointer contract (E1); MEMORY-ARCH doctrine verifies the pointer|repo-local (scaffold)|doctrine=MEMORY-ARCH
docs/CLAIM_VERIFICATION.md|reader_navigation|adopted portable standard; revisions only via the §17 upstream re-check plus deliberate local review (no append pressure)|director
SECURITY.md|reader_navigation|the disclosure + supported-version policy record (the `.2.2` leaf); revisions ride task-tree leaves, never append pressure|director
README_POLICY.md|hot_live|governed by itself: revisions only via deliberate local review (no auto-sync); the guard refuses when this file is absent|director|guard_required=scripts/check_readme_stability.sh
scripts/update_scaffold.sh|reader_navigation|explicit scaffold pull tool; never an overflow destination|repo-local (scaffold)
docs/tasks/|author_overflow|partitioned task collection: bounded index `docs/TASK_TREE.md` (one row per tree) + per-tree changelog and verification discipline; trees close when exhausted|repo-local|index_entry=docs/TASK_TREE.md
docs/TASK_TREE.md|author_overflow|bounded index, one row per active tree; TABLE-ARITY-RATCHET and TASK-TREE-OWNERSHIP doctrines enforce its shape|repo-local|doctrine=TABLE-ARITY-RATCHET doctrine=TASK-TREE-OWNERSHIP
LIVE_STATUS.md|hot_live|BOUNDED since its split. The control this row once declared — `overwritten rather than appended` — was REFUTED by its own 632 versions, and the file is now what that control claimed: the 14-row status snapshot it is named for sits FIRST, its entire correction log is SEALED into git history through the chain notice the file's own footer carries, and 620,448 bytes became 13,995. Threshold 55,000 bytes, DERIVED from the pinned project live window and enforced by LEDGER-RUNWAY through the shared rotation mechanism; the record boundary exists from the first entry written after the split, since the sealed log had none that could be derived. ⛔ Rotate, never raise the threshold. Owner SIGNOFF-REPAIR.11.4.2.6.6|repo-local|ceiling=55000 doctrine=LEDGER-RUNWAY growth=below_peak
DEV_NOTES.md|append_only_history|BOUNDED since its first rotation. Threshold 76,000 bytes, DERIVED from the changelog's own live window of 20.279 p90-entries expressed in this ledger's measured entry size, and enforced by LEDGER-RUNWAY through the shared rotation mechanism. The debt this row was opened to record is DISCHARGED: 430 records retired into git history at the first rotation, 12 kept, 908,850 bytes to 37,874, with ~10 commits of runway. Retrieval is git history through the chain notice the file's own footer carries. ⛔ Rotate, never raise the threshold. Owner SIGNOFF-REPAIR.11.4.2.6.3|repo-local|ceiling=76000 doctrine=LEDGER-RUNWAY growth=below_peak
CHANGELOG.md|append_only_history|query-first (git log is the access path; CHANGELOG is a digest) + 96,000-byte rotation threshold enforced by the README-STABILITY guard (the 48,495-byte baseline is recorded as governed debt); rotation = git history|repo-local|ceiling=96000 doctrine=README-STABILITY growth=below_peak
COMMIT.md|reader_navigation|the exact commit workflow (spine doctrine); changes are deliberate spine maintenance, never an overflow destination|repo-local (scaffold)
docs/adr/|author_overflow|one ADR per file + `docs/adr/INDEX.md` entry; supersede, never mutate; ADRs are accepted only by the recorded authority|repo-local|index_entry=docs/adr/INDEX.md
docs/decisions/|author_overflow|one record per file + INDEX entry (MEMORY-ARCH enforces index sync); supersede, never mutate|repo-local|index_entry=docs/decisions/INDEX.md doctrine=MEMORY-ARCH
TOOLBOX.md|reader_navigation|tools-first doctrine; the tool-registry table grows only with new diagnostic tools (deliberate)|repo-local
KNOWLEDGE_MAP.md|generated_index|regenerated and staged by the pre-commit hook; KNOWLEDGE-MAP doctrine checks sync against sources|repo-local (scaffold)|doctrine=KNOWLEDGE-MAP
knowledge-map/|generated_index|sources + checkers behind `KNOWLEDGE_MAP.md`; KNOWLEDGE-MAP doctrine checks sync|repo-local (scaffold)|doctrine=KNOWLEDGE-MAP
