# RB-SEED: land the execution-baseline roadmap and seed the programme trees

## Metadata

- Tree ID: `RB-SEED`
- Status: `active`
- Roadmap lane: programme setup (pre-Phase 0)
- Created: `2026-09-05`
- Owner: repo-local workflow

## Goal

Take the director-dropped `ROADMAP.md` (ReasonBraid v0.4.1) and its companion
`KICKOFF.md` (Phase 0 immediate execution plan), make them the project's
canonical sources of truth, convert the entire programme into task-trees, and
adopt the Claim-Verification architecture — so a lost session can resume from
the trees rather than from chat.

## Non-Goals

- This tree does not implement Phase 0 code, experiments, or adapters.
- This tree does not un-freeze roadmap v0.4.1 or start v0.5.0.
- This tree does not adopt live-document size containment (not needed yet).

## Acceptance Criteria

- `ROADMAP.md` and `KICKOFF.md` are tracked, cross-linked, and treated as a pair.
- Every Phase 0–9 work package, backlog item, gate, and ADR queue entry is
  owned by a named task-tree leaf.
- `CLAIM_VERIFICATION.md` is project-owned and discoverable from the bootstrap.
- Live docs, the mdBook, and `docs/TASK_TREE.md` stay in lockstep.
- Each completed leaf is committed through `COMMIT.md`.

## Task Tree

- ID: `RB-SEED`
  Status: `active`
  Goal: seed the ReasonBraid programme from the dropped roadmap pair
  Children: `RB-SEED.1`, `RB-SEED.2`, `RB-SEED.3`

- ID: `RB-SEED.1`
  Status: `done`
  Goal: land `ROADMAP.md` v0.4.1 and companion `KICKOFF.md` as the canonical pair; record the companion decision; update live docs and the mdBook so a reader sees ReasonBraid, not the bedrock placeholder
  Acceptance: both files tracked; each names the other; decision record exists; MEMORY/LIVE_STATUS/CHANGELOG/DEV_NOTES/book reflect the drop; no product code changed
  Verification: recorded below
  Commit: pending until `COMMIT.md` step 6

- ID: `RB-SEED.2`
  Status: `pending`
  Goal: convert the entire v0.4.1 roadmap and Phase 0 kickoff into detailed task-trees (`PROGRAM`, `PHASE-0` … `PHASE-9`) and register them
  Acceptance: every phase, WP, backlog item 1–40, gate G0–G9, and ADR 001–028 maps to a named leaf; `docs/TASK_TREE.md` lists the active programme trees
  Verification: pending
  Commit: pending

- ID: `RB-SEED.3`
  Status: `pending`
  Goal: adopt `docs/CLAIM_VERIFICATION.md` (portable architecture #5) as a project-owned copy
  Acceptance: file present; bootstrap points at it; decision record exists; adoption checklist items that can be done without product claims are recorded
  Verification: pending
  Commit: pending

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `RB-SEED.2` | `pending` | roadmap is not yet represented as task-trees |
| 2 | `RB-SEED.3` | `pending` | CLAIM_VERIFICATION is not adopted |

## Decisions

- `2026-09-05`: `KICKOFF.md` is the Phase 0 companion to `ROADMAP.md`, not a
  competing roadmap. Scope and gates live in `ROADMAP.md`; day-to-day Phase 0
  execution lives in `KICKOFF.md`.
- `2026-09-05`: `.doctrine/code_paths.txt` excludes `docs/book/src/` from
  TASK-ACCEPTANCE. The default `(^|/)src/` glob matches mdBook chapters; this
  project's Rust lives under `crates/`.
- `2026-09-05`: Live-document size containment is **not** adopted in this tree.
  `MEMORY.md` is well under cap; revisit if a live surface starts scaling.

## Open Questions

- None that block `RB-SEED.1`. Naming/legal clearance is `PHASE-0` / ADR-001.

## Blockers

- None.

## Acceptance Checklist (required for any leaf that lands a CODE change)

Enforced by the `TASK-ACCEPTANCE` doctrine (`scripts/check_task_acceptance.sh`).
`RB-SEED` lands documents only; the hard-gated boxes below stay unticked unless
a later leaf stages code.

- [ ] **ROOT CAUSE (WHY + WHERE)** — N/A unless a leaf stages code
- [ ] **ADDRESSED (verified)** — N/A unless a leaf stages code
- [ ] **NO REGRESSION** — N/A unless a leaf stages code
- [ ] **FIX** — document/programme seeding only
- [ ] **LOCKSTEP** — live docs, book, decisions, and task index updated per leaf

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-09-05` | `RB-SEED.1` | `rg KICKOFF.md ROADMAP.md` → 2 hits (header + §20.1.2); `rg ROADMAP.md KICKOFF.md` → 2 hits (governing roadmap + companion note); `wc -lc MEMORY.md` → 19 lines / 912 bytes; `scripts/check_doctrines.sh` after regenerating `KNOWLEDGE_MAP.md` | companion pair cross-linked; layer-A under cap; doctrines re-run at commit |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `RB-SEED.1` | `REASONBRAID-SEED-0001 (leaf RB-SEED.1): land ROADMAP v0.4.1 and companion KICKOFF` | docs only; no product code |

## Changelog

- `2026-09-05`: Created task tree.
- `2026-09-05`: `RB-SEED.1` landed the roadmap pair. Frontier is `RB-SEED.2`.
