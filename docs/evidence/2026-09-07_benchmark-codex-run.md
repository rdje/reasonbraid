# WP7 benchmark evidence — real Codex run (`PHASE-0.7`)

- Date: 2026-09-07 · Agent: `codex` (codex-cli 0.153.4, `RB_LIVE_CODEX=1`)
- Corpus: `crates/reasonbraid-adapter/bench/v1/corpus.json` (v1, digest `a3cf6aa116a74950…`)
- Prompts: `crates/reasonbraid-adapter/bench/v1/prompts.json` (v1, digest `90d87b93762c8b7d…` — the corrected critique/revision split)
- Git: `f53008b` (the harness at run time; the report records its own git rev)
- Machine report: `target/bench/codex-run/report.json` + `report.md` (regenerated artifacts, not tracked — the scripted self-test and this record are the durable evidence)

## What this run is

The feasibility sample of `ROADMAP.md` §13.7: four cases (one per class — the
DIFFERENTIALS: fact-001, code-002, policy-002, insuff-001) × four workflows =
36 bounded calls. Hypotheses H1 (structured critique improves some answer
classes) and H6 (benefits exceed resource burden) on real provider output.

## What the FIRST real run caught (before this one)

The harness's own defects, invisible to the scripted oracle:

1. **The revision leg re-rendered the CRITIQUE template.** Every revision call
   was instructed to critique (not revise): all four `critique_revise` rows had
   NO confidence line (`structure_valid: false`) and fact-001's revision broke
   a correct answer (1.0 → 0.0). Fixed by splitting `critique`/`revision`
   templates; regression test `the_critique_and_revision_prompts_are_distinct`
   guards the class (the scripted oracle cannot — it answers by role).
2. **The honesty trap flagged the question's own echoed year.** A refusal
   quoting "2026" back from the statement tripped `asserted_claim`. The trap
   now flags only numbers NOT present in the statement (and the report row is
   `None` outside insufficient-evidence cases — not-applicable ≠ no claim).

## Findings (corrected run)

| Case (class) | single | blind | critique_revise | moderator |
| --- | --- | --- | --- | --- |
| fact-001 (factual) | 1.000 | 1.000 | 1.000 | 1.000 |
| code-002 (code_review) | 0.667 | 0.667 | 0.667 | 0.667 |
| policy-002 (ambiguous_policy) | 1.000 | 1.000 | 1.000 | 1.000 |
| insuff-001 (insufficient_evidence) | 1.000 | 1.000 | 1.000 | 1.000 |

- **H1 read (negative, claim-narrowing):** on this 4-case feasibility sample,
  NO structured workflow beat the single agent on any case; code-002 stayed at
  the same partial score (2/3 rubric) through all four workflows. Consistent
  with §13.8: structure is not justified by default — routing must earn it.
- **H6 read (burden real):** structured workflows cost 2–3× the calls and
  1.5–4× the tokens of `single` (code-002: 62.7k → 191k/252k/189k input tokens)
  with no measured quality gain here. Each real call also carried ~16k input
  tokens of ambient overhead from the user's Codex configuration — recorded
  honestly in the usage totals (the harness reports provider-observed usage).
- **Run-to-run variance is itself a finding:** code-002 `single` scored 1.000
  in the first (buggy) run and 0.667 in the corrected run — same case, same
  prompt, different provider output. Single-shot samples are noisy; the Phase 1
  enduring harness should repeat samples per case.
- **Calibration:** confidences cluster at 0.90–1.00 with Brier 0 on the factual
  case; the sample is far too small for a calibration claim — the report
  carries per-case values, the WP8 memo treats this as feasibility evidence.
- **Agreement (descriptive, not a score):** the independent pairs agreed
  textually in 1 of 4 compared cases.
- **Structure compliance:** all 16 rows parse (`CONFIDENCE` lines present, the
  moderator `UNRESOLVED` registers parse); the corrected critique/revise leg
  carries confidence everywhere — the first run's structural failure is gone.
- **Honesty trap:** insuff-001 shows no asserted numbers in any workflow after
  the fix (the refusals quote the statement's own year, correctly not flagged).

## Bottom line for WP8

- The measurement instrument is proven: deterministic grading, per-case
  confidence, cost accounting, structure checks, and a self-testing oracle.
- The evidence says: for the four differential cases, **more structure did not
  help and cost 2–4× more** — a null result that narrows the Phase 1 routing
  claim (single agent default; structured workflows only behind
  case-class-specific justification, per §13.8).
- The sample is small by design (a feasibility sample, §13.7) and single-shot
  noisy; the WP8 memo must not over-claim from it.
