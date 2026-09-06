# Decision & Fact Records — Index (memory layer C)

Durable, cross-cutting facts and decisions live here, one record per file (ADR-style). Every
record must be listed below (the MEMORY-ARCH doctrine check enforces it). New record: copy
`TEMPLATE.md` → `<type>_<short-kebab-slug>.md`, fill it in, and add its row.

| Record | Type | One-line hook |
| --- | --- | --- |
| [2026-09-05_kickoff-companion-to-roadmap.md](2026-09-05_kickoff-companion-to-roadmap.md) | decision | `KICKOFF.md` is the Phase 0 companion to `ROADMAP.md` |
| [2026-09-05_roadmap-v0.4.1-frozen.md](2026-09-05_roadmap-v0.4.1-frozen.md) | decision | roadmap v0.4.1 frozen until Phase 0+1 evidence |
| [2026-09-05_claim-verification-adopted.md](2026-09-05_claim-verification-adopted.md) | decision | architecture #5: re-derive · falsify · durability |
| [2026-09-05_adr-001-working-name.md](2026-09-05_adr-001-working-name.md) | decision | ReasonBraid is an uncleared working name |
| [2026-09-06_accountable-owners.md](2026-09-06_accountable-owners.md) | decision | Richard DJE accountable for architecture decisions + release/security gates |
| [2026-09-06_g0-contract-id-scheme.md](2026-09-06_g0-contract-id-scheme.md) | decision | G0 requirement IDs (ID/AUTH/THREAD/DELIV/BUDGET) + `spec/` location for contract drafts |
| [2026-09-06_id-representation.md](2026-09-06_id-representation.md) | decision | IDs are branded newtypes over UUIDv7 with per-kind wire prefixes (`ten`/`hpr`/`hst`/`nod`/`rol`/`inc`/`run`/`thr`) |
| [2026-09-06_envelope-representation.md](2026-09-06_envelope-representation.md) | decision | command/event envelopes: client expresses intent, server assigns actor/tenant/sequence/authority/timestamps; `deny_unknown_fields` rejects forgery |
| [2026-09-06_state-transitions.md](2026-09-06_state-transitions.md) | decision | thread/participation/provider-attempt lifecycles are minimal state machines with deterministic fallible `apply`; `patt` prefix for `ProviderAttemptId`; partially superseded by node-journal (`failed_known` landed proof-gated) |
| [2026-09-06_reason-codes.md](2026-09-06_reason-codes.md) | decision | typed errors + the complete §9.8 reason-code registry; unknown codes preserved verbatim via `ReasonCode::Unknown` |
| [2026-09-06_atomic-transaction.md](2026-09-06_atomic-transaction.md) | decision | the WP2 atomic transaction: state + event + idempotency + outbox in one PostgreSQL transaction; claim-first idempotency, replay vs conflict |
| [2026-09-06_outbox-worker-fencing.md](2026-09-06_outbox-worker-fencing.md) | decision | the WP2 leased outbox worker: claim with a per-claim fencing token + lease, deliver into a deduped sink, acknowledge only with the current token and a live lease; kill points 3–5 proven |
| [2026-09-06_node-journal.md](2026-09-06_node-journal.md) | decision | the WP3 node journal: WAL + synchronous=FULL recorded in journal_meta, the dispatch boundary recorded before the adapter runs, outcome_unknown recovery with prove/reconcile exits, read-only rb-journal CLI |
| [2026-09-06_node-channel.md](2026-09-06_node-channel.md) | decision | the WP3 node channel: the node reports its durable resume facts (cursor + pending operations), the server replays the tail and reconciles from receipts, and schedulability gates on the applied handshake |
| [2026-09-06_fake-adapter.md](2026-09-06_fake-adapter.md) | decision | the WP4 adapter boundary: a narrow capability-declaring contract (ack ≠ completion, no credentials, unsupported lookup is never retry advice), the deterministic scripted fake, the sanitized outcome corpus, and the supervisor's honest ambiguity path |
| [2026-09-06_real-adapter-codex.md](2026-09-06_real-adapter-codex.md) | decision | the first real adapter is the Codex-family CLI behind `codex exec --json` (supervised subprocess); the streamed thread id is the provider handle, status lookup is honestly unsupported, and the second adapter (Claude) is recommended for Phase 1 |
| [2026-09-06_authority-boundary.md](2026-09-06_authority-boundary.md) | decision | the development authority engine: the enrollment boundary is a ceiling, grants are subsets enforced at creation AND evaluation, membership grants nothing, and every decision — allow or deny — is audited with the policy digest + version |
| [2026-09-06_budget-reservation.md](2026-09-06_budget-reservation.md) | decision | the development budget engine: reservations against the ceiling before any dispatch, denials recorded at the server and refused at the node, settlement with actual usage (overruns reported, never clamped), indeterminate attempts keep their hold |
