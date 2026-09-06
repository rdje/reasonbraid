# Governance charter — draft

**Status: draft — not normative.** This is the Phase 0 governance-charter
draft (`ROADMAP.md` §4, §20.2 deliverable; backlog item 2). It is **not**
binding: it names a bootstrap root authority only so the authority model has an
explicit, reviewable starting point, and it records the authority structure the
code will eventually enforce. Adoption as a binding charter is itself a governed
decision under §4.1 — it does not happen here.

## Bootstrap root authority

The bootstrap root authority is a **named human**: **Richard DJE**, the
accountable owner for architecture decisions and release/security gate records
(`docs/decisions/2026-09-06_accountable-owners.md`).

- Root actions are conspicuous, rare, and fully audited (§4.1).
- Hardware-backed credentials are used where practical; the protocol does not
  require a particular vendor (§16.2).
- Bootstrap uses this human root; the protocol does not create a machine root
  that could act without a named human authorizing it.

## Owner and operators

| Role | Holder | Source |
| --- | --- | --- |
| Accountable architecture decision owner | Richard DJE | `2026-09-06_accountable-owners.md` |
| Release / security gate record owner | Richard DJE | `2026-09-06_accountable-owners.md` |
| Operators (deployment, recovery) | to be named as the project gains hosts | §4.1 |

One person fills both accountable roles while the repository is private and
pre-clearance; a separation-of-duties split requires a new decision record, not
an edit of this one.

## Authority sources

Authority flows downward and never upward (`ROADMAP.md` §4):

1. **Bootstrap root** — the named human root, source of first grants.
2. **GovernanceCharter** — a tenant's versioned framework (principal/target
   types, domains, risk classes, grant issuers, decision rules, approval
   thresholds, precedence, review intervals).
3. **EnrollmentAuthorityBoundary** — the root/parent-granted ceiling, shown to
   and acknowledged by the enrolled target (§4.4).
4. **AuthorityGrant** — a scoped grant over actions/targets/domains, always a
   subset of the applicable boundary.

Precedence chooses among *authorized* rules; it never creates authority (§4.2).
"Subscribed" is not used as a euphemism for mandatory control (§4.4).

## Target classes (`ROADMAP.md` §4.3)

Policy and decisions may target: tenant/organization; organizational unit or
team; project or repository; branch, path, package, or component; host, node, or
runtime environment; agent role, harness class, or tool capability; service/API;
operational or administrative process; and named human role.

## Scoped grants (`ROADMAP.md` §4.2)

An `AuthorityGrant` carries: grant id, tenant id, issuer, subject, `actions[]`,
target selector, policy domains, risk ceiling, decision-rule constraints,
spend/autonomy limits, `delegable`, validity interval, conditions, signature,
and status. Authorization requires **all** dimensions to match.

## Consent, mandate, and veto (`ROADMAP.md` §4.4)

- **Advisory/project-owned domain** — the target subscribes, pins a version, and
  may reject or unsubscribe.
- **Organization-mandated domain** — an authorized policy may bind enrolled
  targets in its declared scope; targets acknowledge, report inability, or
  request a waiver; they do not silently veto a legitimate mandate.
- **Delegated domain** — the target grants a central body limited, expiring,
  revocable authority.
- **Unmanaged target** — ReasonBraid may recommend but cannot bind or deploy.

Every decision exposes whether it is advisory, offered, mandated, accepted,
waived, suspended, or noncompliant.

## Decision rules and veto (`ROADMAP.md` §4.5)

The governance engine deterministically evaluates: eligible electorate snapshot;
identity and authority at action time; quorum and denominator; abstention,
recusal, absence, replacement, and timeout; veto ownership and scope; the
proposal revision digest voted upon; separation-of-duties; affected targets and
domains; charter and grant versions. A veto is owned and scoped — it is not a
generic "no".

## Emergency powers (`ROADMAP.md` §4.7)

Emergency suspension is fast, scoped, expiring, and conspicuous, and
auto-expires/escalates. It is distinct from permanent retraction. Typical
authority: an incident authority scoped to the target/domain.

## Appeals

Waiver, suspension, exit, and appeal rules are part of the
`EnrollmentAuthorityBoundary` (§4.4) — a target can request a waiver or appeal a
mandate it cannot satisfy, and the boundary records the exit/appeal path.

## Correction (`ROADMAP.md` §4.7)

| Operation | Effect | Typical authority |
| --- | --- | --- |
| Emergency suspension | temporarily stops effect; auto-expires/escalates | incident authority scoped to target/domain |
| Deployment rollback | restores a previously approved deployed version | operational rollback authority |
| Permanent retraction | marks canonical policy invalid from an effective point | charter-defined revocation authority |
| Supersession | adopts a replacement and links history | normal adoption authority |
| Waiver | time-bounded exception for named targets | waiver authority for the policy/risk class |
| Historical correction | adds metadata without deleting the record | records/governance authority |

Reversal is technically fast and operationally safe; that does not imply
universally lower authority (§4.7).

## Authority graph

```text
Bootstrap human root (Richard DJE)
  └─ Tenant
       ├─ GovernanceCharter (framework)
       └─ EnrollmentAuthorityBoundary (ceiling, acknowledged by target)
            └─ AuthorityGrant (scoped, ⊆ boundary)
                 └─ Principal / AgentRole / HumanPrincipal
                      └─ action on a TargetResource / policy domain
```

The graph is a ceiling, not a ladder: a grant cannot exceed its boundary, a
boundary cannot exceed the charter, and no amount of conversational agreement
moves authority upward (§4.5, §16.1).
