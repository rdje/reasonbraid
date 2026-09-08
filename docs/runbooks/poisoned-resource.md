# Runbook: poisoned resource

- Owner: the director (the accountable owner, `docs/decisions/2026-09-06_accountable-owners.md`)
- Leaf: `PHASE-7.4.2` · Date: 2026-09-08 · Profile: dev (trusted LAN)
- Scope: a submitted or acquired resource is malicious or corrupt — the
  extraction, the acquisition, or the derived content behaves badly.

## Detection

- **The extraction refusal:** the worker's named refusals (the encrypted/JS
  PDF, the nested archives, the traversal, the ratio brake) — the
  `process`-class worker quarantine kills the job on the budget trip.
- **The acquisition refusal:** the SSRF classification refuses the
  private/loopback/metadata destination (the 18-case matrix).
- **The suspicious snapshot:** a derived chunk that fails the citation
  validation (the excerpt absent from the bytes) or the drift check.

## Authority

- Submitting/acquiring: the granted principals (the resolver verbs).
- Quarantining + the retention: `tenant_admin`.
- Reading: the inspection surfaces (read-only).

## Safe first actions

1. **Do not open the resource again.** The refusals are the quarantine —
   a refused acquisition/extraction never lands (the typed refusal, not a
   partial trust).
2. **Freeze the picture:** the submission record, the digest, the refusal
   reason, the source.

## Diagnostic queries

- `GET /v1/resources/{id}` — the reference + the digest + the isolation class.
- The derivation graph (`rb inspect …`/the claims surface) — what derived
  from the resource.
- The snapshot store's quarantine status (the `.6` pipeline rows).

## Containment

- The worker quarantine (the fresh process per extraction — the poison never
  escapes its process).
- The budget bound: a poisonous extraction trips the time budget and dies.
- The snapshot quarantine_status row marks the suspect derived content.

## Recovery

- Remove the poisoned reference's DERIVED chunks from the active set (the
  tombstone + the retention path — the `.6.1` store).
- Re-derive from a clean source when the evidence demands it (the derivation
  graph names every dependent).
- The submission stays (the evidence of the attempt — the retention, not the
  deletion).

## Evidence preservation

- The resource row + the digest (the poison's identity).
- The refusal rows + the worker's termination evidence (the kill-on-budget).
- The derivation edges (what it touched).

## Communication

- The operator reports the poison (the digest, the refusal, the derived
  set) to the accountable owner.

## Closure tests

- The extraction suite (the nine refusals) + the SSRF suite (the 18-case
  matrix) + the citation-validation legs on every guard pass.
- The hostile-resource scenario (the profiles suite's refusal legs) on every
  guard pass.
