# 2026-09-06_explicit-participants.md

## Context

`PHASE-1.3.1` (backlog 16 + 15) landed the explicit-participants contract: an
invitation is a PENDING offer with a lifecycle, the invited role must ACCEPT
before it may act, and the work dispatch moved from the invite transaction to
the accept transaction. The `.6.2` wiring had dispatched work at invite time
with an auto-accept on first contribution — implicit membership.

## Decision

- **The invitation IS the acceptance capability (offer/reserve).** Accept and
  decline authorize against a PENDING invitation naming the actor — the
  reservation of the slot is the right. A NEW `thread_invitation_respond` grant
  rides underneath as the "may participate in invitation flows" gate every
  role's default carries: grants say what a principal may do at all; the
  invitation says whether THIS offer is open. No grant could ever stand in for
  the offer, and the offer could never authorize a non-role.
- **Invitations are first-class projection facts, separate from participation
  state.** The participants map carries the lifecycle state (the core machine,
  now with `revoked` for admin removal); a NEW additive `invitations` map
  carries the offer's TIME facts (`invited_at`, `expires_at`). The separation
  keeps the projection growth `#[serde(default)]`-additive (old projections
  parse) and keeps the offer history visible after a decline/removal.
- **Expiry is DERIVED, never swept** — the lease-presence pattern: a pending
  offer past `expires_at` reads as `expired` in the inspection view and refuses
  accept/decline at the command boundary. No background sweeper, no stored
  `expired` flag, no expiry event (expiry is a time fact, not a transition —
  exactly like the channel lease).
- **The work dispatch rides the ACCEPT transaction.** The invite enqueues
  NOTHING; the accept transaction enqueues the contribute work item with its
  reservation (`work_{accept_event_id}`) — an accepted invitation exists iff
  its work does (the `.6.2` invariant, moved). Splitting the lifecycle leaf
  from the dispatch move would have left an incoherent interim (work arriving
  to a role that may no longer act), so they are ONE contract and ONE leaf (the
  decomposition was amended accordingly).
- **An invited role may not act until accepted.** The auto-accept on first
  contribution is gone: `thread.contribute` from an `invited` role is a typed
  `invalid_transition` naming the accept verb. Challenge/revise already
  required `accepted`.

## Consequences

- New wire ops/events: `thread.accept_invitation` /
  `thread.invitation_accepted`, `thread.decline_invitation` /
  `thread.invitation_declined`, `thread.remove_participant` (tenant_admin) /
  `thread.participant_removed`; the invite body gains the typed optional
  `expires_in_seconds`; the invite event body carries `invited_at`/`expires_at`.
- The core participation machine gained `Revoked` + the `Revoke` transition
  (invited or accepted → revoked); the grant registry gained
  `thread_invitation_respond` (wire-name canary extended first).
- The wiring suites and the two-host demo moved to the explicit contract:
  invite → pending; `rb thread accept` as the role; then the node starts. The
  demo's invite step no longer produces work.
- Re-invitation is allowed over any TERMINAL participation state
  (declined/expired/left/revoked) — the new offer overwrites the meta; an OPEN
  membership (invited or accepted) refuses a second offer.

answers:

- **A capability and a grant answer different questions.** The invitation
  answers "is this offer open to you?"; the grant answers "may you participate
  in invitation flows at all?" Conflating them would either grant the
  acceptance right to non-invitees (grant-only) or bypass the audited
  authorization flow (invitation-only). Both checks, two layers, one command.
- **Don't split a contract across leaves when the interim is incoherent.** The
  lifecycle change (invited roles may not act) and the dispatch move (work
  rides accept) only hold together: separating them leaves work arriving to a
  role that cannot accept it. The decomposition was amended the same day —
  decompositions are hypotheses; a contradiction is an amendment, not a
  workaround.
- **Derived expiry needs no event.** An expiry event would have to ride SOME
  command's transaction (the observing accept refuses — refused commands
  commit nothing). Deriving at read + enforcing at the boundary gives
  observability and enforcement without inventing a sweeper; the same shape as
  channel-lease presence.
- **Race tests assert the SNAPSHOT, not a winner.** Concurrent accept/remove
  has no predetermined winner (the head lock serializes, whichever lands
  first); the test asserts exactly one 200, exactly one transition event, and
  that the final snapshot matches the winner — and that the spent invitation
  refuses a late accept either way.
