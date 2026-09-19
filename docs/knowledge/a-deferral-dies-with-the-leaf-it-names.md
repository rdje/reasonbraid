answers: where did my deferred work go; why does a done task tree have a gap; is "this rides leaf N" enough to keep an obligation alive; how do I find work that two tasks each handed to the other; why is a finished feature unreachable

# A deferral dies with the leaf it names

- **Type:** `knowledge`
- **Date:** `2026-09-19`
- **Owner / source:** leaf `SIGNOFF-REPAIR.6.5`

## The question

A task is complete except for one part, and that part clearly belongs to the
next task along. You write *the transport rides `.3.4`* and close. `.3.4` gets
done too. Where is the transport?

## The answer

> **Nowhere.** A sentence in a CLOSED leaf pointing at another leaf is not an
> obligation — it is a note about what its author intended. The leaf it names
> has its own goal and its own acceptance, and neither of those grew a clause
> because a sibling mentioned it. When that leaf closes, the pointer still
> reads correctly and points at something finished.

The measured instance: an MCP tool surface, fully implemented, authorized by
calling the HTTP handlers it re-expresses, and covered by live tests against a
real database — **unreachable by any MCP client**, because nothing serves it.
Two `done` leaves each deferred the transport to the same third leaf, in those
words, and that leaf shipped a different thing and closed `done`. Nothing was
skipped, nobody forgot, and no gate fired. The deferral simply had no owner.

⭐ **The tell is that the two documents disagree inside one leaf.** Its *Goal*
named the transport and the resources; its *Done* list enumerated three tools
and named neither. Both sentences were written by the same person on the same
day, and the leaf closed on the second one. A goal line is a promise and a done
list is a receipt, and nothing compares them.

## The rule

> Deferred work is owned by a leaf **of its own**, opened at the moment of the
> deferral, or it is not owned. Naming a sibling in prose is a cross-reference,
> not a transfer.

The cheap discipline: when you catch yourself writing *X rides `.N`*, open the
leaf for X before you finish the sentence. It costs a paragraph, and the
alternative costs the feature.

## Finding the ones already lost

Two censuses, neither expensive, and they answer different questions:

| Ask | Instrument | Finds |
| --- | --- | --- |
| which promises were never receipted | for each `done` leaf, diff the nouns in its goal line against its done list | work the author knew about and did not do |
| which finished code nothing reaches | count non-test call sites outside the defining file, excluding re-exports | work that was done and then stranded |

The second is [[a-re-export-is-not-a-caller]], and the two catch the same
defect from opposite ends: the first sees the promise with no code, the second
sees the code with no caller. A surface can fail both at once — implemented,
tested, and served by nothing.

⚠️ **A goal-versus-done difference is a POPULATION, not a defect list.** Goal
lines are written before the work and legitimately describe more than one leaf
delivers; what makes an instance real is that the missing noun is reachable from
nothing else either. Classify before you publish a count
([[a-scoping-defect-errs-in-one-direction]] for which way to err while you do).

## Related

- [[a-re-export-is-not-a-caller]] — the runtime half of the same question: is
  this thing actually reached?
- [[a-control-that-passes-for-an-unrelated-reason]] — a suite can be green over
  a surface nothing serves, because the tests are the only caller.
