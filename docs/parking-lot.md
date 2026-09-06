# Parking lot

Non-blocking ideas that must **not** enter `ROADMAP.md` v0.4.1 or the Phase 0
critical path. The master roadmap is frozen (`docs/decisions/2026-09-05_roadmap-v0.4.1-frozen.md`).
v0.5.0 cannot be opened from items here unless they cite Phase 0/1 executable
evidence.

## Convention

Each row needs:

- **Idea** — one sentence
- **Why not now** — freeze, sequencing, or unproven need
- **Revisit trigger** — a measurement, failure, or gate that would reopen it
- **Date parked** — absolute

An idea without a revisit trigger is not parked; it is dropped. Do not silently
promote a row into `ROADMAP.md` or a `PHASE-*` frontier.

## Parked

| Idea | Why not now | Revisit trigger | Date parked |
| --- | --- | --- | --- |
| Re-review `README_POLICY.md` against the newer fsmgen local revision (adds "Authority and provenance" + "Routing-pressure closure" template sections) | README policy already adopted + enforced here (`README-STABILITY`, root `README_POLICY.md` = scaffold 0.2.0 template revision); the routing-pressure machinery targets megabyte-scale neighbor sinks that do not exist in this repo (README ~2.6 KB, live docs capped), and the policy says origin copies are not upstreams — sync only by deliberate review | a live doc approaches its cap, or the README/policy body drifts from the neutral template | 2026-09-06 |
