# Errors and reason codes

Every refusal this product returns carries a **stable reason code** — a
snake_case string in the response body's `code` field — alongside an HTTP status
and a safe human message. The code is what a client should branch on; the
message is for a person, and it may change.

```json
{ "code": "quota_exceeded", "message": "the tenant's monthly call quota is exhausted" }
```

## Two lists, and why both exist

`ROADMAP.md` §9.8 publishes a **stable registry** of 20 codes, mirrored in
`reasonbraid_core::KnownReasonCode`. That list is deliberately the *complete*
error model rather than the subset this build happens to use: a client and a
server can name a code consistently even when neither has reached the feature
that emits it.

The product also emits codes that postdate that list. A client decoding one gets
`ReasonCode::Unknown`, which **preserves the string verbatim** rather than
dropping it — the designed forward-compatibility path, not an accident.

What was missing until `SIGNOFF-REPAIR.11.7` is the list below: nothing told a
client author which codes this build actually emits, so the only way to learn
about `quota_unconfigured` was to receive one.

## Every code the server emits

Derived from the server's sources and held in step by the `REASON-CODE-DOC`
gate — a code the server can emit and this table does not name fails the build.
Re-derive it with `python3 -B scripts/census_reason_codes.py`.

**§9.8** marks a code in the stable registry; **ext** marks one this build added
after §9.8 was published.

| Code | HTTP | Registry | What it means |
| --- | --- | --- | --- |
| `unauthenticated` | 401 | §9.8 | No principal was presented, or the header was malformed. |
| `unauthorized` | 401 / 403 | §9.8 | A principal was presented and is not permitted this action on this resource. |
| `scope_hidden` | 404 | §9.8 | The resource may exist, but naming that is outside the caller's scope. Existence is never leaked across a tenant boundary. |
| `invalid_command` | 400 / 409 | §9.8 | The request is structurally wrong, or names something the command cannot accept. |
| `invalid_transition` | 409 | §9.8 | The aggregate cannot make this move from the state it is in. |
| `version_conflict` | 409 | §9.8 | An optimistic-concurrency check failed; re-read and retry. |
| `idempotency_mismatch` | 409 | §9.8 | The idempotency key was reused with a different payload. |
| `dependency_unavailable` | 500 | §9.8 | A dependency this request needs did not answer. |
| `protocol_incompatible` | 400 | §9.8 | The node channel version does not match the server's. |
| `not_found` | 404 | ext | The named resource does not exist within the caller's scope. |
| `unknown_node` | 404 | ext | The node id is not enrolled. Distinct from `not_found` so a channel client can tell "re-enrol" from "wrong id". |
| `quota_exceeded` | 429 | ext | A declared usage quota for this scope is exhausted. The window slides; retry later. |
| `quota_unconfigured` | 503 | ext | The scope has **no** configured quota, and the surface fails closed rather than admitting an unbounded caller. Retrying will not help until an operator declares a bound. |
| `storm_control` | 429 | ext | A fan-out or invitation-rate breaker tripped. |
| `classification_unqualified` | 409 | ext | The thread's classification requires a qualified evaluator profile and the deployment registers none. |
| `idempotency_conflict` | 409 | ext | A bootstrap request id is already bound to a different request. Distinct from `idempotency_mismatch`, which is about a replayed command's payload. |
| `commit_outcome_unconfirmed` | 500 | ext | The transaction's outcome is genuinely unknown — **not** a failure. Inspect the target before retrying; the write may have committed. |
| `publication_repository_unconfigured` | 503 | ext | The deployment declares **no** publication repository root, so the publish verb is closed. Like `quota_unconfigured` this is a deployment gap, not a request fault: retrying with another `repo_path` will not help until an operator configures one. |

⚠️ `commit_outcome_unconfirmed` is the one a client must not treat as a failure.
It is the honest answer when the server cannot observe whether its own commit
landed, and retrying blindly can duplicate an effect. The CLI's
`--resume-bootstrap` exists for exactly this case.

## A code this build used to emit

`locator_digest_conflict` (409) was emitted by `POST /v1/resources` until
`SIGNOFF-REPAIR.11.14.3.2` and is **retired**. It refused a second
`expected_digest` for a locator — but a reference's identity is the
`(original_locator, expected_digest)` pair, so the refusal denied a legitimate
second version of a changing page (§12.6), and its 409 told the caller that
another principal had registered that locator at a digest they were never shown,
which is the cross-tenant existence §9.8 forbids. A client that branched on this
code should treat the submission as accepted: the same pair replays, a different
digest is a second reference. See
[the citation registration](deployment.md#how-a-deliberation-registers-the-evidence-it-cites).

## Codes in the registry this build never emits

Eleven §9.8 codes are registered and not emitted anywhere:
`rate_limited`, `budget_unavailable`, `resource_unresolvable`, `resource_denied`,
`evidence_quarantined`, `provider_outcome_unknown`, `retry_requires_authorization`,
`no_quorum`, `approval_expired`, `publication_conflict`, `deployment_partial`.

That is deliberate and is **not** drift. They belong to features this build has
not reached, and a stable registry that is carved down to whatever the current
build uses stops being stable. A client may name them today; it will simply not
receive them yet.

## What a client should do

- **Branch on `code`, never on `message`.** Messages are written for people and
  are not a contract.
- **Preserve a code you do not recognize.** The registry is versioned and will
  grow; `ReasonCode::Unknown` keeps the string so a log or a report stays
  truthful.
- **Treat `commit_outcome_unconfirmed` as "unknown", not "failed".**
- **Do not infer existence from `404`.** `scope_hidden` and `not_found` are
  deliberately hard to tell apart from outside: §9.8 requires that cross-tenant
  existence is not leaked.
