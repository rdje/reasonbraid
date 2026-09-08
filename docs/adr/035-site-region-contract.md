# ADR-035 — The site/region contract: the declared regions, the store-and-forward over the shipped substrate, and the export is the exit path

- **Status:** `accepted` (evidence-gated — the §20.10 regional-routing
  vocabulary: the declared site/region model, the store-and-forward
  contract, the export/import ladder, the exit-path shape)
- **Date:** `2026-09-08`
- **Leaf:** `PHASE-8.5.1`
- **Requirements:** `ROADMAP.md` §20.10 (the Phase-8 regional routing)

## Context

The `.5` census measured the seams: the node-level store-and-forward
substrate ships (the outbox worker + the durable inbox + the lease/
presence + the replay — the demo's crash/reconnect scenario); the
export seeds ship (the backup/restore + the portable cards' ladder);
the region machinery is the ADR-034 named deferral (the regions are
DECLARED, no machinery). This record fixes the vocabulary the
`.5.2`–`.5.4` leaves implement.

## Decision

- **The regions are DECLARED, never inferred.** A site declares its
  region in the deployment profile (the ADR-034 stance applied to the
  region vocabulary: the declaration is the only seam). The dev
  profile declares the single region (`dev-local`); an undeclared
  region is the typed refusal at the routing boundary. A site is the
  deployment instance; the region is the named scope its deliveries
  and its data ride.
- **The store-and-forward rides the SHIPPED substrate.** The
  intermittent-site delivery is the outbox + the inbox with the
  site-level pairing: the buffered rows persist until the peer site's
  delivery window (the leased worker's existing pattern — a delivery
  is never discarded for a disconnected site), the reconnect flushes
  in order, and the delivery NEVER invents a completion — the
  possible-gap surfaces when the peer offers no replay (the MCP
  listen-stream's discipline, applied to the site pair).
- **The export is the exit path's machinery.** The tenant/site data
  export is the DIGEST-PINNED BUNDLE (the ADR-011 `sha256:<hex>`
  scheme + the ADR-027 signature) over the backup/restore's dump
  shape; the import is the ORDERED ladder — the digest → the schema
  version → the agreement-allowlist → the local-grant capability (the
  portable cards' four-rung pattern applied to the tenant data). The
  documented exit path is the runbook over this ladder: the export →
  the verification → the local continuation — the §25.1 kill/pivot's
  leaving story, never a lock-in.
- **The exit path is DOCUMENTED, not invented at exit time.** The
  runbook ships with the export; the subtraction discipline stays —
  the unsupported shapes are NAMED, never silently dropped at the
  export boundary.

## answers:

- **The region is a declaration, not a database location**: the
  routing refuses an undeclared region — the declaration is the
  fail-closed seam.
- **The export bundle is the re-derivable unit**: the digest + the
  signature over the canonical bundle — the import verifies BEFORE any
  row lands (the ladder's order is the security property).
- **The exit path is the export path**: leaving ReasonBraid is the
  export followed by the verification — a documented runbook, never an
  emergency invention.
