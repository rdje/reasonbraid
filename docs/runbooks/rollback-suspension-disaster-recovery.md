# Runbook: rollback / suspension / full disaster recovery

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN)
- Scope: the whole system must step BACK (a bad release), STOP (a systemic
  incident), or come back from total loss.
- **Honest limit (stated, not hidden):** the dev profile's DR is the
  restore path (the database + the rebuildable binaries); a production
  RPO/RTO pair is the Phase-7 boundary, not invented here.

## Detection

- **The bad release:** the guard red (the doctrine gate, the suites, the
  demo) on the new build — the release-built proof is the check.
- **The systemic incident:** any of the runbooks above at the
  whole-tenant/whole-system scale.
- **The total loss:** the machine/volume is gone.

## Authority

- Suspending a tenant: `tenant_admin` (the boundary freeze — writes stop,
  reads stay).
- Rolling back: the operator (the versioned migrations + the restore).
- Declaring the incident: the accountable owner.

## Safe first actions

1. **Suspend, don't panic-delete.** The freeze stops the writes; the
   evidence stays (the quarantine-preserving rule everywhere).
2. **Freeze the picture:** the guard's last green state, the last dump, the
   release manifest (`release-manifest.json` — the signed record of what
   shipped).

## Diagnostic queries

- The SLO record's guard checks (the demo + the suites — which leg broke).
- `scripts/restore.sh` into an isolated database (the pre-mutation state).
- The migration status + the release manifest (what is running).

## Containment

- The tenant freeze + the breaker arm (nothing new starts).
- The rollback keeps the DATA: the schema rollback is the migration path,
  never a data delete.

## Recovery

- **Rollback (the bad release):** the previous binary (the manifest's
  digests name the artifacts — the signed manifest is the rollback target's
  identity) + the migration status it expects; the data survives (the
  versioned migrations are additive).
- **Suspension lift:** after the cause, the boundary re-issues (the freeze
  is reversible only by a superseding act — the recorded decision).
- **Full DR:** the database rebuilds from the migrations + the last dump;
  the binaries rebuild from the source (`make release` — the signed
  manifest re-verifies); the nodes reconcile (the journals' `outcome_unknown`
  paths); the replacement drill recovers the lost nodes.

## Evidence preservation

- The release manifest (the signed before/after identity).
- The last dump + the restore assertions.
- The incident's runbook outputs (whichever family triggered the DR).

## Communication

- The operator declares the incident (the scale, the chosen path) to the
  accountable owner; the closure is a recorded decision.

## Closure tests

- The restore suite + the migration-upgrade suite + the demo's kill points
  on every guard pass.
- The release-built demo proof (the debug AND release-built acceptance)
  before any rollback is undone.
