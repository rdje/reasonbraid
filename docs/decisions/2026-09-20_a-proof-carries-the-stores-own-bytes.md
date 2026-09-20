# A proof carries the store's own bytes, not a value that agrees with them

- Date: 2026-09-20
- Status: accepted
- Leaf: `SIGNOFF-REPAIR.11.30` (`REPAIR-0319`)

## The decision

When the server hands a node a value the node will later **verify against the
ledger** — a reservation window, a decision instant, any expiry that gates an
action — that value is read back from the row with `RETURNING` or `SELECT`. It is
never the in-process expression that was bound to the write, even when the two are
the same quantity.

## Why

`crates/reasonbraid-server/src/budget.rs` bound `at + held_for` to the insert and
put the *same Rust expression* in the `ReservationReference`. PostgreSQL
`TIMESTAMPTZ` is microsecond-precision, so the row truncated it. The held-amount
query stops counting an active reservation at `r.expires_at > $2`, reading the
**stored** column — so the ceiling re-lent the capacity up to 999 ns before
`ReservationReference::applicable_at` stopped honouring the proof.

Two values that are "the same instant" are not one value. A store has a type, and
a type has a precision; the moment a quantity crosses into the store, the store's
copy is the authority and every other copy is a derivation.

The channel already did this correctly elsewhere: `decided_at`, the other instant
a node verifies under §14.3, is `clock_timestamp()` sampled inside the authorizing
transaction and read back with a `SELECT` (`.3.4.3.1.2`). The reservation was the
last proof still dated from a Rust clock.

## What this rules out

A comment asserting the invariant. The site carried one — *"one value, written
once, so the proof the node verifies and the ledger that lends cannot disagree"* —
while the code maintained no such thing. The `RETURNING` clause is the mechanism;
the comment is not.

## The control it requires

A test that compares a carried instant to its row must **choose** an input with
sub-microsecond digits. Comparing two values that both come from `Utc::now()`
makes the outcome a property of the host: on macOS `CLOCK_REALTIME` is
microsecond-granular (20,000 consecutive samples, zero nonzero sub-microsecond
remainders), so the truncation cannot occur and the assertion is unfalsifiable.
See `[[2026-09-20_a-declaration-outranks-a-measurement]]` for the sibling shape,
and `[[2026-09-20_a-proof-of-allowance-must-carry-its-own-window]]` for why the
reference carries a window at all.
