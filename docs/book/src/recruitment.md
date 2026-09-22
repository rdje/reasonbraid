# Recruitment and automatic initiation

A thread does not need its participants named in advance. An initiator opens a
**recruitment call** describing who is eligible; enrolled roles answer it; the
initiator closes it and the server snapshots the panel it selected, with the
reasons. Separately, a node holding an explicit grant can **start a thread on its
own** when a server-side checklist says it should.

Both are the "ask a durable network without knowing who is online" path, and
both refuse rather than guess.

## The routes

```text
POST /v1/calls                        open a call on a thread
GET  /v1/calls/{call_id}              inspect one call
POST /v1/calls/{call_id}/respond      answer a call (the response vocabulary)
POST /v1/calls/{call_id}/close        close it and snapshot the panel
POST /v1/threads/auto                 a node initiates a thread itself
```

## Opening a call

The gate is `ThreadInvite` **on that thread**: a call rides the same invitation
machinery as naming a participant by hand (ADR-015), so opening one is not a
weaker act than inviting someone.

```bash
curl -s -X POST localhost:4310/v1/calls \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "tenant_id": "ten_0192…",
        "thread_id": "thr_0192…",
        "expression": {"all": [{"capability": "retention-policy"}]},
        "min_participants": 2,
        "max_participants": 5,
        "recommendations_allowed": true,
        "join_deadline": "2026-09-22T09:00:00Z",
        "expires_at": "2026-09-23T09:00:00Z"
      }'
```

- `expression` is the **eligibility expression** the server re-resolves against
  each respondent's current facts — not a list of names.
- `min_participants` must be at least 1 and `max_participants` at least
  `min_participants`; anything else is refused.
- `recommendations_allowed` decides whether a respondent may answer `recommend`
  and point at somebody else.

⛔ **Open calls are capped per tenant and per initiator.** Exceeding either
returns a typed `429` that names the limit it hit — the dev-scale storm control,
so one initiator cannot flood the directory.

## Answering a call

`POST /v1/calls/{call_id}/respond` takes one tagged response object, and the same
vocabulary rides the MCP `join_call` tool. The eight kinds, their members and the
strict decoding are documented once in
[Responding to an open call](profiles.md#responding-to-an-open-call) — the
respondent's side of the same exchange.

The server checks four things before recording one: the call is **open**, the
**join deadline has not passed**, the respondent is an **enrolled role**, and the
call's eligibility expression **still resolves** against the respondent's current
facts. An ineligible response is refused with the stage-1 reasons, so a
respondent is told why rather than silently dropped.

⚠️ Eligibility is re-resolved at response time, not at open time. A role whose
facts changed after the call was advertised is judged on the facts it has now.

## Closing a call and the panel snapshot

```bash
curl -s -X POST "localhost:4310/v1/calls/call_0192…/close" \
  -H 'x-reasonbraid-principal: hpr_0192…'
```

The **initiator or the tenant owner** may close a call, and the choice is
audited. Closing one that is not `open` is refused as an invalid transition, and
an unknown call is `404`.

Closing **snapshots the selected panel**: the joiners, ranked, capped at
`max_participants` — together with the **selection explanation**, which carries
each panelist's stage-1 reasons and stage-2 features. The explanation is stored
with the panel, so why this panel was chosen survives the call.

⛔ The snapshot is taken at close. Roles that join, change or become ineligible
afterwards do not alter it — that is what makes it a record rather than a view.

## A node initiating a thread itself

`POST /v1/threads/auto` is how a role starts work nobody asked it to start.

```bash
curl -s -X POST localhost:4310/v1/threads/auto \
  -H 'x-reasonbraid-principal: rol_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "tenant_id": "ten_0192…",
        "subject": "retention dispute 4471",
        "objective": "decide whether the evidence may be deleted",
        "topics": ["retention", "evidence"],
        "confidentiality_class": "internal",
        "budget_amount": 5.0
      }'
```

Only an **enrolled role** may call it — a human principal is refused — and it
takes the **explicit `thread:create:auto` grant**. Holding the ordinary
thread-creation authority is not enough: initiating without being asked is a
separate permission.

Before the initiation lands, the server evaluates the wake checklist itself:

| Check | What it asks |
| --- | --- |
| topic gate | do the role's **declared interests** cover the `topics`? |
| confidentiality match | does the role's clearance match `confidentiality_class`? |
| concurrency gate | is the role already running as many autonomous threads as it may? |
| spend bound | does `budget_amount` fit inside the grant's own bound? |

⭐ **The checklist is evaluated server-side, before the thread exists.** A node
cannot assert that it passed; failing any check refuses the initiation rather
than creating a thread and stopping it afterwards.

⛔ **Replies do not inherit the permission.** A child thread needs its own
`thread:create:auto` grant, so one authorized initiation cannot become a tree of
unauthorized ones.

### How far a chain of initiations can go (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.1`)

A role that starts a thread **because of** another thread names it:

```json
{ "tenant_id": "ten_…", "subject": "…", "objective": "…",
  "topics": ["retention"], "caused_by": "thr_…" }
```

The server records the new thread's place in the chain on the thread itself,
so `GET /v1/threads/{id}` shows it:

| field | meaning |
| --- | --- |
| `autonomous_depth` | 0 for a thread a person created; 1 for an initiation with no `caused_by`; otherwise one more than the cause's |
| `caused_by` | the thread named as the cause, or `null` |
| `autonomous_initiators` | every role that auto-initiated along the chain, this one included |

Three things are refused before anything is written:

| the initiation | answer |
| --- | --- |
| names a cause the role has not accepted participation in | `403` — a role cannot place itself in a chain it is not part of |
| comes from a role already on the chain | `429 storm_control` — *autonomous initiation cycle* (ROADMAP §10.7) |
| would be deeper than **3** | `429 storm_control` — *autonomous initiation depth 4 exceeds the maximum of 3* |

⚠️ **Why this matters now.** Until this check existed, the only thing stopping a
chain from running on (A starts a thread that wakes B, B starts one that wakes
C, and so on) was a defect: a role could auto-initiate only once per tenant,
because every later attempt replayed the first thread. That defect is owned by
`SIGNOFF-REPAIR.5.2`, and its repair is locked behind this check.

⚠️ **Causation is declared.** A role that names no cause starts a new chain at
depth 1, and the server cannot tell a genuinely spontaneous wake from an
omitted cause. The limit on how often a role may start a chain is the rate
check, which is owned by `SIGNOFF-REPAIR.11.4.7.2.1.5.3.2`. The maximum depth is a
development-profile constant; §14.1 makes it a budget dimension that a tenant
profile will own.
