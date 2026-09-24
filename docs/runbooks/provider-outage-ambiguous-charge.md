# Runbook: provider outage / ambiguous charge

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN, CLI adapters)
- Scope: a provider call never returns, returns a partial charge, or its outcome
  is unknowable — "the call may have happened, the money may have moved".

## Detection

- **The attempt terminal:** the node's journal lands `outcome_unknown` (the
  adapter's honest ambiguity — a lost response without a lookup proof, the
  `.4.1` supervisor contract).
- **The adapter refusal:** `failed_known` with the stderr tail (the typed
  provider failures, the `.4.1` vocabulary).
- **The ledger signal:** `rb inspect usage` shows the held reservation for the
  attempt (the hold survives the ambiguity — the `.2.3` policy).

## Authority

- Deciding re-ask vs leave-unresolved: the human whose thread it is decides;
  the tenant's administrator takes the possible-duplicate re-ask, because it is
  a budget authorization (§11.3, §14.6): `rb node replay --node … --command …
  --allow-possible-duplicate --reason "…"` (`SIGNOFF-REPAIR.4.4.7.2`).
- Reading: `rb-journal` + `rb inspect runs` (read-only).

## Safe first actions

1. **Never re-fire the call to "check".** The ambiguity policy exists because a
   re-fire can double-charge — the retry decision is a human flag, not a reflex.
2. **Freeze the picture:** the journal entry, the attempt id, the held
   reservation, the provider-reported usage (when the provider later reports it).

## Diagnostic queries

- `rb-journal --dir <node-dir> attempts` — the attempt's boundary + its terminal.
- `rb inspect ambiguous` — the tenant's OPEN ambiguous attempts as the server
  last heard them from each node's handshake, without reaching the node
  (`GET /v1/admin/nodes/ambiguous-attempts`, `SIGNOFF-REPAIR.4.6.1.1`).
- `rb inspect runs` — the run row (one result = one run).
- `rb inspect usage` — the held/settled split for the ceiling.

## Containment

- The held reservation stays held (the indeterminate attempt keeps its hold,
  so the budget cannot double-spend it). It stays held past its ten-minute
  window: when the node reports `retry_requires_authorization`, the server
  stamps the reservation's `outcome_unknown_at`, and the ledger counts a stamped
  hold whatever its `expires_at` (`SIGNOFF-REPAIR.4.5.1`). The thread's budget
  view shows the stamp. ⚠️ Nothing releases it yet, even after an adjudication
  (`SIGNOFF-REPAIR.4.5.1.1`).
- No new work on the same attempt id: the idempotency claim serializes.

## Recovery

- **Provable non-completion** (the provider's lookup says no): settle the
  reservation with zero usage. A REVISE can be re-asked by a new challenge (a
  new operation, not a retry). A CONTRIBUTE cannot be re-asked today: work
  reaches a node only from an accepted invitation or a challenge of a role's
  contribution, and the role is already a participant.
- **Unknowable:** leave `outcome_unknown`; the human decides:
  - close the thread with the unresolved register (the honest close, `.1.5.3`); or
  - accept the duplicate risk. On its next tick the node dead-letters the item
    as `retry_requires_authorization`, and the administrator replays it with
    `--allow-possible-duplicate --reason "…"`. A fresh reservation pays for the
    possible duplicate, the original stays held, and the node re-runs the item
    once. The effect is audited as `node_command_replay_possible_duplicate`.
- **A provider-reported charge for an unknown attempt:** settle against the
  reported usage; the overrun reports, never clamps.
- **An operator adjudicates** (`SIGNOFF-REPAIR.11.4.7.2.1.5.5`): when the
  provider's own records settle the question and the node cannot prove it,
  the tenant's administrator records the verdict —
  `POST /v1/admin/nodes/ambiguous-attempts/adjudicate` with `completed` or
  `failed_known` and a reason. The node applies it at its next handshake and
  the row closes `adjudicated` with the admission as evidence
  (`docs/book/src/node-channel.md`, *An operator adjudicates*).

## Evidence preservation

- The journal's attempt row + the adapter's stderr (the `.6.2` failure-fixture
  corpus pins the replay shapes).
- The reservation row + the settlement row (the ledger, never hand-edited).

## Communication

- The operator notes the outage/ambiguity in the task tree (a leaf with its
  acceptance checklist); the affected threads' humans are named.

## Closure tests

- The adapter conformance suite (the six §19.4 invariants, three adapters) +
  the failure-fixture corpus (the versioned manifest) on every guard pass.
- The budget suite's indeterminate-hold legs (the hold survives; the settlement
  reports the overrun) on every guard pass.
