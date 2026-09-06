# G0 contract drafts: requirement-ID scheme extended, and the `spec/` location

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** director decision during leaf `PHASE-0.0.8`
answers: what stable IDs name the five G0 boundaries; where do the G0 contract drafts live?

## The fact / decision

1. The five G0 boundaries get stable requirement-ID prefixes **`ID-*`**,
   **`AUTH-*`**, **`THREAD-*`**, **`DELIV-*`**, **`BUDGET-*`** — one per boundary
   (`ROADMAP.md` §20.2). `THREAD-*` and `BUDGET-*` are **added** here because
   §19.1 names six illustrative prefixes (ID / AUTH / DELIV / RES / POL / SEC)
   that omit the thread and budget boundaries §20.2 actually gates on. `RES-*`
   (Phase 4), `POL-*` (Phase 6), and `SEC-*` (Phase 7) are **reserved, not used**
   in G0. Requirement-ID form is `FAMILY-NNN` (three zero-padded digits).

2. The G0 contract drafts live under **`spec/`** (glossary, requirements,
   lifecycle, threat-model, governance/charter), beside the code as §19.1
   intends — separate from `docs/`, which stays the project's decision/task
   layer. Every draft file is headed **"draft — not normative"**.

## Why

`PHASE-0.0.8` requires "stable IDs exist for identity/authority/thread/
delivery/budget." The §19.1 list names the *seed families* but not all five
§20.2 boundaries, so assigning IDs forced a choice: either leave thread and
budget unnumbered (breaking the "stable IDs" acceptance) or extend the scheme.
Extending is a gap-fill for backlog 4 ("assign stable IDs"), not a new feature,
so it does not touch the frozen roadmap — it is recorded here as the cross-cutting
convention the later phases will extend without renumbering.

`spec/` is where the roadmap already places the contract (§7.1 "beside the
code", §19.1). Keeping contract drafts out of `docs/` prevents the decision/
task layer from absorbing normative artifact text.

## How to apply

- Requirements cite `FAMILY-NNN` IDs; tests and gate records cite the ID, never
  a paraphrase (`spec/requirements.md`).
- `THREAD-*` and `BUDGET-*` are first-class now; `RES-*`/`POL-*`/`SEC-*` are
  declared but owned by their later phases — do not assign their numbers in G0.
- Contract drafts live in `spec/`, each marked "draft — not normative"; the
  decision/task layer (`docs/`) references but does not inline them.
- When a later phase makes these drafts normative, that is its own governed
  decision — supersede this record's "not normative" status, don't silently edit.
