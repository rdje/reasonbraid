# ADR-027 — Plugin/adapter signing and distribution: the release identity signs the digest-pinned manifest, and a downloaded adapter verifies against the allowlist ladder

- **Status:** `accepted` (evidence-gated — the §16.10 contract: the
  release-identity key hierarchy, the digest-pinned manifest + the
  signature scheme, the allowlist verification ladder; the
  distribution channel + the reproducible builders are the named
  deferrals)
- **Date:** `2026-09-08`
- **Leaf:** `PHASE-7.2.1`
- **Requirements:** `ROADMAP.md` §16.10 (the software supply chain)

## Context

The `.2` census mapped §16.10 against the shipped surface. The
SHIPPED halves: the dependency pinning + the lockfile review
(committed `Cargo.lock`), `cargo deny` + the vulnerability
advisories + the license policy + the secret scanning + the static
analysis (the `make deny`/`make secret-scan`/clippy gates), and the
adapter ledger (the locally installed CLI adapters qualified by the
live probes — the pinned wire interfaces + the qualification
evidence). The GREENFIELD: the SBOM generation, the signed
provenance, the binary/manifest signing, the downloaded-adapter
verification, the disclosure policy, and the public enrollment.
This record fixes the SIGNING-AND-DISTRIBUTION vocabulary; the
`.2.2`–`.2.4` leaves implement it.

## Decision

- **One Ed25519 release identity per release channel.** The signing
  key signs the release manifest; the dev stance places it with the
  releaser (the same honest placement as the workload CA's key —
  ADR-007's consequence). The protected release identities + the
  isolated reproducible builders are the NAMED deferrals (the
  ops/configuration upgrades, never a vocabulary change).
- **The release manifest is the single verification unit.** Per
  release: the per-binary digests (`sha256:<hex>` — the ADR-011
  scheme), the manifest's own digest, and the Ed25519 signature over
  the canonical manifest JSON (the ring provider — the workspace
  single-provider rule). Binaries verify THROUGH the manifest, never
  individually.
- **The downloaded-adapter verification is an ORDERED, FAIL-CLOSED
  ladder** (§16.10's "allowlist, digest, signature, API
  compatibility, and declared capability manifest"): (1) the
  allowlist membership — the ledger row; (2) the digest — the
  manifest's; (3) the signature — the release identity's; (4) the
  API compatibility — the pinned wire-interface version; (5) the
  capability manifest — the adapter's declared capabilities against
  the deployment's grants. An adapter that fails ANY rung is the
  typed refusal at that rung — never a partial trust. The shipped
  dev adapters satisfy the ladder BY CONSTRUCTION (the ledger rows +
  the pinned interfaces + the qualification evidence).

## answers:

- **The manifest is the trust anchor**: digest-pinned + signed once
  per release — the verification is re-derivable from the manifest
  alone (the same re-derive principle as the claim verification).
- **The ladder's order is the security property**: the allowlist
  first (the cheapest, the most stable), the capability last (the
  most deployment-specific) — a refusal names its rung.
- **The dev profile ships the vocabulary, not the channel**: nothing
  distributes in the dev profile (the `make release` artifacts are
  local); the distribution channel + the reproducible builders +
  the protected identities are the named deferrals with the
  public-beta trigger.
