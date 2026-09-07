# ADR-020 — The canonical publication: a nine-step state machine over the staged record — the Git refs are the publication truth, the reconciler never silently promotes

- **Status:** `accepted` (evidence-gated — the §15.7–15.8 contract:
  the nine-step publication pipeline, the publication records, and
  the reconciliation matrix are the shapes the `.4.2`/`.4.3` leaves
  implement)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-6.4.1`
- **Requirements:** `ROADMAP.md` §23 queue item 020 (the Git
  publication refs, the signatures, and the reconciliation);
  §15.7 (the canonical publication protocol), §15.8 (the
  reconciliation matrix)

## Context

The `.4` census mapped §15.7–15.8 against the shipped surface.
The publication INPUTS ship: the decisions + the approvals
(`.2` — the authority proofs), the byte-identical projections +
the digests (`.3` — the verification primitives). The SUBSTRATE
ships: the Git machinery (Phase 4's gix pack + the snapshots),
the CA/signature keys (the Phase-2 certificate infra), the
transactional outbox (the §15.7 step-4's one-transaction
staged + outbox). The GREENFIELD: no publication record, no
staged state, no ref protocol, no reconciliation — the §15.8
matrix's rows exist nowhere.

## Decision

- **The publication is a nine-step state machine over the
  staged record** (§15.7, verbatim): (1) lock the publication
  aggregate and verify the decision, the approvals, the
  authority proof, and the immutable inputs; (2) compile the
  canonical bundle + the publication manifest in a clean
  worker (the `.3` crate — already hermetic); (3) hash and,
  for the required profiles, sign the manifest; (4) in ONE
  transaction store the `publication_staged` row, the
  manifest digest, the desired Git operation, and the outbox
  item; (5) the publisher writes the content to a staging
  branch/ref carrying the publication id + the digest; (6)
  verify the fetched-back content, the signatures, the tests,
  and the parent/reference preconditions; (7) update the
  dedicated IMMUTABLE publication ref/tag and then the
  effective channel ref via the COMPARE-AND-SWAP; (8) record
  the Git object ids and mark the publication effective; (9)
  emit the deployment offers separately. Each step is a typed
  state; a step that cannot complete is the typed failure,
  never a skip.
- **The publication record is the machine's aggregate.** The
  `policy_publications` row carries the decision + the
  approval + the projection references, the manifest digest,
  the state (staged/effective/failed), and the Git object
  ids. The record never folds into the approval (the
  ADR-032 separate-record rule extends to the publication).
- **The Git refs are the publication truth; PostgreSQL
  coordinates.** The immutable publication ref and the
  effective channel ref are the durable truth (§15.2); the
  database row is the workflow's coordinator, not the
  content's store. The compare-and-swap is the idempotency
  primitive — a ref that moved underneath is the typed
  failure, never a force-push.
- **The reconciliation matrix is six idempotent rules**
  (§15.8, verbatim): the staged/absent pair retries the safe
  staged write; the staged/matching pair verifies and
  advances; the staged/conflicting pair STOPS with a security
  alert + the human resolution; the effective/missing-or-moved
  pair freezes the deployment and restores only through the
  authorized repair; the failed/later-appearing pair
  quarantines + adjudicates (never a silent promote); the
  no-record/out-of-band-object pair verifies the signature and
  alerts. The reconciler is idempotent and regularly exercised
  under kill points; the destructive Git rewrites are
  prohibited for the published history.
- **The signatures are the manifest's, never the prose's.** A
  signed publication signs the MANIFEST digest (the bundle +
  the lock + the authority basis), not a rendered file — the
  verification is a digest comparison, not a document read.

## Consequences

- `.4.2` implements the publication records + the staging
  (the aggregate, the manifest, the stage machine) and `.4.3`
  the Git publication + the reconciliation (the ref protocol,
  the matrix, the kill-point tests) — each against this
  contract verbatim; a deviation is a contract change.
- The `.5` deployment lane consumes the effective
  publications: the deployment offers are step 9's records,
  the per-target receipts reference the effective ref's
  object ids.
- The outbox's existing delivery guarantees carry the staged
  write: the Git operation is idempotent by the digest
  (a redelivered write is the same write).

answers:

- **The staged record is the machine's honesty.** The
  nine-step pipeline exists so that no step can pretend a
  later one happened: the staged row, the verified fetch-back,
  the immutable ref, and the effective record are each typed —
  the same separate-record doctrine as the lifecycle (`.2`).
- **Compare-and-swap is the publication's idempotency.**
  Force-writes would make the reconciliation unanswerable;
  the CAS turns the ref race into a typed failure the
  reconciler can act on deterministically.
- **The reconciler never promotes silently.** The §15.8
  matrix's failed/later-appearing row is the trap: a write
  that appears after a failure must be quarantined and
  adjudicated — promoting it would be the same dishonesty as
  the thread's silent retry (Phase 2's bounded-retry rule,
  applied to the publications).
