---
answers:
  - Which of the OpenTelemetry sink, operator dashboards, SLO baselines and game days are owed before G9?
  - Why does one deferral row carrying four nouns hide three of them?
  - Is the OpenTelemetry sink owed now, and on what condition a later pass can evaluate?
  - Which of ROADMAP §18.5's nine operator surfaces are actually shipped?
  - Why are SLO baselines and game days discharged rather than built again?
  - What does an OpenTelemetry sink cost against what it buys at one process on loopback?
---
# The observability row is four commitments, two were already discharged, and the sink is deferred on a readable fact

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.4.6`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §18.1 (four distinct records), §18.2 (OpenTelemetry and correlation),
  §18.3 (minimum metrics), §18.4 (service objectives), §18.5 (operator surfaces),
  §18.6 (runbooks and exercises), §6.6 (deployment profiles);
  `docs/decisions/2026-09-07_phase2-slo-hypotheses.md`;
  `docs/decisions/2026-09-08_game-days-pentest.md`;
  `docs/decisions/2026-09-22_the-remaining-roadmap-gaps-are-sequenced-by-exposure-then-deletion.md`
  (this is that record's item 3)

## Context

`SIGNOFF-REPAIR.4.6` was opened by tranche 2's `fired and open` verdict on the Phase-1
deferral ***OpenTelemetry, operator dashboards, SLO baselines, game days*** — one row, four
nouns, carried as a unit since `docs/decisions/2026-09-07_phase1-subtraction-record.md`
routed it to Phase 2.

⛔ **The leaf's own acceptance named why that is the defect**: *a single verdict over four
nouns is what hid this for a phase*. Each of the four is adjudicated separately below, with
the command that produces its verdict.

## Decision — four verdicts, not one

| # | commitment | verdict | the command that produces it |
| --- | --- | --- | --- |
| 1 | OpenTelemetry sink (§18.2) | ⏸️ **DEFERRED**, on the trigger below | `grep -riE "opentelemetry\|prometheus\|otlp\|statsd" crates/*/Cargo.toml` → **none**; `git grep -cniE "span!\|tracing::span\|#\[instrument\]" -- crates/*/src` → **none** |
| 2 | operator dashboards (§18.5) | ⚠️ **PARTIAL — KEPT and scoped**: 4 of 9 covered, 3 partial, 2 absent | the nine-bullet census below |
| 3 | SLO baselines (§18.4) | ✅ **DISCHARGED** | `docs/decisions/2026-09-07_phase2-slo-hypotheses.md` carries §18.4's **exact nine fields** — population, exclusions, window, statistic, target, error budget, owner, consequence — over SLO-1…SLO-5 |
| 4 | game days (§18.6) | ✅ **DISCHARGED** | `docs/decisions/2026-09-08_game-days-pentest.md` maps **eight shipped exercises** to runbook closure tests; `ls docs/runbooks/` → **13 runbooks**, one per §18.6 family |

⭐ **Two of the four delete outright, and neither needed building** — they were discharged by
records written on 2026-09-07 and 2026-09-08, *before* the audit that graded the row open.
The row hid them because a verdict over four nouns can only be as good as its worst noun.

### Verdict 3 in full — why SLO baselines are discharged, not merely "documented"

§18.4's requirement is a RECORD with a stated shape: *the SLO record defines population,
exclusions, window, statistic, target, error budget, owner, and consequence*. The
2026-09-07 record is a table with those nine columns, and its content is a decision rather
than a placeholder: SLO-1…SLO-4 target 100 % with a **zero error budget**, whose consequence
is that a red pass halts the frontier; SLO-5 is the one measured latency baseline (issuance
p50 63 µs / p95 69 µs). ⭐ **Every unmeasured latency family is named with its trigger rather
than given an invented number** — which is the §18.4 sentence *initial targets are hypotheses
established by load/recovery experiments*, honoured rather than quoted.

### Verdict 4 in full — why game days are discharged

§18.6's requirement is *game days exercise [the runbooks]; findings become backlog items with
owners*. The 2026-09-08 catalogue names eight exercises that run on every pass — the
replacement drill, the restore exercise, the demo's kill points, the migration upgrade, the
hostile suite, the non-escalation suite, the load harness, and the adapter conformance
fixtures — each mapped to a runbook's closure tests. ⛔ Its three named gaps (multi-node
churn, the human tabletop, the load-driven game) each already carry an evaluable trigger, so
they are not re-opened here.

### Verdict 2 in full — the §18.5 census, bullet by bullet

§18.5 says *the admin UI/CLI must expose* nine things. Route surface read from
`git grep -oE '"/v1/[^"]*"' -- crates/reasonbraid-server/src/api.rs`.

| § | operator surface | verdict | evidence |
| --- | --- | --- | --- |
| 1 | service and dependency health with freshness | 🔴 **absent** | `git grep -cniE "health\|readyz\|livez" -- …/api.rs` → **no match**; the router has no health route at all |
| 2 | node leases, versions, capabilities, last reconciliation, quarantine | ✅ covered | `/v1/admin/nodes/presence`, `/v1/admin/incarnations`, `/v1/nodes/quarantine`, `/v1/directory/presence` |
| 3 | thread lifecycle, stop reason, budget, unresolved blockers, pending humans | ✅ covered | `/v1/threads/{id}`, `/{id}/budget`, `/{id}/events`, `/{id}/commands` |
| 4 | ambiguous attempts and safe resolution actions | ⚠️ **partial** | the RESOLUTION ships — `crates/reasonbraid-server/src/node_channel.rs:284` issues a `Directive` per ambiguous attempt at the handshake — but no operator surface LISTS them; `/v1/admin/runs` returns every run, not the ambiguous ones |
| 5 | outbox/inbox/dead-letter queues with authorized replay | ✅ covered | `/v1/nodes/inbox`, `/v1/nodes/replay`, `/v1/nodes/inbox/prune`; `dead_lettered` is a delivery state (`api.rs:1595`) |
| 6 | evidence acquisitions and resolver denials | ⚠️ **partial** | acquisitions are `/v1/snapshots` + `/v1/resources/{id}/resolve` + `/v1/resolvers`; the DENIALS are not listed anywhere |
| 7 | policy publication/deployment/drift state | ✅ covered | `/v1/policy-publications`, `/v1/deployments`, `/v1/policy-drift` |
| 8 | audit-chain verification and checkpoint age | ⚠️ **partial** | verification ships as `/v1/audit/receipts` + `/v1/threads/{id}/audit`; `git grep -cni "checkpoint" -- …/api.rs` → **no match**, so the AGE is absent |
| 9 | backup/restore status and active incidents | 🔴 **absent** | `git grep -cniE "backup\|restore_status" -- …/api.rs` → **no match**, although `tests/backup_restore.rs` exercises the procedure |

⭐ **The two absences share one shape and it is worth naming**: health and backup status are
the two bullets that describe the SYSTEM rather than a domain aggregate. Every covered bullet
already had a domain surface to hang off; the two with no aggregate behind them are the two
nobody built. ⚠️ And the three partials share another: in each, the MECHANISM ships and only
the operator's view of it is missing.

⇒ **§18.5 is KEPT, scoped to five bullets** (1, 4, 6, 8, 9), and owned by `.4.6.1`.

## The OpenTelemetry deferral, with a trigger a later pass can EVALUATE

⛔ `.11.4.7.2.1` measured **27 of 27** deferrals naming a phase that had already closed, and
this leaf exists because the module's own header restated its deferral where nothing evaluates
it (*the OpenTelemetry sink stays the ADR-023 trigger*, `telemetry.rs:3`). So the trigger is a
fact anyone can read on any day:

> Build the OpenTelemetry sink when **either** `rb-server` is started with a `--host` that is
> not a loopback address — the moment the deployment leaves §6.6's **Developer** profile
> (*Loopback/single host*) — **or** a second control-plane process is deployed.

**Why that condition and not a date.** §18.1 class 3 is *distributed traces: causal timing
across API, coordinator, node, adapter, resolver, and publisher*. At one process bound to
`127.0.0.1` (the shipped default, `rb-server.rs:24`) there is no second process for a trace to
cross, and the operator is the developer reading stderr — which is precisely what
`telemetry.rs` already emits, as structured JSON lines, beside an in-process registry served at
`GET /v1/admin/metrics`. The sink becomes load-bearing when the reader stops being the person
holding the terminal.

**The cost, stated rather than implied.**

| option | cost | what it buys at one loopback process |
| --- | --- | --- |
| keep JSON-line logs + the pull surface | 0 | the operator already sees everything the process knows |
| adopt an OTLP exporter now | a vendor dependency tree in a workspace that has deliberately taken none for this, a collector to run and operate, span-attribute redaction to get right *first* (§18.2 forbids prompt text, credentials, full URLs with secrets, private evidence and model output in attributes), and a second egress path to threat-model | a trace with exactly one process in it |

⚠️ **The redaction cost is not a makeweight.** §18.2's prohibition is a security requirement,
and adding an egress path before there is anything to correlate would ship that risk early in
exchange for nothing measurable.

## Consequences

- ✅ **The row's four nouns are now four verdicts with four commands**, so the next reader
  re-derives rather than trusts — and the deferral row itself can be closed.
- 🔴 **`.4.6.1` is opened** for §18.5's five unmet bullets. It is a BUILD and therefore wave C.
- ⚠️ **The sink's deferral is the leaf's ownership, not a park.** It names a readable
  condition, and the condition is checked by whoever changes the bind — which is the one
  moment it matters.
- ⚠️ **`telemetry.rs:3`'s prose trigger is superseded by this record.** The comment stays
  accurate but it is no longer the place the condition lives.
- ⭐ **Second leaf running that shrank under measurement** — `.9.3.5` went from *build a
  publication store* to three bounded gaps, and `.4.6` goes from four open commitments to one
  scoped build plus a deferral. Both were graded open by the same adjudication pass.

## What would make this wrong

- ⛔ If §18.5's *admin UI/CLI* is read as requiring a rendered DASHBOARD rather than an
  exposed surface, then the four covered bullets are covered as API and not as UI, and the
  verdict for those four is partial too. The reading taken here is §18.5's own verb — *expose*
  — and `crates/reasonbraid-server/web/` ships a UI over these routes.
- ⛔ If a profile above Developer is instantiated, the sink's trigger fires by design and must
  be honoured rather than re-argued.
- ⛔ If SLO-1…SLO-5's zero error budget is ever relaxed, verdict 3 rests on a record whose
  consequence column has changed, and it must be re-derived rather than cited.

## Alternatives considered

1. **Build the OTLP exporter now, since §18.2 says *use OpenTelemetry-compatible*.** Rejected:
   §18.2 asks for traces, metrics and structured logs that are OpenTelemetry-COMPATIBLE, and
   the sentence is about correlation and attribute hygiene, not about a vendor SDK. Shipping an
   exporter before the redaction rules are enforced would invert §18.2's own priority.
2. **Grade the whole row `discharged`, since three of four are met or partial.** Rejected: it
   is the defect this leaf was opened to repair, one noun later.
3. **Grade the whole row open and build all four.** Rejected on measurement: two are discharged
   by records that predate the audit, and rebuilding them would be the rework wave B exists to
   prevent.
4. **Defer §18.5's five bullets too.** Rejected: unlike the sink, they need no new dependency
   and no new egress path, and two of them (health, backup status) are what an operator reaches
   for first in an incident the 13 runbooks already assume can happen.
