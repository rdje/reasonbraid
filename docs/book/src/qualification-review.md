# Current qualification and corrective work

The startup source review is complete. It read the roadmap, all tracked code and
all book sources before changes. Runner cleanup, process-creation races and test-side
database ownership now have runtime controls and fixes. Missing ownership refuses
before public tables are created; forged proof and replacement connections are
covered. Foreign-tenant grant and boundary revocation, and the repeated-revoke
epoch defect, are now reproduced and corrected under `.3.1`; 34 focused tests pass. `docs/tasks/SIGNOFF-REPAIR.md` owns the repairs and their verification.

Historical test results in this manual describe the assertions exercised at those
commits. They do not establish current production qualification. Internet exposure
remains gated by G6/G7; binding governance and quality-lift claims retain their
existing restrictions. Current progress is in `LIVE_STATUS.md`.

## Shared registry authority

The former adapter and region handlers accepted a tenant-admin grant from any tenant
and omitted enrollment-boundary validation in that check. An owned live reproduction
confirmed shared writes by two distinct tenant admins, including adapter and region
writes after tenant-boundary revocation. Those historical success tests did
not prove site isolation or administrative freeze after revocation.

The accepted repair requires explicit **site-operator grants**, issued through
protected deployment tooling. Tenant enrollment cannot issue them. Mutations must
check the grant's actual boundary and commit with an attributable audit record,
serialized with revocation. The separate [site-authority service](site-authority.md)
now implements that contract under `.3.2.1`; ten live controls and strict focused
lint pass. The `rb-site` operator CLI passed its live controls under `.3.2.2`,
including the documented operator privileges and bounded audit walk. HTTP routing
now uses the site service under `.3.2.3`; the affected fixtures use explicit site
grants. All eight HTTP controls pass, including invalid wire-input refusals and both
revocation orders. The selected security run passed 58 tests; the final corrected HTTP/registry run
passed 12. Both owned clusters were stopped and removed.

| Scenario | Required repaired behavior |
| --- | --- |
| A tenant administrator declares a region | Refuse: tenant administration grants no site mutation authority. |
| An explicitly authorized site operator declares a region | Apply only within the live grant and boundary; record the audit. |
| An operator grant or its boundary is revoked | Refuse subsequent writes, preserving previously committed history. |
| A mutation races revocation | Transaction order decides; a revocation committed first fences the mutation. |
| A tenant administrator inspects its frozen tenant | Preserve the approved tenant-scoped administrative read behavior. |

## Other findings under repair

These are source observations and test limitations except for the explicitly
measured revocation controls above. Each row has executable repair ownership rather than an inert issue list.

| Surface | Current limitation identified in source | Repair leaves |
| --- | --- | --- |
| Authority and administration | Foreign grant/boundary mutation and shared site-registry authority are corrected with matched controls. Core subject serialization is reproduced and corrected with direct/enclosing and live compatibility controls; bound evaluation now passes core/evaluator/live authority controls and strict lint, with command API confirmation pending. Effect auditing, caller/target paths and actual-parent grant selection remain. | `.3.1`–`.3.5` |
| Node recovery and budgets | Receipt identity, cursor retention, result durability, uncertain retry, settlement and concurrency guarantees need additional enforcement and proof. | `.4.1`–`.4.5` |
| Directory and recruitment | Candidate visibility and call/thread binding are incomplete; automatic creation reuses a fixed idempotency key. | `.5.1`–`.5.3` |
| MCP and A2A | MCP reads do not uniformly enforce target authority; listen dedup needs correction. A2A qualification currently demonstrates serialization rather than an independent transport peer. | `.6.1`–`.6.3` |
| Resource acquisition | Redirect/subresource isolation, credential forwarding, worker bounds and advertised pipeline support require corrective validation. | `.7.1`–`.7.3` |
| Evidence and evaluation | Metadata/author binding, freshness and retention need repair. Citation excerpt presence does not prove entailment; missing gate measurements must not pass. | `.7.4`, `.8.2` |
| Deliberation and governance | Repeated challenge resolution and supplied attribution need correction; policy authority, lifecycle and Git reconciliation need stronger binding. | `.8.1`, `.9.1`–`.9.3` |
| Adapters and console | Subprocess bounds, certification evidence and numeric timeline rendering have source-review findings. | `.10.1`–`.10.2`, `.11.1` |
| Verification and operations | The supported launcher and disposable runner now localize stores and validate test connections. Remaining direct-entrypoint storage, artifact cleanup, script checks and historical claim accuracy require correction. | `.11.2`–`.11.4`; completed prerequisite evidence in `.2.1`–`.2.2` |

Full records: `docs/tasks/artifacts/signoff_review/INDEX.md`. The corrective tree
must give every finding a reproducible fixed or refuted disposition before its
requalification leaf closes. The next roadmap features remain store-and-forward,
verified export/import and the independent G8 implementation exercise.
