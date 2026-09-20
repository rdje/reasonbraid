# PEER-COLLAB: two peer agents resolve a problem end to end over ReasonBraid

## Metadata

- Tree ID: `PEER-COLLAB`
- Status: `active`
- Roadmap lane: none — a director-requested acceptance track, off the frozen v0.4.1 roadmap (`ROADMAP.md` §26 owns Demonstrations A and B; this is a candidate C and is NOT added to §26 while the freeze holds)
- Created: `2026-09-20`
- Owner: repo-local workflow

## Goal

The director's stated first sign of a working ReasonBraid:

> any one of my running agents can establish a communication channel with one of
> its peers via ReasonBraid and work together until a specific problem involving
> both of them is fully addressed.

The concrete instance is a **bug report between two repositories**. B has git
submoduled A because A provides a feature B uses. Using that feature B observes a
misbehaviour, logs it and tracks it in its own repo, then tells A over ReasonBraid.
The report carries everything A needs to reproduce, fix and validate. Where it does
not, A asks B over the same channel. When A believes it is fixed, A pushes and tells
B to update the submodule and re-test.

## Non-Goals

- Un-freezing `ROADMAP.md` v0.4.1 or writing v0.5.0 (`docs/decisions/2026-09-05_roadmap-v0.4.1-frozen.md`). This tree MEASURES; it does not promote features into the frozen plan.
- External/third-party federation (that is `PHASE-8`'s G8 question). Both peers here are the director's own agents, on one machine or a LAN.
- A general file-transfer service. The question is the narrow one the scenario forces: what must cross the channel for a bug to be reproduced.

## Why this is measurement, not a feature addition

`ROADMAP.md` §68 freezes new features to a parking lot and says the next roadmap
version **must cite measurements, failure observations, or implementation
constraints produced by working code**. This tree produces exactly that: the
scenario is walked against the shipped system and each step is graded with
evidence. The two capability gaps it is expected to confirm are parked in
`docs/parking-lot.md` with this tree as their revisit trigger, so nothing is
silently promoted.

## Acceptance Criteria

- Every step of the director's scenario is graded against the SHIPPED system with a named artefact (a test, a route, a demo line), not an opinion.
- Each gap names the measured constraint that makes it a gap.
- The end-to-end run uses the director's REAL agents through the live adapters, not the demonstration's scripted fake.
- Live docs, the book and the parking lot stay in lockstep.

## Task Tree

- ID: `PEER-COLLAB`
  Status: `active`
  Goal: the director's peer-collaboration scenario, measured then closed
  Children: `.1`–`.4`

  - ID: `PEER-COLLAB.1`
    Status: `pending`
    Goal: the census — walk the scenario step by step against the shipped
      system and grade each step with an artefact. The first pass (session
      note, NOT yet evidence) put most of it on Demonstration A's machinery:
      `ROADMAP.md` §26.1 already specifies two nodes enrolling from different
      LAN hosts, durable invitations, independent contribution, challenge and
      revise rounds, crash/duplicate handling and audit reconstruction — and
      `scripts/demo_two_host.sh` runs it green. The census must confirm or
      refute that mapping per step rather than inherit it.
    Acceptance: a table with one row per scenario step, each carrying the
      artefact that proves it or the measurement that shows the gap; no row
      graded from reading alone where a run is possible.
    Verification: `pending`
    Commit: `pending`

  - ID: `PEER-COLLAB.2`
    Status: `proposed`
    Goal: peer artifact exchange — what B attaches to a bug report and A
      retrieves. Blocked on a MEASURED constraint, not a preference:
      `snapshot_objects.bytes` is `BYTEA NOT NULL`, so the evidence store can
      only hold a byte string already in process memory, and
      `SIGNOFF-REPAIR.11.24.1.3` recorded that the git pack's product is an
      on-disk object database addressed by a path, which that column cannot
      hold. `SIGNOFF-REPAIR.11.24.1.3.2` already owns the storage question and
      is this leaf's prerequisite.
    Acceptance: `pending` — opens only after `.1` grades what the scenario
      actually needs to cross the wire. ⛔ Do NOT design a transfer
      surface before that: a single log or patch fits the existing snapshot
      bytes today, and a tree does not, and those are different features.
    Verification: `pending`
    Commit: `pending`

  - ID: `PEER-COLLAB.3`
    Status: `proposed`
    Goal: a source-revision pin vocabulary — "I fixed it at `<sha>`" and "I
      tested your fix at `<sha>`" as facts the thread can carry and verify,
      rather than prose in a message body. The scenario's closing handshake
      (update the submodule, re-test) is unverifiable without it.
    Acceptance: `pending` — needs `.1`'s grade on whether the existing
      evidence-reference machinery (`.1.5.1`, demonstrated in the two-host
      run) already carries this.
    Verification: `pending`
    Commit: `pending`

  - ID: `PEER-COLLAB.4`
    Status: `proposed`
    Goal: the end-to-end run with the director's REAL agents. The live
      adapters exist — `crates/reasonbraid-adapter/src/claude.rs` shells to the
      `claude` binary and `codex.rs` to `codex` — while
      `scripts/demo_two_host.sh` uses the scripted fake, so the demonstration
      that passes today does NOT prove this step.
    Acceptance: `pending`
    Verification: `pending`
    Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PEER-COLLAB.1` | `pending` | the census — nothing else may open until the scenario is graded against the shipped system with artefacts; `.2` and `.3` are guesses about which gaps are real until it runs |

## Decisions

- `2026-09-20`: the scenario is NOT added to `ROADMAP.md` §26 as Demonstration C. The roadmap is frozen at v0.4.1 and §68 sends new features to a parking lot; a demonstration added to the frozen plan without executable evidence is exactly the architecture ratchet `docs/risks.md` R-SCOPE tracks. The tree measures; §26 changes at v0.5.0 if the measurement justifies it.
- `2026-09-20`: the two capability gaps are parked in `docs/parking-lot.md` rather than opened as build leaves. The director's greenlight is recorded as the trigger that opened THIS tree; it does not by itself establish that either feature is the right answer, because `.1` has not yet graded what the scenario needs.

## Open Questions

- Does the scenario need a directory to cross the channel, or only a patch and a log? Owner: `.1`. It does not block `.1`; it blocks `.2`, which is why `.2` is `proposed` rather than `pending`.
- Are the two peers one tenant or two? The scenario says "my agents", which suggests one — and that keeps `PHASE-8`'s federation question out of scope. Owner: `.1`.

## Blockers

- `PEER-COLLAB.2` is blocked on `SIGNOFF-REPAIR.11.24.1.3.2` (the evidence-storage question). Not a blocker for the tree's frontier.

## Acceptance Checklist (required for any leaf that lands a CODE change)

- [ ] **ROOT CAUSE (WHY + WHERE)** — pending.
- [ ] **ADDRESSED (verified)** — pending.
- [ ] **NO REGRESSION** — pending.
- [ ] **FIX / LOCKSTEP** — pending.

## Verification Log

- `pending`

## Commit Log

- `pending`

## Changelog

- `2026-09-20`: tree opened at the director's greenlight, after the A↔B bug-report scenario was raised as `[DBINP]` and then explicitly greenlit for task-tree ownership.
