# The evaluation harness

The harness records how a workflow performed against a fixed corpus: the corpus
version, the runs taken against it, shadow routing trials, calibration over named
runs, and gates that block on a regression against a baseline.

⛔ **Read the [limits](#what-this-harness-does-not-establish) at the foot of this
chapter before using any of it as evidence.** Several are known defects with an
owning repair leaf, and one of them makes an unmeasured gate report `pass`.

## The routes

```text
POST   /v1/evaluations/corpora                      register one corpus version
GET    /v1/evaluations/corpora                      the registered corpora
POST   /v1/evaluations/runs                         record one experiment run
GET    /v1/evaluations/runs                         the runs
POST   /v1/evaluations/trials                       create a shadow routing trial
GET    /v1/evaluations/trials                       the trials
POST   /v1/evaluations/trials/{trial_id}/results    append one per-arm results row
GET    /v1/evaluations/trials/{trial_id}/results    the trial's results
POST   /v1/evaluations/calibrations                 record one calibration
GET    /v1/evaluations/calibrations                 the calibrations
POST   /v1/evaluations/gates                        record a gate (baseline + threshold)
GET    /v1/evaluations/gates                        the gates
POST   /v1/evaluations/gates/{gate_id}/evaluations  evaluate the gate against scores
GET    /v1/evaluations/gates/{gate_id}/evaluations  the gate's results
```

## Who may call them

Every route here admits any **enrolled principal** and nothing more.

⛔ **The harness is deployment-wide, not tenant-scoped.** Unlike the policy
lifecycle tables, none of the seven `evaluation_*` tables carries a `tenant_id`
column, and no handler passes a tenant or a principal into the service. So any
enrolled principal can register a corpus, record a run, or evaluate any gate in
the deployment, and can read every other caller's records.

⚠️ This is a **recorded finding with an owning repair**, not an accepted design:
`SIGNOFF-REPAIR.8.2` clause 1 holds it, and its note records why the obvious fix
is not yet available — no `GrantAction` and no `TargetSelector` can name a corpus
or a gate, so there is nothing for an authority check to bind to yet.

## Corpora

A corpus version is content-addressed and never overwritten. The digests are the
**declared** 64-hex file digests; the harness re-derives them at run time.

```bash
curl -s -X POST localhost:4310/v1/evaluations/corpora \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "corpus_id": "retention-disputes",
        "version": 3,
        "cases_digest": "9f2b…64 hex…",
        "prompts_digest": "41ca…64 hex…",
        "cases": [{"case_id": "c1", "prompt": "…", "expected": "…"}]
      }'
```

A digest that is not 64 hex characters is refused. Re-registering an existing
`(corpus_id, version)` is refused as a duplicate — **register a new version
rather than overwriting one**.

## Runs

A run records one workflow executed against one registered corpus version.

```bash
curl -s -X POST localhost:4310/v1/evaluations/runs \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "run_id": "run_0192…",
        "workflow": "wf_retention_review",
        "corpus_id": "retention-disputes",
        "corpus_version": 3,
        "seed": 42,
        "deterministic": false,
        "trial_count": 20,
        "results": {"c1": 0.91}
      }'
```

- The corpus version must already be registered; an unregistered one is refused.
- **`seed` declares the randomness.** A run with `deterministic: false` and no
  `seed` is refused — a non-deterministic result that cannot say what its
  randomness was is not a record anybody can re-derive.
- `trial_count` must be positive.

## Shadow routing trials

A trial assigns each case to an arm and **never changes production routing** — it
is a shadow. The client does not supply the draw; the server computes it from the
`(case id, seed)` pair with `splitmix64`, so the stored record alone reproduces
the assignment.

```bash
curl -s -X POST localhost:4310/v1/evaluations/trials \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "trial_id": "trl_0192…",
        "corpus_id": "retention-disputes",
        "corpus_version": 3,
        "seed": 42,
        "arms": ["baseline", "candidate"],
        "cohorts": [{"label": "eu", "kind": "case", "members": ["c1"]}],
        "case_ids": ["c1", "c2"]
      }'
```

The response carries the computed `assignment`, a `case_id → arm` map. A cohort's
`kind` is `case` (the case ids it covers) or `subject` (the subject ids).

`POST /v1/evaluations/trials/{trial_id}/results` appends one per-arm results row.
It is **append-only**: the record's identity is its content, and nothing rewrites
an earlier row.

⚠️ **The assignment is not portable to a 32-bit target.** `splitmix64` returns a
`u64` and the arm index is `draw as usize`, which truncates to 32 bits where
`usize` is 32 bits — while the code comment beside it says the draw must be
stable across platforms. Recorded as `SIGNOFF-REPAIR.8.2` clause 2.

## Calibration

A calibration accumulates over **named runs**, each of which must already be
registered:

```bash
curl -s -X POST localhost:4310/v1/evaluations/calibrations \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "calibration_id": "cal_0192…",
        "corpus_id": "retention-disputes",
        "corpus_version": 3,
        "workflow": "wf_retention_review",
        "run_ids": ["run_0192…"],
        "brier": 0.11,
        "confidence": {"bins": [[0.9, 0.87]]}
      }'
```

## Gates

A gate is a baseline plus a threshold. **It only blocks** — a gate never promotes
anything, and passing one is not evidence of improvement.

```bash
curl -s -X POST localhost:4310/v1/evaluations/gates \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{
        "gate_id": "gat_0192…",
        "corpus_id": "retention-disputes",
        "corpus_version": 3,
        "workflow": "wf_retention_review",
        "baseline": {"c1": 0.90, "c2": 0.85},
        "threshold": 0.05
      }'
```

Recording a gate is strict: the baseline must name at least one case, every
baseline score must be numeric, and every one must lie in `[0, 1]`.

Evaluating it compares measured scores against `baseline − threshold` per case
and **appends** the result; a gate never rewrites one.

```bash
curl -s -X POST "localhost:4310/v1/evaluations/gates/gat_0192…/evaluations" \
  -H 'x-reasonbraid-principal: hpr_0192…' \
  -H 'content-type: application/json' \
  -d '{"c1": 0.91, "c2": 0.79}'
```

```json
{
  "passed": false,
  "failures": [
    {"case_id": "c2", "baseline": 0.85, "measured": 0.79, "delta": 0.06}
  ]
}
```

## What this harness does not establish

⛔ **The Phase 5 gate (G5) was met as a *subtraction* gate, and the quality-lift
claim was withdrawn.** Nothing in this chapter demonstrates that deliberation
improves an answer. The harness records measurements; it does not establish that
the thing being measured got better.

🔴 **A gate that measured nothing reports `pass`.** `evaluate_gate` skips any
baseline case the submitted scores do not mention — "the caller owns the
coverage" — so an empty score object compares no case, finds no failure and
returns `passed: true`. Measured scores are also subject to no `[0, 1]` bound,
although the **write** side of the same file refuses exactly that in a baseline.
Recorded as `SIGNOFF-REPAIR.8.2` clause 3; **do not treat a green gate as
evidence until it is repaired.**

🔴 **A storage failure is reported as the caller's mistake.** Six write paths
turn any database error into "already exists — register a new version or run id
instead of overwriting", and three read paths turn one into "the corpus is not
registered" / "no such run" / "no such gate". So an outage is indistinguishable
from a duplicate. Recorded as `SIGNOFF-REPAIR.8.2` clause 4.

⛔ **A recorded result is a claim by whoever posted it.** The server checks
shapes, references and vocabularies. It does not run the workflow, does not
verify that the scores came from the corpus they name, and — per the admission
note above — does not check who the caller is beyond enrolment.
