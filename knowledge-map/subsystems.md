<!-- knowledge-map/subsystems.md — the ONE hand-curated input to the derived Knowledge Map.
     Edit this to give a fast orientation to the project's key subsystems / entry points.
     gen_knowledge_map.sh embeds this section verbatim; the task-tree and decision sections
     are generated automatically. -->

- `crates/reasonbraid-core/` — the domain-model crate (`KICKOFF.md` §3): strong identifiers and command/event envelopes landed (WP1); minimal thread/provider-attempt state to come. Entry point `src/lib.rs`; envelopes in `src/envelope.rs` with golden fixtures/schemas under `fixtures/` and `schema/`. Owner: repo-local workflow.
