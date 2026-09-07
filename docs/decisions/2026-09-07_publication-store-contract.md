# 2026-09-07_publication-store-contract.md

## Context

`PHASE-6.4.3.1` (the Git-publication half's store contract, the ADR-020
implementation): the `.4.3` census found the WRITE half is a greenfield —
the Phase-4 R1 pack only ACQUIRES (the gix clone/fetch; no commit/ref-write
path exists) and the §15.8 reconciler exists nowhere. This record pins the
store's shape before the `.4.3.2` publisher writes a byte.

## Decision

- **The store is a LOCAL bare repository** (the §15.2 "Git is a reviewable
  representation" shape for the LAN slice). The server (or the publication
  worker) owns the repo path; the remote-publication profile is a named
  deferral (a remote write surface needs the ADR-020 signatures + the
  transport authorization the `.5`/`.7` gates decide).
- **The ref scheme (ADR-020's §15.7 steps 5–8):**
  - the STAGING branch: `refs/rb/staging/<publication_id>` — the publisher
    writes the bundle + the manifest there (the branch name carries the
    publication id + the digest rides the manifest);
  - the IMMUTABLE publication ref: `refs/rb/publications/<publication_id>`
    — written ONCE, never moved (the destructive rewrites are prohibited);
  - the EFFECTIVE channel: `refs/rb/effective` — the compare-and-swap ref:
    the update carries the EXPECTED old object id; a ref that moved
    underneath is the typed failure, never a force-push.
- **The write path's gix surface:** the commit-tree (the bundle + the
  manifest as the blobs), the reference update with the expected old id
  (the CAS — gix's `reference::edit` with the `PreviousValue`), and the
  fetch-back read (the re-derived content digest must match the manifest's).
  No shelling out to the git CLI (the pure-Rust doctrine — the R1 pack's
  rule).
- **The reconciliation's DB half is the `.4.2` records** (the
  staged/effective/failed states + the git_object_ids); the Git half is the
  refs above. The six §15.8 rules consume the pair.

## Consequences

- `.4.3.2` implements the publisher over this store verbatim; `.4.3.3` the
  reconciler over the same refs. A deviation is a contract change.
- The `.5` deployment receipts reference the immutable ref's object id —
  the drift detection compares the digests, never the prose.

## Indexed under

- `PHASE-6.4.3.1` (the owning leaf); the `.4.3.2`/`.4.3.3` leaves cite it.
