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

- Deciding re-ask vs leave-unresolved: the human whose thread it is (the
  explicit `allow_possible_duplicate` flag — never a silent retry).
- Reading: `rb-journal` + `rb inspect runs` (read-only).

## Safe first actions

1. **Never re-fire the call to "check".** The ambiguity policy exists because a
   re-fire can double-charge — the retry decision is a human flag, not a reflex.
2. **Freeze the picture:** the journal entry, the attempt id, the held
   reservation, the provider-reported usage (when the provider later reports it).

## Diagnostic queries

- `rb-journal --dir <node-dir> attempts` — the attempt's boundary + its terminal.
- `rb inspect runs` — the run row (one result = one run).
- `rb inspect usage` — the held/settled split for the ceiling.

## Containment

- The held reservation STAYS held (the indeterminate attempt keeps its hold —
  the budget can't double-spend what is still held).
- No new work on the same attempt id: the idempotency claim serializes.

## Recovery

- **Provable non-completion** (the provider's lookup says no): settle the
  reservation with zero usage; the thread's human re-asks normally.
- **Unknowable:** leave `outcome_unknown`; the human decides — re-ask with
  `allow_possible_duplicate` (the flag rides the retry policy) or close the
  thread with the unresolved register (the honest close, `.1.5.3`).
- **A provider-reported charge for an unknown attempt:** settle against the
  reported usage; the overrun reports, never clamps.

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
