# State transitions: minimal orthogonal lifecycles with deterministic, fallible `apply`

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `superseded in part by 2026-09-06_node-journal.md` (the `failed_known` exclusion below was narrowed in `PHASE-0.3.1`; everything else stands)
- **Owner / source:** engineering decision during leaf `PHASE-0.1.3` (WP1 minimal state machines)
answers: how do the thread, participation, and provider-attempt lifecycles move, so every invalid transition is rejected deterministically and no state is reachable except through a documented edge?

## The fact / decision

Phase 0 keeps the *minimal* subsets named in `KICKOFF.md` §3 WP1 — not the full `ROADMAP.md`
§8.4 tables — as three small state machines in `crates/reasonbraid-core/src/state.rs`.
Each state enum's only operation is `apply(transition) -> Result<state, TransitionError>`:
it is total, deterministic, and fallible — it never panics, and an invalid move returns
`TransitionError` (carrying aggregate, source state, and rejected transition) rather than
reaching a state through an undocumented edge. The valid edges are:

| Aggregate | Valid transitions | Terminal states |
| --- | --- | --- |
| Thread | `open → closing → closed`; `open → cancelled`; `closing → cancelled` | `closed`, `cancelled` |
| Participation | `invited → accepted`; `invited → declined`; `invited → expired`; `accepted → left` | `declined`, `expired`, `left` |
| Provider attempt | `prepared → dispatched`; `prepared → failed_before_dispatch`; `dispatched → completed`; `dispatched → outcome_unknown`; `outcome_unknown → reconciled` | `completed`, `failed_before_dispatch`, `reconciled` |

State enums serialize `snake_case` (`"open"`, `"outcome_unknown"`, …) so records are
self-describing. The transition enums are *transient* domain commands — they are **not**
wire types; the operation/event catalogue that names them on the wire is backlog 6.

The provider-attempt aggregate also gains its identifier here: `ProviderAttemptId` with
wire prefix `patt` (deferred from `PHASE-0.1.1`; see [[2026-09-06_id-representation]]).

## Why

`ROADMAP.md` §8.4 lists states but not edges, and §8.6 requires "a message cannot itself
mutate governance state; it submits a command that a deterministic aggregate may accept
and translate to an event." So the transitions had to be chosen. The choices here keep
the demo honest with the smallest edge set:

- Thread close is two-step (`open → closing → closed`), not a direct `open → closed`,
  because the §8.4 model has no single "close now" action — closure is a wind-down then a
  finalize, and `closing` is where unresolved objections are surfaced.
- A participation invitation is consumed by one decision: `accepted → declined`/`expired`
  is invalid once the invite is accepted. Re-invitation is a *new* participation revision
  (a back-edge via explicit new record, per §8.4), not a state rewind.
- A *proven* post-dispatch failure (`failed_known` / `cancelled_known` in §8.4) was out of
  Phase 0 scope **as of this record (`PHASE-0.1.3`)**. `PHASE-0.3.1` later landed
  `failed_known` with **proof-gated edges only** — a definitive runtime rejection, or the
  §11.3 provider-lookup recovery edges `outcome_unknown → completed | failed_known` — so a
  PROVEN failure is never confused with a guess. `cancelled_known` remains out of Phase 0.
  The honest minimal answer to an indeterminate attempt is still
  `outcome_unknown → reconciled` (kill-risk Q4), never a guessed failure or a blind retry.
  See [[2026-09-06_node-journal]] for the superseding decision.
- `apply` returning `Result` (not panicking, not silently clamping) makes "invalid
  transitions rejected deterministically" (the WP1 acceptance) a compile-level, testable
  property rather than a convention.

## How to apply

- Add a future edge only by extending the owning enum's `apply` match and its `*_VALID`
  table in `state.rs`; the exhaustive test rejects every (state, event) pair not in the
  table, so an undocumented edge fails CI.
- Never reach a state except through `apply`; never add `#[serde(skip)]`-style rewinds or
  a blanket `From<state> for state` — back-edges are new revisions/actions, not history
  rewrites (§8.4).
- Keep state enums `snake_case` on the wire; keep transition enums non-wire until backlog 6
  defines the operation/event catalogue.
- `failed_known` landed in `PHASE-0.3.1` as an extension of `ProviderAttemptState` (not a
  new aggregate), proof-gated per [[2026-09-06_node-journal]]; `cancelled_known` remains
  deferred until it can be recorded honestly. The exhaustive `*_VALID`-table rule above is
  how that extension was added.
- See [[2026-09-06_id-representation]] (identifier newtypes) and
  [[2026-09-06_envelope-representation]] (how commands/events cross the wire).
