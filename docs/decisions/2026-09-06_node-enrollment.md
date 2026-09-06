# 2026-09-06_node-enrollment.md

## Context

`PHASE-1.2.1` (backlog 11) landed dev-profile node enrollment: an authorized
human issues a one-time token (bound to tenant, expected node id, host claim,
nonce, expiry — §16.2), and the node consumes it with a dev signing secret.
Certificate issuance (X.509/mTLS) stays deferred to Phase 2 (ADR-007).

## Decision

- **The Phase 1 credential is the token plus a dev shared secret, and the
  server is the trust store** — the same documented stance as the `.6.1` dev
  principal header. The secret's SHA-256 fingerprint is stored alongside it;
  the HMAC key-proof over the handshake arrives with `.1.2.2`.
- **The token row is the serialization point, not the handler.** One token,
  one use, ever: `FOR UPDATE` on the token row + the `used_at` marker makes
  "one-time" a database fact; a second use is refused AND audited, and the
  refusal writes nothing else.
- **Every refusal is a committed audit row** (the budget engine's denial-row
  pattern): the `node_enroll_audit` row commits before the typed error
  returns, so a refused enrollment is as auditable as an accepted one.
- **Enrollment lands host + node + key + token-consumption + audit in ONE
  transaction** — an enrolled node implies all of its identity rows.

## Consequences

- The operator surface: `rb node issue-token` (tenant_admin authority, audited
  by the authorization engine) and `rb-node --enroll-token …` consuming it
  before any channel traffic.
- The 0008 tables joined every suite's purge list in FK order.
- `.1.2.2`'s authenticated handshake builds on `node_keys` instead of
  re-bootstrapping credentials.

answers:

- **"One-time" is a row property, not a handler branch.** `FOR UPDATE` plus a
  nullable `used_at` makes replay impossible at the database level; the
  handler only maps the row state to a typed, audited refusal. A token bound
  to a node id, host claim, and nonce means a stolen token cannot enroll a
  different identity.
- **A refusal that must be durable rides the denial-row pattern:** write the
  audit row in the caller's transaction, commit, then return the error. The
  budget engine established it; enrollment reuses it — refusals are data,
  not just responses.
- **sqlx's `Transaction::commit(self)` consumes the transaction** — a helper
  cannot commit a `&mut Transaction` it was passed. The working shape is:
  validate → write the audit row on the borrowed transaction → commit once in
  the caller → return the typed error. Same lesson as the budget engine's
  in-tx bodies; worth remembering for every future refusal path.
