# G0 contract drafts

**Status: draft — not normative.** These files are the Phase 0 **G0 contract
drafts** (`ROADMAP.md` §19.6 gate "G0 Contract", §20.2 deliverables). They are a
faithful distillation of the frozen roadmap v0.4.1 into stable, traceable
reference artifacts — **not** new architecture. Nothing here is binding on the
implementation until a later leaf promotes it to normative and wires it to a
test or gate. Pre-code changes are limited to factual errata, security
corrections, and Phase 0 blockers (`ROADMAP.md` §0.4.1 errata); any idea beyond
that belongs in `docs/parking-lot.md`.

This directory is the roadmap's `spec/` surface (`ROADMAP.md` §7.1 workspace
layout, §19.1 "maintain these versioned artifacts beside the code"). It is
distinct from `docs/` (working documentation) — `spec/` holds contracts that
code, tests, and gates will cite.

## Traceability map

Each artifact below distills one Phase 0 deliverable into its canonical home.

| Artifact | Distils | Backlog item | Primary source | G0 boundary |
| --- | --- | --- | --- | --- |
| [`glossary.md`](glossary.md) | frozen term distinctions | 3 | `ROADMAP.md` §8.1–§8.5, §4 | identity |
| [`requirements.md`](requirements.md) | requirement catalogue with stable IDs | 4 | `ROADMAP.md` §3, §19.1 | identity · authority · thread · delivery · budget |
| [`lifecycle.md`](lifecycle.md) | lifecycle / state-machine tables | 5 | `ROADMAP.md` §8.4, §10.6, §11.3, §15.6 | thread · delivery · budget |
| [`threat-model.md`](threat-model.md) | threat-model skeleton | 7 | `ROADMAP.md` §16, §25 | all |
| [`governance/charter.md`](governance/charter.md) | governance charter draft + authority graph | 2 | `ROADMAP.md` §4 | authority |

## Requirement-ID scheme

Requirements are addressed by a stable `FAMILY-NNN` id so tests, gates, and
task-tree leaves can reference the same sentence unambiguously (`ROADMAP.md`
§19.1: "natural-language requirements use stable IDs … tests and gate records
reference them").

| Family | Boundary | Source | Status in G0 |
| --- | --- | --- | --- |
| `ID-*` | identity, enrollment, presence | `ROADMAP.md` §3.1, §8.1 | in scope |
| `AUTH-*` | authority, authorization, governance | `ROADMAP.md` §4, §16.3–§16.4 | in scope |
| `THREAD-*` | conversation, thread lifecycle, deliberation | `ROADMAP.md` §3.2, §3.4, §8.4 | in scope |
| `DELIV-*` | delivery, notification, idempotency, ordering | `ROADMAP.md` §3.3, §6.4, §9, §10.6 | in scope |
| `BUDGET-*` | budgets, reservation, settlement | `ROADMAP.md` §3.8, §14 | in scope |
| `RES-*` | resource acquisition and evidence | `ROADMAP.md` §12 | reserved (Phase 4) |
| `POL-*` | policy and doctrine governance | `ROADMAP.md` §15 | reserved (Phase 6) |
| `SEC-*` | security and trust | `ROADMAP.md` §16 | reserved (Phase 7) |

**Deviation from the roadmap, recorded as a decision:** `ROADMAP.md` §19.1 names
six illustrative prefixes (`ID`, `AUTH`, `DELIV`, `RES`, `POL`, `SEC`), but
§20.2 gates G0 on five boundaries — *identity, authority, thread, delivery, and
budget*. The roadmap assigns no prefix to **thread** or **budget**, so those two
are added here (`THREAD-*`, `BUDGET-*`) to complete the G0 boundary set. This is
a gap-fill for backlog 4 ("assign stable IDs"), not a new feature. The decision
is recorded in
`docs/decisions/2026-09-06_g0-contract-id-scheme.md`.

## Status of each draft

- `glossary.md`, `requirements.md`, `lifecycle.md`, `threat-model.md` — drafted
  for the G0 scope; further phases extend them (not rewrite them).
- `governance/charter.md` — a **draft** charter. It is not binding until a
  governed adoption decision under `ROADMAP.md` §4 exists; until then it names a
  bootstrap root authority only so the authority model has an explicit,
  reviewable starting point.
