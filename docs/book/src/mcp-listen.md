# The MCP listen gateway

ReasonBraid can act as an MCP **client** to an upstream MCP server (`ROADMAP.md`
§9.6). When it subscribes to that server's `subscriptions/listen` stream, the
stream itself is treated as **ephemeral transport state**: MCP 2026-07-28 does
not auto-resume a listen stream over a Streamable HTTP reconnect, so ReasonBraid
keeps everything that matters on its own side and resumes from there.

⛔ **The rule this whole chapter serves:** the MCP continuation is never
advertised as stronger than the upstream can prove.

## What is durable, and what is not

| Fact | Where it lives | Why |
| --- | --- | --- |
| the subscription | ReasonBraid (`mcp_listen_state`) | the stream can die; the subscription does not |
| the last accepted cursor | ReasonBraid | the resume point is **ours**, never the dead connection's |
| the recent delivery ids | ReasonBraid (a 64-id window) | a replayed delivery must not be applied twice |
| the connection, the stream | the transport | disposable by design |

A delivery is recorded in **the same transaction** that commits its effects, so
a delivery can never be applied without being remembered, or remembered without
being applied. The window keeps the **most recent** 64 ids, the cursor is a
**high-water mark** that never rewinds, and a window that is not an array of
delivery ids **refuses the delivery** rather than silently deduplicating
nothing.

## The reconnect ritual

After a disconnect, the gateway performs five steps, in this order:

1. **Reauthorize.** A fresh authorization is obtained for the recreated
   request. ⛔ The remote MCP metadata grants ReasonBraid **no authority**: the
   credential admits us to the upstream, and what may then be done with what it
   sends is decided by ReasonBraid's own grants, on the ordinary path. The
   credential is never copied into thread content, and it is redacted from
   diagnostic output.
2. **Recreate the listen request**, carrying the cursor read from the durable
   row — not from anything the dead connection held.
3. **Reconcile the source-specific gap**, if the upstream supports one.
4. **Resume from our own cursor.** This step does not negotiate: whatever the
   upstream says about replay, delivery resumes from the last cursor
   ReasonBraid accepted.
5. **Surface the possible-gap condition** when continuity cannot be proved.

## When the possible-gap flag is raised

The gap is reported **closed** in exactly one case — the upstream can replay
from a point at or below the cursor we already hold, so nothing between the two
can be missing:

| What we hold | What the upstream proves | Possible gap |
| --- | --- | --- |
| cursor 11 | no replay mechanism | **yes** |
| cursor 11 | replays from 0 | no |
| cursor 11 | replays from 11 | no |
| cursor 11 | replays from 50 | **yes** |
| nothing accepted yet | anything | **yes** |

⚠️ **Row four is the one worth reading twice.** An upstream can offer a replay
and still be unable to reach back to our cursor — it replays from 50, we hold
11, and deliveries 12 through 49 are simply unprovable. *Offers a replay* and
*covers our gap* are different facts. An earlier version of this machine took a
single yes/no answer about replay, so it reported continuity in exactly this
case; it now takes what the upstream can actually produce.

The last row is the same rule applied to our own ignorance: with nothing
accepted there is no cursor to compare the upstream's floor against, so no
continuity is claimed.

## What this surface does and does not qualify

✅ **Built and exercised against a real socket.** The ritual runs end to end in
`crates/reasonbraid-server/tests/mcp_listen.rs` against an upstream that cuts a
chunked response mid-body: the client observes the transport failure, the
resume that follows carries the cursor from the durable row, and the upstream
records which steps it saw and in what order. The possible-gap flag is
exercised in **both** directions, including the replay-floor case above, and a
listen request presenting a superseded credential is refused by the upstream —
so step 1 is enforced there, not merely asserted here.

⛔ **Not qualified, and no document here may say otherwise.** The transport used
by that control is ordinary HTTP, not the MCP Streamable HTTP client from the
official Rust SDK: `rmcp` is pinned for its **server** half only, and the client
transport is a dependency this workspace has not taken. So there is no
SDK-level or conformance-tested claim for this surface
(`SIGNOFF-REPAIR.6.6` owns that decision).

⛔ **There is no operator-facing gateway yet.** Nothing configures an upstream
MCP server, so the ritual ships as a library surface with no route, no CLI verb
and no enrolled-upstream registry behind it. Treat this chapter as the contract
that half is built against, not as a feature an operator can switch on.
