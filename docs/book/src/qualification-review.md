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
| Authority and administration | Foreign grant/boundary mutation and shared site-registry authority are corrected with matched controls. Core subject serialization is reproduced and corrected with direct/enclosing and live compatibility controls; bound evaluation passes core/evaluator controls, 32 live authority/command API tests and strict lint. Actual-parent command selection passes 37 live tests, six evaluator controls and strict lint; frozen-read eligibility passes 40 live tests, ten pure controls and strict lint. Provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all results consumed and the owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint, including audit/query failure recovery; all results and shutdown consumed. Exact scoped receipt lookup passes 18 live authority tests and 30 HTTP tests with strict lint; all results/shutdown consumed. The `.3.3.4.1` census maps guard/effect integration children. `.3.3.4.2` qualifies the primitive guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass. Cancelled-BEGIN pooling and a stale embedded-migration executable are reproduced and repaired; all results/shutdown consumed and owned clusters/probes removed. Seven checker controls also correct nested-evidence owner selection, with broader doctrine accuracy still open. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. Bounded HTTP waits and restart qualification follow; ordinary no-key behavior remains intentionally distinct. The final administrative effect representation is defined under `.3.3.4.7.1`: a closed set of fourteen operations, each carrying a caller-supplied target that exists at refusal as well as at success, and a closed `applied`/`no_op`/`refused` outcome whose refusal names a §9.8 reason code. The fourteen were measured from the 27 guarded-admission call sites rather than chosen, and the four other tenant_admin mutations are deliberately outside it because other leaves own their gates. 10 representation controls, 68 core tests and strict core lint pass; the object-only control is falsified against a permissive decoder. NOTHING writes an effect record yet: the schema and writer are `.7.2` and the first route is `.8`. Effect auditing and other caller/target paths remain. | `.3.1`–`.3.5` |
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

The resumed **8d1504d** checkpoint passes eight gates but workspace testing stops
at one state-writer lock failure; PostgreSQL and the demo never start. All sixteen
browser controls pass. Six controlled actual-CLI scenarios then prove that an
inherited child descriptor can retain exclusion after parent success, error or
cancellation. Exact attribution of the original deleted fixture is unavailable.
All 341 source hashes match and fifty recorded groups are absent. That diagnostic commit left production unchanged; the explicit-release repair
and permanent controls follow below, before checkpoint resumption/public push. Broader inherited-descriptor process-loss
qualification remains owned separately. See
`docs/tasks/artifacts/signoff_review/state-writer-lock-lifetime.md`.

Inherited state-lock release is now repaired under .11.4.3.1.2.11.2. A guard
explicitly unlocks before closing its File, including post-acquisition failures;
two successful test probes do likewise. The permanent five-path child regression
fails on unchanged production and passes after repair. All 32 selected CLI tests,
the final twelve-writer rerun, six raw-fork actual-API scenarios and strict CLI
lint/format pass. Original assertions and 339 other non-Markdown sources remain
unchanged. The original holder remains uncaptured; inherited references after
abrupt owner death retain separate restart ownership. Resume the full checkpoint
from this committed repair before public push/remote CI. Evidence:
`docs/tasks/artifacts/signoff_review/state-writer-lock-release.md`.


The **ec8df08** checkpoint next stops at two PDF tests: zero chunks instead of
one, and a JavaScript-bearing fixture unexpectedly accepted. Eight other gates
pass; PostgreSQL/demo never start. An unchanged extractor rerun receives another
test's outer.zip. The exact old helper then reproduces three native-clock filename
collisions among 32 simultaneous writers. Original checkpoint paths are missing;
passing isolated/instrumented reruns do not erase the failures. Exclusive
repository-local unit/stdio inputs are repaired under .11.4.3.1.2.12, with all
original parser assertions preserved. All twelve selected tests, strict lint/format
and three independent locality cases pass. The analogous server input risk has concrete
next repair owner .7.3.3 before checkpoint resumption. See
`docs/tasks/artifacts/signoff_review/extraction-fixture-ownership.md`.


The subsequent production-boundary diagnosis under .7.3.3.1 uses the exact
server input-name/write span and unchanged extraction module/worker. With 32
simultaneous input owners, six paths collide and eight worker responses describe
another owner's bytes. Two positive controls return their own bytes correctly.
This reproduces input interference before the worker; it does not execute the
HTTP handler or prove an incorrect database write. The API currently lacks a
source/response parent-digest equality check. Production remains unrepaired at
this diagnosis: .7.3.3.2 establishes confirmed direct-worker completion, then
.7.3.3.3 adds exclusive same-volume inputs, exact digest binding and checked
cleanup/retention. For example, two acquired documents must never share a worker
input, and a response for different bytes must be refused before persistence.
Unconfirmed reader completion must retain its input. Pipe/output bounds,
descendant containment and aggregate retained-storage limits remain .7.3.4.
See `docs/tasks/artifacts/signoff_review/extraction-input-boundary.md`.

Direct-worker completion .7.3.3.2 is now repaired and qualified. The permanent
controls first reproduced the defect on the unchanged spawner: an early request
failure returned while its worker was still in the process table. Every exit
path now passes through one bounded stop and reap, and every return reports
never-started, consumed or explicitly unconfirmed completion without claiming an
unobserved termination. Sixteen process/evidence controls, six spawner controls
(four synthetic injections), 81 server library tests, twelve adjacent extractor
tests and strict lint pass. Exclusive same-volume inputs and digest-bound
cleanup remain .7.3.3.3; pipes, descendants and retained storage remain .7.3.4.
See `docs/tasks/artifacts/signoff_review/extraction-worker-completion.md`.

Exclusive input ownership .7.3.3.3.1 now provides the store the API will use:
one private 0600 file per request under a runtime-discovered repository root,
occupied candidates skipped whole, and removal gated on both a finished reader
and an unchanged file identity. Eleven controls pass, including 32 simultaneous
creators holding 32 distinct documents and the real worker describing an owned
input by its own digest. One control was itself racy and is root-caused by its
own retained input; both now serialize worker selection. No production caller
uses the owner yet at that child. `.7.3.3.3.2` then wires the R2 API onto it:
the handler's extraction leg is one bound call, and a response whose parent
digest is not the supplied bytes' digest is refused as
`extraction_source_mismatch` before any snapshot or derivation is written. Seven
integration controls, the live `profiles` suite (31 passed) and strict lint pass.
The production R2 input boundary `.7.3.3` is complete. One gap is explicit: no
live test drives a successful acquisition through to a snapshot and derivation,
because the live R2 test refuses at the loopback gate; `.7.3.3.4` owns that join.
See `docs/tasks/artifacts/signoff_review/extraction-owned-input.md`.

That join is closed under `.7.3.3.4.1`. `ApiState::with_acquisition` lets a
deployment supply the R0 fetcher its acquisition legs use, while `new`,
`with_gate` and both existing routers keep building the shipped https-only
public-destination policy. A live control resolves one reference through three
deployments differing only in that fetcher — the shipped state refuses at the
scheme, the shipped destination policy refuses the `loopback` class by name, a
loopback-admitting policy acquires — then reads the evidence back and asserts
the snapshot's digest and byte length and the derivation rows' content and
digests against the bytes the origin actually served. The profiles suite passes
32 of 32 live, alongside 97 library tests, 23 extraction controls and strict
lint; the owned cluster was removed. Three reverted injections prove the control
goes red on a wrong byte, a discarded fetcher and a wrong derivation.

The mismatch refusal has its own live control under `.7.3.3.4.2`. A dishonest
worker injected through the `R2_WORKER_BIN` override returns a well-formed reply
describing bytes the request never supplied; the handler refuses it
`extraction_source_mismatch` and the control asserts an ABSENCE — zero snapshots
for that reference, zero derivations joined to it, and the whole store's counts
unchanged across the request. Writing a snapshot before reporting the same
refusal leaves the refusal's own name intact and still fails that control, which
is why the counts are there and the error kind alone is not enough. The suite
passes 33 of 33 live with its cluster removed, and production source is
byte-identical after both injections were reverted.

That work also measured a finding: the R2 pack advertises five media types
its acquisition leg refuses. The R0 sniff accepts a declared content type only
when it is `text/html`, `application/xhtml+xml` or `text/*`, so a feed served
under its own `application/atom+xml` is refused `media_type_refused` before the
worker is reached, while the identical bytes served as `text/xml` succeed. A
caller is therefore ranked onto a resolver that cannot acquire its document and
receives an acquisition refusal rather than the explicit `resource_unresolvable_now`
the resolution contract reserves for "no eligible resolver". The per-format
census and the repair decision are owned by `SIGNOFF-REPAIR.7.3.3.5`. See
`docs/tasks/artifacts/signoff_review/r2-acquisition-join.md`.

The census is complete under `.7.3.3.5.1`, with no production change, and it
disciplined the finding. A declared type is accepted only from `text/html`,
`application/xhtml+xml` and `text/*`; untyped, the verdict depends on whether
the bytes are printable, not on the format, and ZIP and tar cannot pass that
branch by construction. The leg's rule is therefore not format-shaped at all, so
no subset of the advertisement satisfies it and narrowing the registry row is
rejected as unable to express the truth. The accepted repair is that the
acquisition leg admits the ranked resolver's own advertised media types, with
the destination policy, scheme list and every ceiling explicitly outside the
change. Decision: `docs/decisions/2026-09-12_r2-acquisition-accept-set.md`.

`.7.3.3.5.2` ships it. The advertised types are read from the ranked pack's own
registry row per resolution; `fetch_admitting` carries them and has exactly one
caller, so R0-ranked acquisitions keep the shipped accept set. A missing or
malformed row yields an empty set. The live control now acquires the same feed
served under its own `application/atom+xml` type and records that type on the
snapshot, while a type the pack does not advertise stays refused — a bound
proved live by admitting one such type and watching the negative control fail.
The suite passes 33 of 33 with its cluster removed; the R0 and R1 resolver
controls pass unchanged in the same run.

The full pre-push checkpoint now passes on source `7233122` — the first complete
run recorded. All eight commands return 0, 40 of 40 database suites run with 291
tests and no failures, the two-host demonstration reports all acceptance checks
passed, the browser controls render against the pinned runtime, and both
supply-chain scanners pass as real gates. This is a local qualification only:
remote CI has never run, G6/G7 Internet exposure, name clearance and the license
decision remain open, and the historical phase closures remain under corrective
review. See `docs/tasks/artifacts/signoff_review/checkpoint-7233122.md`.
