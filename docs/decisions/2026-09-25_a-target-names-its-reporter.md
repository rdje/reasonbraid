---
answers:
  - Who may file a deployment receipt for a target?
  - Why is the reporter a principal the target names, rather than a grant or the target's owner?
  - What happens to a target registered before targets named a reporter?
---
# A target names its reporter, and only that principal files its receipts

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.9.3.3.2`
- **Date:** 2026-09-25
- **Work unit:** `REASONBRAID-REPAIR-0526`
- **Cites:** `docs/adr/021-target-deployment.md` (*the receipt attests what the
  target OBSERVED, never what the operator hoped*); ROADMAP §15.9 step 4
  (*collect repository/runtime receipts and policy digest attestations*), §15.10
  (*nodes attest loaded versions*); `REASONBRAID-DOC-0071` (a target is site-wide;
  an assignment is owned by its publication's tenant); `docs/book/src/policy-lifecycle.md`
  (*Deployment and receipts*)

## The fact / decision

1. **A target names one reporter** when it is registered: `reporter`, a principal
   id (`hpr_…` for a human, `rol_…` for an agent role), required, and enrolled at
   registration. It is stored in canonical form (`migrations/0114`) and shown by
   `GET /v1/deployment-targets`.
2. **A receipt is accepted only from the target's reporter**, in addition to the
   existing tenant binding (the assignment's publication belongs to the caller's
   tenant). The assignment is looked up first, so a caller of another tenant gets
   the same *unknown assignment* answer as before; a caller of the right tenant who
   is not the reporter is refused by name.
3. **A target registered before `migrations/0114` has no reporter and takes no
   receipt.** The refusal says so. Nothing is inferred for it.
4. **Naming or replacing a reporter on an existing target is deferred**
   (`SIGNOFF-REPAIR.9.3.3.2.1`), with the trigger: a target must take receipts
   from a principal other than the one it was registered with (a node or role
   replaced), or a target registered before `migrations/0114` must take a receipt.
5. **The reporter need not be in the registrant's tenant** (a target is site-wide,
   DOC-0071). Its receipts cover only the assignments whose publication is in the
   reporter's own tenant, because the tenant binding still applies.

6. **Read reach** (`scripts/census_registry_read_reach.py`, re-adjudicated here):
   `deployment_targets` gains one reader, `record_receipt`, and its statement is
   tenant-predicated (the join through the assignment's publication), so it is
   recorded without `!`. The reporter it reads is compared, never returned: the
   refusal names the CALLER, not the reporter. `GET /v1/deployment-targets` shows
   `reporter` to any enrolled principal, as it already shows `owning_authority`,
   whose grant id embeds the registrant's principal id: a target is site-wide by
   design (DOC-0029, `2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`).
   A principal id is an identifier, not a credential, outside the dev profile,
   whose trusted principal header is the known G6 gate.

## Why

- **The owner is the wrong attester.** The principal that registers a target
  holds its owning authority; it is the operator. ADR-021 separates what the
  operator *hoped* (the desired pair) from what the target *observed* (the
  receipt). Letting the registrant file receipts would let the hope attest itself.
- **A grant cannot name a target.** A grant action such as
  `deployment_receipt_record`, held and covering, is tenant-wide: no target
  selector names a deployment target (`.9.3.4` owns the narrowing), so any holder
  would report for every target. The parent leaf's acceptance is *a receipt from a
  node the target does not name is refused*, and only the target can name one.
- **The reporter is a principal, not a node certificate, for now.** For
  `node_policy` targets the end state in §15.10 is the node attesting over its own
  channel, but no node-channel verb for receipts exists. An agent role is the
  identity the control API already authenticates for the software acting on a
  target. Its strength is the deployment profile's (the dev profile trusts the
  principal header; G6 gates Internet exposure).
- **Fail closed on the old rows.** Guessing a reporter for a pre-0114 target would
  hand the observed half to a principal nobody named, which is the defect.

## How to apply

- Register a target with the principal that runs where the policy is applied as
  its `reporter`, not with the operator.
- A test that files a receipt posts it as the target's reporter.
