answers: did this project adopt the live-document containment doctrine; where is this repository's containment contract written down; why is there no LIVE_DOCUMENT_SIZE_CONTAINMENT.md at the root; what enforces each clause of the containment contract; has the donor adoption guide been revised since we read it; what does a fresh session need in order to resume without loading a monolith

# The containment contract is adopted; the donor's document is not

- **Type:** `decision`
- **Date:** `2026-09-21`
- **Owner:** leaf `SIGNOFF-REPAIR.11.4.2.7`, executing `.11.4.2`'s named
  *broader donor-package review and deliberate local adoption if appropriate*,
  deferred there by `.11.4.1` on 2026-09-09.
- **Status:** accepted.

## The question

The director-authorized donor package (fsmgen) is a guide plus a neutral
doctrine document. Its **What the receiving project adopts** section says to
store an authoritative copy of `LIVE_DOCUMENT_SIZE_CONTAINMENT.md` at the
receiving repository's root and to preserve its project-neutral normative body.

This repository has no such file and has never had one. It does have, built
over `SIGNOFF-REPAIR.11.4.1` and `.11.4.2`, the machinery the guide's
deliverables describe. So: copy the document, or not?

## The donor's revision state, measured rather than assumed

Director instruction §18 asks whether the guide has been updated. It has not
been, since before this project first consulted it:

```bash
cd <fsmgen> && git log --format='%h %ad %s' --date=short -- \
  docs/LIVE_DOCUMENT_SIZE_CONTAINMENT_ADOPTION_GUIDE.md | head -1
# 727e0d086  2026-09-05  …: delegate live surface-count authority
```

SHA-256 of the guide as read: `8f77fa39c9bcb9cfc43166259a627a6ced64682030088400b727dccc5d674a53`.

⛔ The donor is read-only reference material. Its thresholds, surface
identifiers, task ids, decisions, debt allowances and retention guarantees are
evidence about the donor project, which the guide itself says are not portable
policy. Nothing of that kind is copied here.

## The decision

> **The contract is adopted and met. The document is NOT copied, and the reason
> is the guide's own stop condition.**

The guide says to stop and request direction when *"two sources plausibly claim
canonical authority for the same information"*. `README_POLICY.md` is already
this repository's adopted landing-page and routing-pressure policy; its guard
`scripts/check_readme_stability.sh` enforces the routing closure and the caps;
`.doctrine/readme_routes.txt` is the registry of governed surfaces and now
declares each one's machine-readable assertions. A second root document
restating the same normative body would be a second claimant to that authority,
and a 32 KB body nobody here wrote and nothing here derives is the mirror this
project has repeatedly had to remove.

⭐ What the absence of that document actually cost was **discoverability**, and
that is what this record repairs: one addressable answer to *what is this
project's containment contract, and what enforces each clause*.

## The contract, and what enforces each clause

| Clause | Where it lives here | What enforces it |
| --- | --- | --- |
| Bounded live views | `README_POLICY.md`; `.doctrine/readme_routes.txt` | `README-STABILITY` (line AND byte caps), `LEDGER-RUNWAY` (rotation thresholds) |
| Every routed destination is governed | `.doctrine/readme_routes.txt` | `README-STABILITY` routing-pressure closure |
| A declared control is TRUE, not merely present | the registry's fifth field | `ROUTE-CONTROL` |
| Lifecycle per surface | the registry's class column | `README-STABILITY` arity leg |
| Lossless transitions with exact retrieval | the chain notice in each ledger's footer | `scripts/rotate_changelog.py` refuses to retire a record absent from the predecessor |
| Derived-state truth for the resume pointer | `MEMORY.md` | `POINTER-CURRENCY`, calibrated over 646 pointer versions |
| Transition debt recorded, never hidden | the registry's control sentences | `ROUTE-CONTROL` ceilings |
| Unconditional hook and CI wiring | `.githooks/pre-commit`, `.github/workflows/doctrines.yml` | both run `scripts/check_doctrines.sh` |
| Project data on the repository volume | `CLAUDE.md` §13 | `STORAGE-LOCALITY` |

## The completion test, evaluated

The guide defines completion behaviourally rather than documentarily. Each
clause, with the evidence:

| The test says | Here |
| --- | --- |
| load a small bounded current view | `MEMORY.md`, 6,214 bytes under a 7,168-byte cap |
| follow one exact pointer to the relevant unit | its `active_work_unit` and `next_action`, enforced by `POINTER-CURRENCY` |
| retrieve any retained history deterministically | each ledger footer carries a `git show <sha>:<path>` chain notice, losslessness verified before the notice is written |
| one unconditional gate rejecting every undeclared or over-limit path | `scripts/check_doctrines.sh`, run by the pre-commit hook and by CI |
| without loading an ever-growing monolith | the five core live documents total roughly 145 KB, from 1,559,250 before this lane |
| exact current-state fields derived at use or authority-checked | `POINTER-CURRENCY` for the resume pointer; `ROUTE-CONTROL` evaluates each declared assertion at use |
| revision-bound evidence stays clearly historical | dated ledgers are history; the snapshot is a separate role, split at `.11.4.2.6.6` |

⚠️ Re-derive these figures rather than reading them here — `wc -c MEMORY.md`,
and `python3 -B scripts/census_route_controls.py`. A number restated where
nothing derives it drifts (`SIGNOFF-REPAIR.11.16`).

## The one deliverable that is NOT met, with an owner

The guide asks for a **health target** as well as an **enforcement ceiling** for
every governed surface. This repository has both only for the three ledgers:
`LEDGER-RUNWAY` requires headroom for the next entry and fires well before the
ceiling. `README.md` and `MEMORY.md` have a ceiling and no health target — they
refuse at the cap with no earlier signal.

🔴 That is not a theoretical gap. `MEMORY.md`'s cap has been crossed **seven
recorded times**, each resolved by evicting a standing warning under pressure at
the moment of the crossing. A health target is exactly the mechanism that turns
that into a deliberate shed with runway. Owner: `SIGNOFF-REPAIR.11.4.2.7.1`.

## What was deliberately not done

- No donor threshold, path, identifier, measurement, debt allowance or retention
  guarantee was copied. Every number here was derived locally.
- No new root document. See the stop condition above.
- No cap or ceiling was raised. The guide's own anti-pattern list names that
  first, and so does this project's.

## Related

- `docs/decisions/2026-09-06_readme-policy-readoption.md` — the adopted policy
  this record says holds the containment authority.
- `docs/decisions/2026-09-21_live-status-carries-two-roles-and-a-pointer-that-never-resolved.md`
  — the split that made the snapshot a bounded current view.
- `docs/decisions/2026-09-21_the-second-ledgers-threshold-is-the-first-ledgers-window.md`
  — how a ceiling is derived here, and why it is pinned.
