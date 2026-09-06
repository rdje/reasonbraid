# 2026-09-06_thread-api-completion.md

## Context

`PHASE-1.1.3` (backlog 15's API-shape portion) completed the thread command
API: `thread.cancel` (the `open|closing → cancelled` abandonment terminal,
distinct from a decided close) and the three typed create fields —
classification, workflow profile, participant rules — with deny-unknown wire
typing and stated defaults.

## Decision

- **Cancel and close are different terminals and never mix:** close records
  `close_reason` and rides the `open → closing → closed` fold; cancel records
  `cancel_reason` and rides the single `open|closing → cancelled` core edge.
  Both are lifecycle authority — `thread_cancel` joins the grant registry as
  its own action (the registry lives in one enum, `GrantAction`, with the
  wire-name test enumerating it).
- **The typed create fields are enums/structs with STATED defaults, not empty
  profiles:** `classification` defaults `general`; `workflow_profile` defaults
  `single_agent` (ADR-002's routing decision, made visible on the wire and in
  the CLI); `participant_rules` defaults explicit-invites-on / join-requests-
  off (§20.3 explicit participants first). Serde enums give deny-unknown
  typing: a foreign field or an out-of-registry value is a typed
  `invalid_command`, never a silently stored string. The non-default workflow
  profiles are recorded today and executed as single-agent until routing work
  lands (`.1.3`+ — stated, not silently ignored).
- **Projection growth is additive:** the new projection fields are
  `#[serde(default)]`-ed, so a projection written before `.1.1.3` still
  parses — the JSONB projection evolves forward-compatibly in the dev profile.

## Consequences

- The CLI gains `thread cancel` and the three create flags; inspection shows
  classification/workflow and the cancel reason.
- `thread_cancel` is part of the dev admin set (the bootstrap human's grant);
  a role must be granted it explicitly.
- `.1.3` (invitation accept/decline/timeout semantics) builds on the
  participant-rules field instead of re-typing it.

answers:

- **An enum-shaped wire field stays honest when the default is a documented
  variant, not an empty profile.** `serde(rename_all = "snake_case")` enums +
  `deny_unknown_fields` + `#[default]` variants make "unnamed" mean exactly
  one stated value (`general`, `single_agent`, explicit-invites-only) — the
  tests assert the defaults, so the contract cannot silently drift to an
  empty profile.
- **A registry grows by naming the new entry in the one place the registry
  lives and letting the enumerating test fail first.** Adding `thread_cancel`
  to `GrantAction` broke exactly the wire-name round-trip test until it was
  extended — the registry's test is the canary that a new authority name was
  not forgotten anywhere.
- **Two terminals must not share a reason field.** `close_reason` and
  `cancel_reason` are separate projection fields and separate events — a
  cancelled thread that answered "why did it close?" would be lying; the API
  test asserts `close_reason` stays null on cancel.
