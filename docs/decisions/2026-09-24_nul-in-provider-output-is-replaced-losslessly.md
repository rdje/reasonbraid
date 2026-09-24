---
answers:
  - Does a node deliver provider output verbatim?
  - What happens to provider output that holds U+0000 (the NUL character)?
  - How can the original provider output be recovered after a NUL was replaced?
  - Why is a NUL replaced rather than the attempt failed?
---
# NUL in provider output is replaced losslessly, and the positions reach every reader

- **Type:** decision
- **Status:** active. Supersedes `2026-09-24_nul-in-provider-output-is-replaced-visibly.md`.
- **Owner:** `SIGNOFF-REPAIR.4.4.10.3.1`
- **Date:** 2026-09-24
- **Work unit:** `REASONBRAID-REPAIR-0488`
- **Source:** the director's review of the superseded record: *"this is yours to make but better be sota and signoff"*.

## The fact / decision

No store in the platform can hold U+0000. PostgreSQL refuses it in `jsonb` (SQLSTATE `22P05`) and in `text` (`22021`), as `DOC-0154` measured. When a node builds a result, each U+0000 becomes U+FFFD, and the result carries **`nul_positions`**: canonical maximal runs `[start, len]` over Unicode scalar indices into the stored content. The control plane validates the claim and carries it onto the `thread.contribution_submitted` or `thread.revised` event. The claim must be runs in order, non-empty, never adjacent, in bounds, and covering U+FFFD only; any author may send it, and a false one is `400 invalid_command`. The original output is exactly the stored content with the run scalars set back to U+0000.

## Why

- **Keep the result, lose nothing.** Failing the attempt throws away a paid answer over a character that carries no meaning in a text contribution. Replacing it with only a count, the superseded decision, kept the answer but lost information. A provider's own U+FFFD could not be told from a replaced NUL, so the verbatim output was unrecoverable. With positions, the transformation is a bijection on everything the platform can store.
- **Visible where it is read.** The superseded record's marker lived only in the raw node event. The server never read it, and the contribution, which is what readers and agents consume, showed no sign of the change. The positions now sit on the event itself.
- **A claim is only worth what checks it.** Because the positions are validated against the content, a reader can rely on a marked scalar having been a NUL. Canonical runs make the same content always carry the same claim, so an idempotent redelivery hashes identically.
- **Standard parts, stated precisely.** U+FFFD is Unicode's sign for a character that could not be represented. Scalar indices, rather than UTF-8 bytes or UTF-16 units, match what the text IS rather than one encoding of it; a UTF-16 consumer converts. Runs keep pathological output compact: 256 KiB of NUL is one run.
- **Only `content` needs it.** Censused: the other text a node sends into PostgreSQL is a retry-gate constant or an error about the server's own payload. Provider failure reasons stay in the node's SQLite journal.

## How to apply

- The adapter contract's *verbatim* has ONE exception, U+0000 → U+FFFD, located by `nul_positions`. Any further exception needs its own decision.
- To recover the original: take the content's Unicode scalars, set each scalar in each run back to U+0000.
- A blind contribution's positions are withheld with its content until the commitment point (the blind read rule replaces the whole body).
- Rejected alternatives: storing the verbatim bytes as `bytea` or base64 beside the content (doubles every such result and gives readers two copies to reconcile); escaping NUL inside the text (every reader must unescape, and a literal escape sequence becomes ambiguous); U+2400 SYMBOL FOR NULL instead of U+FFFD (it is a printable glyph a provider can emit too, so it is no less ambiguous without positions, and it is not the standard's replacement sign).
