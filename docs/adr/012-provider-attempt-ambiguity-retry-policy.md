# ADR-012 — Provider-attempt ambiguity and the per-adapter retry policy, accepted with evidence

- **Status:** `accepted` (evidence-gated — the machinery this record formalizes
  was built by the Phase-0 WP3/WP4 leaves and hardened by the `.2.3` retry
  policy before this ADR was written)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-2.3.1` (reconciliation record — the ambiguity contract and
  the retry policy were decided and shipped by the Phase-0/1/2 delivery leaves)
- **Requirements:** `ROADMAP.md` §23 queue item 012; §14.6 (no silent retry);
  §1104–1108 (the possible-duplicate rule); backlog 23's retry sliver

## Context

The roadmap queued "provider-attempt ambiguity and per-adapter retry policy"
as ADR-012, but the implementation ran ahead of the record: the WP3 node
journal proved the boundary-before-boundary ordering and the honest
`outcome_unknown` (`docs/decisions/2026-09-06_node-journal.md`), the WP4
supervisor wired the prove/adjudicate exits, and `PHASE-2.2.3` landed the
per-adapter retry classes as a pure, tested decision. The record was never
promoted, leaving the queue item nominally open while the ambiguity contract
is shipped and green.

## Options

1. **Adopt the shipped ambiguity + retry machinery as the ADR** (promote the
   decision records with their evidence).
2. Re-open the ambiguity design (idempotency keys, provider-side dedup) —
   nothing in the measured evidence asks for this: the journal's
   boundary-first ordering and the prove/adjudicate exits survived real kill
   points, and the retry classes are pure + tested.

## Evidence

- **The WP3 journal** (`docs/decisions/2026-09-06_node-journal.md`): the
  dispatch boundary record commits BEFORE the adapter is invoked (a second
  connection sees it); a crash after a possible provider dispatch recovers as
  `outcome_unknown`, never a guess.
- **The WP4 supervisor** (`docs/decisions/2026-09-06_fake-adapter.md` +
  `…real-adapter-codex.md`): ack ≠ completion, unsupported lookup is never
  retry advice, and the honest ambiguity path rides the journal.
- **The core state machine** (`reasonbraid-core/src/state.rs`):
  `ProviderAttemptState` with deterministic `apply` — the only
  `outcome_unknown` exits are proof (`completed`/`failed_known`) or
  adjudication (`reconciled`).
- **The `.2.3` retry policy** (`reasonbraid-core/src/retry.rs`): the §14.6
  classes as a pure decision — never-crossed re-dispatches; a budget-denied
  item is terminal; a reserved pre-dispatch refusal retries bounded; an
  ambiguous outcome retries ONLY with the delivery's explicit
  `allow_possible_duplicate` flag, the refusal naming §9.8's
  `retry_requires_authorization` (5 tests; the core suite 49, the worker legs 4).

## Choice

Option 1. The ambiguity contract is the shipped machinery: boundary-before-
boundary ordering, `outcome_unknown` with proof-or-adjudication as its only
exits, and the pure per-class retry policy whose risky re-run requires the
explicit possible-duplicate authorization.

## Consequences

- No silent retry, structurally: every re-dispatch either provably cannot
  duplicate (the boundary never crossed) or carries the authorization.
- The retry classes live in the core crate as a pure function — new adapters
  ride the same decision, no per-adapter code.

## Rollback / revisit trigger

- A measured duplicate-cost problem (a provider with billable effects the
  boundary record cannot see) — reopen provider-side idempotency keys with
  numbers, not anticipation.
- A provider whose ambiguity window cannot be resolved by proof/adjudication
  (e.g. no status lookup at all) — the `allow_possible_duplicate` flag is the
  escape hatch the policy already types.
