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
GET  /v1/calls/offered                the calls offered to the calling role
GET  /v1/calls/{call_id}              inspect one call
POST /v1/calls/{call_id}/respond      answer a call (the response vocabulary)
POST /v1/calls/{call_id}/close        close it and snapshot the panel
POST /v1/threads/auto                 a node initiates a thread itself
```

## Opening a call

The gate is `ThreadInvite` **on that thread**: a call rides the same invitation
authority as naming a participant by hand (ADR-015), so opening one is not a
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
- `thread_id` names a thread **in `tenant_id`**. A thread that does not exist
  there, or belongs to another tenant, answers `404 scope_hidden` — the same
  answer every thread view gives — so a call is never opened on nothing
  (`SIGNOFF-REPAIR.5.2`).

⛔ **A call on a thread a role started on its own is bounded by that role's
grant** (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.3`). The thread records the grant
that admitted its initiation as `initiating_grant`, and when that grant declares
`auto_bounds.audience: "tenant"`, a call whose eligibility `scope` is `network`
is refused with a `403` naming the bound. `"network"` admits any scope, and a
grant with no audience bound leaves the call as free as one on a person's
thread.

⛔ **A `network`-scope call is also offered across a federation agreement**
(`SIGNOFF-REPAIR.5.3.5.1.2`). When the expression's `scope` is `network`, the
offers reach matching subscribers in every tenant that holds the **effective
directory-visibility agreement** with the call's tenant — both directions
accepted, both carrying `directory_visibility`, neither expired — in the same
statement and the same transaction as the tenant's own offers. A `tenant`-scope
call, a one-sided or revoked agreement, or an agreement without
`directory_visibility` offers nothing outside the tenant, which is ADR-026's
opt-in: the remote recruitment is agreement-scoped, never the default. A
federated subscriber can list the offer but cannot yet respond to it — its
response is refused until `SIGNOFF-REPAIR.5.3.5.2` records it as a join request
for the call's tenant to resolve by importing the role's card.

⛔ **Open calls are capped per tenant and per initiator.** Exceeding either
returns a typed `429` that names the limit it hit — the dev-scale storm control,
so one initiator cannot flood the directory. The refusal is recorded (below).
The caps are decided under a per-tenant serialization, so two opens racing at
the cap admit exactly one (see *Each transition is one transaction*).

### Every storm-control refusal is recorded, and an operator can list them

Until `SIGNOFF-REPAIR.11.4.7.2.1.5.3.4`, a `429 storm_control` reached only the
caller: the two fan-out caps here and the cycle and depth controls on
[autonomous initiation](#how-far-a-chain-of-initiations-can-go-signoff-repair1147215331)
answered and stored nothing. That mattered beyond bookkeeping: the roadmap's
per-origin and global circuit breakers were deferred until *the first
multi-tenant storm observed*, and nothing could observe one.

Now every storm-control refusal is written before it is answered — by the one
function that spells the code, so a control cannot refuse without recording —
and listed newest first:

```bash
curl -s 'localhost:4310/v1/admin/storm-refusals?tenant_id=ten_0192…' \
  -H 'x-reasonbraid-principal: hpr_0192…'
```

```json
{ "tenant_id": "ten_0192…", "limit": 500,
  "refusals": [
    { "refusal_id": "srf_…", "initiator": "hpr_0192…",
      "control": "open_calls_per_initiator", "limit_value": 4, "target": "thr_0192…",
      "message": "the initiator's open-call fan-out limit (4) is reached",
      "refused_at": "2026-09-23T10:41:02.117Z" },
    { "refusal_id": "srf_…", "initiator": "rol_0192…",
      "control": "autonomous_depth", "limit_value": 3, "target": "thr_0192…",
      "message": "autonomous initiation depth 4 exceeds the maximum of 3",
      "refused_at": "2026-09-23T10:40:57.004Z" } ] }
```

| field | meaning |
| --- | --- |
| `control` | the limit's own name: `open_calls_per_tenant`, `open_calls_per_initiator`, `autonomous_cycle`, `autonomous_depth`, `offline_backlog` ([the node's cap](node-channel.md#the-offline-backlog-is-capped)) |
| `limit_value` | the numeric limit, when the control has one; a cycle has none |
| `target` | the thread the refused request named, when it named one |
| `message` | exactly the words the caller was given |

The read needs `tenant_admin` and shows the asking tenant's own refusals only.
The breakers' trigger is now a question with an answer: two or more tenants
refused within an hour is *a multi-tenant storm observed*. The breakers
themselves stay deferred on it, and the condition can be read rather than
remembered.

## Seeing the calls offered to you

`GET /v1/calls/offered`, as a role, lists the open calls that were offered to
it — every call whose expression named one of the role's declared interests
when it was opened — oldest offer first, inside their join window, each with
the role's own response when it has made one:

```json
{
  "role_id": "rol_0192…",
  "offered": [
    {
      "call_id": "cal_0192…",
      "thread_id": "thr_0192…",
      "expression": {"scope": "tenant", "interests": ["parser trivia"], "…": "…"},
      "min_participants": 1,
      "max_participants": 4,
      "recommendations_allowed": false,
      "join_deadline": "2026-09-23T10:00:00Z",
      "expires_at": "2026-09-23T11:00:00Z",
      "offered_at": "2026-09-23T09:00:00Z",
      "responded": null
    }
  ]
}
```

A closed call, or one past its join deadline, leaves the list. A person has no
offers — calls are offered to roles — and answers `403`; an unenrolled
principal is refused rather than shown an empty list it could mistake for an
answer.

Each offer names the call's tenant (`call_tenant_id`) and whether it is
`foreign`. An offer from another tenant — a `network`-scope call offered across
an effective directory agreement — carries the call, its expression and its
window and **no `thread_id`**: the thread lives in a tenant whose records the
role cannot read, and cross-tenant existence is not leaked
(`SIGNOFF-REPAIR.5.3.5.1.2`). The thread is named when a join lands.

⛔ **This is the durable half of the advertisement** (`SIGNOFF-REPAIR.5.3.5.1`).
The offer row is written when the call opens and survives the role's node being
offline, and this read is how the node learns of it when it next asks. Until
this read an offer was a row nothing carried further: a role learned of a call
out of band and answered by its id. The prompt half is the node channel's
handshake, which carries `offers_pending` — the count of open, in-window,
unanswered offers for the node's role — so an online node is told rather than
left to ask (`SIGNOFF-REPAIR.5.3.5.1.1`, [the node channel](node-channel.md#reconnect-and-cursor-resume)).
An offer never rides the inbox as work: it has no admission and nothing to run.

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

A role may answer again while the call is open. The new response **replaces** the
earlier one, which is not kept, and the call's inspection shows the current
response with the time it was given.

⛔ **A federated subscriber's `join` is a request, not a join**
(`SIGNOFF-REPAIR.5.3.5.2`). A role in another tenant cannot act in the call's
tenant without a local grant (ADR-026), and the only path to one is the card
import the call's tenant performs. So when a role that was **offered** a
network-scope call answers `{"kind": "join"}`, the server records a
`join_request` on the call — carrying the role's own exported card and its
digest, exactly what `GET /v1/profiles/{role_id}/card` would mint for it — and
answers `200 {"response": "join_request"}`. Three gates stand before it: only a
`join` is a request (a foreign decline, observe or recommend records nothing);
the call must have been offered to the role; and the two tenants must hold the
**effective recruitment agreement**, because a card crosses only under the
operators' consent on both sides — without it the answer is `403` naming the
missing agreement. A foreign role that was never offered the call hears exactly
what it always heard, so a call's existence cannot be learned by probing ids.

The close never seats a request — it counts `join`. The initiator and the
administrator see the request in the call's inspection, with the card, and
resolve it with the ordinary [card import](profiles.md#importing-a-card); the
imported role's provenance names the origin role. What the imported role can
then *do* in the call waits on `SIGNOFF-REPAIR.5.3.5.3`: it has no node here.

⛔ A response that arrives while the call is being closed **waits for the close
and is then refused** — `409 invalid_transition`, *the call is closed* — with
nothing recorded. It is not written as a late response on the closed call: a
response on a closed call is on no panel, and a row nothing consumes is not a
record (`SIGNOFF-REPAIR.5.2.4`).

## Closing a call and the panel snapshot

```bash
curl -s -X POST "localhost:4310/v1/calls/call_0192…/close" \
  -H 'x-reasonbraid-principal: hpr_0192…'
```

The **initiator or the tenant owner** may close a call, and the choice is
audited. The initiator is the caller the call names **and** one that still holds
the invitation authority the call was opened under: the close re-runs the same
guarded `thread_invite` authorization the open ran, so an initiator whose grant
or boundary has been revoked since cannot close, and a caller who holds that
authority but did not open the call cannot either. Until `SIGNOFF-REPAIR.11.47`
the close compared names only; measured, an initiator whose grant was revoked
closed its call and wrote its panel. Closing one that is not `open` is refused as an invalid transition, and
an unknown call is `404`. Two closes at once are ordered: one snapshots the
panel and the other is refused the same way, never answered with a database
error.

Closing **snapshots the selected panel**: the joiners, ranked, capped at
`max_participants` — together with the **selection explanation**, which carries
each panelist's stage-1 reasons and stage-2 features. The explanation is stored
with the panel, so why this panel was chosen survives the call.

The panel is a record, not a seating: closing invites and seats no one. A
panelist takes part in the thread once it is invited like any participant
([explicit participants](cli.md)).

⛔ The snapshot is taken at close. Roles that join, change or become ineligible
afterwards do not alter it — that is what makes it a record rather than a view.

### What the panel is known to share

The close ranks the joiners with the [six features](profiles.md#matching-the-directory),
all weighted `1`, and it is the one surface that loads the **dependence
facts** the `diversity` feature needs. Each joiner has five, read at close:
`provider`, `model_family` and `harness` from the role's latest incarnation,
`owner` (the call's tenant), and `lineage`, which nothing declares yet.

**Only a fact known on both sides counts** (`SIGNOFF-REPAIR.5.1.3`). For each
of the five, a joiner that declares it is compared with the *other* joiners
that declare the **same** attribute, and scores `1 −` the share of them holding
its value. Every other attribute scores 0. The feature is the mean over all
five:

| Joiners | `diversity` of A |
| --- | --- |
| A `provider: openai`, B `provider: openai`, C `provider: anthropic` | provider shared by 1 of 2 → (1 − 0.5) / 5 = **0.1** |
| A `provider: openai`, B `provider: anthropic` | provider shared by none → 1 / 5 = **0.2** |
| A declares nothing | **0** — *none of the 5 dependence attributes is known for both this candidate and another (unknown contributes nothing)* |
| A `provider: x`, B `harness: x` | the provider meets no other provider → **0**; a provider is never compared with a harness |

Dividing by five, not by the facts a joiner happens to declare, is deliberate:
an undeclared fact scores exactly what a fact shared with everyone scores, so
**declaring less can never rank a joiner higher**. The explanation names every
attribute it compared and how many it could not.

The snapshot also stores one **dependence indicator** per attribute: the groups
of two or more panelists sharing a value, and how many declare nothing. The
wording separates the three cases:

```text
2 of 2 panel members share a provider with at least one other (1 overlap group)
no two panel members share a provider (it varies across the 2 that declare one); 1 of 3 do not declare one
no panel member declares a lineage, so nothing is known about it
```

These are **indicators, never an independence claim** (§10.4): they say what
the panel is known to share, not how likely its members are to fail together.

⚠️ Until that repair an absent fact read as variation. A joiner with no facts
scored the maximum diversity, `1.0`; leaving a shared provider undeclared hid
the overlap; an attribute nobody declared was reported as *varies across the
panel*; and a value was matched against all five of another joiner's facts,
so a provider named `x` counted as sharing with a harness named `x`.

**The minimum is a minimum on the selected panel** (`SIGNOFF-REPAIR.5.2.5`).
Eligibility is re-resolved at close against each joiner's current facts, and a
joiner who no longer satisfies the expression is dropped before ranking. The
close is then refused `409 invalid_transition` when fewer eligible joiners
remain than `min_participants`, naming both counts — *the panel needs at least
1 eligible joiners: 0 remain eligible of 1 who joined* — and the call stays
open. Until this repair the minimum counted who had said `join`, so a call
whose only joiner had since lost the required attestation closed with an empty
panel.

## Each transition is one transaction

Every recruitment-call transition is one database transaction
(`SIGNOFF-REPAIR.5.2.4`), so a client sees either all of a transition's effects
or none, and two transitions racing on one call are ordered rather than
interleaved.

| Transition | What it holds | What that orders |
| --- | --- | --- |
| open | the tenant's opens, serialized by a transaction-scoped advisory lock keyed on the tenant | the two fan-out caps are counted after every earlier open in the tenant committed; the call row and its offers commit together, so a failed offer write leaves no call behind |
| respond | the call row, shared | responses do not queue behind each other; a close in progress makes the response wait and then read the status the close committed |
| close | the call row, exclusive | a second close waits, reads `closed`, and is refused `409`; no response lands between the responses the close reads and the panel it writes; the panel row and the status commit together |

Why an advisory lock for the open, rather than a row: the caps are counts over
a *set* — the tenant's and the initiator's open calls — and no row stands for
that set. Locking the `tenants` row would queue every other reader of that row
behind an open, and the tenant's authority guard is for issuances. The lock is
released with the transaction, commit or rollback, and holds nothing else in
the database. A refused open rolls back before its refusal is recorded, so the
record is written on its own commit and the lock is not held for it.

Before this repair each transition was several statements on the connection
pool: two concurrent opens both counted under the cap and both landed; a join
that arrived during a close was recorded on a call whose panel it was not on;
and two concurrent closes both read `open`, so the second's panel insert raised
the panel table's primary key into a `500`. Each is now a control that runs
the two requests deterministically on either side of one write.

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
| audience | `auto_bounds.audience` | `tenant` or `network`: the widest eligibility `scope` a call opened on the thread may target |
| side-effect | — | not yet carried: no verb attributes a side effect to a thread (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.4`) |

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

**What the thread keeps from the request** (`SIGNOFF-REPAIR.11.65`). The checks
judge the request, and the thread carries only what the thread model can hold:

- **The class, failing closed.** No `confidentiality_class`, or `general`, makes
  a `general` thread. Any other class the role declared, such as `internal` in
  the example above, makes it `confidential`, because a thread has no level
  between the two. A confidential thread's work needs an evaluator qualified
  for it, like any confidential thread's. Until that repair every automatic
  thread was `general`, whatever it declared. Measured: an initiation declaring
  `internal` produced a `general` thread.
- **The budget is a declaration, and the ceiling is the bound.** No ledger
  meters money, so `budget_amount` is checked against the admitting grant's
  `spend_limits.amount` and not carried. What bounds the thread's spending is
  its ceiling of calls, tokens and wall-clock seconds, the default ceiling for
  an automatic thread. An initiation without `budget_amount` skips that check
  and runs under the same ceiling.
- **Topics gate the initiation and are not stored.** Nothing reads a thread's
  topics after it exists.

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
| `initiating_grant` | the grant that admitted this initiation, whose bounds calls on the thread honour; absent on a person's thread |

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
