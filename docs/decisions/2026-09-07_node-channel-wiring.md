# Node-channel wiring and the two-host demonstration (PHASE-0.6.2)

- Date: 2026-09-07
- Status: accepted
- Owners: Richard DJE (engineering + product)
- Scope: WP6 second half — wiring the `.6.1` control surface into the WP3 node
  channel (server→inbox dispatch, node result→thread contribution) and the
  reproducible two-host crash/reconnect demonstration (`KICKOFF.md` WP6,
  `ROADMAP.md` §26.1 Demonstration A).

## Decisions

1. **A node id IS the agent role wire id it serves** (`NODE-BINDING-001`, dev
   profile). The demo profile has no directory: the node presents `rol_…` as its
   channel identity, the server synthesizes the contribution's principal from
   that id, and dispatch targets the role id as the inbox key. One node = one
   role until the Phase 1 directory replaces the rule. Honest and explicit; the
   authenticated streaming profile + workload identity stay deferred (WP7).

2. **Dispatch rides the command transaction** (`DISPATCH-001`). An accepted
   `thread.invite` / `thread.challenge` enqueues the work item into the target
   node's inbox in the SAME transaction as the thread event — an invitation
   exists iff its work does. The work item's command id is `work_{event_id}`,
   correlating the work with the event that produced it.

3. **Reserve before dispatch, at the server, best-effort with a visible denial**
   (`DISPATCH-002`). The dispatch transaction reserves `WORK_RESERVATION`
   (1 call, 2k/2k tokens, 120 s wall-clock) against the thread ceiling. When the
   ceiling refuses, the work item is STILL enqueued — without a reservation,
   with the denial reason — and the denial row is committed. The node's budget
   gate then refuses `failed_before_dispatch` BEFORE any provider contact: the
   denial is visible and bounded at both boundaries, never a silent skip and
   never an unreserved dispatch.

4. **A node result folds into the thread through the same claim → authorize →
   validate → apply flow** (`RESULT-001`), with the idempotency key = the work
   item's inbox command id. Duplicate transport produces one domain effect at
   two independent layers: the `node_events` receipt dedupe (same event id →
   `accepted:false`) and the idempotency claim (redelivered work → replay of the
   ORIGINAL result). A rejected application (e.g. the thread closed) is STORED
   as the work command's idempotent rejection while the receipt still commits —
   the node did emit the event; the domain refused it.

5. **The revise work item targets the CHALLENGE event, not the contribution**
   (`RESULT-002`). First implementation dispatched the challenged contribution's
   id as the revise target; the live-PG suite caught it (`invalid_command:
   revision target … is a contribution, not a challenge`). The domain's
   `thread.revise` targets a challenge; the contribution's id is only the
   author-lookup key.

6. **No silent retry: the worker executes a work item only when its attempt is
   absent or `prepared`** (`RETRY-001`). `dispatched`/`outcome_unknown` waits for
   proof or adjudication; terminal states are done. A crash after dispatch
   recovers `outcome_unknown` (WP3) and the handshake's `NeedsAdjudication`
   keeps it bounded and visible.

7. **Settlement rides the result transaction** (`SETTLE-001`). The reservation
   settles with the node-reported usage in the same transaction as the
   contribution (idempotent — a replay settles nothing twice).

8. **The demonstration is the acceptance test.** `scripts/demo_two_host.sh`
   drives the real binaries with real `SIGKILL` kill points (server restart,
   node killed after dispatch, duplicate delivery re-POSTed verbatim, budget
   exhaustion) and asserts every KICKOFF WP6 acceptance point by grep-able
   evidence in a bundle under `target/demo/<run-id>/`. It runs in the focused
   gate (`scripts/run_pg_tests.sh`, `make demo`) and CI. Limitation recorded in
   every bundle: the fake adapter (deterministic, no tokens); the real-harness
   leg stays `RB_LIVE_CODEX=1`-gated.

## Wire contract (work items, `PHASE-0.6.2`)

- Inbox payload: `{kind: contribute|revise, agent_role, subject, objective,
  target_event_id?, reservation: ReservationReference|null, reservation_reason?}`.
- Node `work_result` event: `{kind: work_result, work_kind, command_id,
  reservation_id, attempt_id, content (verbatim chunks), usage}`.
- The node journal treats exactly `kind ∈ {contribute, revise}` as thread work;
  `rb-journal events` exposes the emitted events (original ids + payloads).

## Bugs the suite/demo caught (all fixed with regression coverage)

- The revise-target inversion (decision 5).
- `node_work` test-harness race: the table purge ran BEFORE the suite mutex —
  reordered, matching `command_api`'s correct pattern.
- Demo script: `$$` inside a subshell is the SCRIPT's pid — `node_kill` killed
  the demo itself; pidfiles now capture `$!` (local) / remote `$!`.
- Demo script: `wait_for` timeout returned nonzero under `set -e`, aborting
  before the failure summary; timeouts now record the FAIL and continue.
