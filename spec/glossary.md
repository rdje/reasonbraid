# Glossary — frozen term distinctions

**Status: draft — not normative.** Distills `ROADMAP.md` §8.1–§8.5 (identity
hierarchy, aggregates, identifiers, lifecycles, typed messages) and §4
(governance) into one frozen set of distinctions. Backlog item 3: "freeze
distinctions among tenant/human/host/node/role/incarnation/run/thread/
contribution/decision/publication/deployment." New terms enter here, not into
prose that drifts.

The governing rule for every row: **distinct things stay distinct in types and
on the wire** (`ROADMAP.md` §8.3, "use newtypes for every ID; never interchange
plain UUID strings"; §3.1 "separate a durable agent role from a model/harness
incarnation and a run/session"). A model name never carries authority.

## Identity hierarchy (`ROADMAP.md` §8.1)

| Term | Definition | Distinguish from |
| --- | --- | --- |
| **Tenant** | The organization/scope boundary; `tenant_id` is part of every aggregate key, authorization decision, and object-store namespace (§16.8). | A human, a host, a target — a tenant is a *scope*, not an actor. |
| **HumanPrincipal** | A durable human identity that can hold authority and act. | A host or an agent role. |
| **Host** | A machine (physical or virtual) that runs one or more nodes. | A node *instance* (one process) and a node *identity*. |
| **NodeInstance** | A running `reasonbraid-node` daemon on a host; holds local credentials and a durable run/delivery journal (§11.1). | The host it runs on; the node's workload *identity* (reimage ≠ inherit history, §16.2). |
| **HarnessInstallation** | A harness (e.g. Claude Code, Codex) installed on/controlled by a node. | The node; the agent *role*; the model. |
| **AgentRole** | The durable identity: purpose, subscriptions, authority, and history. | An incarnation or a run — the role **outlives the model** (§2.2 principle 5). |
| **AgentIncarnation** | One provider/model/harness/configuration of a role, with a validity interval. | The role (durable) and the session/run (transient). |
| **AgentSession** | A conversation/harness continuity handle. | A `Run` (supervised execution); a role. |
| **Run** | One supervised execution with a budget and provider receipts. | A session (continuity) and an incarnation (configuration). |

Authority attaches to the narrowest appropriate identity; a model name never
carries authority (§8.1).

## Core aggregates and entities (`ROADMAP.md` §8.2)

| Term | Definition | Distinguish from |
| --- | --- | --- |
| **Thread** | The aggregate for scope, question, workflow profile, visibility, and overall open/closed status. | A `Decision` (one thread may yield many) and a `Publication`. |
| **Message / Contribution** | A typed contribution referencing artifacts/evidence (Question, Position, Claim, Challenge, Revision, Vote, …). | A *command*: a message cannot itself mutate state — it submits a command an aggregate may accept (§8.5). |
| **Claim** | A normalized assertion with author, scope, confidence, and status. | A `Message` (one message kind) and an `EvidenceSnapshot` (the bytes). |
| **Decision** | Electorate, rule, votes, objections, and a deterministic result. | Approval (human review) and Publication (policy distribution). |
| **ProviderAttempt** | One supervised provider execution with its ambiguity state (§11.3). | A `Run` (the supervised unit) — the attempt is the *state machine* of one dispatch. |
| **Budget / Reservation / Charge** | Multi-resource authorization, hold, and settlement (§14). | Authority (who may act) — budget is *what may be spent*. |
| **EnrollmentAuthorityBoundary** | The root/parent-granted ceiling, shown to and acknowledged by the enrolled target (§4.4). | A grant (which must be a subset of the boundary). |
| **GovernanceCharter** | The versioned tenant charter: principal/target types, decision rules, precedence, veto, waiver, review (§4.1). | A policy (doctrine content) and a grant (one authorization). |
| **AuthorityGrant** | A scoped grant of actions over targets/domains, with ceilings and expiry (§4.2). | The charter (the framework) and a self-declared capability. |
| **Publication** | A canonical immutable bundle + effective pointer (§8.2, §15.7). | A `Deployment` (per-target application) — publication is *not* global rollout (§15.9). |
| **Deployment** | Per-target desired/observed application + receipts (§15.9). | Publication (the immutable content) and a `Receipt` (one target's evidence). |

## Lifecycle states (`ROADMAP.md` §8.4)

Lifecycle states are enumerated in [`lifecycle.md`](lifecycle.md). The glossary
only freezes the *meaning* of the state families:

- **Thread** states record whether the conversation is open, paused, closed,
  cancelled, or expired.
- **Participation** states record one role's join/invite/decline/observe/leave.
- **Provider attempt** states record whether one dispatch is planned, reserved,
  dispatching, accepted, streaming, completed, failed, cancelled, or — critically —
  `outcome_unknown` (the system does not know whether a billable call happened).

Every transition carries an actor, authorization, precondition, idempotency
behavior, emitted event, and recovery action (§8.4).

## Non-interchangeability table (the point of this file)

| Confusable pair | Why they must not be confused | Source |
| --- | --- | --- |
| role vs incarnation | a role is durable; changing the backing model must not create a new vote or inherit identity | §8.1, §13.3 |
| incarnation vs run | one role can have many runs; authority is not attached to a run | §8.1 |
| message vs command | a message is content; only a command can transition state | §8.5 |
| claim vs evidence | a claim asserts; evidence is an immutable snapshot with a receipt | §12.6–§12.7 |
| publication vs deployment | canonical content vs per-target rollout (never global-atomic) | §15.9 |
| budget vs authority | spend vs permission | §14, §16.4 |
| presence vs enrollment | offline ≠ nonexistent; presence does not change enrollment | §10.2 |
