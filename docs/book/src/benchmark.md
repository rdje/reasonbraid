# The deliberation benchmark

The WP7 experiment: does more structure beat one good agent — and at what
cost? A small versioned corpus is run through four workflows, graded by
deterministic rules (never by another model), and reported per case with
confidence and spread — no bare averages, no independence score.

## Run it

```bash
target/debug/rb-bench --agent scripted                # deterministic, no tokens
RB_LIVE_CODEX=1 target/debug/rb-bench --agent codex \  # real calls, deliberately
    --case fact-001,code-002,policy-002,insuff-001 --max-calls 36
```

The report lands under `target/bench/<run>/`: `report.json` (machine-readable,
with the corpus and prompt digests) and `report.md` (the review surface).
The scripted agent plays the corpus's own scripted answers through the real
adapter contract — the harness's self-test proves the scoring pipeline before
any tokens are spent; the Codex agent produces the actual numbers.

## The four workflows

| Workflow | Calls | Shape |
| --- | --- | --- |
| `single` | 1 | one well-prompted agent |
| `blind` | 2 | two independent answers; a DETERMINISTIC adjudicator (the grader) presents the higher-scoring one |
| `critique_revise` | 3 | answer → critique → revision (revision quality = the score delta) |
| `moderator` | 3 | two independent answers → synthesis with a REQUIRED structured `UNRESOLVED` register |

## The corpus (v1, hashed into every report)

Eight cases — two per class — with ground truth where it exists, presence
rubrics otherwise, and a source list where citations apply:

- **factual** — known answers (calibration: Brier vs the binary outcome).
- **code_review** — seeded-bug snippets scored against a rubric.
- **ambiguous_policy** — no single right answer; the rubric rewards covering
  both sides and stating the conditions.
- **insufficient_evidence** — the correct behavior is an honest refusal; the
  harness traps asserted numbers and counts fabricated `CITE[n]` references.

## What the report refuses to do

- **No LLM judge** — a judge model would share the measured models' correlated
  errors. All scores are re-derivable rules.
- **No independence score** — agreement between the independent answers is a
  descriptive count with the warning that agreement ≠ correctness.
- **No average-only summaries** — every aggregate carries n / min / mean / max.
- **No invented calibration** — Brier is factual-only; rubric confidence is
  reported, not scored.
- **Unmeasured stays unmeasured** — human-review minutes are owned by the WP8
  study and recorded as `not_measured`, never zero.

The results feed the WP8 decision memo: which workflows deserve Phase 1
support, and which stay experiments. A null or negative result is an accepted
outcome — it narrows the routing claim honestly.
