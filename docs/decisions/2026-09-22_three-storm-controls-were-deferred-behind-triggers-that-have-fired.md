---
answers:
  - Which of Phase 3's deferred storm controls have had their triggers fire?
  - Why is automatic thread initiation bounded today, and why is that bound accidental?
  - What must land before repeated auto-initiation is repaired?
  - Which §11.5 wake-checklist items did PHASE-3.5.3 name and not build?
---
# Three storm controls were deferred behind triggers that have fired, and the only bound on auto-initiation is a defect

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.5.3`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §10.7 (notification and storm controls), §11.5 (autonomous wake and
  initiation), §14.1 (*autonomous child threads and recursion depth*), §3.2 (*recursion
  depth*); `docs/tasks/PHASE-3.md` `.4` (the named deferrals) and `.5.3`;
  `SIGNOFF-REPAIR.5.2` (the once-per-tenant auto-initiation key)

## Context

`PHASE-3.4` built the storm controls' dev-scale core (fan-out caps, call expiry) and
deferred six more, each with a named trigger. `.11.4.7.2.1.5`'s re-derivation of row 16
noticed one trigger that read as fired. This record adjudicates all six against the present
code.

## The six, adjudicated

| deferred control | its trigger | fired? | how it is decided |
| --- | --- | --- | --- |
| parent/causation chains, max autonomous depth, cycle detection | *the first agent-initiated call* (the `.5` autonomous-initiation lane) | 🔴 **FIRED** | `POST /v1/threads/auto` ships (`PHASE-3.5.3`, `done`); `AutoCreateRequest` has no parent or causation field, and the handler checks no depth |
| quiet hours / local node policy | the `.5` wake-policy lane | 🔴 **FIRED** | `PHASE-3.5.2` (`done`) made the wake policy an enforced gate; the profile carries `operating_hours` (`matching.rs`), and nothing enforces it at auto-wake |
| max offline backlog | the `.5` subscriptions | 🔴 **FIRED** | `PHASE-3.5.2` shipped subscriptions; `node_inbox` has no per-node cap |
| per-origin and global circuit breakers | *the first multi-tenant storm observed* | ⚠️ **UNOBSERVABLE** | the `storm_control` 429 is returned and recorded NOWHERE, so the trigger can never be seen to fire: `.11.4.7.2.1`'s *a trigger nothing evaluates* shape |
| duplicate-thread suggestions | the `.5` subscriptions' semantic layer | not fired | the semantic layer is ADR-014's, behind its own trigger; `matching.rs` is structural |
| emergency broadcast authority | an emergency class exists | not fired | no emergency class exists in the vocabulary |

## The finding that orders the work

⛔ **Automatic thread initiation is bounded today only by a defect.** `SIGNOFF-REPAIR.5.2`
records that `create_thread_auto`'s idempotency key is `auto_{role}_{tenant}`: a function
of the role and tenant alone. So a role can auto-initiate exactly ONCE per tenant, ever,
and every later attempt replays the first thread. That defect is the only thing preventing
an unbounded chain. A thread that wakes a role, which auto-initiates a thread, which wakes
another role, cannot recur today only because the second initiation replays the first.

⇒ **Repairing `.5.2`'s repeated initiation without a causation chain, a depth limit and
cycle detection would OPEN a runaway.** The depth/cycle build (`.11.4.7.2.1.5.3.1`) must land
before, or together with, `.5.2`'s repeat-initiation repair. `.5.2` carries that
sequencing lock.

## The second finding — a `done` leaf that built a third of its goal

`PHASE-3.5.3`'s goal names the grant's bounds as *the topic + the audience + the rate + the
depth + the spend + the side-effect bounds*, and the full §11.5 checklist:
- auto-wake;
- advertisement;
- confidentiality;
- *concurrency + operating hours*;
- *central + local reservation*;
- *recursion/duplicate/notification controls*;
- allowed tools;
- adapter health;
- billing route.

Its `Done` note records four gates: topic, confidentiality, concurrency and spend. The
rest are neither built nor named as deferred. `.11.4.7.2.1.5.3.2` owns adjudicating each
one, deciding server-side or node-side, then built or given a trigger.

## Decision

- 🔨 `.11.4.7.2.1.5.3.1` — causation chain + max autonomous depth + cycle detection for
  auto-initiation. **Frontier**, and a prerequisite of `.5.2`'s repeat-initiation repair.
- 🔨 `.11.4.7.2.1.5.3.2` — the §11.5 items `PHASE-3.5.3` named and did not build, quiet
  hours among them.
- 🔨 `.11.4.7.2.1.5.3.3` — the max offline backlog.
- 🔨 `.11.4.7.2.1.5.3.4` — record every `storm_control` refusal, so the circuit-breaker
  trigger becomes evaluable. The breakers stay deferred on that now-observable condition.
- ⏸️ Duplicate-thread suggestions stay behind ADR-014's trigger. Emergency broadcast waits
  for an emergency class. Both conditions are readable today.
