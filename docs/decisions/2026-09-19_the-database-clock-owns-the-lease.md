# The database clock owns `node_leases.lease_expires_at`

- Date: 2026-09-19
- Status: accepted
- Owner: `SIGNOFF-REPAIR.4.2.3.1`
- Related: `SIGNOFF-REPAIR.4.2.3` (which met this and deliberately did not bundle
  it), `SIGNOFF-REPAIR.4.1.5` (which added the third writer after `.4.2.3`
  counted two), `SIGNOFF-REPAIR.3.4.3.1.2` (the node's clock offset, measured
  against `clock_timestamp()`), `docs/book/src/node-channel.md`.

## The question

`lease_expires_at` was WRITTEN as `Utc::now() + LEASE_TTL` — the server process
clock — and READ by SQL comparisons against the database clock. With the two
apart by `S`, a lease is observably live for `LEASE_TTL + S` (process ahead) or
`LEASE_TTL - S` (behind). The published 60 s was a nominal TTL: no reader got it.

`.4.2.3.1` offered two ways out — one clock owns the column, or the book's TTL
sentence carries the skew term — and required the decision to name which and why.

## What was measured first

**Three writers, not two.** `.4.2.3`'s census recorded two (`issue_lease`,
`renew_lease`) and was true when written; `.4.1.5` (REPAIR-0167) added a third
afterwards — the replacement's `UPDATE node_leases SET lease_epoch = …,
lease_expires_at = $2`, which ends the replaced machine's session. A number
restated from a census that has since moved: `a-restated-number-needs-a-producer`.

**Five comparison sites, on two clocks.** `node_presence.online` (migration
0009, carried through 0012 and 0017), `renew_lease`'s own `WHERE` and
`classify_renewal_refusal` read `now()`; `verify_fencing` and
`verify_fencing_in_tx` read `Utc::now()`.

**The node never reads the field.** `git grep -n "\.lease_expires_at" --
crates/reasonbraid-node` returns zero uses against two struct-field
declarations, so changing what the handshake returns changes no node behaviour.
The blast radius the leaf asked to be measured before choosing is empty.

🔴 **And the finding that decided it: one handshake response carries two
instants on two different clocks, with nothing saying so.** `HandshakeResponse`
returns `lease_expires_at` (process clock) beside `server_time`, which
`epoch_and_server_time` reads as `clock_timestamp()` — the database's.
`server_time` is the anchor `.3.4.3.1.2` gave the node to correct every server
instant against, and that leaf's own record says the offset "applies to every
instant, including `lease_expires_at` when it starts being read". Applying it
would have been WRONG, because the field was not in the terms of the clock the
product publishes as its own.

## The reproduction

The leaf forbade writing up a skew that had not been driven (`.3.4.3.1.3`'s
prohibition): the two clocks are the same host in every fixture this project
has. `issue_lease`/`renew_lease` take the server's own instant as a parameter,
and that parameter is the seam — it is what a process whose clock is wrong would
pass. Driven ten minutes ahead, against the unrepaired product:

```
.4.2.3.1 issue_lease with the process clock 10 min ahead: 660.0 s of lease left on the DATABASE clock
.4.2.3.1 renew_lease with the process clock 10 min ahead: 660.0 s of lease left on the DATABASE clock
test result: FAILED. 38 passed; 2 failed
```

An eleven-fold overrun of the published TTL, on both writers, with the other 38
controls green — so the two controls discriminate exactly this defect. Each
asserts `last_seen_at` FIRST: that column is the server's own observation, so it
carries the full injected skew and proves the injection LANDED
(`an-injection-must-be-shown-to-land`).

## Decision

**The database clock owns the column — it is produced by the database and every
liveness comparison is made by the database.** Three reasons, in order:

1. **It is the clock the product already publishes as its own.** `server_time`
   is `clock_timestamp()`. A column the wire hands out in *another* clock is one
   the node cannot correct, and correcting it with the offset it has would make
   the error worse rather than better.
2. **It is the only clock every reader can share.** `node_presence.online` is a
   VIEW — the database evaluates it and there is no process there to ask. Each
   server process, by contrast, has its own clock, so a process-written TTL
   means one lease duration per replica.
3. **It removes the failure mode instead of describing it**, which `.11.4.5.3`
   prefers and which the second option cannot do: a sentence in the book cannot
   make two replicas agree.

**Which database time function, derived per site rather than chosen once.**
`now()` is `transaction_timestamp()`. Where the statement IS its transaction it
is the database clock, and it is used: `issue_lease`, `renew_lease` (whose
single `now()` both checks the old expiry and grants the new one, so the check
and the grant share one reading rather than merely agreeing about the clock) and
`verify_fencing`. Where the statement runs inside a longer transaction, `now()`
is that transaction's START — measurably stale, because `events` takes the
tenant guard's shared mode first and blocks behind a revocation holding the
exclusive mode. Those two sites use `clock_timestamp()`: `verify_fencing_in_tx`,
where `now()` would admit a lease that lapsed during the wait — `.4.2.3`'s
revival defect re-entering through the clock instead of through the write — and
the replacement writer, where `now()` would record the session as having ended
before the work that ended it.

**`last_seen_at` and `issued_at` stay the server's own observation.** Nothing in
`crates` or `migrations` compares them; they are records, not predicates. It is
also what leaves a fixture able to drive a disagreeing process clock on one
host, and a property with no such seam is one no control can falsify.

## What is NOT claimed

⚠️ **Three of the five sites are repaired under the rule and are not falsifiable
by any control this project can write**, and they are labelled rather than
covered: the replacement writer samples its own instant internally, and both
admission checks read a clock that on a fixture host is the same clock. What
would falsify them is two hosts with a driven offset, which no fixture here has.
`.3.4.3.1.1` measured this server's two clocks agree within a second, so the
live exposure is small — but "small on this host" is what the repair replaces
with "zero by construction".

⛔ **No clock security.** This makes one clock authoritative for one column. A
database whose own clock is wrong is not addressed and cannot be by this change.

## Alternatives rejected

- **Keep the process clock and restate the TTL with its skew term** (the leaf's
  second option). The honest restatement would have had to say the instant is in
  a clock the product does not publish — while the same response publishes
  `server_time` from the other one. It also documents a per-replica TTL rather
  than removing it.
- **Move the readers to the process clock instead** — one clock, the other one.
  Refused by the view: `online` is evaluated inside the database, which has no
  process instant available to it.
- **Leave the writers and have the node correct the field through its existing
  offset.** Measured to repair nothing: the node does not read the field. It
  would also leave the server's own five readers split across two clocks.
- **`now()` at all five sites**, for one uniform spelling. Refused by the
  measurement above at `verify_fencing_in_tx`.
