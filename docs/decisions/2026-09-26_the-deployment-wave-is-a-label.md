---
answers:
  - Does a deployment assignment's `wave` order or hold back a rollout?
  - When would the wave become a sequencer, and what would that require?
---
# The deployment wave is a label; nothing orders a rollout by it yet

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.9.3.3.4`
- **Date:** 2026-09-26
- **Work unit:** `REASONBRAID-DOC-0182`
- **Cites:** ROADMAP §15.9 (*Rollouts use waves/canaries: … pause on failure
  thresholds; continue, waive, reject, or roll back per target*);
  `docs/adr/021-target-deployment.md` (*the rollouts use the waves/canaries*);
  `docs/book/src/policy-lifecycle.md` (*Deployment and receipts*)

## The fact / decision

1. **`wave` is a LABEL.** An assignment stores the integer its caller gives it and
   returns it on every read. Nothing compares, orders, gates or pauses on it.
   Measured at REPAIR-0527: `git grep -n -w wave -- 'crates/*/src/*.rs'
   'crates/*/src/**/*.rs'` finds the field declared, bound into the INSERT and
   selected back out, and two doc comments; no expression reads it.
2. **Nothing executes a deployment at all.** The server records an assignment, a
   target's receipts and drift; no code path applies a publication to a target,
   opens a change for it, or dispatches work from an assignment (the same census
   over `deployment_assignments` finds only the recorder, the receipt path, the
   drift existence check and the reads). There is no rollout for a wave to order.
3. **The sequencer is deferred** (`SIGNOFF-REPAIR.9.3.3.4.1`), with the trigger:
   the first code path that ACTS on an assignment (applies it, opens a change for
   it, or dispatches work from it) rather than recording it. At that point wave
   N+1 must wait for wave N's coverage, and §15.9's pause/continue/waive/reject/
   roll-back per target must be built with it.
4. **The book says so.** It called the field a *canary wave*, which a reader takes
   as a staged rollout; it now says the wave is a label the caller chooses.

## Why

- **A sequencer with nothing to sequence is decoration.** Ordering is a property
  of execution; until something executes an assignment, "wave 2 waits for wave 1"
  has no observable meaning, and a control could not be written for it.
- **The label is still worth keeping.** It is the caller's own grouping of targets
  (the §15.9 wave plan), recorded with each assignment and readable back, so a
  later sequencer inherits the plan that was declared rather than inventing one.
- **The honest wording is cheap and the misleading one is class 3.** "Canary"
  promises that a failing first wave stops the rest; nothing does.

## How to apply

- Do not describe waves as ordering, gating or pausing anything until
  `.9.3.3.4.1` lands.
- A change that makes the server act on an assignment evaluates `.9.3.3.4.1`'s
  trigger in the same commit.
