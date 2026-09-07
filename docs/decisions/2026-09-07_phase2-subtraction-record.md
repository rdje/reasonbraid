# Phase 2's subtraction record: what the delivery, identity, and recovery phase did NOT build (`PHASE-2.7.4`)

- Date: 2026-09-07 · Leaf: `PHASE-2.7.4` · Decision record · §19.8 mandatory

## Context

§19.8 requires every phase gate to examine what can be REMOVED or POSTPONED,
not only what could be added. Phase 2 delivered: the workload-identity
lifecycle (cert issuance/rotation/revocation + the delegation context + the
cached decisions + the incarnation/run writers), the delivery hardening
(ADR-005, the lease epoch, the retry policy, the two-way quarantine), the
spend breakers + the reconciliation surface, the backup/restore + the
measured upgrade path, the observability lane (ADR-023, the metrics, the
SLO record, the runbook), the adapter conformance kit, and the exit lane
(the non-escalation suite, the replacement drill, ADR-022, the retry-safety
inventory). This record names what it ACTIVELY DID NOT build — each item
with the profile whose arrival re-opens it.

## The subtraction list (each item = a named not-built, with its trigger)

| # | Not built in Phase 2 | Why not | The profile that re-opens it |
| --- | --- | --- | --- |
| S-1 | Internet exposure of any surface | the G6 gate is the named blocker; nothing ships exposed before it | the Internet qualification gate (§16.12 + external review) |
| S-2 | The SSRF / DNS-rebinding / redirect / archive-bomb suite | no arbitrary-reference surface exists (no object store, no fetching) | the first arbitrary-reference feature (Phase 4's resource store) |
| S-3 | The prompt-injection action-boundary suite | the adapters stream verbatim; no policy projection boundary exists | the first policy projection (Phase 6) |
| S-4 | The dependency/SBOM/provenance/release-signing pipeline | the dependency ledger + `cargo deny` exist; signing needs a signing key | the first signed release (Phase 9's G9) |
| S-5 | Rate-limit machinery | no provider rate-limit signal was ever observed | the first parseable provider rate-limit signal (the `.6.3` deferral) |
| S-6 | The audit hash chain + checkpoint + verification policy | the linkage is shipped; the chain answers a threat model the trusted LAN lacks | the first non-loopback deployment / the G7 ops gate (ADR-022) |
| S-7 | An OpenTelemetry sink | the four-record separation is shipped; no sink, no deployment, no measured need | the ADR-023 trigger (a non-loopback deployment / the G7 ops gate) |
| S-8 | Load tests and capacity numbers | no workload measurement exists; §5's provisional-objective rule | the first load experiment (Phase 7 territory; SLO-5's trigger) |
| S-9 | Chaos injection beyond the kill beats + the replacement drill | the drill measures the real failure classes; fault injection at scale is a game-day instrument | the production-beta game days (G7, Phase 7) |
| S-10 | HA (multiple servers, multiple nodes per role) | the dev profile is one server, one node per role (trusted LAN) | the deployment profile that requires it (G7) |
| S-11 | The object-store inventory / the canonical Git mirror / the signing-key recovery / the multi-store reconciliation / the backup encryption | nothing to bind (no object store, no mirror, no key) | each names its trigger in the `.4.3` deferral record |
| S-12 | Production RPO/RTO pairs | the restore exercise measures CORRECTNESS, not wall-clock targets | the production profile (SLO-3's record) |
| S-13 | The per-adapter rate-limit/backoff, output-size limits, tool-call validation, the provider error taxonomy | no machinery or no surface to bind (the `.6.3` deferrals) | each names its trigger in the `.6.3` record |

## answers:

- **The phase's subtraction is the G6–G7 gap list, not a wish list** — every
  item above names the exact gate or surface whose arrival re-opens it; none
  is an undisclosed missing control.
- **The dev profile's honesty boundary is explicit** — the trusted-LAN
  profile claims linkage + reconstruction (ADR-022), not tamper-evidence of
  the stream; recovery correctness, not production RPO/RTO; and no Internet
  exposure by construction.
- **Nothing on this list blocks Phase 3** — each item is either a later
  phase's own territory or a trigger the roadmap already queues.
