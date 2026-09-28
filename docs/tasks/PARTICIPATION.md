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

And on 2026-09-28, after first trying the showcase (`SHOWCASE.1`):

> I have access to several chatbots and coding agents. Right now they all are
> running in my LAN. I use the codex, claude, kimi code, qwen code and pi CLIs. I
> use gpt, opus, kimi, qwen and deepseek models for coding. Any of these
> harness/model can connect to the network and [deliberate], discuss, come to
> conclusions, disagree on a subject. the chat interface should, long term
> resemble that of Teams, slack, discord for there flexibility when humans are
> involved, on the other end agents do not care.

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
    Director (2026-09-28): the chat interface should, long term, resemble Teams, Slack or Discord *"for their flexibility when humans are involved"*; agents do not care about the interface. The showcase's first note (`SHOWCASE.3`, `.4`) is the earliest measurement of what the director expects of a human client.
    Goal: human clients, CLI and web, production-grade, elegant, fool-proof and intuitive. The survey found the `rb` CLI the primary surface and the web console READ-ONLY by construction, both trusting a development identity header; a human has no production sign-in.
    Acceptance: `pending` — opens after `.1`; its first question is human authentication, which the rest depends on.
    Verification: `pending`
    Commit: `pending`

  - ID: `PARTICIPATION.5`
    Status: `proposed`
    Director (2026-09-25, `REASONBRAID-DOC-0173`): greenlit — *"connect to the internet over HTTPS and fully support sota OAuth sign-in"*, security paramount. Served first as the OWNER-ONLY profile once its own gate record is signed (`REASONBRAID-DOC-0175`); serving anyone else waits on the full G6 record (`docs/runbooks/external-security-review.md`).
    Goal: the CONNECTOR path — a chat app or agent comes to ReasonBraid. MCP is the protocol ChatGPT, Claude and Gemini accept for this: a remote Streamable-HTTP MCP server with OAuth 2.1 per the MCP specification 2026-07-28 (RFC 9728 protected-resource metadata, RFC 8707 resource indicators, client ID metadata documents), and a tool vocabulary that covers what a participant does. The survey found `rb-mcp` stdio-only (HTTP priced and deferred, `docs/decisions/2026-09-20_the-mcp-server-transport-is-stdio-first.md`), six tools, and identity passed as a tool argument.
    Director (2026-09-25, discussion): *"Using chatbots will allow more testing of REASONBRAID."* The recorded view: agreed for exploratory testing (confusing refusals, missing tools, realistic misuse, hostile content coming back through tool results). A chatbot never repeats a run exactly, so it adds to the repeatable controls rather than replacing them: each problem one finds becomes a repeatable control before a fix is claimed.
    Acceptance: `pending` — depends on `.4`'s authentication and on the G6/G7 exposure gate for any cloud-hosted app.
    Verification: `pending`
    Commit: `pending`

  - ID: `PARTICIPATION.6`
    Status: `proposed`
    Director (2026-09-28): the harnesses in use on the director's LAN are the codex, claude, kimi code, qwen code and pi CLIs, over gpt, opus, kimi, qwen and deepseek models; any of them should be able to join, deliberate, conclude and disagree. Two of the five harnesses (claude, codex) have adapters today, as library code `rb-node` does not construct; kimi code, qwen code and pi have none.
    Goal: the ADAPTER path — ReasonBraid runs the agent. The survey found two real adapters (the Claude and Codex CLIs) and a fake; every other surveyed vendor offers an OpenAI-compatible chat API with tool calling, so one qualified OpenAI-compatible API adapter would reach most of them, and their agent CLIs are candidates for further CLI adapters.
    Acceptance: `pending` — opens after `.1`; each adapter is qualified by the §19.4 checklist (`docs/book/src/adapter-boundary.md`).
    Verification: `pending`
    Commit: `pending`

  - ID: `PARTICIPATION.7`
    Status: `proposed`
    Director (2026-09-25, a discussion message, not a decision): test locally first with the chat apps, then host ReasonBraid on a cloud provider (Alibaba Cloud, Google Cloud, AWS and Azure named as candidates); asked whether testing on one provider is enough. The director will open the provider accounts.
    Goal: hosting — the first cloud provider and what changes between providers. The view given in reply: qualify ONE provider fully, and keep the deployment portable (a container, standard PostgreSQL, no provider-only service on the critical path). Then run a smoke test on a second provider before claiming portability. A matrix of four or five providers adds cost without much extra signal, because the differences sit in a few known places: the platform addresses the egress guard must refuse (`SIGNOFF-REPAIR.18` found one it does not), load-balancer idle timeouts on long-lived MCP streams, managed-PostgreSQL restrictions (no superuser, extension allow-lists), secret stores and outbound-network rules. Alibaba Cloud's mainland-China regions add a separate question (ICP filing; reaching model APIs outside China), which is a market choice rather than a test.
    ⚠️ Local testing depends on each client's MCP transport: `rb-mcp` is stdio-only today (`.5`), which desktop and CLI clients can use locally, while ChatGPT accepts only a remote server, so local ChatGPT testing needs the HTTP transport and a reachable endpoint, which is the exposure gate `.5` already names.
    Prerequisite owned by the director: an account on the chosen provider (and later on a second one for the smoke test), with multi-factor sign-in, a billing alert and least-privilege access. ⏰ The director asked to be TOLD WHEN (2026-09-25). The signal is `SIGNOFF-REPAIR.14`'s owner-only candidate being built and needing a real host to be tried on, which comes after the corrective exit bar (`REASONBRAID-DOC-0162`). The agent raises it then as a named blocker in its reply, and not before.
    Acceptance: `pending` — opens after `.5`'s owner-only gate record is signed; the provider is the director's choice.
    Verification: `pending`
    Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PARTICIPATION.1` | `pending` | the census — nothing else opens until each requirement is graded against the shipped system; sequenced after the corrective exit bar (`REASONBRAID-DOC-0162`), measurement only if run earlier |

## Decisions

- `2026-09-25`: two paths, not one. A chat app acts only when its person prompts it, so it takes part through a CONNECTOR (MCP) and finds its pending items in its inbox; an autonomous agent takes part through an ADAPTER that ReasonBraid runs. The director's question *"we should be able to achieve all these using what REASONBRAID calls adapters, right?"* is answered in the record: adapters are half of it.
- `2026-09-25`: sequenced after the corrective exit bar, because every leaf here widens who can reach the server (`REASONBRAID-DOC-0162`). The director agreed the same day.
- `2026-09-25`: the Internet gate splits (`REASONBRAID-DOC-0175`, the director's choice B): owner-only first, gated by our own evidence and a tested owner allowlist; opening to others keeps the independent review and penetration test, *"with extrem care"*.
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
