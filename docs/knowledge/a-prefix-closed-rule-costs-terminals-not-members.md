answers: a completeness rule looks too expensive, how do I price it; should every file in the tree need a registry entry; how many entries would it take to govern everything; I declined a rule because the population was too large, was that right; what is the unit of cost for a rule over a hierarchy; how do I decide between a narrow anchor and a complete one

# A prefix-closed rule costs terminals, not members

- **Type:** `knowledge`
- **Date:** `2026-09-21`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.4.2.7.3`, which measured it while
  censusing what a routing closure cannot see. The leaf that opened it had
  already declined the complete rule in writing — *"a rule that demands a row for
  every `.md` in the tree is a different and much worse rule"* — on a population
  it had not counted.

## The question

You are deciding whether a completeness rule is affordable: *every X must be
declared somewhere*. You count the X's, the number is large, and the rule looks
absurd. So you propose a narrower anchor that catches the instances you have
actually seen.

## The answer

> **Count the entries the rule would require, not the things it would cover.**

If the registry, index or manifest the rule points at resolves by **prefix** —
a directory entry governing everything beneath it, a namespace covering its
members, a glob standing for its matches — then one entry can discharge an
arbitrary number of members, and the member count says nothing about the cost.

The cost is the **minimal covering set**: for each undeclared member, the
shallowest container all of whose members are undeclared; a container holding
even one declared sibling is mixed, so the search goes deeper and may end at the
member itself.

⭐ That is a computation, not a judgement, and it is worth doing before the rule
is argued about. It changes the conversation from *"432 entries is absurd"* to
*"14 entries, here they are, which of them do we actually want to govern?"*

## The measured instance

| quantity | count |
| --- | --- |
| tracked Markdown documents | 432 |
| already governed by a registry row | 343 |
| **undeclared members** | **89** |
| **minimal covering terminals** | **14** |

Of the 14, four are whole directories — one carries 49 members, another 13,
another 11, another 6 — and the remaining ten are individual files that sit
beside governed siblings and so cannot collapse. The rule the owning leaf had
declined as demanding 432 entries demands 14.

## ⚠️ Cheap is not the same as right

The measurement removes the **cost** objection. It does not decide the rule.
Three of the fourteen terminals are collections nobody has yet decided should
carry a lifecycle, and a rule that forces that decision by arithmetic rather
than on the merits is the same error one layer up.

> Price the rule mechanically; decide it deliberately. The number belongs in the
> argument, not instead of it.

⛔ And the narrower anchor is not automatically the safer choice. A narrow anchor
that catches every instance *observed so far* is calibrated on a sample of two —
the sample being the instances somebody happened to notice, which is exactly the
population a blind spot suppresses.

## Restated outside software

A council requiring every building to hold a safety certificate does not need one
certificate per flat. A single certificate covers the block, and the real cost is
the number of **blocks** — plus the handful of detached houses that no block
covers. Counting flats makes the policy look impossible; counting blocks makes it
a morning's work, and only then is it worth arguing about which buildings should
have been in scope at all.

## Related

- [[an-instruments-zero-describes-its-reach]] — the other half of the same
  census: before trusting *nothing is undeclared*, ask what the instrument's
  anchor can reach at all.
- [[a-census-is-an-instrument-not-a-table]] — why the 14 above is produced by a
  tracked instrument rather than counted by eye.
- [[calibrate-over-the-history-that-contains-the-instance]] — why an anchor
  scored only against the instances already found is scored against a sample the
  blind spot selected.
- [[an-absence-claim-is-a-census-over-the-corpus]] — *nothing governs X* is a
  quantifier, and this note is what it costs to make it false.
