---
answers:
  - Which of SIGNOFF-REPAIR.5.2's goal items are still open after the six repairs that discharged its findings?
  - Is any recruitment-call transition — open, respond, close — one transaction, and what can interleave?
  - Where is a call's minimum enforced, and against which set?
  - How are the remaining items split and ordered?
---
# No call transition is one transaction, and the minimum is checked before the filter

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.5.2`
- **Date:** 2026-09-23
- **Cites:** ROADMAP §10.5 (*a call specifies … minimum/maximum participants … the server
  snapshots the selected panel and selection explanation*), §10.7 (fan-out limits);
  `docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md`;
  `docs/decisions/2026-09-07_node-channel-wiring.md` (`DISPATCH-001`: *an invitation exists
  iff its work does* — the one-transaction rule this lane follows elsewhere)

## Context

`SIGNOFF-REPAIR.5.2`'s goal line names four things: *bind call/thread/tenant/actor and
grants*, *make panel/close/offer transitions atomic*, *enforce post-filter minimums and
concurrent caps*, and *repeated legitimate auto initiation*. The findings annotated on the
leaf are discharged — the once-per-tenant key (REPAIR-0417), the spend gate reading expired
grants (REPAIR-0421), the call bound to a real thread (REPAIR-0422), the respondent bound to
the call's tenant (REPAIR-0185). This record censuses what the goal line still owes. Every
claim cites the code as of `3687850`.

## The census

**Bind.** The open is authorized on `ThreadInvite` over the named thread and now loads the
thread tenant-bound; the respond derives the respondent's tenant from the call and re-resolves
eligibility against current facts; the close is gated on the initiator or the tenant's
administrator. ✅ Met by the repairs above.

**Atomic transitions — none is one transaction.** `crates/reasonbraid-server/src/api.rs`
and `recruitment.rs`:

| transition | statements, each on the pool | what interleaves |
| --- | --- | --- |
| open (`open_recruitment_call`) | the two fan-out counts (`open_calls_by`), then `open_call`'s INSERT, then one INSERT per offered subscriber | two concurrent opens both read a count under the cap and both insert — the cap is read-then-write; a crash after the call's INSERT leaves a call with no or partial offers, the advertisement's durable trace incomplete |
| respond (`respond_to_call_core`) | `call()` read, status/deadline/expiry/eligibility checks on that stale row, then `record_response`'s upsert | a respond that reads `open` while a close is running lands its join on a call the close has already read the responses of — on the record, off the panel |
| close (`close_call`) | `call()` read, the gate, `responses()` read, candidate and lineage reads, then `snapshot_panel`: the panel INSERT and the status UPDATE as two statements | two concurrent closes both read `open`; the second's panel INSERT hits `recruitment_panels.call_id PRIMARY KEY` — a RAISED violation answered `500`, the shape the knowledge note forbids — and neither the first's panel nor its status is fenced against a join landing between its reads and its UPDATE |

**Post-filter minimum — checked before the filter.** `close_call` compares
`min_participants` with the RAW joiner count (`responses` of kind `join`), then
`rank_with_dependence` keeps only candidates whose re-resolved verdict is `eligible`
(`matching.rs`, `.filter(|(_, verdict)| verdict.eligible)`), then truncates to the maximum.
So a joiner who became ineligible after joining — its profile changed, its node went — still
counts toward the minimum and is then dropped, and the call closes with a panel SMALLER than
its minimum, down to empty. §10.5's minimum is a minimum on the selected panel; today it is a
minimum on who once said *join*.

**Concurrent caps.** `max_participants` is a selection cap applied at close (`truncate`),
which is what §10.5's *selected panel* means, and not a defect. The fan-out caps are the
concurrent gap, and they ride the open's transaction above.

## Decision — two children, the small one first

1. **`.5.2.5` — the minimum on the selected panel.** `close_call` refuses when the ELIGIBLE,
   ranked set is below `min_participants`, naming both counts. RED first: a role joins, its
   profile drops the required interest, the close with `min_participants: 1` today snapshots
   an empty panel and reads `closed`.
2. **`.5.2.4` — each transition one transaction.** Open: the fan-out caps decided and the
   call and its offers written under one transaction, the cap serialized per tenant (the
   mechanism — the tenant's shared authority guard, a row lock over the initiator's open
   calls, or a single conditional INSERT — is the leaf's to decide with the reason). Close:
   the call row locked `FOR UPDATE`, status re-checked inside, responses read, panel and
   status written, all in one transaction, so a second close reads `closed` and answers
   `409` rather than a raised primary key. Respond: the call row locked `FOR SHARE` with the
   upsert, so a join cannot land on a call whose close has begun. RED first with the
   race harness `enrollment_transaction.rs` already carries (`hold_issuance` / `waiter`).

⛔ Not decided here: whether a join that lost the race to a close should be refused or
recorded as *late* on the closed call. The leaf reproduces it before choosing.
