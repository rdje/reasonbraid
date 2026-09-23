---
answers:
  - What is a grant condition, and what makes something a member of the vocabulary?
  - Why a closed typed set rather than a policy language or free-form predicates?
  - What is the first member, how is it evaluated, and what was rejected for now?
  - How is a member added, and what keeps a declared-and-unread condition out?
---
# A grant's conditions are a closed, typed, evaluable vocabulary

- **Type:** decision
- **Status:** accepted — decided under the director's delegation of 2026-09-23 (*your decision, state of the art and signoff-grade, in the project's best interest*)
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.5.4.3`
- **Date:** 2026-09-23
- **Cites:** ROADMAP §4.2 (`conditions[]`), §4.5 (*separation-of-duties conditions*), §4.6 (*low-risk
  standing authorization is a signed rule template with strict target/domain/value limits, not a
  blanket bypass*); `docs/decisions/2026-09-23_of-the-four-missing-grant-fields-one-is-owed-now-one-waits-on-a-domain-one-on-the-director-and-one-is-two-halves.md`
  (DOC-0140, which found the field undefined); `crates/reasonbraid-core/src/authority.rs`
  (`GrantCondition`), `crates/reasonbraid-server/src/authority/conditions.rs`

## Context

§4.2 lists `conditions[]` among a grant's dimensions and says *authorization requires all
dimensions to match*; nothing in the roadmap or the code defined a condition. DOC-0140 refused
to build the field until it had a vocabulary, because a declared-and-unread field is the defect
`SIGNOFF-REPAIR.11.4.7.2.1` names. The candidates on the table were a purpose, a time window
and a requesting network. The director delegated the choice.

## Decision

**A closed, typed, evaluable vocabulary**, evaluated at every admission, validated at issuance,
extended one member at a time by a fixed rule. This is the shape the state of the art converged
on for authorization conditions once it stopped writing predicates in prose: IAM condition keys,
Cedar's `when` clauses and XACML conditions are all typed operators over facts the evaluator
already holds, with an unknown key refused at write time. What this project does NOT adopt is
the general policy language: a grant here is a scoped mandate under a boundary, evaluated by a
deterministic core function whose inputs are recorded in the authorization record, and a
condition must keep that property — decidable from the admission's own facts, refusable at
issuance, nameable in the denial.

1. **A member is** a typed variant on `reasonbraid_core::GrantCondition` (`kind`-tagged,
   `deny_unknown_fields`), an issuance rule in `conditions::issuance_violations` refusing every
   declaration it cannot give a meaning to, an evaluator arm in `conditions::holds` over facts
   the admission already holds (the database time `at`, the `CommandAuthz`'s actor, subject,
   action and target), a control that issues it, is refused by it and is admitted under it, and
   a paragraph in the book. Anything short of all five is not a member. An empty list is refused
   (it binds nothing); an unknown kind is refused by the wire type; the grant listing shows the
   conditions typed.
2. **The first member is `within_hours { window }`**: the grant acts only inside a daily UTC
   window, `HH:MM-HH:MM`, start inclusive, end exclusive, wrapping midnight — the profile's
   `operating_hours` format, read by the same parser (`wake::OperatingHours`), so one format has
   one meaning across the product. It is §4.6's *standing authorization with strict limits* in
   time, and it is evaluable today because the admission already samples database time.
3. **Evaluated after the action, before the target kind**, in `authority::evaluate`: a covered
   action must still satisfy its conditions, and the denial names the condition and the fact —
   *the grant's within_hours condition does not hold: 17:03 UTC is outside `09:00-17:00`*. A
   stored window the parser cannot read denies, fail-closed: the issuance refused it, so the row
   was hand-edited.
4. **Roles only, in the dev profile**, like every other bound: a human's dev grant is the admin
   set and carries no bound (`api::enroll`'s rule, unchanged in spirit and extended by one name).
5. **Rejected for now, each for a measured reason.** *Purpose*: no command carries a declared
   purpose for a grant to be conditioned on; `DelegationConstraints.purpose` binds a delegation,
   not an admission. *Requesting network*: the admission holds no source address — the principal
   header is the identity, and a network condition would be evaluated against a fact nobody
   records. *`human_present`* (the target thread has a human participant): evaluable in
   principle, but the evaluator is a pure function over the grant and the context, and the
   thread's participants are a read the selection would have to make per candidate grant;
   the next member, once the admission carries the target's participant facts.
   *Separation of duties* (§4.5): belongs to the policy chain's approval, where the author and
   the approver are both on the record, and is that lane's to add here as a member.

## Consequences

- `authority_grants.conditions JSONB` (migration 0099, NULL = unconditional, which every
  existing grant was); the grant row is decoded by name (a seventeen-column tuple exceeds what
  sqlx decodes); the enrolment body accepts `conditions`; the operator's grant listing shows them.
- ⚠️ A cached admission (`CACHED_ALLOW_TTL_SECONDS = 60`) can carry a `within_hours` admission
  up to a minute past the window's end at the node's dispatch; the condition is evaluated at the
  admission, and the cache's own TTL bounds the drift. Stated, not hidden.
