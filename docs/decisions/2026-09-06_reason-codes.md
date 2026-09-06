# Reason codes and typed errors: the complete §9.8 registry, unknown codes preserved

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.1.4` (WP1 typed errors + reason-code registry)
answers: how are machine-actionable errors represented so the demo has stable reason codes and a code from a newer build is preserved rather than dropped?

## The fact / decision

`crates/reasonbraid-core/src/error.rs` encodes the *complete* `ROADMAP.md` §9.8 registry
as [`KnownReasonCode`] — all 20 codes, snake_case on the wire — wrapped by [`ReasonCode`],
which adds an `Unknown(String)` arm. `ReasonCode` is `#[serde(untagged)]`, so it tries the
known registry first and otherwise captures the raw string: a code this build does not
recognize is **preserved verbatim** and re-serializes to the same string, never collapsed
to a generic name and never a deserialization error.

[`DomainError`] is the typed, machine-actionable error (`ROADMAP.md` §9.8): a stable
`code`, a tri-state [`Retryability`] (`no`/`yes`/`requires_authorization`, the last mapping
§9.8's `retry_requires_authorization`), a safe human `message`, an optional
`correlation_id`, and visibility-filtered `details`. Secrets, policy internals, and
cross-tenant existence are not carried on the type.

`TransitionError` (from `PHASE-0.1.3`) maps to `invalid_transition` via
`From<TransitionError> for DomainError`, so the state machine's deterministic rejection
classifies into the registry rather than staying a bare string.

## Why

§9.8 lists the codes but does not specify how a build treats a code it has never seen.
The two naive choices both fail: a closed enum (`#[serde(rename_all)]` only) *rejects* an
unknown code (deserialization error), and a plain string *loses* the typing of the known
set. The `Known` + `Unknown(String)` split gets both properties at once — known codes are
typed and matchable, unknown codes round-trip — which is exactly the WP1 acceptance
"unknown codes remain preservable."

The registry is the *complete* §9.8 list rather than a demo-only subset because a
"stable registry" that gets re-carved every leaf is not stable: client and server must be
able to name any §9.8 code consistently, even if Phase 0's demo only ever emits a few of
them. Forward-compatibility (codes beyond §9.8) is `Unknown(String)`'s job, not a reason
to trim the known list.

## How to apply

- Source reason codes only from §9.8; do not invent a new known code without a §9.8 entry.
- Keep the `Known` + `Unknown(String)` split; never replace `ReasonCode` with a bare
  `String` (loses typing) or a closed enum (rejects the future).
- Any code this build does not recognize must deserialize as `ReasonCode::Unknown(raw)` and
  re-serialize to the same `raw`; the `unknown_codes_are_preserved_verbatim` test is the
  guard.
- New typed errors carry a `ReasonCode` + `Retryability` + safe `message` (+ optional
  `correlation_id` / filtered `details`) rather than a bare string; extend `DomainError`
  or add a `From` impl for a specialized error, don't add a parallel error shape.
- See [[2026-09-06_state-transitions]] (whose `TransitionError` feeds `invalid_transition`)
  and [[2026-09-06_envelope-representation]] (where a rejected command becomes a typed
  error response).
