# Lifecycle tables

**Status: draft — not normative.** Models the orthogonal lifecycles of the
system (`ROADMAP.md` §8.4: "one thread does not have one all-encompassing linear
state; linked aggregates evolve independently"). Backlog item 5: "model thread,
invitation, membership, attempt, proposal, publication, deployment, suspension,
and retraction."

Scope note: **thread, invitation/membership, provider attempt, and delivery**
are the G0 boundaries and are modeled in full below. **Proposal, decision,
approval, publication, deployment, suspension, and retraction** are
governance/policy lifecycles of later phases; they are sketched here for
completeness (backlog 5) but are not G0-gated and stay minimal.

Every transition carries an actor, authorization, precondition, idempotency
behavior, an emitted event, and a compensating/recovery action (§8.4). A
back-edge occurs only through an explicit new revision or action, never by
overwriting history.

## Thread (`ROADMAP.md` §8.4; minimal Phase 0 subset per `KICKOFF.md` §WP1)

| State | Meaning |
| --- | --- |
| `draft` | created but not yet open to participants |
| `open` | accepting participation and contributions |
| `paused` | temporarily halted by an authorized actor |
| `closing` | draining toward a terminal outcome |
| `closed` | a terminal outcome was produced and recorded |
| `cancelled` | terminated without an outcome |
| `expired` | its expiry/decision deadline passed |

Phase 0 (WP1) implements only `open`, `closing`, `closed`, `cancelled`; the
remaining states arrive with the workflow engine.

## Participation / invitation & membership (`ROADMAP.md` §8.4; §10.5–§10.6)

| State | Meaning |
| --- | --- |
| `eligible` | matches the call's deterministic eligibility |
| `advertised` | the call was offered (open call) |
| `invited` | explicitly or capability-recruited |
| `joined` | accepted and participating |
| `observing` | joined without contributing |
| `declined` | explicitly refused |
| `deferred` | postponed with an `until` |
| `left` | withdrew after joining |
| `revoked` | removed by an authorized actor |

Recruitment responses (§10.5) map onto these: `join`→`joined`,
`observe`→`observing`, `decline`→`declined`, `defer`→`deferred`. Phase 0 (WP1)
implements the minimal subset `invited`, `accepted`, `declined`, `expired`,
`left`.

## Provider attempt (`ROADMAP.md` §8.4, §11.3)

The attempt state machine records state before and after every irreversible
boundary, so a crash after a possible billable dispatch yields `outcome_unknown`
rather than a blind retry.

| State | Meaning |
| --- | --- |
| `planned` | intent recorded, not yet reserved |
| `reserved` | budget held for this dispatch |
| `dispatching` | the provider call has been sent |
| `accepted_known` | provider acknowledged with a durable ID |
| `streaming` | partial output arriving |
| `completed` | terminal success |
| `failed_known` | terminal failure, confirmed by provider |
| `cancelled_known` | cancellation confirmed |
| `outcome_unknown` | whether a billable call happened is unknowable |

Phase 0 (WP1) uses the reduced set `prepared`, `dispatched`, `completed`,
`failed_before_dispatch`, `outcome_unknown`, `reconciled`. `outcome_unknown`
resolves via provider status lookup, an authorized possible-duplicate retry, or
human/owning-agent adjudication — never silent retry (§11.3).

## Delivery / inbox (`ROADMAP.md` §10.6)

| State | Meaning |
| --- | --- |
| `queued` | the inbox entry exists |
| `offered` | presented to a transport |
| `transport_received` | the transport accepted it |
| `acknowledged` | the consumer acked it |
| `consumed` | processed (and deduplicated) |

Out-of-path exits: `expired`, `revoked`, `dead_lettered`. Transport receipt does
not mean an agent read or acted; acknowledgement semantics are explicit per
event type.

## Sketches for later phases (not G0-gated)

These are minimal and will be completed by their owning phase; they are listed
only so the full orthogonal set is named once (`ROADMAP.md` §8.4, §15.6).

| Lifecycle | States (sketch) | Owning phase |
| --- | --- | --- |
| Proposal revision | `draft` · `submitted` · `under_review` · `withdrawn` · `superseded` | 5 |
| Decision | `forming_electorate` · `collecting_actions` · `decided` · `deadlocked` · `no_quorum` · `cancelled` | 5 |
| Approval | `pending` · `claimed` · `changes_requested` · `approved` · `rejected` · `expired` · `cancelled` | 6 |
| Publication | `planned` · `staged` · `verified` · `effective` · `superseded` · `retracted` · `failed` | 6 |
| Deployment | `planned` · `offered` · `applying` · `applied` · `rejected` · `waived` · `drifted` · `rolled_back` · `failed` | 6 |
| Correction (suspension / retraction / supersession) | distinct operations, not one state (§4.7) | 6 |

Correction is a set of *operations*, not a single lifecycle: emergency
suspension (temporary, auto-expires), permanent retraction (invalidates from an
effective point), supersession (adopts a replacement and links history), waiver
(time-bounded exception), and historical correction (adds metadata without
deleting the record) — `ROADMAP.md` §4.7.
