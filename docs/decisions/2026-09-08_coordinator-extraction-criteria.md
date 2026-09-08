# The coordinator extraction criteria — nothing scales until a measurement names the bottleneck (`PHASE-7.3`)

- Date: 2026-09-08 · Leaf: `PHASE-7.3` · Decision record (the ADR-002 extraction criteria, applied to the coordinator)

## Context

The `.3` lane's mandate is "horizontally scalable coordinator workers only
where measurements require them" (ADR-002's extraction criteria). The
census measured the state: the coordinator is a deliberate SINGLE-WRITER
design (ADR-002's named property — "the single-writer assumptions must be
retired before exposure, no exception"); the measurements are
catalogue-named-not-instantiated (the SLO record: the control-plane
ingress→commit latency, the worker throughput, the channel latency are
named families with NO workload measurement — the ONE empirical number is
SLO-5, the issuance baseline). No measurement names a bottleneck, so
nothing extracts — the criteria below turn that into a mechanical rule,
not a feeling.

## Decision

- **The extraction trigger is a MEASUREMENT, never a hunch.** Each
  single-writer seam names the exact load measurement that would justify
  its extraction; the measurement comes from the `.4` lane's load harness
  (the capacity tests). Until a measurement names a seam as the
  bottleneck at the target profile, the seam stays single-writer.
- **The seam map (the candidates + their trigger measurements):**
  (1) the aggregate write path — the per-tenant/aggregate primary-key
  serialization → the trigger: the ingress→commit p95 at the target
  concurrency; the horizontal form: the partitioned writers (the
  claim-first idempotency already rides the partition key). (2) the
  outbox worker — the single poller → the trigger: the delivery
  throughput (the items/sec) against the target delivery SLO; the
  horizontal form: the partitioned workers over the EXISTING lease +
  fencing (the WP2 machinery extracts, it does not rebuild). (3) the
  node channel — the per-node inbox + the handshake state → the trigger:
  the concurrent-node count at the target profile; the horizontal form:
  the node affinity (the per-node state is already keyed by the node).
  (4) the CA issuance — the measured baseline (p50 63 µs) is not a
  bottleneck; the trigger: the issuance p95 crossing the enrollment SLO.
  (5) the budget/authority evaluation — the in-tx checks → the trigger:
  the per-command evaluation share of the ingress→commit latency; the
  horizontal form: the same checks in the partitioned transaction.
- **The single-writer stays until the exposure profile names it.** The
  ADR-002 line stands verbatim: the retirement is a consequence of the
  G6/G7-qualified profile's measurements, never a preemptive rework.

## answers:

- **The extraction is measurement-gated, seam by seam**: five seams,
  five trigger measurements, zero extractions today — the criteria make
  the "only where measurements require" mandate checkable.
- **The horizontal forms reuse the shipped machinery**: the partitions
  ride the existing claim keys + the lease/fencing + the node keying —
  the extraction is a re-arrangement, not a rebuild.
- **The `.4` load harness is the criteria's input**: the capacity tests
  exist to feed THIS record — a seam whose measurement stays green is a
  seam that never extracts.
