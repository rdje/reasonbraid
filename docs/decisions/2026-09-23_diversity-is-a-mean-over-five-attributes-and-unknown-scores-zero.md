---
answers:
  - How does the ranking's `diversity` feature treat a dependence fact nobody declared?
  - Why is it a mean over all five attributes rather than the heaviest overlap?
  - What does a dependence indicator say about an attribute nobody declared?
---
# Diversity is a mean over the five attributes, and an unknown fact scores zero

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.5.1.3`
- **Date:** 2026-09-23
- **Work unit:** `REASONBRAID-REPAIR-0444`
- **Cites:** ROADMAP §10.3 (stage 2; *visibility-safe explanation*), §10.4 (*dependence indicators*, never an independence claim); `docs/decisions/2026-09-23_the-directory-isolation-goal-line-three-items-met-three-live-defects.md` (DOC-0149, the census that found the three shapes).

## The fact / decision

The `diversity` feature (`matching::diversity`) is the **mean over the five
dependence attributes** (`dependence::ATTRIBUTES`: provider, model family,
harness, lineage, owner). An attribute contributes only when it is known on
both sides — the candidate declares it and at least one other eligible
candidate declares the **same** attribute — and then contributes `1 −` the share
of those declarers holding the candidate's value. Every other attribute
contributes 0. The denominator is always five.

A dependence indicator counts the members that do not declare its attribute
(`undeclared`), and its explanation says *varies* only when at least two
members declare the attribute and no two share it.

## Why

- **Unknown contributes nothing** was already the rule the code's own doc
  stated; the code scored a candidate with no facts `1.0`, the maximum.
- **The heaviest-overlap form cannot be repaired by skipping unknowns.**
  Excluding an undeclared attribute from a maximum still lets a candidate hide
  a shared provider and score as if it had none. With a fixed denominator an
  undeclared attribute scores what an attribute shared with everyone scores,
  so declaring less can never rank a candidate higher. That removes the
  incentive to withhold facts, which a max cannot.
- **Attribute to attribute.** A provider and a harness are different
  conditions; the same string in two of them is no evidence of shared failure.

## Consequences

- Scores are lower in absolute terms than before (a candidate unique on one
  known attribute scores `0.2`, not `1.0`). The ranking is ordinal and every
  candidate is divided by the same five, so this changes no order that the
  known facts justify.
- `lineage` is declared by nothing today, so it contributes 0 to everyone;
  `owner` is the call's tenant, shared by every joiner, so it contributes 0 at a
  single-tenant close. Both are uniform across candidates and reorder nothing.
- A snapshot taken before this repair carries indicators without `undeclared`;
  the stored record is kept as it was taken.
