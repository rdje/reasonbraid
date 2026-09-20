---
answers:
  - Can an operator configure an upstream MCP server?
  - Why is there no route for enrolling an MCP upstream?
  - What must be built before the listen gateway gets an operator surface?
  - Does an outbound MCP dial get SSRF classification?
---
# The listen gateway owes a dialler before an operator surface

- **Type:** decision
- **Status:** accepted; the operator surface deferred, its prerequisites named
- **Owner:** `SIGNOFF-REPAIR.6.7`
- **Date:** 2026-09-20
- **Related:**
  `docs/decisions/2026-09-20_the-mcp-http-transports-are-blocked-by-one-upstream-split.md`,
  `docs/decisions/2026-09-18_lan-completeness-precedes-internet-exposure.md`,
  ADR-018 (whose options section names the placeholder-infrastructure failure
  this record avoids), ADR-024

## The question, and a sharper answer than it expected

`SIGNOFF-REPAIR.6.7` asks whether the listen gateway should get an operator
surface — a route, a CLI verb, an enrolled-upstream registry — before the LAN
bar is met, on the reasoning that an outbound dial to a third-party MCP server
is an Internet-facing act and `SIGNOFF-REPAIR.14`'s deferral therefore applies.

🔴 **That reasoning is sound and it is not the binding constraint. There is
nothing to operate.** `crates/reasonbraid-server/src/mcp_listen.rs` declares
`pub trait ListenUpstream`, and `impl … ListenUpstream` appears exactly **once**
in the workspace — in `crates/reasonbraid-server/tests/mcp_listen.rs`. **No
production type dials anything.** The reconnect ritual and its durable state are
both correct and both unreachable, which is the same shape the MCP tools were in
before `SIGNOFF-REPAIR.6.8` gave them a transport.

⛔ **So an operator surface would configure an upstream that no code can reach.**
A route that writes a registry row nothing consumes is the
placeholder-infrastructure lie ADR-018's own options section rejects — machinery
with nothing to drive.

## The decision

**Deferred, and the ordering is the substance of the decision: the production
dialler comes first, the operator surface second.**

Three requirements the dialler owes, recorded now so whoever builds it inherits
them rather than rediscovering them:

1. ⛔ **The destination must be classified.** `crate::ssrf::evaluate` is what the
   R0 fetch and R1 git packs put in front of every dial, and an outbound MCP
   connection is the same act against the same hazard. A dialler that reaches
   the network without it would be the one egress path in this server with no
   destination policy — and it would be reached from a registry row an operator
   supplied, which is the caller-controlled-destination shape the classifier
   exists for.
2. ⛔ **The enrolment must ride a named authority, not enrolment alone.**
   `SIGNOFF-REPAIR.7.1.1` measured 33 routes writing site-global state on
   enrolment alone; naming an upstream MCP server for the whole deployment is
   site-global by nature, and this route must not become the 34th.
3. ⚠️ **The credential has a holder question that is not answered here.**
   `UpstreamAuthorization` carries a credential in memory; where it comes from,
   who may register it and how it is redacted in the audit trail are decisions
   the R5 broker already had to make, and the answer is likely to be *reuse
   that*, but this record does not decide it.

⚠️ **And the transport it would use is itself constrained.** An SDK-backed
dialler is blocked by the `base64` split
(`SIGNOFF-REPAIR.6.6`); a hand-written one is what the current tests use and is
available, but choosing it is a decision with its own conformance consequences.
That ordering means the dialler is not simply the next thing to build either.

## What stays true in the book

The chapter's *there is no operator-facing gateway yet* paragraph is **kept and
sharpened** rather than replaced: the acceptance's replace-it clause was
conditional on the surface being taken, and it was not. What the paragraph gains
is the reason — the trait has no production implementor — which is a fact a
reader can check, where *not yet* is a schedule nobody can.
