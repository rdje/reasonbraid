# ADR-015 — Recruitment policy baseline: the explicit invitation promotes; the open call is the matching lane's consumer

- **Status:** `accepted` (evidence-gated — the shipped `.1.3` explicit-participants
  contract IS the recruitment baseline: the human names the participants, the
  invitation is the real capability, the dispatch fires on accept)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-3.4.1`
- **Requirements:** `ROADMAP.md` §23 queue item 015 (dependence indicators and
  recruitment policy baseline); §10.5 (the recruitment protocol), §10.7 (the
  storm controls)

## Context

The roadmap queued "dependence indicators and recruitment policy baseline" as
ADR-015. The `.4` census found the recruitment INPUTS shipped — the `.1.3`
explicit-invitation flow (invite → accept/decline/remove, dispatch-on-accept,
the invitation as the real capability) and the `.3` matching surface (the
eligibility + the ranking) — but the protocol between them is unbuilt: no
§10.5 response vocabulary, no call artifact, no storm controls. The ADR must
pin the baseline BEFORE the `.4.2`/`.4.3` children build, so the open-call
machinery lands as a STRENGTHENING of the explicit contract, not a
replacement for it.

## Options

1. **Promote the explicit invitation as the baseline; the open call consumes
   the matching lane** — the human's named invite stays the recruitment
   primitive; the open call (the `.4.2` artifact) advertises to the ELIGIBLE
   set and the typed responses (join/observe/decline/defer/…) land the panel;
   the dependence indicators arrive with the `.6` lane (the trigger named).
2. Build the open-call machinery as a parallel recruitment path now — a
   second invitation system duplicating the shipped one, before any open-call
   artifact exists to ride it.

## Evidence

- **The explicit contract is shipped + measured**: the `.1.3` flow is the
  demo's own beat (invite → accept → dispatch; the pending invitation
  enqueues nothing — the work exists iff the accept does).
- **The matching lane is shipped + measured**: the `.3` surface resolves the
  eligibility + the ranking over the shipped facts — the open call's
  candidate source already exists.
- **The dependence indicators have a home**: `.6` (the roadmap's own lane)
  computes them from the typed facts (the lineage, the participation
  patterns) — nothing in `.4` needs them before the panel exists to score.

## Decision

Accept option 1. The explicit invitation is the baseline; the open call is
the matching lane's consumer (the eligibility expression + the audience + the
window/deadline/min-max/slots), and the §10.5 response vocabulary + the
§10.7 storm rules pin the shapes the `.4.2`/`.4.3` children build. The
dependence indicators land with `.6` — the trigger that re-opens them early
is a recruitment decision that needs them (a measured case where the panel
selection must weigh the lineage/participation patterns before `.6` runs).

## Consequences

- The `.4.2` call artifact rides the shipped invitation machinery (the
  thread's participants flow), never a parallel system.
- The `.4.3` storm controls gate BOTH paths (the explicit invite and the open
  call share the fan-out limits + the expiry + the depth/cycle checks).
- No dependence-indicator machinery lands in `.4` (the `.6` lane owns it).

## Revisit trigger

The first recruitment decision that needs the dependence indicators before
the `.6` lane runs — they land early as their own leaf with this ADR's
evidence package.
