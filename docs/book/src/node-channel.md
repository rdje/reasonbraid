# The node channel

The node connects **outbound** to the control plane (`ROADMAP.md` §9.3: nodes
initiate connections; ordinary deployments expose no inbound ports). This
chapter describes the Phase 0 channel: what it exchanges on reconnect, how
delivery stays duplicate-safe, and how reconciliation gates the node's
schedulability.

## Transport (Phase 0)

HTTP/1 JSON over the loopback development profile, served by the control plane
(axum) and spoken by the node (reqwest):

```text
POST /v1/nodes/handshake   the reconnect exchange
POST /v1/nodes/events      node results, deduplicated by the node-assigned event id
POST /v1/nodes/ack         cursor acknowledgement (delivery state)
GET  /v1/nodes/poll        the live delivery tail after a cursor
```

Every message carries a `channel_version` and rejects unknown fields, so a
forged authoritative field or a future version fails loudly, on both sides.
The channel is **unauthenticated** in Phase 0 — it exists for the loopback
experiment only; workload identity arrives with WP5.

## Reconnect and cursor resume

The node is authoritative for what it **durably holds**. On reconnect it
reports:

```json
{
  "channel_version": 1,
  "node_id": "nod_…",
  "last_acked_cursor": 3,
  "pending_operations": ["op_…"],
  "ambiguous_attempts": [
    { "attempt_id": "patt_…", "operation_id": "op_…" }
  ]
}
```

The server replies with every command after `last_acked_cursor` (the replay
tail), plus reconciliation guidance:

- **directives** — one per ambiguous attempt: `adjudicated` when the server
  holds a receipt for the operation's event (the node marks the attempt
  `reconciled`), `needs_adjudication` otherwise (the attempt stays visibly
  `outcome_unknown` — bounded, never silently retried).
- **known_events** — server-held receipts for the node's pending operations,
  so a result whose acknowledgement was lost is not re-sent.

A node that reports a cursor ahead of the server's ledger is **refused** with a
typed error: its journal saw commands this server cannot reproduce.

## Duplicate safety

The server replays, the node's journal deduplicates:

- a duplicated command never creates a second local operation (`operations` is
  keyed 1:1 on the command id), and
- a re-emitted event carries its **original id**, so the server's
  event-receipt primary key turns redelivery into a duplicate, never a second
  event.

## Schedulability gate

A node is `Offline → Reconciling → Schedulable`, and it becomes `Schedulable`
**only** after the full handshake round-trip is applied: crashed attempts
classified, replay journaled, directives applied, pending results re-emitted
(or acknowledged as known), cursor acknowledged on both sides. New work
(`emit_event`) is refused until then, and any failure drops the node back to
`Offline` — retrying the whole protocol is always safe because every step is
idempotent.

## Honest limits (Phase 1, `.1.2.1`)

- The channel is still **unauthenticated** at the handshake: `.1.2.1` lands the
  enrollment bootstrap (one-time tokens bound to tenant + node id + host claim,
  `rb node issue-token` / `rb-node --enroll-token …`, the node's dev key in the
  identity store), and `.1.2.2` wires the key-proof handshake + leases onto it.
- No node-command leases yet (`.1.2.2`); duplicate safety rests on the
  journal's dedupe keys.
- Live delivery is a poll of the tail; the streaming profile is the formal
  ADR-006 decision (WP8).
- The server's inbox is seeded by tests in `.3.2`; the production wiring from
  the WP2 outbox worker to the inbox arrives with the WP6 vertical slice.
