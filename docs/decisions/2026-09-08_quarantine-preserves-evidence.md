# The quarantine preserves the evidence — the retention never deletes a quarantined row (`PHASE-7.1.3.3`)

- Date: 2026-09-08 · Leaf: `PHASE-7.1.3.3` · Decision record (the §16.11 quarantine-preserving-evidence rule)

## Context

ADR-034 fixed the stance: "the quarantine preserves the evidence — a
quarantined node/thread keeps its records (the tombstone doctrine, applied to
the abuse cases)". The `.1.3` census measured the shipped quarantine: the
explicit operator quarantine + the node's dead-letter auto-quarantine are
UPDATE-only row facts (`quarantined_at` + `quarantine_reason`, migration
0010) — no quarantine path deletes. The census then measured the RETENTION
path and found the gap the rule exists to close: `prune_node_inbox` deletes
every row with `acknowledged_at IS NOT NULL AND acknowledged_at <= cutoff` —
and a dead-lettered row is precisely a row the node DELIVERED (acknowledged)
and then reported dead — so the disposition DESTROYED the evidence. The rule
below articulates the contract and closes that gap.

## Decision

- **The quarantine is a ROW FACT.** Quarantining writes `quarantined_at` +
  `quarantine_reason` on the inbox row; the row, its payload, and the
  event-log history behind it stay — the evidence is the record, never a
  side table.
- **The retention never deletes a quarantined row.** The prune's DELETE
  gains `AND quarantined_at IS NULL`: the preservation SURVIVES the
  disposition. A quarantined row is only removed by an explicit,
  authorization-recorded operator action aimed AT that row (a future
  evidence-review disposition), never by the age-based sweep.
- **The replay re-arm clears the MARK, never the evidence.** `POST
  /v1/nodes/replay` clears `quarantined_at`/`quarantine_reason` on the
  target row — the delivery-state fact changes (the retry re-arms); the
  row, the payload, and the events stay. The re-arm is a deliberate
  operator decision under the authorization record (the audit names the
  operator and the decision), never a sweep.
- **The disposition vocabulary stays explicit**: the prune is the ONLY
  age-based removal, it is operator-invoked, and its response is the
  measured receipt (before/deleted/after in ONE transaction).

## answers:

- **The preservation rule is mechanical, not aspirational**: the prune's
  quarantine exclusion is the enforceable line — a row with a
  `quarantined_at` fact cannot be removed by the retention path, period.
- **The tombstone doctrine's honest limit stands**: the dev profile keeps
  the evidence on the row itself; an archive/export disposition (moving the
  evidence, never deleting it) is the named deferral with the
  Internet-profile trigger.
- **The re-arm is a disposition, not a destruction**: clearing the mark
  changes the delivery state; the evidence (the row + the event log + the
  node's journal) survives the re-arm by construction — no path in the
  quarantine vocabulary deletes the cited evidence.
