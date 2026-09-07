# ADR-014 — Directory semantic-index engine and embedding lifecycle: deterministic eligibility first, embeddings behind their trigger

- **Status:** `accepted` (evidence-gated — the two-stage matching §10.3 pins is
  the SHIPPED contract's shape: the eligibility stage is structural, the
  ranking stage names the semantic slot the embedding machinery would fill)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-3.1.1`
- **Requirements:** `ROADMAP.md` §23 queue item 014; §10.1 (the registration
  profile), §10.3 (two-stage matching — "an ineligible role is never restored
  by a high semantic score"; "learned ranking begins in shadow mode and never
  controls authorization")

## Context

The roadmap queued "directory semantic-index engine and embedding lifecycle"
as ADR-014. The `.1` census found the profile itself is a greenfield (no
table, no verbs, no visibility policy). The ADR must decide the engine
BEFORE the profile lands, so the profile's fields carry the shape the
matching stage will consume: does the dev profile build a learned embedding
index now, or pin the deterministic eligibility stage and let the semantic
machinery arrive behind a trigger?

## Options

1. **Deterministic structural eligibility first; the embedding engine behind
   its trigger** — the §10.3 stage-1 fields (scope, status, capability
   requirements, policy restrictions, separation rules, ceilings, budget
   availability) are typed profile data evaluated by deterministic rules;
   the stage-2 "semantic relevance, when enabled" slot stays EMPTY in the
   dev profile (structural features: exact capability/subscription match,
   affinity, latency class, workload balance, the dependence indicators).
2. Build an embedding index now (a vector store, embedding generation, and a
   semantic search path) — before any recruitment surface exists to search,
   and before the profile has data to embed: machinery with no consumer
   (the subtraction doctrine's placeholder-infrastructure lie).

## Evidence

- **The eligibility stage is structural by the roadmap's own table** —
  §10.3's stage-1 list has no semantic term in it; every field is a typed
  fact (enrollment/suspension status, ceilings, exclusions) the shipped
  authority machinery already evaluates deterministically.
- **"An ineligible role is never restored by a high semantic score"** pins
  the ordering: the semantic engine can only ever rank WITHIN the eligible
  set — so building it first would answer a question nobody can ask yet.
- **"Learned ranking begins in shadow mode"** pins the lifecycle: the
  embeddings arrive as a shadow instrument, never an authorization control.

## Decision

Accept option 1. The dev profile's directory is the typed profile + the
deterministic eligibility evaluation; the embedding lifecycle arrives
behind its trigger — the first recruitment run whose stage-2 ranking is
measured to need semantic relevance beyond the structural features (a
non-loopback network with enough profiles that the exact-match features
under-select). When it fires, the shadow-mode rule and the §10.1 provenance
rule (a high self-declared score is never equivalent to verified
competence) ride the engine's design.

## Consequences

- The `.1.2` profile schema carries NO embedding columns (no vector fields,
  no learned scores) — the versioned profiles stay interpretable typed
  facts.
- The `.3` two-stage matching lane builds stage 1 + the structural
  features of stage 2; the semantic slot is named, not filled.
- The dependence indicators (`.6`) are computed from typed facts (lineage,
  participation patterns), never from embedding geometry.

## Revisit trigger

The first recruitment run whose stage-2 ranking under-selects without the
semantic feature (measured on a non-loopback profile corpus) — the engine
lands as its own leaf in shadow mode, with the §10.1 provenance rules.
