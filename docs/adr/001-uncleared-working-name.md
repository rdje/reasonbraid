# ADR-001 — ReasonBraid is an uncleared working name

- **Status:** working name accepted; public repository authorized by the director; name clearance remains open
- **Date:** `2026-09-05`
- **Leaf:** `PHASE-0.0.1`
- **Requirements:** backlog 1; `ROADMAP.md` §2.7; `KICKOFF.md` WP0

## Context

The project needed a pronounceable working name after Concord, ADEL, AgentBraid,
and Deliberon were rejected (`ROADMAP.md` §2.7). A preliminary exact-name Web
screen found no relevant software/company result for “ReasonBraid.” That is
negative search evidence, not legal or namespace availability.

## Options

1. Keep using Concord (crowded name; promises agreement).
2. Use ADEL / AgentBraid / Deliberon (collisions; some are close products).
3. Use ReasonBraid as a working name and isolate branding; defer package, domain
   and marketing commitments until professional clearance.

## Evidence

- Naming table in `ROADMAP.md` §2.7 (document date 2026-09-04).
- Closest screened products: Deliberon; AgentBraid (MCP orchestration).
- No professional trademark, company, package, domain, or app-store search has
  been run for this repository.

## Choice

Option 3. `ReasonBraid` is the **working name** for architecture prose and
prototypes. It does not promise unanimous agreement.

The director clarified on 2026-09-09 that the private-repository instruction was
wrong: this project is public and must remain public. Repository visibility is
authorized independently of name clearance. This corrects the repository clause
and its revisit trigger; see docs/decisions/2026-09-09_public-repository-policy.md.

## Consequences

- Keep the repository public, as explicitly directed. Do not make it private
  because name clearance remains open.
- Do not assume `reasonbraid`, `reasonbraid-server`, crate names, domains,
  or social handles are obtainable.
- Keep wire type names product-neutral where practical.
- Isolate branding constants so another rename is mechanical.
- Retain “formerly Concord” only in document provenance and migration notes.
- Public crates.io/domain/handle reservation is **out of scope** until clearance.

## Rollback / revisit trigger

Before package, domain, or marketing release: professional
trademark, company/product, package, executable, domain, app-store, and
repository clearance in intended jurisdictions. The naming ADR shall then
record search evidence, counsel or owner decision, rename trigger, and
selected identifiers. A collision found at that gate is a rename, not a waiver.
