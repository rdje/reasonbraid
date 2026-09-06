# Requirement catalogue — G0 boundaries

**Status: draft — not normative.** Assigns stable IDs to the requirements of the
five G0 boundaries — identity, authority, thread, delivery, budget
(`ROADMAP.md` §20.2). Backlog item 4: "assign stable IDs and trace them to
phases, tests, risks, and gates." Later phases add `RES-*` (Phase 4), `POL-*`
(Phase 6), `SEC-*` (Phase 7); the prefixes are declared now so nothing renumbers.

ID form: `FAMILY-NNN` (three digits, zero-padded). A requirement is one
imperative sentence; tests and gate records cite the ID, never a paraphrase.
Schema validation does not replace semantic validation (§19.1).

## ID — identity, enrollment, presence (`ROADMAP.md` §3.1, §8.1)

| ID | Requirement | Trace |
| --- | --- | --- |
| `ID-001` | Enroll tenants, humans, hosts, nodes, agent roles, harnesses, services, and target resources as distinct principals or resources. | §3.1; gate G0 |
| `ID-002` | Keep a durable agent role, a model/harness incarnation, and a run/session as separate identities that cannot be confused in types or wire records. | §8.1; gate G0 |
| `ID-003` | Issue or bind cryptographic workload identities and rotate/revoke them; reimaging a host or changing a model must not silently inherit historical identity. | §3.1, §16.2; gate G0 |
| `ID-004` | Record capability, interest, subscription, visibility, authority, cost class, availability, and wake policy per role. | §3.1, §10.1; gate G0 |
| `ID-005` | Maintain lease-based presence that treats offline as a *presence state*, never as nonexistence. | §3.1, §10.2; gate G0 |
| `ID-006` | Use newtypes for every ID — globally unique, non-semantic, never interchangeable with plain UUID strings. | §8.3; gate G0 |

## AUTH — authority, authorization, governance (`ROADMAP.md` §4, §16.3–§16.4)

| ID | Requirement | Trace |
| --- | --- | --- |
| `AUTH-001` | Enforce deny-by-default authorization over typed actions and resources at every boundary (API, command handler, node, resolver, publisher, deployment, admin). | §16.4; gate G0 |
| `AUTH-002` | Represent a scoped `AuthorityGrant` whose every dimension (actions, target selector, domains, risk ceiling, spend/autonomy limits, expiry) must match before authorization. | §4.2; gate G0 |
| `AUTH-003` | Bound every charter provision and grant by the enrollment ceiling: a tenant's charter, grant, vote, or delegation cannot widen its `EnrollmentAuthorityBoundary`. | §4.4; gate G0 |
| `AUTH-004` | Record an authorization decision with policy digest, subject, actor, action, resource, context facts, decision, reason codes, and obligations. | §16.4; gate G0 |
| `AUTH-005` | Attenuate delegation: a delegate cannot widen a grant, duration, tenant, target set, cost ceiling, or approval power; model output never becomes authority. | §16.3; gate G0 |
| `AUTH-006` | Record actor, subject (if delegated), grant/boundary reference, decision, and policy digest/version on every command. | §4.4, §16.4; gate G0 |

## THREAD — conversation, lifecycle, deliberation (`ROADMAP.md` §3.2, §3.4, §8.4)

| ID | Requirement | Trace |
| --- | --- | --- |
| `THREAD-001` | Allow any principal with `thread:create` for a scope to create a thread carrying subject, context, desired outcome, mode, scope, budget, decision rule, and audience. | §3.2; gate G0 |
| `THREAD-002` | Make thread creation durable and idempotent, returning a subscription handle and an initial estimate range. | §3.2; gate G0 |
| `THREAD-003` | Enforce the thread lifecycle states with actor, authorization, and precondition per transition (see `lifecycle.md`). | §8.4; gate G0 |
| `THREAD-004` | Carry typed messages/contributions; a message is content that submits a command — unknown extension kinds remain preservable and cannot mutate state. | §8.5; gate G0 |
| `THREAD-005` | Produce deterministic terminal outcomes; never rewrite disagreement as consensus; preserve dissent and uncertainty. | §3.4, §13.4; gate G0 |

## DELIV — delivery, notification, idempotency, ordering (`ROADMAP.md` §3.3, §6.4, §9, §10.6)

| ID | Requirement | Trace |
| --- | --- | --- |
| `DELIV-001` | Accept client commands as at-least-once submissions with idempotent processing within a documented retention window. | §6.4, §9.2; gate G0 |
| `DELIV-002` | Scope idempotency to (tenant, principal, operation, key); same key + request hash returns the original result, a different hash is a conflict. | §9.2; gate G0 |
| `DELIV-003` | Append events per thread in a unique monotonic sequence; reject client-supplied actor, tenant, sequence, and timestamp. | §6.4, §9.1; gate G0 |
| `DELIV-004` | Maintain a durable inbox with explicit delivery states and at-least-once semantics; consumers deduplicate by delivery/event ID. | §10.6; gate G0 |
| `DELIV-005` | Expose no global exactly-once guarantee; an indeterminate provider call remains visible as `outcome_unknown`, never silently retried. | §6.4, §11.3; gate G0 |

## BUDGET — budgets, reservation, settlement (`ROADMAP.md` §3.8, §14)

| ID | Requirement | Trace |
| --- | --- | --- |
| `BUDGET-001` | Represent a multi-dimensional `ResourceBudget` (money, tokens, calls, time, compute, concurrency, bytes, human minutes, recursion); record unknown dimensions as unknown, not zero. | §14.1; gate G0 |
| `BUDGET-002` | Follow estimate → reservation → settlement: a reservation atomically holds the maximum authorized next step or batch. | §14.3; gate G0 |
| `BUDGET-003` | Start no provider dispatch without an applicable reservation and authorization; reservation plus settled/estimated charges never silently exceeds the hard ceiling. | §14.6; gate G0 |
| `BUDGET-004` | Keep the ledger append-only in logical accounting terms; corrections are compensating entries, never rewritten history. | §14.3; gate G0 |
| `BUDGET-005` | Authorize only the charter's named principals to enlarge or transfer a budget; child-thread budgets are subsets of (or explicitly separate from) parent authority. | §14.6; gate G0 |

## Traceability legend

- **Gate G0** — the Phase 0 exit gate for these five boundaries (`ROADMAP.md`
  §19.6, §20.2). G0 blocks implementation of an affected boundary until the
  contract holds.
- **Risk** — each requirement has a stop/reframe risk registered in
  `docs/risks.md` (the live subset of `ROADMAP.md` §25): e.g. `ID`→R-NAME,
  `AUTH`→R-AMB/R-SECRET, `DELIV`→R-EXACT, `THREAD`→R-VALUE, `BUDGET`→R-EXACT.
- **Later phases** — `RES-*` (Phase 4), `POL-*` (Phase 6), `SEC-*` (Phase 7)
  are declared but out of G0 scope; their rows are added by those phases.
