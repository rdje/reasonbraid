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
| Authority and administration | Foreign grant/boundary mutation and shared site-registry authority are corrected with matched controls. Core subject serialization is reproduced and corrected with direct/enclosing and live compatibility controls; bound evaluation passes core/evaluator controls, 32 live authority/command API tests and strict lint. Actual-parent command selection passes 37 live tests, six evaluator controls and strict lint; frozen-read eligibility passes 40 live tests, ten pure controls and strict lint. Provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all results consumed and the owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint, including audit/query failure recovery; all results and shutdown consumed. Exact scoped receipt lookup passes 18 live authority tests and 30 HTTP tests with strict lint; all results/shutdown consumed. The `.3.3.4.1` census maps guard/effect integration children. `.3.3.4.2` qualifies the primitive guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass. Cancelled-BEGIN pooling and a stale embedded-migration executable are reproduced and repaired; all results/shutdown consumed and owned clusters/probes removed. Seven checker controls also correct nested-evidence owner selection, with broader doctrine accuracy still open. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. Bounded HTTP waits and restart qualification follow; ordinary no-key behavior remains intentionally distinct. Effect auditing and other caller/target paths remain. | `.3.1`–`.3.5` |
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


### Proposed semantic introspection

The director has proposed a clean semantic API, usable through MCP, for agents to
trace operations, explain authorization, inspect dependencies and check invariants.
`SIGNOFF-REPAIR.6.4` owns assessment of existing evidence and missing capabilities;
`docs/tasks/artifacts/signoff_review/semantic-introspection-proposal.md` preserves
the proposal. This is not an implemented diagnostic or autonomous repair surface.
The suggested progression is bounded read-only diagnosis, qualified isolated
replay, then separately permissioned repairs with preconditions and verification.
A diagnostic response must distinguish facts, hypotheses and missing evidence,
including committed, rolled-back and unknown operation outcomes. The current
bootstrap recovery work continues without a pivot.
