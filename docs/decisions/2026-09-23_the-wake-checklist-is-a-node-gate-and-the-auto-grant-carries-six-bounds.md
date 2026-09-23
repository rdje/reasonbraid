---
answers:
  - Which of ROADMAP §11.5's wake-checklist items are evaluated today, and where?
  - Which bounds must a thread:create:auto grant carry, and which does it carry?
  - Where did the operating-hours and wake-policy checks go between PHASE-3.5.2 and PHASE-3.5.3?
  - In what order are the missing gates built?
---
# The wake checklist is a node gate, and the auto grant carries six bounds — most of both are missing

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.5.3.2`
- **Date:** 2026-09-23
- **Cites:** ROADMAP §11.5 (*Before wake, the node evaluates* … and *an agent may
  autonomously initiate a new thread only under a separate `thread:create:auto` grant with
  topic, audience, rate, depth, spend, and side-effect bounds*), §10.7, §14.1;
  `docs/tasks/PHASE-3.md` `.5.2` and `.5.3`;
  `docs/decisions/2026-09-22_three-storm-controls-were-deferred-behind-triggers-that-have-fired.md`

## Context

`PHASE-3.5.3` was marked `done`. Its goal named both halves of §11.5; its `Done` note records
four initiation gates. §11.5 is actually two different requirements:
- a **wake checklist** the NODE evaluates before it wakes for delivered work;
- **six bounds** that a `thread:create:auto` GRANT carries for an initiation.

`PHASE-3.5.3` evaluated some wake items at the initiation boundary instead, which answers
neither requirement fully.

## The census (read-only; every row cites file and test in the leaf)

**Wake checklist:**

| item | today |
| --- | --- |
| auto-wake for mode/topic | ❌ not on wake. Checked only at initiation, against the profile's `interests`. `wake_policy` is declared (`crates/reasonbraid-server/src/profiles.rs:106`) and never read. |
| advertisement/context visible | ⚠️ closest: the cached admission and epoch check before dispatch (node) |
| confidentiality vs local capability | ⚠️ the server refuses classified dispatch with no evaluator. Nothing on the node. |
| concurrency | ⚠️ only `concurrency == 0` holds delivery. A non-zero cap is reported as `busy` and gates nothing. |
| operating hours | ❌ `operating_hours` is declared (`crates/reasonbraid-server/src/profiles.rs:102`) and never read |
| central + local reservation | ✅ node, before dispatch |
| recursion | ✅ initiation only (REPAIR-0415) |
| duplicate | ✅ node (command-id dedupe; the `allow_possible_duplicate` gate) |
| notification controls | ❌ |
| required tools/resources | ❌ no tool list on a run, no allowed-tools field |
| adapter health / billing route | ❌ |

**The auto grant's six bounds:**
- **spend:** carried by the grant ✅.
- **topic:** checked against the profile's interests, not the grant ⚠️.
- **depth:** a constant (REPAIR-0415) rather than a grant bound ⚠️.
- **audience, rate, side-effect:** absent ❌.

## Where the two checks went

`PHASE-3.5.2` shipped the zero-concurrency hold and **named** mode/topic and operating hours as
deferred *"with the `.5.3` trigger"*. `PHASE-3.5.3` then built initiation gates, and its
`Done` note never mentions either. A deferral handed to a sibling leaf disappeared in the
hand-off. That is the same shape `.11.4.7.2.1` found in prose, one leaf later.

## Decision — four children, in this order

1. **`.5.3.2.1` — the RATE bound, on the grant** (server, at initiation). A windowed ceiling
   on `POST /v1/threads/auto` per role, recorded like every quota event. ⭐ **First,
   because `SIGNOFF-REPAIR.5.2`'s repeat-initiation repair must land with it**: an
   undeclared chain is bounded today only by `.5.2`'s once-per-tenant defect. The two ship
   together.
2. **`.5.3.2.2` — the wake gate on the node:** `wake_policy` (mode/topic), `operating_hours`
   and a non-zero concurrency cap, evaluated before wake. Two profile fields that are
   declared and never read are the defect `.11.4.7.2.1` names; each becomes a gate, or the
   field is removed.
3. **`.5.3.2.3` — audience and side-effect bounds on the grant**, and topic and depth MOVED
   onto the grant from the profile and the constant.
4. **`.5.3.2.4` — the node-local items:** notification controls, required tools, adapter
   health and billing route. Each is adjudicated against the adapters that actually ship
   (the fake, Codex and Claude CLI adapters; `rb-node` constructs only the fake), then built
   or given an evaluable trigger.

## Correction (2026-09-23, `SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2`, `REASONBRAID-REPAIR-0419`)

Item 2 above says *the wake gate on the node*. The gate was built at the **server's
delivery boundary** — `node_channel::replay`, where `PHASE-3.5.2` had put the
zero-concurrency drain switch — and not on the node, for a reason this record did
not weigh: the profile and the clock are the server's, and the node holds a copy of
neither. A node-side evaluator would have been a second reader of the same three
fields against a second clock. §11.5's *before wake, the node evaluates* is met by
the server evaluating the block before the node is handed anything: a held role's
rows stay `queued` and are never offered. The census table's *auto-wake for
mode/topic* and *operating hours* rows now read: ✅ server, before the tail is read
(`crate::wake`). The non-zero concurrency cap remains open as
`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.2`.
