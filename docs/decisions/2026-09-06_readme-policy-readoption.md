# README_POLICY re-adoption: derived caps, routing-pressure closure, and a guard that proves itself

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** director directive §14 (adopt `fsmgen/README_POLICY.md`) + the director's word on `PHASE-0-MAINT-1` timing (2026-09-06 session decision)
- answers: what is the README growth guard's enforcement contract after the re-adoption, how were the derived caps computed, and how does every routed destination end at a governed terminal?

## The fact / decision

`README_POLICY.md` is re-adopted at the upstream 2026 revision (fenced ReasonBraid adoption note + the
neutral body: Authority and provenance · duplication probe · Routing pressure closure · derived caps ·
unconditional check · the 9-step checklist). `scripts/check_readme_stability.sh` now enforces:

- **Derived caps, not template values:** `README.md` measured 47 lines / 1,772 bytes at review; the
  inclusive ceilings are **60 lines / 2,400 bytes** (the survivor + explicit bounded headroom). A raise
  needs a reviewed decision in the task tree — never "raise to land content".
- **Routing-pressure closure:** the data-only registry `.doctrine/readme_routes.txt` (17 rows) maps every
  destination named by the README, the policy, and the guard's own hint to `path|class|control|owner`.
  The guard fails when a linked or emitted destination has no governed row (prefix closure), when a row
  is malformed, or when the registry is missing. A row's path governs that path and everything under it.
- **Append-only-history threshold:** `CHANGELOG.md` carries a 96,000-byte rotation threshold (rotation =
  git history). The measured 48,495-byte baseline is recorded as governed debt, not treated as an ideal.
- **A self-test arm** (`--self-test`) proves the extraction filter, the closure verdict, and the cap
  verdict against ground truth before the guard may judge the tree — same discipline as TABLE-ARITY.
- **Unconditional execution:** the guard runs on every commit and CI build regardless of changed paths
  (the registry rows and the CHANGELOG threshold are properties of the resulting tree).

## Why

The §14 upstream re-check (2026-09-07 session start) found the fsmgen source revised with seven changes
that close measured defect classes: a fenced local-adoption note, "Authority and provenance", "Routing
pressure closure" (with the 1,547,057-byte neighboring-sink cautionary tale), derived caps instead of
example values, the unconditional-check rule, a duplication probe, and a 9-step checklist. The repo-local
copy was the older revision and the guard still shipped template defaults (300 lines / 16,384 bytes) —
meaningless ceilings for a 47-line landing page — with no routing-closure inventory.

The closure leg paid for itself on its FIRST run against this repo: it caught three genuinely unrouted
destinations (`COMMIT.md`, `docs/adr/001-uncleared-working-name.md`, and the scaffold-URL placeholder
inside the scaffold span) and a real measured legacy ceiling (`CHANGELOG.md` at 48,495 bytes vs the
provisional 10,240) — the exact defect class the upstream policy documents. Each was either given a
governed row or reworded, and the falsification arms (cap override, injected unrouted link, malformed
registry row) each turn the guard red and restore the tree byte-identical afterward.

## How to apply

- A README cap, the CHANGELOG rotation threshold, or any registry threshold may increase only through an
  explicit reviewed decision recorded in the owning task-tree leaf.
- New destinations in the README, in the guard's routing hint, or inside a declared control require a
  registry row (`path|class|control|owner`) in the SAME commit, or the guard fails the commit.
- Upstream `README_POLICY` revisions are adopted only through deliberate local review (`PHASE-0-MAINT-*`
  leaves); there is no automatic synchronization — the origin is not an upstream.
- The env overrides (`README_LINE_CAP`, `README_BYTE_CAP`, `README_CHANGELOG_BYTE_CAP`) exist for
  falsification runs only, never as a growth path.

## Consequences

- The guard now REFUSES (exit 2) when `README.md`, `README_POLICY.md`, the registry, or `CHANGELOG.md`
  is missing — an absence can no longer read as a pass.
- `make gate` stays the single enforcement point (the README-STABILITY check inside the 13-check driver);
  CI runs the same driver.
- Renaming or retargeting a routed destination without updating the registry now fails loudly — the
  "silently retarget" hole from the upstream cautionary tale is closed.
