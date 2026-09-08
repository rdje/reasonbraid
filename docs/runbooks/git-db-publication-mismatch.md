# Runbook: Git/DB publication mismatch

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN)
- Scope: the published Git store and the database's publication records
  disagree — the effective ref, the staged content, or the record state.

## Detection

- **The reconciliation matrix:** the six-row §15.8 rules compute the drift
  (the reconciler suite pins the matrix) — the inspection surface names the
  mismatch row.
- **The publication state:** a record in `staged` whose Git ref is missing,
  or an `effective` record whose content changed (the CAS check fails).

## Authority

- Publishing: the granted principals (the publication verbs).
- Reconciling: the operator reads the matrix (the reconciler is the
  IDEMPOTENT FUNCTION — it computes, it never silently promotes).

## Safe first actions

1. **Do not hand-edit either store.** The reconciler's six rules are the
   only promotion path — a hand "fix" breaks the CAS chain.
2. **Freeze the picture:** the record's state, the Git refs (the
   staging/immutable/effective scheme), the matrix row.

## Diagnostic queries

- The publication records (the aggregate row + the chain-verified refs).
- The reconciler's matrix computation (the `.4.3.3` function — re-run it,
   it is deterministic).
- The Git store's refs (the three-ref CAS scheme).

## Containment

- The never-silent-promote rule: the reconciler reports the mismatch row —
  it does not "heal" it by promotion (the promotion is a human decision).
- The effective ref stays where it is (no partial truth becomes effective).

## Recovery

- **The staged-vs-Git split:** the reconcile function names the exact
  action (the six rules cover the six rows) — the operator applies the
  named action, then re-runs the reconcile to the idempotent fixed point.
- **The corrupted Git store:** the records + the immutable refs are the
  truth — the store rebuilds from the chain-verified references (the
  fetch-back proof: what was written is what is read).
- **The effective content changed:** the CAS check names it — the
  publication re-stages from the verified content.

## Evidence preservation

- The reconcile matrix output (the before/after rows).
- The Git refs + the record states (the chain).
- The CAS digests (the content identity).

## Communication

- The operator reports the mismatch (the matrix row, the chosen action)
  to the accountable owner; the promotion is a declared decision.

## Closure tests

- The publisher suite (the written-once immutable + the CAS effective
  legs) + the reconciler suite (the six-row matrix + the idempotence) on
  every guard pass.
