# The evaluation harness

The harness records how a workflow performed against a fixed corpus: the corpus
version, the runs taken against it, shadow routing trials, calibration over named
runs, and gates that block on a regression against a baseline.

⛔ **Read the [limits](#what-this-harness-does-not-establish) at the foot of this
chapter before using any of it as evidence.** Several are known defects with an
owning repair leaf.

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

⚠️ **The missing tenant column is the DESIGN; the missing gate is the defect.**
`docs/decisions/` records the verdict for this family: the seven tables are
site-wide by design and gated by **site-operator** grants, because the harness is
release engineering rather than a tenant product surface. What was never applied
is that gate.

⛔ **An earlier version of this page said the fix was unavailable — *no
`GrantAction` and no `TargetSelector` can name a corpus or a gate, so there is
nothing for an authority check to bind to yet*. That was wrong twice**, and
`SIGNOFF-REPAIR.15` withdrew it: a site-operator gate does not bind through
`GrantAction` at all, and four site-wide surfaces already use it. Owned by
`SIGNOFF-REPAIR.8.2.5`, and not blocked.

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

✅ **The assignment is portable across pointer widths.** `splitmix64` returns a
`u64`, and the arm is chosen with `draw % arms.len()` taken in `u64` before the
result is narrowed — so the index cannot lose information on any target. Until
`SIGNOFF-REPAIR.8.2.3` the draw was narrowed FIRST, with `draw as usize`, which
truncates to 32 bits on a 32-bit host and would have picked a different arm for
the same `(case_id, seed)` than a 64-bit host does.

## Calibration

A calibration accumulates over **named runs**, each of which must already be
registered **and eligible** — a run taken against a different corpus version or
a different workflow is refused, naming the run and what disagreed. Until
`SIGNOFF-REPAIR.8.2.4` only existence was checked, so a calibration row could
assert a provenance its runs did not share:

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
anything, and passing one is not evidence of improvement. It names a corpus
version and is **bound** to it: an unregistered one is refused, as it already was
for a run and a trial (`SIGNOFF-REPAIR.8.2.4` — the gate was the one surface of
the three that named a corpus and never asked whether it existed).

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
  "compared": 2,
  "unmeasured": 0,
  "failures": [
    {"case_id": "c2", "baseline": 0.85, "measured": 0.79, "delta": 0.06}
  ]
}
```

**Evaluating is as strict as recording, and `compared` is why a partial result is
readable.** A measured score must be numeric and in `[0, 1]`, on the same terms
the baseline is held to — a string used to drop its case from the comparison
silently, and a `5.0` used to clear any threshold.

**A partial evaluation is legal and a vacuous one is not.** Scores that name
fewer cases than the baseline still succeed: the caller owns the coverage, and
`compared` and `unmeasured` say what the verdict rests on. Scores that name
**none** of the baseline's cases are refused with `400`, because a gate cannot
pass on nothing — and before `SIGNOFF-REPAIR.8.2.1` that case returned
`passed: true` and appended it to the gate's durable results.

```bash
# refused: compares no case
curl -s -X POST ".../evaluations" -d '{}'
# refused: the measurement is not a number / is outside [0, 1]
curl -s -X POST ".../evaluations" -d '{"c1": "oops"}'
curl -s -X POST ".../evaluations" -d '{"c1": 5.0}'
```

## What this harness does not establish

⛔ **The Phase 5 gate (G5) was met as a *subtraction* gate, and the quality-lift
claim was withdrawn.** Nothing in this chapter demonstrates that deliberation
improves an answer. The harness records measurements; it does not establish that
the thing being measured got better.

✅ **Repaired: a gate that measured nothing used to report `pass`.**
`evaluate_gate` skipped any baseline case the submitted scores did not mention —
"the caller owns the coverage" — so an empty score object compared no case,
found no failure, returned `passed: true` and **appended that verdict** to the
gate's results. A non-numeric measurement dropped its case just as silently, and
a measurement outside `[0, 1]` was compared as written, so `5.0` cleared every
threshold — while the **write** side of the same file refused all three in a
baseline. `SIGNOFF-REPAIR.8.2.1` made the read side obey the write side's rules
and published `compared` / `unmeasured`. ⚠️ **Partial coverage is still the
caller's to own**, which is a contract and not a defect: read `compared` before
treating any green gate as evidence.

✅ **Repaired: a storage failure used to be reported as the caller's mistake.**
Every write path turned any database error into "already exists — register a new
version or run id instead of overwriting", and every existence check turned one
into "the corpus is not registered" — each as an HTTP **400**, so an outage was
indistinguishable from a bad request. `SIGNOFF-REPAIR.8.2.2` converted **12**
sites: an existence check's error arm is always a store fault, because the
BOOLEAN beside it carries the genuine refusal, and a failed write is the
caller's duplicate only when the database reports a unique violation. A store
fault is now **500 `dependency_unavailable`** with the cause logged server-side,
and every genuine refusal — a real duplicate, an unregistered corpus, a ghost
run or gate — stays a 400.

⛔ **A recorded result is a claim by whoever posted it.** The server checks
shapes, references and vocabularies. It does not run the workflow, does not
verify that the scores came from the corpus they name, and — per the admission
note above — does not check who the caller is beyond enrolment.
