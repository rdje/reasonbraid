# The verification strategy for a networked agent platform is assessed and scoped

- **Type:** `decision`
- **Date:** `2026-09-12`
- **Status:** `active`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.5`; director question 2026-09-11
- **Assesses:** `docs/tasks/artifacts/signoff_review/verification-strategy-proposal.md`
answers: which parts of the verification-strategy proposal become tracked work; does MCP help verify this project; why do assertions over return values miss this project's defects; what must a concurrency control assert?

## The decision

All four proposals are accepted in principle. They are SEQUENCED rather than
started, and one new lane is added ahead of three of them because it is cheaper
than all of them and has already paid.

| # | Proposal | Decision |
| --- | --- | --- |
| 0 | **Clean-state lane** (new) | **Adopt now.** Not in the original document. |
| 1 | A control must have failed on the defect it was written for | **Adopt now**, as a doctrine gate. |
| 4 | Adversarial concurrency lane | **Adopt next.** Its prerequisite is now largely done. |
| 2 | Pipeline-stage coverage registry | **Adopt after `.7.3.3.4`**, built from a real instance. |
| 3 | Deterministic simulator | **Tracked, deliberately not now.** Blocked on a predictable checkpoint. |

## Why the evidence got stronger, not just longer

The proposal rested on six defects, none of which was a wrong return value. Four
more were repaired on 2026-09-11/12 — `REPAIR-0080`, `0083`, `0085`, `0086` —
and the count is now **ten defects, none of them a wrong return value, and none
of them reachable by MCP.** Each was identity, lifetime, ordering, concurrency or
environment.

Two of the four sharpen specific proposals:

- `REPAIR-0086` is the clearest instance of proposal 1 the project has produced.
  The conformance scenarios passed for the entire life of an `ETXTBSY` race, and
  the earlier repair's own comment claimed it had left "no window at all". The
  new control asserts the INVARIANT and fails against the superseded design
  **while all three scenarios still pass.** That asymmetry is the whole proposal,
  demonstrated.
- `REPAIR-0083` and `REPAIR-0086` were both concurrency defects invisible to the
  development platform, which strengthens proposal 4. The proposal named
  fixture-ownership repairs as its prerequisite; `REPAIR-0082`, `0085` and `0086`
  are that work, and it is now substantially complete.

## The new lane, and why it goes first

**Clean state is an instrument, and it is nearly free.** Two of the three
"remote-only" failures repaired on 2026-09-11 needed only a CLEAN machine, not a
remote one:

- the git fixtures read the developer's ambient git identity, reproduced by
  suppressing git configuration;
- a `pg_guard` check-then-act race never fired locally because `target/` stays
  warm between runs, making the racing branch dead code — reproduced by removing
  one directory.

Both cost one command and had been costing CI round-trips instead. Only
`ETXTBSY`, the 108-byte `sun_path` limit and inode reuse are genuine kernel
differences that require a Linux runner. This lane runs the suite against a cold
tree and a suppressed ambient environment, and it is cheaper than every other
proposal here. Recorded in `TOOLBOX.md`.

## What proposal 1 must assert

A control written for a concurrency defect must assert the property that makes
the defect impossible, not the absence of the symptom — and its acceptance
record must show it FAILING against the unrepaired source. The signature of a
good one: it fails against the superseded design while the ordinary scenarios
still pass. If it passes against the old code too, it is not testing the repair.

Its honest limit is the same one `TASK-ACCEPTANCE` already carries: the gate
verifies that a failing run was RECORDED, not that the control is well chosen.

## On MCP, unchanged and now better evidenced

MCP widens who can reach the system; it does not deepen what can be seen inside
it. It would have caught none of the ten defects, because those were absences and
duplicates, and absences are not return values. It is also still unqualified on
its own wire (`.6.1`–`.6.3` open). It stays in a CONFORMANCE lane and never in the
correctness lane, per ROADMAP §25's stop/reframe trigger. Its real value —
heterogeneous real clients, and an external yardstick this project did not write
— is unaffected by that placement.

## The binding constraint

No new lane lands before `SIGNOFF-REPAIR.11.4.3.1.2.15` explains the checkpoint's
~2,982 unaccounted seconds. A tiering story cannot be built on an unmeasured
cost, and a gate people route around is a gate that lies. The clean-state lane is
exempt because it replaces CI round-trips rather than adding to them.
