# ReasonBraid routed-destination registry (README_POLICY.md — Routing pressure closure).
# One row per destination named by README.md, README_POLICY.md, or the guard's
# routing hint. Format: path|class|pressure control|owner|assertions
# classes: reader_navigation | author_overflow | hot_live | generated_index |
#          append_only_history | frozen_legacy | external_service
#
# A trailing `|` continues the vocabulary onto the next comment line; the list
# ends at the first line without one. ROUTE-CONTROL derives the vocabulary from
# these lines and REFUSES a row whose class is not in it -- until
# SIGNOFF-REPAIR.11.4.2.8 the field was validated for non-emptiness alone, so a
# typo was a silent reclassification.
#
# ⚠️ `external_service` has NO INSTANCE and is measured, not assumed: 34 rows
# over 6 of these 7 words (.11.4.2.8). It is kept as a reserved word rather than
# deleted, and named here as untested rather than established, so the next author
# knows no destination has ever exercised it.
#
# ⛔ THE CLASS IS A READER'S TAXONOMY AND DOES NOT BIND A MECHANISM, and that is
# a measurement rather than a design statement: of the six classes in use, only
# the three with two rows or fewer share an assertion kind across their rows, and
# the three largest -- reader_navigation (15), author_overflow (9), hot_live (5)
# -- share none. What a destination's growth is actually held by is the FIFTH
# field, which ROUTE-CONTROL evaluates. Re-derive, never read from here:
# `python3 -B scripts/census_route_controls.py --classes`.
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
MEMORY_ARCHITECTURE.md|reader_navigation|the durable 4-layer memory architecture (spine doctrine): it GOVERNS the layer-A resume pointer — the contract, the one-question rule, the line and byte caps and the eviction remedy — and the MEMORY-ARCH doctrine enforces what it defines. Changes are deliberate spine maintenance, never an overflow destination. It had no row until SIGNOFF-REPAIR.11.4.2.7.2, because the routing closure is anchored at the landing page's link graph and the landing page does not link it|repo-local (scaffold)|doctrine=MEMORY-ARCH
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
# ── Rows reached by the SECOND ANCHOR (SIGNOFF-REPAIR.11.4.2.7.3.1) ────────────
# Every tracked Markdown document must end at a governed row. The three original
# legs are all anchored at what README.md reaches, so these destinations had no
# row for the life of the project -- not refused, not listed, never asked for a
# lifecycle. README.md itself is EXCLUDED by the guard: it is the guard's subject
# and its ceiling is LINE_CAP/BYTE_CAP, so a row would be a second authority for
# one cap. The 89 undeclared documents measured at .11.4.2.7.3 are governed by
# the 13 rows below, because the registry resolves by prefix.
MEMORY.md|hot_live|the layer-A resume pointer: it answers one question -- what is next -- and nothing else, overwritten and never appended. `MEMORY_ARCHITECTURE.md` §6 defines that contract and the MEMORY-ARCH doctrine enforces the line and byte caps; growth is a signal that something belongs in another layer, never that the cap is tight. Owner SIGNOFF-REPAIR.11.4.2.7.1|repo-local (scaffold)|doctrine=MEMORY-ARCH
DOCTRINE_ENFORCEMENT.md|reader_navigation|the four-layer enforcement map and the task-acceptance checklist (spine doctrine); every registered check's exit-code contract is written against it. Changes are deliberate spine maintenance, never an overflow destination. ⛔ NO SIZE CONTROL IS DECLARED BECAUSE NONE EXISTS: .11.4.2.7.3.1 measured 200,000 appended bytes accepted by a green enforcer here AND on `TOOLBOX.md`, `COMMIT.md` and `MEMORY_ARCHITECTURE.md`, so the class is uniformly unbounded and the question is owned by SIGNOFF-REPAIR.11.4.2.7.3.2 rather than answered by an invented ceiling|repo-local (scaffold)
.doctrine/|reader_navigation|project-declared seams that adapt the neutral spine checks without editing them; each file is read by the check that declares it and `.doctrine/README.md` documents the set. Growth is one file per new seam, never prose|repo-local (scaffold)
docs/knowledge/|author_overflow|the retrievable lesson layer and a MANDATED routing destination: LESSON-PROMOTION requires a new dated `DEV_NOTES.md` lesson to land here or be explicitly declined, and the KNOWLEDGE-MAP doctrine derives `KNOWLEDGE_MAP.md` from these sources and checks it in sync. One note per file, superseded rather than mutated|repo-local|doctrine=LESSON-PROMOTION doctrine=KNOWLEDGE-MAP index_entry=KNOWLEDGE_MAP.md
docs/runbooks/|author_overflow|one incident runbook per file, each an operator procedure for a named incident class; the collection grows only when a new class is identified, never as an overflow destination for prose|repo-local
docs/evidence/|author_overflow|one evidence bundle per file plus a `docs/evidence/INDEX.md` entry; a bundle records what a run produced at a moment and is never rewritten, so records are ADDED and never mutated -- the same shape as the ADR and decision collections. ⛔ It was classed append_only_history when it was added at SIGNOFF-REPAIR.11.4.2.7.3.1 and that conflated an append-only COLLECTION, whose bound is an index, with an append-only FILE, whose bound is a rotation threshold; .11.4.2.8 measured the difference and corrected it|repo-local|index_entry=docs/evidence/INDEX.md
spec/|author_overflow|the specification set, indexed by `spec/README.md`; specifications mature beside working code (ROADMAP §1) and each document is revised deliberately under its own task-tree leaf|repo-local|index_entry=spec/README.md
docs/TASK_TREE_README.md|reader_navigation|the task-tree setup guide `docs/TASK_TREE.md` names; it defines the tree file shape and the relative-reference convention, and changes ride task-tree leaves rather than append pressure|repo-local
docs/ci.md|reader_navigation|the CI and supply-chain surface documentation (`PHASE-0.0.7`): it describes the workflows that exist and draws the line between a skeleton and a release claim. It documents mechanisms, never an overflow destination|repo-local
docs/compatibility-matrix.md|hot_live|the SDK compatibility matrix, evidence-bound by its `.4.1` schema -- every cell comes from a run, never from a sibling -- and mechanically re-derived against the contract token and the cited artifacts by the compatibility-matrix guard, which the PROJECT-SPECIFIC doctrine runs every commit (the guard's path is declared in the assertion field, which the transitive leg does not read as a destination -- a control sentence naming it would make the enforcing script a routed destination, which it is not)|repo-local|doctrine=PROJECT-SPECIFIC
docs/risks.md|hot_live|the LIVE subset of the roadmap's §25 risk table, with owners and Phase 0 actions; the full qualitative table stays in `ROADMAP.md`, so this file tracks state rather than accumulating history|repo-local
docs/parking-lot.md|author_overflow|non-blocking ideas kept OUT of the frozen v0.4.1 roadmap; an entry leaves by becoming a task-tree leaf or by being declined, never by accumulating, and v0.5.0 cannot be opened from an item here without executable evidence|director
docs/ARTIFACT_CLEANUP.md|hot_live|the startup-directive §8 cleanup record: exactly ONE entry, the date and a one-line summary of the latest cleanup, OVERWRITTEN by each cleanup and never appended, so earlier entries live only in git history. Narrative: a one-entry file has no growth history to assert a shape over. Owner SIGNOFF-REPAIR.11.4.3.1.10|repo-local
deploy/|reader_navigation|the Phase 1 LAN deployment package: the binaries `make release` produces plus the operator runbook that stands a control plane and nodes up on a trusted LAN. Product documentation lives in the book's deployment chapter, so this is the packaging surface only|repo-local|build_target=release
