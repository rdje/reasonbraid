# Node issuance and revocation gate on a GRANT, never on a kind of principal (`SIGNOFF-REPAIR.4.1.4`)

- Date: 2026-09-13 · Leaf: `SIGNOFF-REPAIR.4.1.4` · Decision record

## Context

`POST /v1/nodes/enroll-tokens` and `POST /v1/nodes/revoke` both documented
themselves as operations "an authorized **human**" performs. Neither ever
checked. Both resolve a principal that may be `GrantSubject::Human(hpr_…)` or
`GrantSubject::Role(rol_…)` and authorize it against `GrantAction::TenantAdmin`.

The question the leaf had to settle is not a typo: an agent role that can issue
enrollment tokens can **extend the node population**, and one that can revoke
can **remove nodes**. Whether that is the product or a defect is a governance
call, and the documentation asserted the opposite of the behaviour.

## Decision

**The behaviour is correct and the documentation was wrong.** Issuance and
revocation gate on the `tenant_admin` grant. A principal's kind — human or agent
role — is never an input to an authorization decision anywhere in this system.

Measured rather than argued, on four independent lines:

1. **The roadmap.** §16.2 specifies what an enrollment token is BOUND to and
   says nothing about who may issue one. §16.4 specifies authorization "over
   typed actions and resources", deny-by-default. §16.3 states outright that "a
   human, service, or agent may delegate a strict subset of its own authority".
2. **The codebase's own model.** Across the server's 117 `resolve_principal`
   call sites, authorization never depends on the principal's kind. Every
   production branch on `GrantSubject::Human` — `reader_tenant`, the unreachable
   invite arm, the bootstrap insert, and `reasonbraid-mcp`'s `principal_is_human`
   — selects which identity TABLE to read. "Human-only" is not a concept this
   authority model has.
3. **The gate itself.** Issuance and revocation both require
   `GrantAction::TenantAdmin`, the same gate every other tenant-administrative
   route uses.
4. ⭐ **An oracle nobody built for this question.** Adding the human-kind check
   the old sentence implied fails three controls, and two of them predate the
   leaf: `a_denied_issuance_records_no_effect_and_no_token` and
   `a_token_does_not_outlive_the_authority_that_issued_it` already drive an agent
   role at the issuance route and require it to be adjudicated by the GRANT. The
   first fails for the reason that matters most — a kind check refuses with `401`
   **before** the authorization that writes the denial record, destroying the
   audit evidence the denial path exists to produce.

## What this means for an operator

⚠️ **Granting `tenant_admin` to an agent lets that agent extend the node
population and revoke nodes.** A deployment that does not want that must
**withhold the grant**. The route will not refuse agents, and it should not be
made to: narrowing who may issue is a change to the GRANT MODEL — a narrower
action than `tenant_admin`, or a constraint on the grant — not a kind check
bolted onto one endpoint, which would be invisible to the policy layer and
unenforceable at the other boundaries §16.4 requires enforcement at.

## answers:

- **Who may issue a node enrollment token or revoke a node**: whoever holds
  `tenant_admin` in that tenant — a human principal or an agent role, with no
  distinction anywhere on the path.
- **Why the kind is not checked**: authorization in this system is over typed
  actions and resources (§16.4). A kind check is authority expressed outside the
  grant model, so it cannot be reasoned about, versioned, delegated, revoked or
  enforced at the other boundaries.
- **How to restrict it anyway**: withhold `tenant_admin`, or introduce a
  narrower action — both are grant-model changes, which is where this decision
  says the lever belongs.
- **What holds this in place**: `an_agent_role_holding_tenant_admin_may_issue_an_enrollment_token`
  asserts both directions (a role WITH the grant issues and is recorded in the
  ledger as the grant subject; a role WITHOUT it is refused `403` and writes no
  token). It is a tripwire on the governance decision: a future kind check
  cannot be added silently.
