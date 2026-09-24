---
answers:
  - Does anything drain the server outbox?
  - Why is there no production outbox runner?
  - When must the outbox runner be built?
---
# The server outbox has a producer and no consumer, so its runner is deferred to the first consumer

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.4.5.5`
- **Date:** 2026-09-24
- **Work unit:** `REASONBRAID-DOC-0157`

## The fact / decision

Every aggregate command writes one `outbox` row, in the same transaction as its event (`crates/reasonbraid-server/src/agg.rs`, step 4; that is the only `INSERT INTO outbox` in the server). Nothing in production claims one. `outbox::claim_ready`, `deliver` and `complete` (the `PHASE-0.2.2` leased worker) are called only by `tests/outbox_worker.rs`. `deliver` writes to `outbox_delivery`, which nothing reads, and no subscriber, integration target or publisher consumes domain events through the outbox. Node work travels through `node_inbox`, a separate path.

**Decision:** no runner is built now. A runner with no destination would lease, "deliver" into a table nobody reads and mark rows dispatched: activity with no effect, plus a dead-letter path for failures that cannot happen. The runner, its attempt cap, and §17.3's poison quarantine with authorized replay are built with the **first consumer**.

## Why

- §17.3 describes workers that deliver to consumers keeping inbox deduplication records. With no consumer there is nothing to deduplicate, nothing to fail, and so nothing to poison.
- What the undrained table costs today is measured, not assumed. It grows one row per domain event, the same cardinality as `event_log`, which is retained by design. It is a second index of the journal, not a new class of growth. Nothing in production deletes from `event_log`, so the `outbox → event_log` foreign key blocks no pruning today.
- No document claims a running worker (`grep -rni "outbox worker|outbox runner|leased outbox|claim_ready|outbox_delivery"` over the book, README and LIVE_STATUS found none).

## How to apply

Build the runner, then, with an attempt cap, a poison or dead-letter state that keeps its reason, and an authorized replay (§17.3, §18.5), when ANY of these becomes true:

1. a component consumes domain events: the Phase 6 publication worker (§15.x, *store … outbox item*), a federation event export, a webhook, or a projection fed asynchronously;
2. anything prunes or archives `event_log`, because an undispatched outbox row then blocks the deletion through its foreign key;
3. the outbox's size becomes an operational concern on its own account.

Until then, `outbox.dispatched = false` on every row is the correct state, not a backlog.
