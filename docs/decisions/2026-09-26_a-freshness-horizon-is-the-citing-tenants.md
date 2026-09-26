---
answers:
  - Does re-acquiring an evidence snapshot refresh its freshness horizon?
  - Whose freshness horizon does the stale list use when several tenants cite the same snapshot?
  - Why is fresh_until on the citation and not on the snapshot row?
---
# A freshness horizon is the citing tenant's

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.7.4.9`
- **Date:** 2026-09-26
- **Work unit:** `REASONBRAID-REPAIR-0545`
- **Cites:** `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`
  (one shared snapshot row, a tenant-bound read); `migrations/0116`;
  `docs/book/src/deployment.md` § *A tenant's own freshness horizon*

## The fact / decision

A re-acquisition refreshes the freshness horizon, and the horizon belongs to the
tenant that declared it. `fresh_until` is kept on the tenant's row in
`evidence_citations`; a first acquisition and every replay write the submission's
value there (including none), the stale list compares each tenant's own horizon,
and the snapshot read shows the reader's. `evidence_snapshots.fresh_until` is
dropped by migration 0116, after every existing citation took the value the row
held.

## Why

- **It was measured, not assumed.** Before this, `snapshots::replay` wrote only
  `refreshed_at` and discarded the new submission's `fresh_until`, so evidence a
  tenant re-acquired stayed on that tenant's stale list: the list asks the
  tenant to re-acquire, and re-acquiring did not answer it. `evidence.md` said
  the replay *"refreshes its freshness horizon"*; `deployment.md` said horizons
  were *"a separate field"*.
- **"Latest replay wins" on the shared row would have been worse.** The snapshot
  row is shared by every tenant that cites the same bytes. Kept there, the
  horizon was already the FIRST acquirer's decision about every later tenant's
  stale list; letting a replay overwrite it would let any citing tenant change,
  or clear, another's. A horizon is a decision about when to re-acquire, and
  only the tenant that makes it should be bound by it.
- **Why a tenant's latest declaration replaces its earlier one.** Only that
  tenant reads it, so there is no one else to protect; the most recent
  acquisition is the tenant's most current statement, including a statement of
  no horizon at all.

## Scope

`refreshed_at` stays on the shared row: it records that the bytes were
re-acquired, not a tenant's decision. The external (git) snapshot class
declares no horizon. Retiring a stored object whose snapshot write failed is
`.7.4.12`'s, and unchanged here.
