# Runbook: notification storm

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN)
- Scope: a flood of invitations, deliveries, or enrollment attempts — the
  §16.11 abuse vocabulary's "notification floods / invitation storms".

## Detection

- **The quota signal:** `quota_exceeded` refusals + the `quota_events`
  `denial` rows (the `.1.3.2` recorded denials — a storm refuses LOUDLY).
- **The inbox signal:** a node's inbox grows without the ack cursor moving
  (`GET /v1/nodes/inbox` + the delivery-state view).
- **The metrics:** the delivery counters + the authorization-denial deltas.

## Authority

- Quarantining: `tenant_admin` (`POST /v1/nodes/quarantine` — the reason is
  REQUIRED, a quarantine without a reason is a silent skip).
- Replaying: `tenant_admin` (`POST /v1/nodes/replay` — the decision-scoped
  re-arm).
- Pruning: `tenant_admin` (`POST /v1/nodes/inbox/prune` — the measured receipt).

## Safe first actions

1. **Quarantine the storm source** with the reason (the row fact) — the
   quarantined commands never re-deliver (the `.1.2.3` contract).
2. **Do not prune blindly:** the retention never deletes a quarantined row
   (the `.1.3.3` evidence rule) — the evidence survives the disposition.

## Diagnostic queries

- `SELECT count(*) FROM quota_events … WHERE kind = 'denial'` — the storm's
  recorded footprint.
- `GET /v1/nodes/inbox?node_id=…` — the queue depth + the delivery states.
- `GET /v1/admin/metrics` — the delivery + the denial counters.

## Containment

- The quarantine (the auto path: a node's dead-letter report quarantines its
  row WITH the reason).
- The per-tenant invite quota (the `.1.3.2` bound: 1000 invites/hour in the
  dev profile — a storm beyond it is refused + recorded).
- The per-thread fan-out caps (the Phase-3 storm controls).

## Recovery

- After the cause is understood: `POST /v1/nodes/replay` the affected
  commands (the decision re-arms against the current epoch) — the operator's
  explicit choice, never a sweep.
- The retention cleanup: the prune (the delivered, never the quarantined —
  the evidence stays).

## Evidence preservation

- The `quota_events` use/denial rows (the recorded storm).
- The quarantine rows with their reasons (the row facts).
- The delivery-state view (the before/after picture).

## Communication

- The operator reports the storm (the source, the recorded denial count, the
  quarantine reason) to the accountable owner.

## Closure tests

- The quota suite (the storm legs: the ceiling crossing records the denial,
  the slide releases) + the delivery-state suite on every guard pass.
- The quarantine suite (the retention-survival legs) on every guard pass.
