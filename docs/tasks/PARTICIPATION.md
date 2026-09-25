# PARTICIPATION: any human and any agent can take part, and nobody has to be online

## Metadata

- Tree ID: `PARTICIPATION`
- Status: `active`
- Roadmap lane: none — a director-requested track, off the frozen v0.4.1 roadmap, and the input the director named for v0.5.0 (`docs/book/src/roadmap.md`, *What is frozen*: opening v0.5.0 is the director's call)
- Created: `2026-09-25`
- Owner: repo-local workflow
- Record: `docs/decisions/2026-09-25_any-human-or-agent-can-take-part.md` (the director's words, the measured state, the platform survey with sources)

## Goal

The director's requirements of 2026-09-25, in their words:

> agents do not have to logged into the REASONBRAID network before an other agents
> can send them questions, bug reports or even feature requests or any other
> demands. As soon that agent come back online it shall receive all its pending
> notifications. it should work like emails.

> we might also need to distinguish between temporarily offline and permanently
> offline

> A human (me) should be able to connect to a REASONBRAID network via either a CLI
> client or a web-based client. both types of clients shall be sota, production
> and elegant and really fool proof and really intuitive to use.

> I should be able to connect to a REASONBRAID instance network via ChatGPT,
> Claude, Gemini, Deepseek, Kimi, Qwen, GLM, minimax and Mi[M]o chat or agent and
> be able to use REASONBRAID to the fullest.

> We need to expose REASONBRAID to those chat-bot or agent in sota, signoff and
> production-grade way.

## Non-Goals

- Un-freezing `ROADMAP.md` v0.4.1 in this tree. The tree measures and designs; v0.5.0 takes what it proves.
- Claiming Internet exposure. A connector a cloud chat app can reach is a PUBLIC endpoint, and G6/G7 is NOT MET (`LIVE_STATUS.md`); `.5` depends on that gate and does not self-certify it.
- Replacing `PEER-COLLAB`. That tree grades one scenario (a bug report between two of the director's agents); this one owns who can take part and how. `.2` is the capability `PEER-COLLAB`'s scenario needs, and `PEER-COLLAB.1`'s census feeds `.1` here.

## Sequencing

⚖️ **After the corrective exit bar, by `REASONBRAID-DOC-0162`.** Blocking corrective leaves (cross-tenant, integrity, false claims, lying gates) hold roadmap work, and they matter MORE here, not less: every requirement in this tree widens who can reach the server, and open blocking leaves include cross-tenant defects (`SIGNOFF-REPAIR.7.1.4.1`, `.9.1`, `.11.1`, `.10.1`) that a wider audience would reach first. `.1` is measurement and may run earlier without changing code. The director can reorder this; the record says what reordering would cost.

## Acceptance Criteria

- Every requirement above is graded against the SHIPPED system with an artefact before anything is designed (`.1`).
- Each build leaf states its security class under the bar before it opens, because each one widens the reachable surface.
- The book says plainly, per platform, what works today, what does not, and why.
- Live docs, the book and this tree stay in lockstep.

## Task Tree

- ID: `PARTICIPATION`
  Status: `active`
  Goal: any human and any agent can take part, and nobody has to be online
  Children: `.1`–`.6`

  - ID: `PARTICIPATION.1`
    Status: `pending`
    Goal: the census — grade each of the five requirements against the shipped system, one row per requirement clause, each with the artefact that proves it or the measurement that shows the gap. The session survey of 2026-09-25 (in the record; NOT yet evidence) is the starting hypothesis.
    Acceptance: a table with one row per clause, each graded by a run where a run is possible.
    Verification: `pending`
    Commit: `pending`

  - ID: `PARTICIPATION.2`
    Status: `proposed`
    Director (2026-09-25, `REASONBRAID-DOC-0173`): wanted; *"64 items is perfectly fine"*. The shape (typed messages, an authorization rule for who may message whom, delivery on the inbox's replay machinery) is in that record.
    Goal: asynchronous delivery to a NAMED participant, like email — a question, bug report, feature request or any demand, addressed by one agent to another that may be offline, kept durably, and delivered on reconnect. The survey found the node inbox durable and replayed by cursor on reconnect, but written only by the SERVER (two dispatch arms), capped at 64 undelivered rows per node, and no primitive by which one agent addresses another.
    Acceptance: `pending` — opens after `.1`.
    Verification: `pending`
    Commit: `pending`

  - ID: `PARTICIPATION.3`
    Status: `proposed`
    Goal: temporarily vs permanently offline — a participant that is away (its items wait) is distinguished from one that is gone (its items are not left waiting for ever). The survey found seven presence states, `offline` and `held` covering "away", revocation reading as a REVERSIBLE `suspended`, no retired or decommissioned state, and a revoked node's queued rows staying `queued` indefinitely.
    Acceptance: `pending` — opens after `.1`.
    Verification: `pending`
    Commit: `pending`

  - ID: `PARTICIPATION.4`
    Status: `proposed`
    Director (2026-09-25, `REASONBRAID-DOC-0173`): the web console must *"behave exactly like the rb CLI tool"*, and both must be *"extremely secure and super intuitive"*: full parity, not a read-only window.
    Goal: human clients, CLI and web, production-grade, elegant, fool-proof and intuitive. The survey found the `rb` CLI the primary surface and the web console READ-ONLY by construction, both trusting a development identity header; a human has no production sign-in.
    Acceptance: `pending` — opens after `.1`; its first question is human authentication, which the rest depends on.
    Verification: `pending`
    Commit: `pending`

  - ID: `PARTICIPATION.5`
    Status: `proposed`
    Director (2026-09-25, `REASONBRAID-DOC-0173`): greenlit — *"connect to the internet over HTTPS and fully support sota OAuth sign-in"*, security paramount. Public service still waits on the signed G6 gate record (`docs/runbooks/external-security-review.md`).
    Goal: the CONNECTOR path — a chat app or agent comes to ReasonBraid. MCP is the protocol ChatGPT, Claude and Gemini accept for this: a remote Streamable-HTTP MCP server with OAuth 2.1 per the MCP specification 2026-07-28 (RFC 9728 protected-resource metadata, RFC 8707 resource indicators, client ID metadata documents), and a tool vocabulary that covers what a participant does. The survey found `rb-mcp` stdio-only (HTTP priced and deferred, `docs/decisions/2026-09-20_the-mcp-server-transport-is-stdio-first.md`), six tools, and identity passed as a tool argument.
    Acceptance: `pending` — depends on `.4`'s authentication and on the G6/G7 exposure gate for any cloud-hosted app.
    Verification: `pending`
    Commit: `pending`

  - ID: `PARTICIPATION.6`
    Status: `proposed`
    Goal: the ADAPTER path — ReasonBraid runs the agent. The survey found two real adapters (the Claude and Codex CLIs) and a fake; every other surveyed vendor offers an OpenAI-compatible chat API with tool calling, so one qualified OpenAI-compatible API adapter would reach most of them, and their agent CLIs are candidates for further CLI adapters.
    Acceptance: `pending` — opens after `.1`; each adapter is qualified by the §19.4 checklist (`docs/book/src/adapter-boundary.md`).
    Verification: `pending`
    Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PARTICIPATION.1` | `pending` | the census — nothing else opens until each requirement is graded against the shipped system; sequenced after the corrective exit bar (`REASONBRAID-DOC-0162`), measurement only if run earlier |

## Decisions

- `2026-09-25`: two paths, not one. A chat app acts only when its person prompts it, so it takes part through a CONNECTOR (MCP) and finds its pending items in its inbox; an autonomous agent takes part through an ADAPTER that ReasonBraid runs. The director's question *"we should be able to achieve all these using what REASONBRAID calls adapters, right?"* is answered in the record: adapters are half of it.
- `2026-09-25`: sequenced after the corrective exit bar, because every leaf here widens who can reach the server (`REASONBRAID-DOC-0162`). The director agreed the same day.
- `2026-09-25`: the director's point-by-point answers (`REASONBRAID-DOC-0173`): the 64-item cap stands; direct addressing wanted; the web console at full CLI parity; Internet exposure over HTTPS with OAuth greenlit, security without exception.

## Open Questions

- Which vendors' consumer chat apps accept a user-supplied remote MCP server today? The survey confirmed ChatGPT (web, developer mode), Claude (web, desktop, mobile) and Gemini (personal US accounts, English; and Gemini Enterprise); for the others it found desktop apps or CLIs with MCP, and the consumer web apps unconfirmed. Owner: `.1`, re-checked when `.5` opens, because vendors change this monthly.
- One tenant or several for the director's agents? Owner: `.1`, with `PEER-COLLAB`'s same question.

## Blockers

- `.5` for cloud chat apps: the G6/G7 Internet-exposure gate (`LIVE_STATUS.md`, Phase 7: NOT MET). The director greenlit the exposure (2026-09-25); the gate's two external lines need an independent reviewer the director engages (`docs/runbooks/external-security-review.md`, `SIGNOFF-REPAIR.13.1`).

## Acceptance Checklist (required for any leaf that lands a CODE change)

- [ ] **ROOT CAUSE (WHY + WHERE)** — pending.
- [ ] **ADDRESSED (verified)** — pending.
- [ ] **NO REGRESSION** — pending.
- [ ] **FIX / LOCKSTEP** — pending.

## Verification Log

- `pending`

## Commit Log

- `REASONBRAID-DOC-0172`: tree opened, requirements recorded.
- `REASONBRAID-DOC-0173`: the director's answers recorded; `.2`, `.4` and `.5` scoped by them.

## Changelog

- `2026-09-25`: tree opened from the director's five messages of the same day.
