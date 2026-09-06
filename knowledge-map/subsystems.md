<!-- knowledge-map/subsystems.md — the ONE hand-curated input to the derived Knowledge Map.
     Edit this to give a fast orientation to the project's key subsystems / entry points.
     gen_knowledge_map.sh embeds this section verbatim; the task-tree and decision sections
     are generated automatically. -->

- `crates/reasonbraid-core/` — the domain-model crate (`KICKOFF.md` §3): strong identifiers, command/event envelopes, minimal thread/participation/provider-attempt state machines, and typed errors + the §9.8 reason-code registry landed (WP1 complete). Entry point `src/lib.rs`; envelopes in `src/envelope.rs` (golden fixtures/schemas under `fixtures/` and `schema/`); lifecycles in `src/state.rs`; errors in `src/error.rs`. Owner: repo-local workflow.
