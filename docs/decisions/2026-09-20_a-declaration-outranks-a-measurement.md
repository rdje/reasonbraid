---
answers:
  - Why can a node now report §10.2's `busy`, and what counts as in flight?
  - Why does `draining` outrank `busy`?
  - Why is `offered` not counted as in flight?
  - Why is the in-flight count a view column rather than an argument at five call sites?
---
# A declaration outranks a measurement

- **Type:** decision
- **Status:** accepted; implemented
- **Owner:** `SIGNOFF-REPAIR.11.24.1.2`
- **Date:** 2026-09-20
- **Raised by:** `SIGNOFF-REPAIR.11.24.1`'s census of advertised-but-unreachable surfaces
- **Related:** `docs/decisions/2026-09-19_a-transport-receipt-is-not-an-acknowledgement.md`,
  `docs/decisions/2026-09-20_the-tail-read-is-the-offer.md`

## The defect, and why deleting the variant was not available

🔴 `PresenceState::Busy` was declared, rendered as the wire string `"busy"`,
and **constructed nowhere**. A client branching on the presence vocabulary had
one value that could never arrive.

⛔ The obvious repair — delete the variant — is refused by the oracle. §10.2:

> A role can be `available`, `busy`, `draining`, `offline`, `suspended`, or
> `unknown`; presence does not change enrollment.

Six states, `busy` among them. Removing it would take the code **out of §10.2
conformance in order to make a vocabulary honest**, trading a real requirement
for a cosmetic one. So the defect is not an over-declared variant; it is a
required state the deployment could not report.

## What counts as in flight

✅ Exactly `node_inbox_state.delivery_state = 'transport_received'` — the node
durably holds the command and has not finished it.

⭐ **The exclusions are the ladder's precedence, not a list anyone maintains.**
Each of these is excluded because a higher rung claims the row first:

| row | excluded because |
| --- | --- |
| `consumed` | a work result came back — the work is done |
| `dead_lettered` | quarantined, so it will never be worked; counting it would hold the node at capacity forever |
| `revoked` · `expired` | withheld from delivery; the node will not be given it |
| `queued` | never handed to a transport |
| `offered` | put on the wire and not confirmed held |

⛔ **`offered` is the one worth arguing rather than assuming.** A row the server
put on the wire and the node has not confirmed holding is not work the node
holds — that is exactly what the rung means — and counting it would leave a
node with a lossy connection permanently `busy` over rows it never received.

⭐ This is only derivable because of `migrations/0075`. Before it, the state
meaning *the node process durably holds this* was called `acknowledged`, and
reading a count of acknowledged-not-consumed rows as *in flight* would have
been an inference from a name that did not mean that.

## Why `draining` outranks `busy`

A node declaring `concurrency = 0` satisfies both readings of *cannot take
work*, so the two must be ordered. ⛔ Deciding by which arm happens to be
written first is how a precedence chain acquires a wrong answer nobody can see.

✅ **The chain already sorts by how durable the fact is**: no enrolment row, a
revoked certificate, a lapsed lease, a **declaration**, a **measurement of this
moment**. `draining` is a declaration and `busy` is a measurement, so
`draining` wins — and reporting `busy` for a node that declared no capacity
would tell a caller to wait for capacity that is not coming back on its own.

⛔ One consequence, stated rather than discovered: with `draining` above it, the
`busy` arm never sees `concurrency = 0`, so *at capacity* cannot be satisfied by
a node that declared none.

⚠️ And an **undeclared** concurrency never reads `busy` either, however much the
node holds. A node that never said what it can take cannot be reported at a
limit it never gave.

## A view column, not a fifth argument

All five readers of `presence_state` already select from `node_presence`, and
each already pays a correlated subquery for `concurrency`. `migrations/0079`
appends `in_flight` to that view: one definition is one place to be wrong
instead of five.

⚠️ `CREATE OR REPLACE VIEW` is legal here and it was checked rather than
assumed. This view **names** its columns, so the new one appends, which is the
one shape a replacement may take. `node_inbox_state` selects `i.*` and had to be
dropped and recreated for exactly that reason two migrations earlier — and a
control drives this one over a populated pre-`0079` database, because an
assertion about a migration is worth what drives it.

## What is not claimed

⚠️ **The delivery path is unchanged.** `concurrency` still gates delivery only
at **zero**; a node declaring two will still receive a third row. `busy` is a
presence report, not a limiter, and the book says so twice now that the number
is read at all.

## Verification

`node_channel` 41, `node_work` 12, `node_inbox` 11, `migration_upgrade` 8,
`profiles` 63, `regions` 3 — **138 passed, 0 failed**; nine library units on the
derivation itself.

⭐ **Falsified four ways, each verdict naming the assertion that caught it** —
`presence.rs` and `migrations/0079` restored byte-identical after every
mutation. Removing the `busy` arm, reading *at capacity* as strictly over, and
counting every inbox row as in flight each turn the live control red.

🔎 **Inverting the precedence is red against the LIBRARY units and green
against the database suites, and that split is worth knowing.** The
`draining`-versus-`busy` conflict exists only at `concurrency = 0`, which no
live fixture constructs — so the precedence decision is guarded by the pure
units alone. A future editor moving that arm will be caught, but only by
`cargo test --lib`.
