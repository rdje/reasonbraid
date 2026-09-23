---
answers:
  - Where do the six §11.5 bounds of a thread:create:auto grant live today, and who can set them?
  - Why is there no API that issues a grant with a spend limit, and what produced the ones the tests use?
  - What does "audience" mean in this codebase, and what is a side-effect bound over?
  - In what order are the grant-carried bounds built?
---
# The auto grant's bounds have no producer, and two of the six have nothing to bound yet

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3`
- **Date:** 2026-09-23
- **Cites:** ROADMAP §11.5 (*a separate `thread:create:auto` grant with topic, audience, rate,
  depth, spend, and side-effect bounds*), §14.1 (*autonomous child threads and recursion depth* as a
  budget dimension), §10.5 (a call's audience);
  `docs/decisions/2026-09-23_the-wake-checklist-is-a-node-gate-and-the-auto-grant-carries-six-bounds.md`
  (DOC-0135, item 3); `SIGNOFF-REPAIR.5.2` clause 2 (the spend gate reads expired grants)

## Context

DOC-0135 decided that `.5.3.2.3` moves topic and depth onto the grant and adds audience and
side-effect bounds. Before building, the leaf censused where a grant's bounds could come from
and what each bound would be evaluated against. Every claim below cites the command that
decides it, run at `5d35b72`.

## The census

**No route issues a grant.** `grep -n 'route("/v1/.*grant' crates/reasonbraid-server/src/*.rs`
→ exactly two: `GET /v1/admin/grants` (list) and `POST /v1/admin/grants/{grant_id}/revoke`.
`grep -rn "create_grant(\|create_grant_in_guard(" crates/reasonbraid-server/src` → two
producers: the enrolment (`crates/reasonbraid-server/src/api.rs:1187`, through `dev_grant`, whose `spend_limits` is
`None` and whose actions are whatever the enrolment body asked for — *grant issuance is
dev-trusted — documented*) and the card import (`crates/reasonbraid-server/src/authority/profile_admin.rs:497`). The
`create_grant` library function is `pub` and is called by tests.

⇒ **Every `thread_create_auto` grant that carries a spend limit in this repository is a
row a test inserted by SQL.** An enrolment-issued auto grant carries no limit, so the
existing spend gate refuses any budgeted initiation from it. A typed bounds field added
now would be a schema with no producer — the shape `migrations/0068` names for the two
quota scopes that *shipped with a vocabulary, a schema and no producer*.

**The spend gate reads the wrong grant.** `create_thread_auto` asks
`SELECT max((spend_limits->>'amount')::float) FROM authority_grants WHERE subject_id = $1
AND status = 'active' AND actions @> '["thread_create_auto"]'` — no validity window, and
`max()` over every grant rather than the one that admitted the request. `.5.2` clause 2
recorded it; it is the same defect as *bounds with no producer* seen from the reader's
side: nothing binds the bound to the authorization.

**Where the record already names the admitting grant.** `authorization_records.grant_id`
(`migrations/0004`) is written for every allowed decision, and `authorize_guarded` returns
the `record_id`. So the grant that admitted an initiation is one lookup away, and the
bounds can be read from *that* grant — live by construction, because authorization only
admits on a live one.

**"Audience" occurs nowhere in the code.** `grep -rn audience crates/*/src` → 0. The nearest
fact is `EligibilityExpression.scope: ReaderClass` (`crates/reasonbraid-server/src/matching.rs:21`; `Tenant` or
`Network`) — the reader scope a recruitment call opened on a thread evaluates candidates at.
That is what §10.5 calls a call's audience. An audience bound on the auto grant is therefore
a bound on the calls an autonomous thread may open, which needs the thread to remember the
grant that initiated it.

**No verb attributes a side effect to a thread.** `ResolveRequest` (`crates/reasonbraid-server/src/api.rs:2341`) and
`SnapshotRequest` (`crates/reasonbraid-server/src/api.rs:5217`) carry no thread id; acquisition and snapshot are principal-bound.
A side-effect bound on an autonomous thread has no enforcement point until a side-effecting
verb is thread-attributed.

**Rate** landed as the role's `initiator` quota row (REPAIR-0417). It stays there by design:
the quota machinery is where windowed counting and recorded denials live, and a per-grant
number would need a second counter. The grant's issuer configures the row beside the grant.

## Decision — four children, in this order

1. **`.2.3.1` — the producer.** The one dev-profile issuer that already declares a role's
   actions — the enrolment body — also declares the auto grant's typed bounds (`spend`,
   `topics`, `max_depth`), and the card import carries them. A typed `AutoBounds` on
   `AuthorityGrant` and on the row. RED first: an enrolment declaring bounds yields a grant
   row that carries them; one declaring none yields a grant that admits no budgeted
   initiation, exactly as today.
2. **`.2.3.2` — the reader.** `create_thread_auto` reads spend, topics and depth from the
   grant named by the admitting authorization record — never `max()` over the subject's
   grants — with `MAX_AUTONOMOUS_DEPTH` as the site ceiling a grant may not exceed. This
   discharges `SIGNOFF-REPAIR.5.2` clause 2. The profile's `interests` stay the role's
   wake-for-topic declaration; the grant's `topics` are the issuer's bound; both apply.
3. **`.2.3.3` — audience.** The autonomous thread records the grant that initiated it, and
   `POST /v1/calls` on such a thread refuses an eligibility `scope` wider than the grant's
   `audience`. Buildable once 1 has landed.
4. **`.2.3.4` — side-effect bounds**, deferred on a readable trigger: the first verb that
   attributes a side effect to a thread. Recorded as a condition, not a promise.

⛔ Not decided here: a general grant-issuance API. That is an authority-model lane of its
own (who may issue, under which boundary, audited how) and the roadmap's §4 owns it. The
enrolment producer is the dev profile's existing trust boundary, extended by three typed
fields, and is documented as such.
