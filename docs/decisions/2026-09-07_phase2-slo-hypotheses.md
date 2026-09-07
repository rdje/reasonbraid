# Phase 2.5's initial SLO hypotheses: the guard IS the dev profile's population (`PHASE-2.5.3`)

- Date: 2026-09-07 · Leaf: `PHASE-2.5.3` · Decision record

## Context

§18.4 requires each deployment profile to instantiate service objectives
from §5, with initial targets as HYPOTHESES established by load/recovery
experiments — and warns that numeric objectives stay provisional until
baseline testing records workload and measurement boundaries. The dev
profile's measured, repeatable signal is the guard: 15 live server suites
(+ CLI e2e) over a fresh PostgreSQL database, the two-host demo with real
kill points (34/34 acceptance checks), and the `.4.1` restore exercise —
every pass is a fresh baseline run. This record instantiates the SLO
HYPOTHESES from THOSE measurements only, and names the rest (the SLO
catalogue's latency families have no workload measurement yet — the
subtraction doctrine: no invented numbers).

## Decision

The dev-profile SLO hypotheses (the §18.4 shape — population, exclusions,
window, statistic, target, error budget, owner, consequence):

| # | Objective (§5 attribute) | Population | Exclusions | Window | Statistic | Target | Error budget | Owner | Consequence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SLO-1 | Guard acceptance availability (availability + durable acceptance + audit + projection, jointly) | every live assertion of the guard's 15 suites + CLI e2e | offline suites, release-build runs, env-gated live adapter runs | each guard pass (one fresh-database run) | fraction of assertions green per pass | 100 % | 0 — one red assertion halts the frontier (the CI policy; §16 full CI before push) | director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`) | the red pass becomes a defect leaf before any new work |
| SLO-2 | End-to-end behavior (the demo's acceptance points: delivery, fencing, kill recovery, budget denial, audit reconstruction) | the demo's 34 acceptance checks | none — the checks ARE the scenario | each guard pass | checks green per pass | 34/34 | 0 — same halt rule | director | same |
| SLO-3 | Recovery success (the §5 recovery attribute's dev-profile instantiation) | restore exercises (`.4.1`: seed → real pg_dump → mutate → restore into an isolated database → assert the pre-mutation state) | none | each guard pass | success fraction | 100 % | 0 — same halt rule | director | same |
| SLO-4 | Node reconcile-after-kill success (notification/reconnect — the §5 notification attribute) | the demo's kill beats (SIGKILL: the dispatch-bound node dies, the attempt recovers `outcome_unknown` — bounded, visible, never silently retried) + the node_channel replay tests | none | each guard pass | honest-terminal fraction (no silent retry, no lost attempt) | 100 % | 0 — same halt rule | director | same |
| SLO-5 | Certificate-issuance latency (a transport-latency BASELINE, not a target) | issuance N=200 (the `.1.1` spike, `target/spike81.log`) | control-plane ingress→commit latency (unmeasured) | the recorded spike run | empirical p50/p95 | p50 63 µs / p95 69 µs (hypothesis to re-measure at the next cert-path change) | none — a baseline records, it does not gate | director | a re-measure rides the next cert-lane leaf |

## The SLO catalogue stays separated (the §5 last line)

Control-plane latency, notification latency, model/provider latency,
human-wait latency, publication latency, deployment convergence are DIFFERENT
numbers — a single "response time" SLO would be misleading. The dev profile
instantiates behavior/recovery objectives (SLO-1…SLO-4) and ONE latency
baseline (SLO-5); the rest of the catalogue is named, not instantiated:

- **Control-plane ingress→commit latency** — no workload measurement exists
  yet; the §5 rule (provisional until baseline testing) defers the number.
  Trigger: the first load experiment (§19.6 / Phase 7 territory) records it.
- **Model/provider + human-wait latency** — deliberately never one end-to-end
  SLO: a thread intentionally waits for human review or external evidence
  (§18.4). Not instantiated; the benchmark's aggregates (Phase 0 `.7`) are
  evaluation numbers, not service objectives.
- **Notification latency** — SLO-4 measures SUCCESS (reconcile), not
  promptness; the promptness number needs the presence-state timing
  measurement. Trigger: the first §18.4 notification experiment.
- **Publication + deployment convergence** — no publication surface (Phase 6)
  and no managed deployment yet. Trigger: the first published policy
  repository / the first converged deployment.

## Revisit

The next re-measure rides the leaf that changes the corresponding path
(SLO-1…SLO-4 ride every guard pass by construction; SLO-5 rides the next
cert-lane change), and the first load experiment (Phase 7) converts the
latency hypothesis into a boundary-recorded SLO.

## answers:

- **The guard IS the dev profile's population** — the hypotheses are
  instantiated from measurements that exist and repeat on every pass (15
  suites, 34 demo checks, the restore exercise), never from a hypothetical
  workload.
- **An unmeasured latency is named with its trigger, not given a number** —
  the §5 provisional rule + the subtraction doctrine agree; SLO-5 is the one
  measured baseline (the issuance spike), and the control-plane latency
  hypothesis waits for the first load experiment.
- **The error budget is zero in the dev profile** — the consequence is
  "halts the frontier", which is already the CI policy's shape; a production
  profile will carry a nonzero budget and a burn-rate story when it exists.
