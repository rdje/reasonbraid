# 2026-09-07_r2-extraction-contract.md

## Context

`PHASE-4.4` (pack R2 — PDFs and selected open formats in sandboxed extraction
workers, backlog 34) decomposed at the census seams: NOTHING extracts — no
PDF/zip/tar/feed crate in the lock, and the codebase's `extract` hits are
axum's extractor and the CA's EC helper. The §12.3 R2 row needs a typed
contract before the workers (`.4.2`) exist — the same order the `.2`/`.3`
lanes used.

## Decision

- **The format set (initial).** PDF (the TEXT layer only — no rendering, no
  fonts, no images: R4's media lane owns those); the archives (zip, tar —
  one level deep); the structured feeds (Atom, RSS). Plain UTF-8 text is NOT
  an extraction — it IS the content the R0 acquisition already returns.
- **The extraction is ALWAYS a Derivation, never the original.** The output
  is the §12.6 Derivation edge: the parent digest (the ADR-011 digest of the
  acquired bytes — the R0/R1 receipt's digest), the extractor version, and
  the derived chunks — each chunk carries its own ADR-011 `sha256:<hex>`
  digest (the §12.6 "derived text/chunk digests").
- **The sandbox claim is `process` — the honest ladder-up.** R0/R1 claim
  `none` (bytes only, nothing executes); R2's parsers EXECUTE untrusted
  content, so the boundary is a DEDICATED WORKER PROCESS per extraction: a
  fresh process, a stdio JSON protocol, per-extraction byte/time budgets
  whose trip KILLS the worker — the quarantine is the process boundary, and
  nothing persists between extractions. The pattern mirrors the adapter
  lane's supervised subprocess.
- **The worker protocol.** JSON on stdio: request `{ input (base64 bytes or
  a temp path), media_type, limits }` → response `{ parent_digest, chunks:
  [{digest, text}], extractor_version }` or `{ error: {kind, message} }` —
  every refusal names its kind.
- **The refusal vocabulary (all named, never skipped):** encrypted PDFs,
  JS-bearing PDFs (the `/JS`/`/JavaScript` names), nested archives (an
  archive inside an archive — one level, then refused), path traversal (the
  `..`/absolute entry names), the archive bomb (the decompression-ratio +
  byte ceilings — the R0 brake again). No rendering, no fonts, no images.
- **The media-type routing.** The R2 pack advertises the ACQUISITION packs'
  schemes (https, git) with its media types (application/pdf,
  application/zip, application/x-tar, application/atom+xml,
  application/rss+xml). A reference WITH a matching `media_type_hint` ranks
  the R2 pack (the pipeline: acquire via the R0/R1 pack per the scheme,
  THEN extract); a reference without a hint stays the acquisition-only
  path. The `.4.3` wiring adds the media-type filter to the resolve order.
- **The parser census (measured `2026-09-07`).** `cargo add --dry-run`:
  lopdf 0.44.0 (the pure-Rust PDF text extractor), pdf 0.10.0 (rejected —
  the lower-level API, more surface), zip 8.6.0 + tar 0.4.46 (flate2 already
  in-tree), atom_syndication 0.12.10 (+ quick-xml 0.42.0). All pure Rust —
  the lean supply-chain doctrine holds; no C additions.

## Consequences

- The `.4.2` workers and the `.4.3` receipt implement this contract
  verbatim; a deviation is a contract change, not an implementation detail.
- The R2 registry entry (`.4.3`) claims sandbox `process` — the first pack
  above the ladder's bottom; callers requiring `constrained_process` or
  above get the explicit unresolvable-now, never a silent downgrade.

answers:

- **Content execution changes the honest sandbox class.** The packs that
  only MOVE bytes claim `none`; the pack that PARSES bytes claims `process`
  — the worker process IS the security boundary, not a preference.
- **Extraction is a Derivation, always.** The original bytes keep their
  digest and their acquisition receipt; the derived text is a NEW object
  with its own digests and its parent link — the distinction the §12.6
  graph enforces.
- **The refusal vocabulary is decided before the parsers.** Encrypted/JS
  PDFs, nested archives, traversal, and bombs are named up front — the
  workers enforce them mechanically instead of discovering them mid-parse.
