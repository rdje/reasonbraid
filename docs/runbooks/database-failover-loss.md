# Runbook: database failover / loss

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN)
- Scope: the PostgreSQL control-plane store is unavailable, corrupt, or gone.
- **Honest limit (stated, not hidden):** the dev profile has NO failover
  machinery — no replica, no automatic promotion. This runbook IS the
  restore path; the production RPO/RTO pair is the Phase-7 boundary named
  in the SLO record, not invented here.

## Detection

- **The server-side error:** the control API returns the internal error
  (the dependency-unavailable shape) — every write path fails closed.
- **The operator's check:** the server log + the PostgreSQL state (the
  cluster down / the disk loss / the corruption).

## Authority

- Running the backup/restore: the operator (the scripts are the automation).
- Declaring the incident: the accountable owner.

## Safe first actions

1. **Stop the writes.** The server refuses anyway (every transaction fails
   closed) — do not "fix" the database by hand; the ledger rows are the
   truth, never hand-edited.
2. **Preserve whatever survives:** the data directory, the last dump, the
   server log.

## Diagnostic queries

- `scripts/backup.sh` — the dump (when the cluster still runs).
- The guard's restore exercise logs (the last good restore's before/after).

## Containment

- The failed-closed writes: nothing new commits while the store is down.
- The node side: the channel's delivery waits (the journal holds the
  dispatch boundary — the nodes reconcile on the reconnect).

## Recovery

- **Partial loss (the data survives):** restore from the last dump
  (`scripts/restore.sh`) — the exercise's own shape: seed → dump → mutate →
  restore into an isolated database → assert the pre-mutation state.
- **Total loss:** the database is rebuilt from the migrations + the last
  dump; the in-flight attempts reconcile through the node journals (the
  `outcome_unknown` ambiguity policy, the node-lost-replaced runbook).
- **The honest RPO/RTO statement:** the dev profile's RPO is the last dump,
  the RTO is the restore's wall time — the measured numbers are the guard's
  restore-exercise log, not a promised pair.

## Evidence preservation

- The last dump + the restore's before/after assertion output.
- The node journals (the surviving dispatch-boundary evidence).

## Communication

- The operator declares the loss (the RPO window — what happened between
  the last dump and the loss) to the accountable owner.

## Closure tests

- The backup/restore suite (the real pg_dump/restore round on every guard
  pass) + the migration-upgrade suite (the schema-path survival).
- The demo's kill points (the server restart legs — the committed commands
  survive) on every guard pass.
