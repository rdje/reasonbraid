---
answers:
  - Which identities must match during tenant authority evaluation?
  - Can a thread selector authorize tenant administration or thread listing?
  - Is authority valid at its exact expiration instant?
---
# Bound authority evaluation and target coverage

- Owner: `SIGNOFF-REPAIR.3.3.2`.
- Status: implemented; core 51 unit + 3 subject tests, six evaluator controls, 32 live authority/command API tests and strict core/server lint pass. All results are consumed and the owned cluster stopped/removed.

A boundary is a named parent in one tenant. Equal permission ceilings are not a
substitute for matching grant.boundary_id and tenant_id. The core subset check
now requires both, at grant creation and again during evaluation. The evaluator
compares the grant's subject to the evaluated principal, or to delegate_subject
for the delegated authority source. It does not infer authentication from the
audit actor handle; the caller still supplies the resolved identity context.

Authority has a nonempty half-open validity interval: the start is inclusive,
expiration exclusive. Status, current liveness and the whole grant's existing
ceiling checks all apply. The policy-digest input format is unchanged: it binds
selected identifiers, subject, action, target and decision, not the complete
contents of every grant and boundary field.

Creation, automatic creation and tenant administration target the tenant.
Inspection can target one thread or the tenant's full thread listing. Other
thread actions require a thread. A tenant target requires a tenant-wide selector;
a thread selector covers only its named threads within the grant's tenant.
Listing all thread metadata cannot be inferred from permission to inspect one.

The normal evaluator still requires a live boundary. The separate approved
frozen-own-tenant administrative read exception is preserved; its actual-parent,
selector and usable-grant checks are owned by `.3.3.3`. That leaf also owns database
loading and candidate selection, including caller/delegated authority sources.
The structural evaluator repair does not establish revocation serialization or
final-effect auditing, owned by `.3.3.4`, nor delegation consent/depth (`.3.4`).

Task evidence records the original core and server failures, corrected controls,
real database issuance/audit checks and adjacent command compatibility. No full
production qualification follows from this bounded repair.
