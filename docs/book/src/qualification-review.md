# Current qualification and corrective work

The startup source review is complete. It read the roadmap, all tracked code and
all book sources before changes. Runner cleanup and process-creation races now
have runtime controls and fixes; reproduction of product findings remains pending. `docs/tasks/SIGNOFF-REPAIR.md` owns the repairs and their verification.

Historical test results in this manual describe the assertions exercised at those
commits. They do not establish current production qualification. Internet exposure
remains gated by G6/G7; binding governance and quality-lift claims retain their
existing restrictions. Current progress is in `LIVE_STATUS.md`.

## Shared registry authority

The current adapter and region handlers accept a tenant-admin grant from any tenant
and omit enrollment-boundary validation in that check. Consequently, their current
success tests do not prove site isolation or administrative freeze after revocation.

The accepted repair requires explicit **site-operator grants**, issued through
protected deployment tooling. Tenant enrollment cannot issue them. Mutations must
check the grant's actual boundary and commit with an attributable audit record,
serialized with revocation. Implementation and command examples are pending
`SIGNOFF-REPAIR.3.2`; there is no new operator command to run yet.

| Scenario | Required repaired behavior |
| --- | --- |
| A tenant administrator declares a region | Refuse: tenant administration grants no site mutation authority. |
| An explicitly authorized site operator declares a region | Apply only within the live grant and boundary; record the audit. |
| An operator grant or its boundary is revoked | Refuse subsequent writes, preserving previously committed history. |
| A mutation races revocation | Transaction order decides; a revocation committed first fences the mutation. |
| A tenant administrator inspects its frozen tenant | Preserve the approved tenant-scoped administrative read behavior. |

## Other findings under repair

These are source observations and test limitations, not newly measured runtime
results. Each row has executable repair ownership rather than an inert issue list.

| Surface | Current limitation identified in source | Repair leaves |
| --- | --- | --- |
| Authority and administration | Some paths validate caller tenant separately from target ownership; grant selection and boundary binding need correction. | `.3.1`–`.3.5` |
| Node recovery and budgets | Receipt identity, cursor retention, result durability, uncertain retry, settlement and concurrency guarantees need additional enforcement and proof. | `.4.1`–`.4.5` |
| Directory and recruitment | Candidate visibility and call/thread binding are incomplete; automatic creation reuses a fixed idempotency key. | `.5.1`–`.5.3` |
| MCP and A2A | MCP reads do not uniformly enforce target authority; listen dedup needs correction. A2A qualification currently demonstrates serialization rather than an independent transport peer. | `.6.1`–`.6.3` |
| Resource acquisition | Redirect/subresource isolation, credential forwarding, worker bounds and advertised pipeline support require corrective validation. | `.7.1`–`.7.3` |
| Evidence and evaluation | Metadata/author binding, freshness and retention need repair. Citation excerpt presence does not prove entailment; missing gate measurements must not pass. | `.7.4`, `.8.2` |
| Deliberation and governance | Repeated challenge resolution and supplied attribution need correction; policy authority, lifecycle and Git reconciliation need stronger binding. | `.8.1`, `.9.1`–`.9.3` |
| Adapters and console | Subprocess bounds, certification evidence and numeric timeline rendering have source-review findings. | `.10.1`–`.10.2`, `.11.1` |
| Verification and operations | Disposable databases, project-local storage, cleanup, script checks and historical claim accuracy require correction. | `.2.1`–`.2.2`, `.11.2`–`.11.4` |

Full records: `docs/tasks/artifacts/signoff_review/INDEX.md`. The corrective tree
must give every finding a reproducible fixed or refuted disposition before its
requalification leaf closes. The next roadmap features remain store-and-forward,
verified export/import and the independent G8 implementation exercise.
