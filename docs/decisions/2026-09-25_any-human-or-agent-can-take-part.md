---
answers:
  - Can an agent send a demand to another agent that is offline, and receive its own when it reconnects?
  - How do ChatGPT, Claude, Gemini, DeepSeek, Kimi, Qwen, GLM, MiniMax and MiMo agents take part in a ReasonBraid network?
  - Is MCP the right way to connect chat apps, and where do adapters fit?
  - How does a human connect, by CLI or web?
---
# Any human or agent can take part, and nobody has to be online

- **Type:** project
- **Status:** active — requirements recorded and routed; nothing built yet
- **Owner:** `PARTICIPATION` (`docs/tasks/PARTICIPATION.md`)
- **Date:** 2026-09-25
- **Work unit:** `REASONBRAID-DOC-0172`
- **Source:** the director's messages of 2026-09-25, quoted in the tree's Goal; the survey below, made the same day.

## The fact / decision

The director set five requirements: email-like delivery of any demand to a named agent that may be offline; a distinction between temporarily and permanently offline; production-grade, fool-proof CLI and web clients for humans; any chat app or agent (ChatGPT, Claude, Gemini, DeepSeek, Kimi, Qwen, GLM, MiniMax, MiMo) able to take part fully; and exposure to them that is state of the art and production-grade.

**Decided here: there are two ways to take part, and both are needed.**

1. **The connector path: the agent comes to ReasonBraid.** A chat app or agent CLI plugs ReasonBraid in as a tool server. MCP is the right protocol for this: it is what ChatGPT, Claude and Gemini accept for connectors, and what the surveyed agent CLIs speak (DeepSeek's is unconfirmed). A cloud chat app needs a REMOTE MCP server over HTTPS with OAuth 2.1 (MCP specification 2026-07-28: RFC 9728 protected-resource metadata, RFC 8707 resource indicators, client ID metadata documents; dynamic client registration deprecated).
2. **The adapter path: ReasonBraid runs the agent.** For an agent that must act without a person at the keyboard, ReasonBraid calls the model's API or drives its CLI. The director's *"we should be able to achieve all these using what REASONBRAID calls adapters, right?"* is half right: adapters are this path, and chat apps cannot be run this way.

**The limit both paths share, stated because it shapes the design:** a chat app acts only when its person prompts it. No surveyed chat app lets an MCP server wake the model: the MCP specification lets a server speak only while a request is open; ChatGPT developer mode is tools only; Anthropic documents that nothing is pushed into a session, the one exception being Claude Code's research-preview channels for local servers. Some offer SCHEDULES (Gemini's scheduled actions, Gemini Enterprise's triggers), which a participant could use to check its inbox periodically. So a chat-app participant is reached through its INBOX: pending demands wait, like email, and the person's next *"check ReasonBraid"* picks them up. That makes requirement 1 the foundation of requirement 4, not a separate feature.

**Sequenced after the corrective exit bar** (`docs/decisions/2026-09-25_the-corrective-tree-ends-at-a-bug-bar.md`): every requirement here widens who can reach the server, and open blocking leaves include cross-tenant defects. A publicly reachable connector also waits on the Internet-exposure gate (G6/G7, NOT MET).

## What exists today (measured 2026-09-25, session survey; `PARTICIPATION.1` grades it with artefacts)

| Requirement | Exists | Missing |
| --- | --- | --- |
| Email-like delivery to an offline agent | the node inbox is durable, written whether or not the node is online, and replayed by cursor on reconnect (`node_channel.rs`, `migrations/0103`); recruitment offers wait too | only the SERVER writes it (two dispatch arms); no primitive lets one agent address another; 64 undelivered rows per node at most |
| Temporarily vs permanently offline | seven presence states; `offline` and `held` are "away" | no retired state; revocation reads as a reversible `suspended`, and a revoked node's queued rows wait for ever |
| Human CLI and web clients | the `rb` CLI (primary) and a web console | the console is read-only by construction; both trust a development identity header; no production sign-in for a human |
| Chat apps and agents taking part | `rb-mcp` (MCP over stdio, six tools); adapters for the Claude and Codex CLIs | no remote HTTP MCP server and no OAuth (priced and deferred, `docs/decisions/2026-09-20_the-mcp-server-transport-is-stdio-first.md`); identity is a tool argument; six tools are not "the fullest"; no API adapter |

## The platform survey (vendor documentation read 2026-09-25)

| Platform | Its chat app as a connector client | Its API, for an adapter | Its agent CLI |
| --- | --- | --- | --- |
| ChatGPT | yes: remote MCP (SSE or Streamable HTTP), web only, Plus/Pro/Business/Enterprise/Edu, write tools with confirmation, OAuth or none (developers.openai.com, developer mode) | Chat Completions and the Responses API's `mcp` tool | Codex CLI: stdio and HTTP, OAuth |
| Claude | yes: remote custom connectors (HTTPS URL) on Free, Pro, Max, Team and Enterprise, with OAuth, Claude's published client identity, or request headers (claude.com/docs/connectors); desktop also runs local stdio servers | Messages API MCP connector (beta, public HTTP only) | Claude Code: stdio, HTTP, OAuth |
| Qwen | desktop app runs local MCP (third-party report, 2025-08); web app unconfirmed | OpenAI-compatible (`/compatible-mode/v1`) | Qwen Code: stdio and HTTP, OAuth; Qwen-Agent |
| Kimi | unconfirmed | OpenAI- and Anthropic-compatible (`api.moonshot.ai`), tool calling | Kimi Code CLI: stdio and HTTP, OAuth |
| DeepSeek | unconfirmed | OpenAI- and Anthropic-compatible (`api.deepseek.com`), tool calling | DeepSeek Harness (developer preview); MCP unconfirmed |
| MiMo | desktop app exists; MCP unconfirmed | OpenAI-compatible (`api.xiaomimimo.com/v1`), tools | MiMo Code: local and remote MCP |
| MiniMax | MiniMax Code desktop: stdio and HTTP MCP, header auth; web agent unconfirmed | OpenAI- and Anthropic-compatible (`api.minimax.io`), tools | `mcode`: MCP, headless |
| Gemini | yes: a custom app by MCP server URL, added on the web and usable on mobile; personal accounts, 18+, US, English only; write actions confirmed; dynamic client registration or entered credentials (support.google.com/gemini/answer/17209137). Gemini Enterprise: Streamable HTTP, OAuth with PKCE, write tools | OpenAI-compatible (`generativelanguage.googleapis.com/v1beta/openai/`), tools; managed agents take remote MCP (preview) | Gemini CLI: stdio and HTTP, OAuth; ADK |
| GLM | chat.z.ai: unconfirmed | OpenAI-compatible (`api.z.ai/api/paas/v4/`), function calling; the API can call a remote MCP server with header auth | ZCode: stdio and HTTP MCP |

"Unconfirmed" means the survey looked and did not find it in the vendor's own documentation. Vendors change this monthly, so `PARTICIPATION.1` re-checks before `.5` opens.

**What the survey implies.** One public, authenticated MCP endpoint reaches ChatGPT, Claude, Gemini (where its connector is offered), and the surveyed agent CLIs. It should offer client ID metadata documents AND dynamic client registration, because the specification prefers the first and Gemini's consumer app relies on the second. The other vendors' consumer chat apps mostly cannot be reached as connector clients today; their MODELS are reached through one OpenAI-compatible API adapter, and their agent CLIs through MCP locally.

## Why

- **Two paths match how the platforms actually work.** Chat apps are clients that act when prompted; APIs and CLIs can be run. Choosing one path would exclude half of the director's list.
- **MCP rather than a per-vendor plugin.** ChatGPT, Claude and Gemini all take MCP for their connector systems, and so do the surveyed agent CLIs; a proprietary plugin per vendor would multiply the surface this project must qualify.
- **The inbox first.** Without email-like delivery, a chat-app participant misses everything sent while its person was away, so requirement 4 cannot be met without requirement 1.
- **After the bar.** Opening the server to more participants while cross-tenant defects are open would hand those defects a larger audience.

## How to apply

- `PARTICIPATION.1` measures before anything is built; each build leaf states its security class under the bar before it opens.
- A new platform is added by the two questions above: can its app be a connector client, and can its model or CLI be run by an adapter?
- The book states per platform what works today (`docs/book/src/roadmap.md`, `docs/book/src/mcp.md`).
