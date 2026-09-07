# ADR-023 — Telemetry storage and redaction: the four records are separate systems, the sink is a named trigger

- **Status:** `accepted` (evidence-gated — the four-record separation this
  record pins is the SHIPPED design: the operational `eprintln!` stream and
  the durable audit/event tables are separate systems by construction)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-2.5.1`
- **Requirements:** `ROADMAP.md` §23 queue item 023; §18.1 (four distinct
  records), §18.2 (OpenTelemetry and correlation)

## Context

The roadmap queued "telemetry storage/redaction" as ADR-023. The census
found the dev profile's observability unstructured (16 `eprintln!` sites, no
metrics, no traces) — but the four-record doctrine (§18.1) is structurally
TRUE: the operational logs and the durable audit/domain tables are separate
systems, so no log line can become an audit record and no audit row is a
log. The open question is what the ADR should PIN before any telemetry
machinery lands: the separation, the redaction rules, and the sink trigger.

## Options

1. **Adopt the shipped separation as the ADR** — the four records stay
   separate by design; the §18.2 redaction rules pin any future sink; the
   OpenTelemetry dependency waits for the named trigger.
2. Adopt an OpenTelemetry stack now (a collector, an exporter, a metrics
   registry dependency) — the dev profile has no telemetry sink, no
   deployment, and no measured need; the dependency would be the
   placeholder-infrastructure lie the subtraction doctrine forbids.

## Evidence

- **The separation is shipped**: the operational surface is `eprintln!`
  (grep: api 8 sites, node_channel 2, worker 6); the durable record surface
  is the event/audit/authorization tables every domain write commits — two
  systems, no shared path.
- **The §18.1 rules hold by construction**: an operational log is not an
  audit record (the audit row is the transaction's committed fact); a trace
  sampler can never decide whether a governance action remains provable (the
  authorization record is).
- **No sink exists to redact for**: nothing ships logs/traces/metrics off
  the hosts today, so the redaction rules are the contract for the FUTURE
  sink — pinned now, enforced when it lands.

## Choice

Option 1. The four records remain separate systems; the §18.2 redaction
rules pin the future telemetry sink (never prompt text, credentials,
secret-bearing URLs, private evidence, or model output in span attributes;
sensitive IDs tokenized at the sink boundary; high-cardinality labels stay
in logs/traces, not unbounded metric dimensions); the OpenTelemetry
dependency waits for the trigger.

## Consequences

- Subtraction: no tracing/metrics dependencies, no collector, no exporter
  in the dev profile.
- The `.5.2` metrics slice (the admin metrics surface over the §18.3
  minimums) is an in-process registry — not a sink — so it rides this ADR
  without an OpenTelemetry dependency.

## Rollback / revisit trigger

- A deployment with a telemetry sink (any non-loopback profile or the G7
  operations gate) — adopt the OpenTelemetry-compatible stack under THESE
  redaction rules, with the tokenization boundary implemented before the
  first export.
