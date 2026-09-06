# 2026-09-06_identity-store.md

## Context

`PHASE-1.1.2` (backlog 10) landed migration 0007: the first-class identity
store — `tenants`, `human_principals`, `agent_roles`, `hosts`, `nodes`,
`incarnations`, `runs` — beside the `.6.1` `enrollments` table. Enroll now
writes the enrollment row AND the identity row in one transaction.

## Decision

The identity tables are the identity RECORDS (`ROADMAP.md` §8.1 hierarchy);
`enrollments` stays what it was — the dev bootstrap's (tenant, kind, name) →
principal-id map. Every material record carries `tenant_id` (§17.2), the wire
forms are the core `Id<K>` UUIDv7 strings, and the foreign keys
(`human_principals`/`agent_roles` → `tenants`, `nodes` → `hosts`,
`incarnations` → `agent_roles`, `runs` → `incarnations`) fail closed: an
identity row whose parent does not exist is refused by the database, never a
silent half-identity. Columns that belong to later features (node certificate
state, run receipts, incarnation attestation) arrive with those features — the
0002 precedent of never guessing a shape early; the incarnation carries only
the §8.1 facts that DEFINE it (provider/model/harness/config, validity
interval).

## Consequences

- Enroll's single transaction now covers four-plus rows (tenant → boundary →
  grant → identity → enrollment); a replay duplicates nothing (the replay
  check returns before any insert, and the unique keys would refuse it anyway).
- The purge lists of the API-driving suites gained the identity tables in FK
  order (children first).
- `hosts`/`nodes`/`incarnations`/`runs` are schema-only until their owning
  flows (backlog 11 workload enrollment, the node channel identity work) write
  them — no dead columns were guessed in.

answers:

- **The identity table is the record; the enrollment table is the map.** Keep
  the dev bootstrap's name→id convenience map as its own table instead of
  upgrading it into the identity schema — the identity tables model §8.1
  exactly, and the map stays disposable without touching identity truth.
- **Write the parent row first inside the same transaction, and let the FK
  enforce the order.** The tenant insert precedes the principal insert in
  `enroll`, and the FK turns any future caller that forgets the order into a
  failed transaction — the database, not a comment, is the enforcement.
- **A re-enroll is a replay at the identity layer too.** Because the replay
  path returns before any insert AND the identity tables carry their own
  unique keys, idempotent bootstrap is double-enforced — proven by the
  `identity_store` suite's count assertions after re-enroll.
