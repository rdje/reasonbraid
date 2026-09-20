# ReasonBraid routed-destination registry (README_POLICY.md — Routing pressure closure).
# One row per destination named by README.md, README_POLICY.md, or the guard's
# routing hint. Format: path|class|pressure control|owner
# classes: reader_navigation | author_overflow | hot_live | generated_index |
#          append_only_history | frozen_legacy | external_service
# scripts/check_readme_stability.sh fails when a linked or emitted destination
# has no row here, when a row is malformed, or when a declared ceiling is
# exceeded. A row's path governs that path and everything under it (prefix
# closure). Raising any ceiling needs the same explicit review as the README cap.
docs/book/|reader_navigation|mdBook built by `make book` (commit workflow + CI); chapters are task-tree-leaf owned and locked to the code|repo-local
ROADMAP.md|reader_navigation|frozen at v0.4.1; changes only per the errata rule (factual or security corrections, Phase 0 blockers) or the v0.5.0 gate|director
KICKOFF.md|frozen_legacy|Phase 0 execution companion; content identity once the PHASE-0 tree closes (Phase 1 execution lives in `docs/tasks/PHASE-1.md`)|director
CLAUDE.md|reader_navigation|one-line bootstrap pointer contract (E1); MEMORY-ARCH doctrine verifies the pointer|repo-local (scaffold)
AGENTS.md|reader_navigation|one-line bootstrap pointer contract (E1); MEMORY-ARCH doctrine verifies the pointer|repo-local (scaffold)
docs/CLAIM_VERIFICATION.md|reader_navigation|adopted portable standard; revisions only via the §17 upstream re-check plus deliberate local review (no append pressure)|director
SECURITY.md|reader_navigation|the disclosure + supported-version policy record (the `.2.2` leaf); revisions ride task-tree leaves, never append pressure|director
README_POLICY.md|hot_live|governed by itself: revisions only via deliberate local review (no auto-sync); the guard refuses when this file is absent|director
scripts/update_scaffold.sh|reader_navigation|explicit scaffold pull tool; never an overflow destination|repo-local (scaffold)
docs/tasks/|author_overflow|partitioned task collection: bounded index `docs/TASK_TREE.md` (one row per tree) + per-tree changelog and verification discipline; trees close when exhausted|repo-local
docs/TASK_TREE.md|author_overflow|bounded index, one row per active tree; TABLE-ARITY and TASK-TREE-OWNERSHIP doctrines enforce its shape|repo-local
LIVE_STATUS.md|hot_live|NO SIZE BOUND. The control this row used to declare — `overwritten rather than appended` — was REFUTED by its own history in SIGNOFF-REPAIR.11.4.2.5: 632 versions, 614 grew against 17 shrank, 646,037 bytes added against 39,251 removed, and the tip is the all-time high. Governed debt recorded at the exact baseline 607,437 bytes, 3,113 lines, 7,305-byte widest line — a single line wider than the whole layer-A pointer cap. The enforcer accepts 200,000 added bytes on it today, demonstrated rather than inferred. Baselines do not widen. Owner SIGNOFF-REPAIR.11.4.2.6|repo-local
DEV_NOTES.md|append_only_history|BOUNDED since its first rotation. Threshold 76,000 bytes, DERIVED from the changelog's own live window of 20.279 p90-entries expressed in this ledger's measured entry size, and enforced by LEDGER-RUNWAY through the shared rotation mechanism. The debt this row was opened to record is DISCHARGED: 430 records retired into git history at the first rotation, 12 kept, 908,850 bytes to 37,874, with ~10 commits of runway. Retrieval is git history through the chain notice the file's own footer carries. ⛔ Rotate, never raise the threshold. Owner SIGNOFF-REPAIR.11.4.2.6.3|repo-local
CHANGELOG.md|append_only_history|query-first (git log is the access path; CHANGELOG is a digest) + 96,000-byte rotation threshold enforced by the README-STABILITY guard (the 48,495-byte baseline is recorded as governed debt); rotation = git history|repo-local
COMMIT.md|reader_navigation|the exact commit workflow (spine doctrine); changes are deliberate spine maintenance, never an overflow destination|repo-local (scaffold)
docs/adr/|author_overflow|one ADR per file + `docs/adr/INDEX.md` entry; supersede, never mutate; ADRs are accepted only by the recorded authority|repo-local
docs/decisions/|author_overflow|one record per file + INDEX entry (MEMORY-ARCH enforces index sync); supersede, never mutate|repo-local
TOOLBOX.md|reader_navigation|tools-first doctrine; the tool-registry table grows only with new diagnostic tools (deliberate)|repo-local
KNOWLEDGE_MAP.md|generated_index|regenerated and staged by the pre-commit hook; KNOWLEDGE-MAP doctrine checks sync against sources|repo-local (scaffold)
knowledge-map/|generated_index|sources + checkers behind `KNOWLEDGE_MAP.md`; KNOWLEDGE-MAP doctrine checks sync|repo-local (scaffold)
