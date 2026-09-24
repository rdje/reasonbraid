---
answers:
  - Does a node deliver provider output verbatim?
  - What happens to provider output that holds U+0000 (the NUL character)?
  - Why is a NUL replaced rather than the attempt failed?
---
# NUL in provider output is replaced visibly, and counted

- **Type:** decision
- **Status:** superseded by `2026-09-24_nul-in-provider-output-is-replaced-losslessly.md` (`SIGNOFF-REPAIR.4.4.10.3.1`, REPAIR-0488). The director's review found this form short of signoff: a count and no positions, so the replacement was lossy and ambiguous, and the marker never reached the contribution. Kept unchanged below as the record of what was decided first.
- **Owner:** `SIGNOFF-REPAIR.4.4.10.3`
- **Date:** 2026-09-24
- **Work unit:** `REASONBRAID-REPAIR-0473`
- **Cites:** `docs/book/src/adapter-boundary.md` (*the stream's chunks pass through verbatim*); `REASONBRAID-DOC-0154` (the measurement); `SIGNOFF-REPAIR.4.4.10.1` (the control plane's `400 unrepresentable_input`) and `.4.4.10.2` (the node's permanent-refusal handling).

## The fact / decision

No store in the platform can hold U+0000. PostgreSQL refuses it in `jsonb` (SQLSTATE `22P05`) and in `text` (`22021`), measured by `DOC-0154`. A work result carrying one could therefore never be accepted: since `.4.4.10.1` the control plane refuses it `400 unrepresentable_input`, and since `.4.4.10.2` the node journals that refusal and stops offering the event. The node no longer wedges, but the whole paid result is lost, and a provider can produce the character (the fixture corpus's own `malformed_output` does).

**Decision:** when the node builds a result (`Worker::process`, `storable_content`), each U+0000 becomes U+FFFD, the Unicode replacement character, and the result carries `nul_replaced: <count>`. Every other character, control characters included, stays verbatim. With no NUL, the content is untouched and the field is absent.

## Why

- The alternative, failing the attempt with a named reason, throws away a paid result because of one character that carries no meaning in a text contribution.
- U+FFFD is the standard's own sign for "a character that could not be represented". It is visible exactly where the NUL stood, so a reader sees that something was there.
- `nul_replaced` states the change in the result itself, so the transformation is never silent, and an operator can find every altered result by that field.
- The node does no parsing: the one change is a fixed, character-for-character substitution, deterministic and reversible in count. It does not interpret the content (§ *model output stays untrusted*).

## How to apply

- The adapter contract's *verbatim* has ONE stated exception: U+0000 → U+FFFD, counted. Any further exception needs its own decision.
- ⚖️ **For the director:** if exact bytes matter more than keeping the result (for example, a binary-carrying workload), the alternative is a `failed_known` naming the character, a one-line change in `storable_content`'s caller. The control plane's refusal (`.4.4.10.1`) and the node's refusal handling (`.4.4.10.2`) stand either way.
