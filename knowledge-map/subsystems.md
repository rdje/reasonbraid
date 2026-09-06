<!-- knowledge-map/subsystems.md — the ONE hand-curated input to the derived Knowledge Map.
     Edit this to give a fast orientation to the project's key subsystems / entry points.
     gen_knowledge_map.sh embeds this section verbatim; the task-tree and decision sections
     are generated automatically. -->

- `crates/reasonbraid-core/` — the domain-model crate (`KICKOFF.md` §3): strong identifiers, command/event envelopes, minimal thread/participation/provider-attempt state machines, and typed errors + the §9.8 reason-code registry landed (WP1 complete). Entry point `src/lib.rs`; envelopes in `src/envelope.rs` (golden fixtures/schemas under `fixtures/` and `schema/`); lifecycles in `src/state.rs`; errors in `src/error.rs`. Owner: repo-local workflow.
- `crates/reasonbraid-server/` — the control-plane crate (`KICKOFF.md` §3), first landed in WP2 `.2.1`: `apply_command` writes idempotency + event + state + outbox in one PostgreSQL transaction (claim-first idempotency). Schema at repository-root `migrations/`; proof via `scripts/run_pg_tests.sh` (ephemeral server) and the `pg-tests` CI job; the HTTP/SSE surface and leased worker are later leaves. Owner: repo-local workflow.
