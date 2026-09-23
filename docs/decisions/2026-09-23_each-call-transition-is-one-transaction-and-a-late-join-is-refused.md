---
answers:
  - What serializes two recruitment-call opens racing at the fan-out cap, and why that mechanism?
  - What happens to a join that arrives while the call is being closed?
  - Why does a second concurrent close answer 409 rather than 500?
  - Which recruitment data functions run on the caller's transaction, and which may read the pool?
---
# Each call transition is one transaction, and a late join is refused

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.5.2.4`
- **Date:** 2026-09-23
- **Cites:** `docs/decisions/2026-09-23_no-call-transition-is-one-transaction-and-the-minimum-is-checked-before-the-filter.md`
  (DOC-0142, which opened the leaf and left the late-join answer to it);
  `docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md`; ROADMAP §10.5, §10.7

## Context

DOC-0142 measured that none of open, respond and close was one transaction, and named the
three races: two opens both under the cap, a join landing between a close's reads and its
writes, two closes raising `recruitment_panels`' primary key into a `500`. It left two things
to this leaf: the serialization mechanism for the open, with its reason, and whether a join
that loses the race to a close is refused or recorded as *late*.

## Decision

1. **The open is serialized per tenant by a transaction-scoped advisory lock**
   (`pg_advisory_xact_lock(class, hashtext(tenant_id))`, `recruitment::serialize_opens`).
   The two caps are counts over a SET — the tenant's and the initiator's open calls — and no
   row stands for that set, so no row lock can guard it. The lock covers the initiator's cap
   because a principal is enrolled in one tenant. Rejected: the `tenants` row, because every
   other reader that locks it would queue behind an open; the tenant's authority guard
   (`tenant_authority_guards`), because an open is not an issuance and must not wait on one;
   a single conditional `INSERT … WHERE (SELECT count(*) …) < cap`, because two of those can
   still both see the count under READ COMMITTED. The call and its offers are one
   `INSERT` each on that transaction, so a failed offer write leaves no call behind. A refused
   open rolls back before `refuse_storm` records the refusal on its own commit, so the lock is
   not held for the record.
2. **The close holds the call row `FOR UPDATE`** from its first read to its commit: the status
   is checked on the locked row, the responses are read under it, and the panel INSERT and the
   status UPDATE commit together. A second close waits, reads `closed`, and answers
   `409 invalid_transition` — the refusal is a value, never a raised constraint.
3. **The respond holds the call row `FOR SHARE`** beside its upsert. Responses do not queue
   behind each other; a close in progress makes a respond wait and then read the status the
   close committed.
4. **A join that lost the race to a close is refused**, `409 invalid_transition` *the call is
   closed*, with nothing recorded. Not recorded as *late*: a response on a closed call is on no
   panel, no reader consumes a late row, and a row nothing consumes is not a record. The
   respond's existing status check already gives this answer once the join is ordered behind
   the close — the decision is to make the ordering hold, not to add a state.
5. **Which functions take the caller's connection.** `serialize_opens`, `open_calls_by`,
   `open_call`, `offer_to_subscribers`, `record_response`, `call_locked` and `snapshot_panel`
   take `&mut PgConnection`: a transition cannot call them on the pool by accident. `call`
   and `responses` are reads any executor may run: the inspection reads them on the pool, the
   close reads `responses` on its transaction after the lock.

## Consequences

- The MCP `join_call` seam is unchanged: it calls `respond_to_call_core`, which now owns its
  transaction.
- The controls hold each race deterministically with the trigger-and-advisory-lock harness
  `enrollment_transaction.rs` introduced: every INSERT into the table under test waits behind
  a holder, so both requests are past their reads before either can write. Weakening the
  close's lock to shared was tried as a mutation: the two closes deadlock (`500`) and the late
  join lands, so the exclusive lock is load-bearing, not ceremony.
