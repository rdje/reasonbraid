# A transport receipt is not an acknowledgement, and the derived delivery state was calling one the other

- **Type:** `decision`
- **Date:** `2026-09-19`
- **Status:** `active`
- **Owner / source:** `SIGNOFF-REPAIR.11.24.1.1` (REASONBRAID-REPAIR-0297) — measured

## The fact / decision

The derived `node_inbox_state.delivery_state` renames its third state:

```text
queued → transport_received → consumed
                   ↘ dead_lettered
```

What was published as `acknowledged` is now published as `transport_received`.
`migrations/0075_node_inbox_transport_received.sql` replaces the view.

The shipped ladder is a true **subset** of `ROADMAP.md` §10.6 in which every
name means what §10.6 says it means. `offered`, `acknowledged`, `expired` and
`revoked` remain **underived**, each with a disposition and, where work is owed,
a leaf. ⛔ A state with no producer is not added to the `CASE`.

## Why

§10.6 gives the ladder and then, in the same paragraph, the distinction:

> `queued → offered → transport_received → acknowledged → consumed`
> `                  ↘ expired / revoked / dead_lettered`
>
> Transport receipt does not mean an agent read or acted. Acknowledgement
> semantics are explicit per event type.

Read from the producer rather than from the column's name, `acknowledged_at`
records a **transport receipt**. The node's reconcile loop journals every
inbound command — `record_inbound_command` then `ensure_operation`, step 4 —
and only at step 7 calls `channel.acknowledge(max_cursor)`. So the column means
*the node process durably holds this command*. The agent has not read it and has
not acted; acting is what writes the `work_result` event the view already
derives `consumed` from.

And the cursor ack cannot be §10.6's `acknowledged` for a second, independent
reason: it covers **every row up to a cursor, whatever their event types**,
while §10.6 defines that state as the one whose semantics are *explicit per
event type*. Nothing in the shipped channel produces it.

So the view used one of §10.6's words for the fact §10.6 gives the other word
to — in a published field, under a specification sentence written to keep those
two apart.

## Consequences

- **A wire-visible vocabulary change.** `delivery_state` is returned by
  `GET /v1/nodes/inbox` and by the MCP `list_inbox` tool; a client matching the
  string `acknowledged` sees `transport_received` after this migration. Taken
  deliberately: the project is pre-1.0, the surface is the development profile,
  and a published vocabulary that reuses a specification's word for a different
  fact is what the §10.6 sentence exists to prevent.
- **The state that was named wrongly was the one state the suite never
  asserted.** `the_delivery_ladder_reads_through_the_inbox_state_view` covered
  `queued`, `consumed` and `dead_lettered`. It now covers all four, with
  `cmd_received` and `cmd_consumed` sharing an `acknowledged_at` and differing
  only in the `work_result` event — so an implementation that ignored the event
  fails rather than passing on the row it happens to check.
- **Four states stay underived, by decision and not by omission:** `offered`
  and `acknowledged` (`.11.24.1.1.1`), `expired` and `revoked`
  (`.11.24.1.1.2`, which is also where `PHASE-3.2.2`'s deferred offline-delivery
  expiry and max age land).

## answers:

- **Is a rename worth a wire break?** Here, yes, and the test is whether the old
  name could mislead a decision. A client that reads `acknowledged` and
  concludes the agent has seen the command will skip a re-offer it should make.
  `transport_received` cannot be read that way, which is the entire point of
  §10.6 spelling both words out.
- **Why not add the missing states while renaming one?** Because a state with no
  producer is an advertisement, not a fact —
  [[a-re-export-is-not-a-caller]]'s shape applied to a vocabulary, and the same
  defect `PresenceState::Busy` carries (`.11.24.1.2`). The four are owed with
  their mechanisms, not with a `CASE` arm.
- **How was this found?** Not by reading the migration. `.11.24.1` adjudicated a
  census population and asked whether `transport_received` existed in `crates/`;
  it returned **0**, which sent a reader to §10.6 and then to the producer of
  `acknowledged_at`. ⭐ The question *is the promised thing there?* found a
  defect in the thing that IS there.
