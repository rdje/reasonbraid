# Moderating a thread

ROADMAP §13.5 describes a moderator. A moderator may classify messages, ask for
clarification, propose closing a round, point out claims nobody has answered, and
draft summaries. A moderator may **not** vote or approve, suppress a visible
dissent, fabricate evidence or change citations, change the electorate, quorum or
proposal, authorize spending or tools, or publish policy.

ReasonBraid builds this from two parts. The first is a small, closed **moderation
vocabulary** that anyone may use. The second is a **moderator seat**: a participant
who can use that vocabulary and nothing else.

## The moderation vocabulary

A moderation act is a normal thread contribution with one of five `kind`s:

| `kind` | What it does |
| --- | --- |
| `classify` | classifies a message |
| `request_clarification` | asks a participant to clarify |
| `propose_close` | proposes closing the current round |
| `identify_unanswered` | points out claims nobody has answered yet |
| `draft_summary` | drafts a summary |

```json
{
  "tenant_id": "ten_…",
  "content": "this is a procedural question, not a position",
  "kind": "classify",
  "ref_event_id": "evt_…"
}
```

Each of these rules is enforced, and each refusal is `400 invalid_command`:

- A moderation act is accepted only while the thread is on a **`moderate`** step.
  None of the eight built-in profiles has one, so a thread that needs moderation
  runs a registered profile that includes it (for example
  `["moderate", "vote"]`).
- A moderation act may **not** carry a verdict, a ballot, an assessment, claims,
  evidence references or a claim target. The vocabulary cannot express those
  things, so a moderation act cannot do them.
- `ref_event_id` is optional. When present, it must name an event in this thread.
  A moderation act **refers to** an earlier contribution. It never rewrites one.

Contributions cannot be removed or edited. No product code updates or deletes the
event log, so there is no verb that could suppress a dissent. ⚠️ That comes from
the code, not from the database: no database trigger prevents it.

## The moderator seat

Anyone who may invite to the thread (the `thread_invite` grant) seats a moderator
by inviting an agent role with `"seat": "moderator"`:

```json
{ "tenant_id": "ten_…", "agent_role": "rol_…", "seat": "moderator" }
```

The role accepts the invitation like any participant. From then on it may:

- accept or decline its own invitation;
- post a moderation-kind contribution.

**Everything else is refused with `403`.** The refusal message says that the
principal is seated as a moderator and names the §13.5 rule being enforced:

| Attempt | Rule named in the refusal |
| --- | --- |
| a `ballot` or a `verdict` | *a moderator cannot add a vote or approval* |
| an `evidence_reference` or `assessment` contribution | *a moderator cannot fabricate evidence or change citations* |
| any other contribution, or a challenge | *a moderator takes no position of its own; it may only moderate* |
| `thread.revise` | *a moderator cannot change a proposal* |
| `thread.invite`, `thread.remove_participant`, `thread.join` | *a moderator cannot change the electorate* |
| `thread.advance_round` | *a moderator proposes round closure (`propose_close`); it does not advance the round* |
| `thread.close`, `thread.cancel` | *a moderator cannot decide the thread or change its quorum* |

The seat takes effect **even when the role holds the grant** for the verb. The
seat only takes authority away; it never adds any.

**A moderator is never in the electorate.** When the thread opens its `vote` step,
the electorate is the accepted participants *minus* the moderators. Counting a
moderator would leave its ballot outstanding forever, because it is not allowed
to cast one. Under `unanimity`, that would turn every vote into `no_quorum`.

A seat lasts as long as the membership. Once the role has declined, left or been
removed, it can be invited again, and the new invitation sets the seat afresh:
as an ordinary participant it is no longer restricted.

## What is not a moderator's job

- **Enforcing format or length is not implemented, on purpose.** It is the only
  §13.5 act that controls *content* rather than *procedure*. §13.6 puts form
  constraints (blind initial positions, randomized order, hidden vote totals) in
  the workflow profile, not in one participant's hands. If it returns, it will be
  a profile property.
- **Spending and publishing** need their own grants. Being seated as a moderator
  grants neither.
- **The benchmark's `moderator` arm** ([The deliberation benchmark](benchmark.md))
  is a routing-shape measurement. It is not this seat, and this seat does not
  replace it.
