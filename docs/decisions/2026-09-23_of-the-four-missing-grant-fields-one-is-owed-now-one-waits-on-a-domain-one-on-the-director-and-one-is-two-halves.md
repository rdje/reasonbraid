---
answers:
  - Which of §4.2's grant fields with no column are owed before the stable release?
  - What does each missing field need to exist before it can bind anything?
  - Where does a decision-rule constraint on a grant get read, and against what?
---
# Of the four missing grant fields, one is owed now, one waits on a domain, one on the director, and one is two halves

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.5.4`
- **Date:** 2026-09-23
- **Cites:** ROADMAP §4.2 (the `AuthorityGrant` field list: `policy_domains[]`,
  `decision_rule_constraints`, `conditions[]`, `grant_signature_or_authorization_record`,
  beside `spend_and_autonomy_limits`), §16.3 (delegation constraints), ADR-022 (the
  non-loopback trigger); `docs/decisions/2026-09-23_the-auto-grants-bounds-have-no-producer.md`
  (DOC-0137, the producer-and-reader-in-one-commit precedent);
  `docs/decisions/2026-09-22_a-counted-rule-derives-its-outcome-and-a-rule-it-cannot-count-is-refused.md`
  (the charter's decision rules)

## Context

`.11.4.7.2.1.5`'s row 10 found four §4.2 fields with no column, type or check anywhere. DOC-0137
settled the fifth, `spend_and_autonomy_limits`, as `spend_limits` plus `AutoBounds`. This record
decides the four. Every claim cites the command that decides it, run at `fa9ce9f`.

## The census

| field | what exists | what a grant-level field would bind today |
| --- | --- | --- |
| `policy_domains[]` | the BOUNDARY carries `permitted_domains` (`migrations/0004`, core `EnrollmentAuthorityBoundary.permitted_domains`), and `grep -c -i domain crates/reasonbraid-core/src/authority.rs` → **2**: the declaration and a fixture. No request, target or evaluator names a domain | **nothing** — the boundary's own domains are compared to nothing, so a grant narrowing them would narrow nothing |
| `decision_rule_constraints` | the tenant's CHARTER names its allowed decision rules and `charters::allows_on` refuses a declared rule outside it, inside the create transaction after authorization (`run_thread_command`'s Create arm). The vocabulary is `charters::DecisionRule` (`owner_decides`, `majority_of_electorate`, `unanimity`, `consensus`, …) | **a real thing**: the rules THIS subject may declare, narrower than the charter — read from the admitting grant at the same point the charter is read |
| `conditions[]` | nothing names a vocabulary. The nearest shape is §16.3's `DelegationConstraints { on_behalf_of, purpose, scope }` | undefined: a field with no vocabulary is the declared-and-unread defect by construction |
| `grant_signature_or_authorization_record` | the core header says *crypto-signed records and full delegation chains are later phases*; the site authority signs nothing; ADR-027 signs release artifacts. The fourteen administrative operations write `administrative_effects` rows (`authority/effects.rs`), but the ENROLMENT's dev grant does not: `grep -c "AdministrativeEffect\|record_effect" crates/reasonbraid-server/src/api.rs` → **0** | two halves: a SIGNATURE (a workload-identity concern that rides ADR-022's non-loopback trigger, with mTLS in `.11.4.7.2.1.5.2`) and a RECORD (an issuance effect the dev grant could carry now) |

## Decision — four children

1. **`.5.4.1` — `decision_rule_constraints`, owed now.** The DOC-0137 shape, one commit:
   a typed field on `AuthorityGrant` (rule names, validated at `create_grant_in_guard` against
   `charters::DecisionRule`; refused on a grant without `thread_create`/`thread_create_auto`;
   an empty list refused), declared in the enrolment body, listed by `GET /v1/admin/grants`,
   and READ in `run_thread_command`'s Create arm from the ADMITTING grant beside the charter
   check: a declared rule must be allowed by the charter AND by the grant when it constrains.
   Absent means the charter alone, exactly as today.
2. **`.5.4.2` — `policy_domains[]`, deferred on a readable trigger:** a request, target or
   evaluator names a domain — `grep -c -i domain crates/reasonbraid-core/src/authority.rs`
   > 2. When the dimension arrives, the boundary's and the grant's domains land together.
3. **`.5.4.3` — `conditions[]`, waits on the director.** 💡 The roadmap names the field and
   nothing defines what a condition is; the candidates (a purpose, a time window, a
   requesting network) are product decisions, not engineering ones. Recorded like
   `.11.4.7.2.1.5.3.2.2.1`.
4. **`.5.4.4` — the authorization-record half, owed now; the signature half on ADR-022's
   trigger.** The enrolment's dev grant gets an administrative effect record like every other
   issuance-class operation, so *every grant has its record* holds; the signature rides the
   first non-loopback deployment with mTLS.

⛔ Not decided here: the signature scheme, and the condition vocabulary. Both are named with
the fact that decides when they are decided.
