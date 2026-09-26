---
answers:
  - Can a thread be decided under role_weighted or human_committee?
  - Why are two of ROADMAP §13.3's rule families refused at thread creation?
  - What would it take to support weighted or committee approval?
---
# Weighted and committee rules stay refused until they are designed

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.8.1.1.4`; the design is deferred to `SIGNOFF-REPAIR.8.1.1.4.1`
- **Date:** 2026-09-26
- **Work unit:** `REASONBRAID-REPAIR-0548`
- **Cites:** `ROADMAP.md` §13.3; `docs/book/src/decision-rules.md`;
  `docs/book/src/governance-charter.md`;
  `docs/decisions/2026-09-25_the-corrective-tree-ends-at-a-bug-bar.md`

## The fact / decision

`role_weighted` and `human_committee` stay **declarable only as refused**. A
thread that names either at creation is refused with `400 invalid_command`, and
the refusal names `SIGNOFF-REPAIR.8.1.1.4.1`. A governance charter may still list
them as allowed, because the charter records the tenant's policy and the create
check is what refuses the thread.

## Why

- **The false claim is already gone.** `.8.1.1.4` was opened because a rule could
  be declared that the close could not count. `.8.1.1.2` made both families
  refused at creation, and the book says so in both chapters, so no reader is told
  they work.
- **What remains is design, not repair.** `role_weighted`'s weights are
  charter-defined by §13.3, and no charter schema holds them. §13.3 names
  `human_committee` approval but gives it no bar: who the committee is, what
  quorum it needs, and what a committee member's ballot means next to a role's.
  Inventing either would put an undesigned rule behind the server's count, which
  is worse than refusing it.
- **The bug bar's own resolution.** *"Narrowing the claim is a valid resolution
  when fixing the code is not warranted"*, and a feature the roadmap has not
  built is its example of a deferral.

## Trigger

`.8.1.1.4.1` is picked up when a roadmap phase schedules role-weighted, chambered
or committee approval, or when a deployment's governance charter allows either
family and a tenant asks to decide a thread under it. It then owns a charter
schema for the weights, a §13.3 bar for the committee, and their counting in the
close.
