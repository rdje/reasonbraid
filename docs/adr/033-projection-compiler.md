# ADR-033 — The projection compiler: byte-identical output from the resolved clauses — the unrepresentable clause names itself, never silently omitted

- **Status:** `accepted` (evidence-gated — the §15.5 contract:
  the deterministic rendering, the target vocabulary, the
  unrepresentable declaration, and the hermetic crate rule are
  the shapes the `.3.2`/`.3.3` leaves implement)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-6.3.1`
- **Requirements:** `ROADMAP.md` §15.5 (the deterministic
  projection compiler)

## Context

The `.3` census mapped §15.5 against the shipped surface. The
compiler is a GREENFIELD: no projection exists, no target
vocabulary, no unrepresentable declaration (`git grep -c
"projection" HEAD -- crates/` finds only the thread/profile
projections — not the policy compiler). The reusable pieces:
the `.1` resolution (the resolved clause set is the compiler's
INPUT — the compiler never re-resolves), the `.1` registry's
digest shapes (the byte-identical proof's substrate), the
policy.lock's inputs (the versions, the digests, the
dependencies, the authority facts — the `.1.2` registry + the
`.1.3` explanation tree already hold them).

## Decision

- **The compiler renders the RESOLVED set, never re-resolves.**
  The input is the `.1.3` `Resolution`'s clause set (each with
  its winning policy/version + its path); the compiler is
  pure rendering — the semantic questions are answered before
  it runs. A compiler that re-resolved would be a second
  judge (the ADR-017 trap, again).
- **The rendering is byte-identical by construction.** The
  compiler is deterministic: the same semantic inputs +
  compiler + profile + target parameters produce
  BYTE-IDENTICAL output (the renderer is a pure function of
  the ordered clause list — the stable sort + the fixed
  templates + no timestamps, no ambient state, no
  nondeterministic iteration). The projection's digest (the
  ADR-011 shape over the output bytes) is the §15.7
  publication's verification primitive.
- **The target vocabulary is the initial §15.5 set:** the
  generic system/developer instruction bundle, the Codex
  `AGENTS.md` bundle, the Claude `CLAUDE.md` bundle, and the
  `policy.lock` (the versions + the digests + the dependency/
  authority resolution). The MCP manifests, the host-config
  fragments, and the human checklists wait for their
  projection adapters (named deferrals — a target without an
  adapter is the typed refusal, never a guessed render).
- **The unrepresentable clause is a DECLARED refusal.** A
  clause that cannot ride a target (the harness-specific
  semantics, the shape the target cannot express) names
  itself in the projection's `unrepresentable` list with the
  reason; it is NEVER silently omitted. The
  consumer-facing rule: a projection with a non-empty
  unrepresentable list is a WARNING-carrying artifact — the
  operator decides, the compiler refuses to pretend.
- **The compiler is a separate hermetic crate.** The
  compilation lives in its own crate (the clean-worker
  doctrine — the Phase-4 extraction precedent): no database,
  no network, no ambient environment — the pure function
  shape is structural, not a promise. The server (or the
  `.4` publication worker) invokes it over the resolved
  inputs.

## Consequences

- `.3.2` implements the compiler core (the generic bundle +
  the `policy.lock`, the deterministic serialization, the
  projection record) and `.3.3` the Codex + the Claude
  renderers with the unrepresentable declarations + the
  projection tests (the loss/ordering/escaping/size/
  harness-conflict coverage) — each against this contract
  verbatim; a deviation is a contract change.
- The `.4` publication lane's step-2 ("compile the canonical
  bundle in a clean worker") invokes exactly this crate; the
  byte-identical guarantee is what makes the publication
  verifiable after the fact.
- The projection digest becomes the deployment receipt's
  anchor (the `.5` lane's drift detection compares the
  digests, never the prose).

answers:

- **Determinism is a shape, not a hope.** The byte-identical
  guarantee holds because the compiler is a pure function of
  the ordered resolved set — no clock, no ambient state, no
  hash-iteration order — the same argument the `.4.3`
  splitmix64 draw makes for the trials.
- **The unrepresentable list is the compiler's honesty.** A
  silent omission would be the same dishonesty as a decided
  close carrying unresolved items (`.2.4.1`) — the projection
  that cannot express a clause says so, in the artifact
  itself.
- **A separate crate is the hermetic rule made structural.**
  The compiler with no database or network handle cannot
  drift on the server's ambient state — the quarantine
  argument the extraction worker already ships under.
