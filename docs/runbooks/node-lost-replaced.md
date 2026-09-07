# Runbook: node lost / replaced

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-2.5.3` · Date: 2026-09-07 · Profile: dev (trusted LAN, one node per role)
- Scope: a node (its machine, its journal, or its credential) is lost, destroyed,
  or replaced — everything from "the lease stopped renewing" to "the box burned down".
  The provider-outage and database-failover runbooks are separate (Phase 7+).

## Detection

- **Lease silence:** the node's lease/presence stops renewing — the server-side
  `lease_refusals` counter rises on stale-epoch renewals, and the inbox stops
  draining (the metrics surface: `GET /v1/admin/metrics`).
- **Dispatch silence:** dispatched attempts stop returning results; the run stays
  `dispatched` (inspect: `rb inspect runs`).
- **The operator knows:** SIGKILL, hardware loss, credential compromise suspicion.
- **The honest ambiguity signal:** after a kill, the surviving journal says
  `outcome_unknown` — the attempt may have reached the provider. That is the
  signal the recovery path must respect, NOT a bug to paper over.

## Authority

- Declaring the node lost + revoking its credential + replaying a dead-lettered
  attempt: `tenant_admin` (the dev principal's grant) — every such write is
  audited (the `.1.3` revocation verbs + the `.2.4` replay verb).
- Re-enrolling a replacement node: the role's credential path (the `.1.2.1`
  enrollment) — the new node is a NEW incarnation of the same role id.
- Reading everything: `rb inspect …` / `rb-journal` (read-only, no database surgery).

## Safe first actions

1. **Do not delete anything.** The node's journal/state dir is the evidence;
   copy it before touching it. The database is the truth — no manual row edits.
2. **Stop the bleeding:** revoke the lost node's credential (`rb node revoke`)
   so no new dispatch can ride the old channel (the `.1.3.1` suspended presence).
3. **Freeze the picture:** capture `rb inspect incarnations`, `rb inspect runs`,
   `rb inspect usage`, and the node's journal listing.
4. **Decide lost vs replaced** — a machine reboot is NOT a replacement: the same
   journal resumes (the `.1.2.2` rotation + the channel replay). Only a dead
   machine/credential is a replacement.

## Diagnostic queries

- `rb-journal --dir <node-dir> attempts` — every attempt boundary the journal
  recorded before the loss (which one was in flight).
- `rb-journal --dir <node-dir> events` — the outgoing events + ack cursor (what
  the server has and has not receipted).
- `rb inspect incarnations` — the §8.1 request facts of every incarnation.
- `rb inspect runs` / `rb inspect usage` — what was running, what is held.
- `GET /v1/admin/metrics` — `lease_refusals`/`dead_letters`/`handshake_refusals`
  for the loss's footprint.

## Containment

- Revoke the node cert (`rb node revoke`): the handshake refuses, presence is
  suspended, and the tenant revocation epoch bumps — every cached admission
  decision the dead node holds becomes stale (the `.1.5.2` dispatch gate).
- No new work is dispatched to the lost node; its inbox rows wait for replay.

## Recovery

- **Attempts in flight at the loss:** recover through the ambiguity policy —
  the attempt lands `outcome_unknown` (visible, bounded), and the human/channel
  decides: re-ask with `allow_possible_duplicate`, or leave it unresolved (the
  `.2.3` retry policy: never a silent retry of a possibly-completed call).
- **Dead-lettered work:** `rb node replay` (the `.2.4` decision-scoped retry
  re-arm) — only after the cause is understood, and only the named attempt.
- **Replacement node (the `.7.2` ritual — measured by the drill):**
  1. `rb node revoke` — the operator declares the loss; the old cert is fenced
     and the tenant's revocation epoch bumps.
  2. Issue a fresh enrollment token (`POST /v1/nodes/enroll-tokens`) and enroll
     the replacement with a NEW secret — the replacement is a NEW incarnation
     (same role id; the old certs stay revoked, the dev secret swaps).
  3. The fresh journal reconciles and the inbox tail replays from cursor 0.
  4. **The fence holds:** the re-delivered work carries the decision the LOST
     incarnation cached (epoch N), so the replacement's dispatch refuses
     FAIL-CLOSED against the current epoch (N+1) — no silent re-dispatch of the
     lost node's in-flight work — and the row dead-letters (auto-quarantine).
  5. `rb node replay` the dead-lettered work (the decision refreshes against
     the current epoch), then the replacement completes it.
- **Total machine loss with a live database:** the database is the durable
  truth; the ritual above. **Total loss including the database:**
  the `.4.1` restore exercise is the control — `scripts/backup.sh` dumps,
  `scripts/restore.sh` restores into an isolated database; a production RPO/RTO
  pair is a Phase-7 boundary, not invented here (SLO-3's record).

## Evidence preservation

- The node journal (WAL + `synchronous=FULL`, verified at open) — copy it
  before any replacement; it is the dispatch-boundary evidence.
- The server-side audit rows (the revocation, the replay, the denial rows) —
  the audit chain, never hand-edited.
- The demo's evidence bundle pattern (`target/demo/<ts>/evidence`) — the same
  shape any incident write-up should follow.

## Communication

- The operator reports the loss to the accountable owner (who declares it in
  the task tree), the incident's shape (what was in flight, what was held), and
  the chosen path (re-ask vs leave unresolved). The declaration is a tree leaf
  with its own acceptance checklist — incidents are work, not chat.

## Closure tests

- **The existing exercise:** the demo's SIGKILL beat (scenario step 7) — the
  node is killed mid-attempt, the restart recovers `outcome_unknown`, the
  challenge stays unresolved, and the audit reconstruction tells the whole
  story — runs on every guard pass (`scripts/demo_two_host.sh`, 34/34 checks).
- **The revocation beat:** the demo's revoke step — the revoked node reads
  suspended through the channel API.
- **The replay beat:** the node_channel suite (22 tests, incl. the replay pair)
  + the `.2.4` dead-letter/replay suite on every guard pass.
- **The total-loss beat:** the `.4.1` restore exercise on every guard pass.
- **The replacement drill (`.7.2`):** the guard's `node_replacement` suite runs
  the whole ritual on every pass — destroy the journal mid-ambiguity, revoke,
  replacement enroll, inbox replay, the stale-decision fence (no silent
  re-dispatch), the dead letter, the operator replay, exactly one fold. The
  Phase-7 game day then EXERCISES the runbook at scale (multi-node churn); it
  no longer has to build it.
