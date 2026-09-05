# Claim-verification architecture is adopted

- **Type:** `decision`
- **Date:** `2026-09-05`
- **Status:** `active`
- **Owner / source:** director startup policy; leaf `RB-SEED.3`
- **Source copy:** `docs/CLAIM_VERIFICATION.md` (from pgen `docs/CLAIM_VERIFICATION.md` as of 2026-09-05)
answers: what does "checked" mean before publishing a number; is CLAIM_VERIFICATION adopted?

## The fact / decision

ReasonBraid adopts the portable claim-verification standard as architecture #5,
alongside task-trees, memory-architecture, knowledge-map, and doctrine-enforcement.
The project-owned copy is `docs/CLAIM_VERIFICATION.md`.

## Why

Checking a claim twice the same way repeats its blind spot. Before anyone acts
on a measurement, it must be re-derived from source, falsified against an oracle
the author did not build, and made durable (tracked producer, watched against
staleness). Missing legs are named, never hidden.

## How to apply

- Working definition of "checked": the three legs in `docs/CLAIM_VERIFICATION.md` §3.
- Publishing contract: `docs/CLAIM_VERIFICATION.md` §4. A missing leg is stated.
- No new doctrine *check* in this leaf. Mechanizing claim tags or derived-constant
  gates is later work, owned when the first product measurement is published.
- Sweep of published product constants at adoption: none (scaffold + roadmap prose
  only). Revisit when Phase 0 evidence reports land numbers.
- Live-document size containment remains not adopted (`RB-SEED` decision).

## Adoption checklist (this leaf)

1. §3 and §4 adopted — this record.
2. Sweep published constants — none to derive yet.
3. Sweep untracked producers of *published* numbers — none found; `git_message_brief.txt` is untracked by design and publishes nothing.
4. Fire every control — no product claim-controls exist yet.
5. Claim-tag in PR template — not added (no PR template yet); first evidence report that publishes a number owns adding tags.
6. Restate in this domain: a gate, cost, quality, or recovery number in a Phase 0 evidence report is unpublished until the three legs are named.
