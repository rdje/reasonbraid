# An offered replay is not a covered gap: a capability answer that cannot carry a bound cannot decide continuity

- **Type:** `decision`
- **Date:** `2026-09-19`
- **Status:** `active`
- **Owner / source:** `SIGNOFF-REPAIR.6.2.4` (REASONBRAID-REPAIR-0293) — measured

## The fact / decision

The MCP listen gateway's reconnect reports a **possible gap** unless the
upstream can replay from a point **at or below the cursor ReasonBraid has
already accepted**. What the upstream offers is no longer a `bool`:

```rust
pub enum UpstreamReplay {
    None,           // no replay mechanism at all
    From(i64),      // replays from this cursor onward, and no earlier
}
```

| What we hold | What the upstream proves | `possible_gap` |
| --- | --- | --- |
| cursor 11 | `None` | `true` |
| cursor 11 | `From(0)` | `false` |
| cursor 11 | `From(11)` | `false` |
| cursor 11 | `From(50)` | `true` |
| `None` (nothing accepted) | anything | `true` |

The predecessor, `resume_plan(own_cursor: i64, upstream_replay: bool)`, set
`possible_gap = !upstream_replay`. It is **deleted**, not wrapped: a wrapper
would have to read `true` as *replays everything*, which is the over-claim
itself, and keeping it would leave the defect reachable behind a shorter name.

## Why

`ROADMAP.md` §9.6 and ADR-024 both close the same paragraph with the same
sentence — *MCP continuation is never advertised as stronger than the upstream
source can prove* — and both list **reconcile any source-specific gap** as a
step distinct from *surface the possible-gap condition*. Two steps, because
there are two facts: whether the upstream offers a replay, and whether that
replay reaches back to where we are. A boolean holds one of them.

Row four is the case that made the difference concrete. An upstream that
replays from cursor 50 while ReasonBraid holds 11 answers *yes, I replay*, and
cannot produce deliveries 12 through 49. Under the boolean that was reported as
continuity — the precise thing both sources forbid — and nothing in the tree
could have noticed, because the only control was a unit test over the boolean's
own two values.

The last row is the same rule turned on our own state rather than the
upstream's: with nothing accepted there is no cursor to compare a replay floor
against, so no continuity is claimed. A scoping defect must err towards
weakening the finding (`docs/knowledge/a-scoping-defect-errs-in-one-direction.md`),
and here the weak direction is *surface the gap*.

## Consequences

- `ResumePlan.resume_from` becomes `Option<i64>`, because *accepted nothing yet*
  and *accepted cursor 0* are different facts and the durable row is written by
  the FIRST delivery, so a reconnect can legitimately precede it.
- `reconnect` performs all five documented steps against a `ListenUpstream`
  seam and returns the resumed stream with the plan — a reconnect that hands
  back no stream has not reconnected to anything.
- The `.6.2.4` control exercises the flag in **both** directions across a real
  socket, and the whole sweep of seven in-situ mutations is caught 7/7.

## answers:

- **Is this the same as §9.5's *absence differs from `false`*?** No, and the
  pair is worth holding together. §9.5 warns that *not stated* must not collapse
  into *not supported*. This is the other end of the same axis: **stated must
  not collapse into sufficient.** A capability answer is a claim about the
  provider, and eligibility is a claim about the caller's case; only a bound
  carries both.
- **Where else does this shape appear here?** `.7.3.6.4` decided that an egress
  claim is a ceiling and a sandbox claim is a floor — a declared capability read
  in the wrong DIRECTION. This one is a declared capability read at the wrong
  GRANULARITY. Related family, different mechanism, so no rule is promoted from
  one instance; the trigger for promotion is a third case where a declared
  capability is compared against a caller's requirement without a bound.
- **Does this qualify the MCP transport?** No. The control uses ordinary HTTP,
  not the official SDK's Streamable HTTP client, which is not a dependency of
  this workspace. `SIGNOFF-REPAIR.6.6` owns that decision, and until it closes
  nothing here may describe this surface as SDK-backed or conformance-tested.
