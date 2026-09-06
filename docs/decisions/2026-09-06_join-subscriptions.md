# 2026-09-06_join-subscriptions.md

## Context

`PHASE-1.3.2` (backlog 14's simple-subscription sliver + 16) landed
`thread.join` and made the thread's participant rules load-bearing: the join
door and the invite door are now ENFORCED at the command boundary, not just
recorded. Filtered delivery by eligibility stays with Phase 3's directory.

## Decision

- **`thread.join` is the self-request path — no invitation, no reservation.**
  A thread whose `allow_join_requests` is on admits the role directly as
  `accepted` (the event `thread.participant_joined` records `via:
  "join_request"` so the path is auditable). The joiner needs the ordinary
  `thread_contribute` grant — a subscription is an act of contribution, not a
  new authority. A closed door, and a second join over an open membership, are
  typed refusals.
- **`allow_explicit_invites=false` is enforced at the invite boundary.** The
  typed rule from `.1.1.3` became load-bearing: the invite arm refuses on a
  join-only thread (the join door still works). Rules recorded but not
  enforced are prose; rules enforced at the command boundary are doctrine.
- **The listing surface is the existing inspection view.** Participants render
  with their full state (invited/accepted/declined/expired/revoked) and the
  `invitations` meta carries the offer facts — no new listing machinery; the
  subscription facts ride the projection.
- **The race test taught a domain fact before it proved the lock.** The first
  shape raced accept vs REMOVE and both succeeded — they are COMPATIBLE
  transitions (the serialized order accept-then-revoke is legitimate; the
  snapshot is `revoked` either way, and a revoked role's late result folds to
  a stored rejection). The conflict pair is accept vs DECLINE (both consume
  the same pending offer): exactly one 200. The lock serializes; the DOMAIN
  decides which transitions conflict.

## Consequences

- New wire op/event: `thread.join` / `thread.participant_joined`; CLI verb
  `rb thread join`. The fourth `invitations` test covers the door, the
  enforcement, the double-join refusal, and the join-then-act path.
- `.1.3` (invitation/subscription semantics, backlogs 15/16) is complete.

answers:

- **Compatible transitions are not a serialization bug.** Two concurrent
  commands that both return 200 can be correct — the question is whether their
  serialized order is a legitimate sequence and whether the final snapshot
  matches the last winner. A race test must first establish WHICH transitions
  conflict in the domain; asserting exactly-one-winner on a compatible pair
  asserts a fiction.
- **A typed rule becomes doctrine the day a command boundary enforces it.**
  `allow_join_requests`/`allow_explicit_invites` were recorded for a year of
  commits; the refusals land in this leaf. Until the enforcement test exists,
  a rule field is a suggestion.
- **The self-request path needs no reservation machinery.** An invitation
  reserves a slot (offer/reserve); a join requests entry and is admitted or
  refused in one command — there is nothing pending to reserve against. The
  symmetry that would have added join-tokens was scope creep.
