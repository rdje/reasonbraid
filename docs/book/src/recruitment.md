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
        "idempotency_key": "init-4471-1",
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

**The grant's bounds are read from the grant that admitted the call**
(`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.2`) — the one the authorization record
names — never from the largest value across the role's grants. An expired or
not-yet-valid grant cannot admit, so it cannot raise a bound either. The bounds
a grant may carry, declared at enrolment ([authority](authority.md#enrollment-boundaries-and-grants)):

| bound | on the grant | how the initiation reads it |
| --- | --- | --- |
| spend | `spend_limits.amount` | `budget_amount` may not exceed it; a grant with no limit admits no budgeted initiation |
| topic | `auto_bounds.topics` | every declared topic must be in the list, **and** in the role's own `interests` — two declarations by two parties, both apply |
| depth | `auto_bounds.max_depth` | the deepest chain position admitted; absent means the site ceiling of 3 |
| rate | — | the role's `initiator` quota row (see below), not the grant |
| audience, side-effect | — | not yet carried: `SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.3` and `.4` |

`idempotency_key` names **this** initiation. Sending the same key again — a
retried delivery — returns the thread the first delivery created, marked
`"replayed": true`, and a refused initiation is replayed the same way. A new
initiation carries a new key. An empty key is refused (`400 invalid_command`),
because a shared key would make every later initiation a replay of the first —
which is exactly what happened until `SIGNOFF-REPAIR.5.2` was repaired.

Before the initiation lands, the server evaluates the wake checklist itself:

| Check | What it asks |
| --- | --- |
| topic gate | do the role's **declared interests** cover the `topics`, and does the admitting grant's `topics` bound, when it has one? |
| confidentiality match | does the role's clearance match `confidentiality_class`? |
| concurrency gate | has the role declared `concurrency: 0` — winding down, so it initiates nothing? |
| wake policy | is the role `manual_only` — woken by no delivery and never initiating on its own? |
| operating hours | is the server's clock, in UTC, inside the role's `operating_hours` window? |
| spend bound | does `budget_amount` fit inside the **admitting** grant's `spend_limits`? |

⭐ **The checklist is evaluated server-side, before the thread exists.** A node
cannot assert that it passed; failing any check refuses the initiation rather
than creating a thread and stopping it afterwards. The three availability rows
are decided by the same evaluator that holds delivery to the node
([the wake gate](node-channel.md#the-wake-gate-three-declarations-hold-delivery)),
so a role that may not be woken may not wake itself either; the refusal is a
`403` whose message names the hold.

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
| would be deeper than the ceiling in force — the admitting grant's `max_depth`, else **3** | `429 storm_control` — *autonomous initiation depth 4 exceeds the maximum of 3* |

⚠️ **Why this check came first.** Before it existed, the only thing stopping a
chain from running on (A starts a thread that wakes B, B starts one that wakes
C, and so on) was a defect: a role could auto-initiate only once per tenant,
because every later attempt replayed the first thread. That defect
(`SIGNOFF-REPAIR.5.2`) is repaired — the key is per initiation now — and the
repair landed together with the initiation quota below, so that at no point was
autonomous initiation bounded by nothing.

⚠️ **Causation is declared.** A role that names no cause starts a new chain at
depth 1, and the server cannot tell a genuinely spontaneous wake from an
omitted cause. What bounds how often a role may do that is the initiation quota
below. The maximum depth is a development-profile constant; §14.1 makes it a
budget dimension that a tenant profile will own.

### How often a role may initiate

Every role carries an **initiation quota** from the moment it is enrolled or
imported from a card: a windowed ceiling on `POST /v1/threads/auto`, the
`initiator` scope of the same usage-quota machinery that bounds invitations, MCP
writes and acquisitions (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.1`, ROADMAP §11.5's
*rate* bound). The development default is 1000 initiations per hour per role —
the shape of the bound, not a measured figure.

| the initiation | answer |
| --- | --- |
| within the window's ceiling | `200`; one `use` is recorded against the role's quota |
| at the ceiling | `429 quota_exceeded` — *the quota is exhausted: 1000 uses within 3600s — the denial is recorded* |
| from a role with no `initiator` quota row | `503 quota_unconfigured` — the surface fails closed rather than admitting an unbounded role |

The check runs inside the creation's own transaction, after the idempotency
claim and the authorization: a replayed key consumes nothing, a refused grant
consumes nothing, and a recorded `use` always has a thread behind it. A refused
initiation is stored under its key like any other refusal, so redelivering it
replays the `429` without recording a second denial. A person's ordinary
`thread.create` is not counted — the bound is on autonomous initiation only.

The bound is a row in `usage_quotas`, like the other four scopes, and an
operator changes it there:

```sql
UPDATE usage_quotas SET ceiling = 60
 WHERE scope_kind = 'initiator' AND scope_id = 'rol_0192…';
```

Roles that existed before this bound received the default row when the server
was upgraded, so no role is left without one.
