# Accountable owners: architecture decisions and release/security gates

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** director decision; leaf `PHASE-0.0.6`
answers: who is accountable for final architecture decisions and for release/security gate records?

## The fact / decision

Richard DJE is the accountable owner for **both** Phase 0 roles:

1. final architecture decisions, and
2. release/security gate records.

One person fills both roles; the split is by function, not by person.

## Why

`KICKOFF.md` WP0 requires "one person is accountable for final architecture
decisions and one for release/security gate records, even if the same small
team fills several implementation roles." The director chose a single person
for both roles while the repository is private and pre-clearance. Naming a
person (rather than a title) is exactly what `PHASE-0.0.6` acceptance requires,
and what `docs/risks.md` was waiting for ("Owner roles are titles until
`PHASE-0.0.6` names people").

## How to apply

- `docs/risks.md` owner roles resolve to Richard DJE (architecture + security).
- Gate records, ADRs, and evidence reports name Richard DJE as the accountable
  sign-off for architecture decisions and release/security gates.
- If the team grows, supersede this record with distinct owners rather than
  silently re-assigning; a separation-of-duties split needs a new decision
  record, not an edit of this one.
