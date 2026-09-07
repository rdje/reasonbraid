# Phase 5 gate record — G5 (`PHASE-5.6.2`)

## Context

`PHASE-5.6.2` (the G5 exit gate package, the Phase-1 `.1.8.2` / Phase-4
`.7.2` pattern): Phase 5 ships the deliberation-quality programme — the
workflow profiles (`.1`), the blind-first structured deliberation (`.2`),
the moderator/synthesizer constraints (`.3`), the evaluation service
(`.4`), and the routing policy (`.5`). The gate: `ROADMAP.md` §20.6's
**G5** — benchmark thresholds and honest inconclusive behavior, blocking
the "deliberation improves answers" claim. The exit condition
(`ROADMAP.md` §25.1): after Phase 5, remove or narrow quality claims if
the controlled evaluation does not beat cheaper baselines.

## Decision

- **G5 outcome: Met as a subtraction gate.** The gate's blocking function
  is discharged by subtraction: the product makes NO "deliberation
  improves answers" claim — the `.6.1` census found the README carries
  only the mechanism claim ("Rust — not an LLM — enforces identity,
  authorization, ordering, budgets, and publication"; the model output
  stays untrusted until the deterministic rules accept it), which ships
  and tests. The first controlled evaluation (`docs/evidence/
  2026-09-07_benchmark-codex-run.md`, the 4-case differential sample)
  reads **H1 null**: no structured workflow beat `single`, at 2–4× the
  cost (H6). Per §25.1, the claims NARROW rather than the gate failing:
  the machinery is built and measured, the default stays the cheapest
  baseline (the `.5` rule-based policy encodes exactly this).
- **The honest-inconclusive half is SHIPPED and tested:** the twelve
  §13.4 terminals with the family rule (profiles 28), the unresolved
  register riding the close, the budget-exhausted denial with the
  honest `Inconclusive` terminal, the blind commitment point (profiles
  27), the honesty trap in the harness, the synthesis's re-derivable
  coverage (profiles 31).
- **The benchmark-threshold half is BUILT as an instrument:** the WP7
  harness's scripted self-test proves the measurement pipeline against
  the corpus's expected scores; the `.4` service persists the registry,
  the seed-declaring runs, the shadow trials, the calibration, and the
  baseline/threshold gates (evaluation 3); the routing records make
  every selection auditable (routing 2).
- **The named deferrals** (each recorded, none silent):
  1. **The enduring multi-run accumulation**: the live evidence is ONE
     feasibility sample (36 calls) — the `.4` service is the instrument
     the accumulation rides; no cross-run quality claim exists.
  2. **The calibration claim**: Brier 0 on one factual case — deferred
     until the accumulation (S-2).
  3. **The learned routing**: shadow-only by design (`.5.3` — recorded,
     evidenced, never applied; S-3).
  4. **The §19.5 axes the harness does not measure yet** (the
     discovery-relevance, the correlated-agent robustness): named as
     unmeasured — nothing claims them.
- **The subtraction record** (`2026-09-07_phase5-subtraction-record.md`)
  lists the five narrowed/withdrawn claims — no empty lists.

## Consequences

- The tree closes (`PHASE-5` `done`), the frontier moves to `PHASE-6.1`
  (the semantic policy lane — §25.1's pre-Phase-6 gate applies: no
  binding policy governance ships unless real owners accept the
  authority/correction model).
- The README's stale status line is fixed (the S-5 candidate) — the
  landing page now reads the current phase.
