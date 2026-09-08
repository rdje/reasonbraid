# Runbook: audit-chain break

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN)
- Scope: the audit chain is suspect — a missing event, a broken version
  sequence, a tampered projection, or a corrupt stored state.
- **Honest limit (stated, not hidden):** the hash-chain + the checkpoint +
  the verification-policy machinery are the ADR-022 NAMED deferral (the
  trigger-named groundwork ships; the chain verification itself does not).
  Until it lands, this runbook's detection is the structural checks that DO
  exist.

## Detection

- **The CorruptState refusal:** the stored projection fails to parse (the
  "database modified outside the supported surface" failure mode) — the
  typed internal error, never a silent repair.
- **The version gap:** a version sequence with a hole (the
  `(tenant, aggregate, version)` uniqueness holds, but the continuity is
  checkable by the inspection surfaces).
- **The golden drift:** the schema golden or the fixture manifest drift
  checks fail (the mechanical guards).

## Authority

- Restoring: the operator (the backup/restore scripts).
- Declaring the break: the accountable owner.

## Safe first actions

1. **Stop the writes** (the affected surface fails closed anyway).
2. **Do not delete the suspect rows.** The audit chain's evidence is the
   rows themselves — the break is diagnosed from them, never erased.

## Diagnostic queries

- The event-log inspection for the suspect aggregate (the ordered versions,
   the digests, the bodies).
- The audit views (the authorization/denial rows for the window).
- The last good restore's before/after (the guard's restore-exercise log).

## Containment

- The failed-closed reads/writes on the corrupt state (the typed refusal).
- The audit-chain break does NOT spread: each aggregate's chain is its own
  sequence (the break is scoped to the suspect aggregate).

## Recovery

- **The row-level corruption:** restore the suspect aggregate's window from
  the last good dump (the restore exercise's shape: seed → mutate →
  restore → assert) — the events are the truth, the restore re-establishes
  them.
- **The schema-level break:** the migration-upgrade path (the all-but-last
  + the upgrade + the survival assertions) proves the chain survives the
  schema change.
- **The named future:** when the ADR-022 chain verification lands, its
  checkpoint is the detection THIS runbook's structural checks approximate.

## Evidence preservation

- The suspect rows (copied, not edited) + the restore's before/after.
- The guard's logs for the failing check (the golden drift, the version
  gap).

## Communication

- The operator reports the break (the suspect aggregate, the diagnosis)
  to the accountable owner.

## Closure tests

- The migration-upgrade suite + the backup/restore suite + the
  golden-drift checks on every guard pass.
- The demo's audit-reconstruction leg (the full story from the events +
  the audit rows) on every guard pass.
