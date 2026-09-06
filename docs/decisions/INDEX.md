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
