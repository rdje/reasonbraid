# Runbook: credential compromise

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN)
- Scope: a node's dev secret or workload certificate, or a principal's
  enrollment credential, is suspected or confirmed compromised.

## Detection

- **The operator's suspicion** (a leaked secret file, a stolen machine).
- **The channel signal:** `handshake_refusals` rises on a fenced identity; the
  cert-proof verification refuses a revoked leaf at the next crossing (the
  `.1.2.2` ladder).
- **The enrollment anomaly:** an unexpected token issuance or a re-enrollment
  attempt against an existing role (`node_enroll_audit` rows).

## Authority

- Revoking: `tenant_admin` — `rb node revoke` + the grant/boundary revoke verbs
  (every write audited).
- Reading: `rb inspect …` + the audit surfaces (read-only).

## Safe first actions

1. **Revoke FIRST, investigate second.** `rb node revoke` fences the identity:
   the handshake refuses, the presence suspends, and the tenant's revocation
   epoch bumps — every cached admission decision the stolen identity holds
   goes stale (the `.1.5.2` dispatch gate).
2. **Do not delete the evidence:** the stolen credential's audit trail (the
   enrollment rows, the token rows, the handshake refusals) is the footprint.

## Diagnostic queries

- `GET /v1/admin/metrics` — `handshake_refusals`, `authorization_denials`.
- `rb inspect incarnations` — what the compromised identity served.
- `rb inspect usage` — what was held/settled under it.
- The `node_certificates` row (the fingerprint + the revocation state).

## Containment

- The revocation fence: no new dispatch rides the compromised identity; the
  suspended presence names it; the epoch bump makes its cached decisions stale.
- A compromised TENANT boundary: the boundary-revocation freeze stops every
  write while the reads stay inspectable (the `.1.3.2` freeze semantics).

## Recovery

- **Node compromise:** the replacement drill (the `.7.2` ritual) — the fresh
  enrollment token + the new secret, the new incarnation, the inbox replay,
  the stale-decision fence, the dead-letter, the operator replay.
- **Principal compromise:** a new grant set under the tenant's authority; the
  compromised principal's grants revoke (the freeze-then-reissue shape).
- **The dev trust-store stance (honest):** the server IS the dev trust store;
  the recovery above is the whole dev-profile ritual — an external trust
  store is the Phase-7 secret-manager profile, not invented here.

## Evidence preservation

- The revocation rows + the enrollment audit rows (the chain).
- The metrics deltas (the denial counter matched the denied record — the
  `.5.2` measurement).

## Communication

- The operator declares the compromise to the accountable owner (a tree leaf
  with the footprint + the chosen path).

## Closure tests

- The revocation beats (the demo's revoke step: the revoked node reads
  suspended) + the `.1.3` revocation suites on every guard pass.
- The replacement drill (the `node_replacement` suite — the full ritual) on
  every guard pass.
- The non-escalation suite's re-arm fence (the revoked identity cannot re-arm
  silently) on every guard pass.
