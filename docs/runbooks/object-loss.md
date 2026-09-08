# Runbook: object loss

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN)
- Scope: a stored snapshot object or a derived chunk is missing or corrupt —
  the content-addressed store cannot serve what the references name.

## Detection

- **The digest miss:** a snapshot read whose `sha256:<hex>` object is absent
  (the content-addressed store fails the lookup).
- **The staleness signal:** the freshness surface (`rb inspect …` /
  the `.6.4` staleness flags) names the expired/missing references.
- **The derivation break:** a derivation edge whose parent or derived chunk
  is missing.

## Authority

- The retention enforcement: `tenant_admin` (`POST /v1/snapshots/expire-due`
  — the explicit operator action, never a sweep).
- Reading: the inspection surfaces.

## Safe first actions

1. **Do not delete more.** The tombstone is the correct disposition — the
   reference row stays (the object's history), the retention marks it.
2. **Freeze the picture:** the missing digests, the derivation edges, the
   last-known-good state.

## Diagnostic queries

- The snapshot store's rows (the digest, the quarantine_status, the
  retention_class).
- The derivation graph (what referenced the missing object).
- The license/retention + the freshness flags (the `.6.4` surface).

## Containment

- The missing object refuses (the digest lookup fails closed — no fallback
  to a "similar" object).
- The derived set that depends on it is flagged (the derivation edges name
  the dependents).

## Recovery

- **Re-acquire:** the resource pipeline re-derives from the source when the
  source still resolves (the derivation is reproducible — the R0/R1 packs).
- **The tombstone:** when nothing can re-derive it, the tombstone records
  the loss (the store's honesty — an object that is gone says so, it never
  fabricates).
- **The retention:** the expiry rides the operator action (the explicit
  `expire-due`), never a sweep that could delete the evidence.

## Evidence preservation

- The tombstone row + the derivation edges (the loss's record).
- The last-known-good digests (the re-acquisition targets).

## Communication

- The operator reports the loss (the missing digests, the dependents, the
  re-acquire vs tombstone decision) to the accountable owner.

## Closure tests

- The snapshot-store suite (the content-addressing + the tombstone legs) +
  the derivation-graph suite (the verified edges) on every guard pass.
- The citation-validation legs (an excerpt must exist in the real bytes —
  a missing object refuses, never satisfies) on every guard pass.
