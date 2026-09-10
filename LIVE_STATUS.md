# LIVE_STATUS.md — authoritative live progress tracker

Rows use only **Done · Mostly Done · In Progress · Not Started**. This is a current
snapshot. Historical implementation and verification records live in the phase
task-trees and git; the pre-review snapshot is `9c2d2ba:LIVE_STATUS.md`.

## Qualification correction

The full startup source read found open invariant failures and coverage gaps.
Historical phase closure does not establish current production qualification.
`docs/tasks/SIGNOFF-REPAIR.md` owns reproduction, fixes and requalification;
`docs/tasks/artifacts/signoff_review/INDEX.md` preserves the source evidence.
The runner cleanup and spawn/signal defects now have runtime controls and fixes;
foreign-target revocation and repeated-revoke epoch defects are reproduced; the
`.3.1` correction passed 34 focused tests and strict lint. Shared writes by frozen
tenant admins are now runtime-confirmed. The new site service passes ten live
controls and strict focused lint; the protected CLI passes its live controls.
All seven HTTP registry operations now use site authority and pass eight focused
HTTP controls plus adjacent checks; other repairs remain open.

| Area | Status | Current evidence and remaining work |
| --- | --- | --- |
| Roadmap and task-tree conversion | Done | `PROGRAM` maps all phases, gates, backlog items, ADRs and demonstrations; `RB-SEED.2` holds the original census. |
| Claim-verification policy | Done | Local policy matches the director-authorized pgen donor at the startup comparison; subsequent claims still need all three verification legs. |
| Discipline and continuity | In Progress | Repair ownership is recorded; `.2` and `.11` own local storage, disposable verification, doctrine defects and document containment. `.11.4.1` rotates the recent changelog through verified Git history; broader containment remains open. |
| Phase 0 — contracts and experiments | Mostly Done | Historical G0 package and owner signoff retained in `PHASE-0`; affected authority/budget/adapter assertions require corrective evidence. |
| Phase 1 — LAN vertical slice | Mostly Done | Historical demonstration retained in `PHASE-1`; current authority, console and demo-script findings remain open. |
| Phase 2 — identity, delivery and recovery | Mostly Done | Historical machinery retained in `PHASE-2`; revocation, fencing, budget and recovery repairs are `.3`–`.4`. |
| Phase 3 — directory and recruitment | Mostly Done | Historical machinery retained in `PHASE-3`; tenant visibility, recruitment and automatic initiation repairs are `.5`. |
| Phase 4 — resources and evidence | Mostly Done | Historical G4 record retained in `PHASE-4`; acquisition isolation, evidence integrity and retention repairs are `.7`. |
| Phase 5 — deliberation and evaluation | Mostly Done | Historical G5 subtraction gate withdrew the quality-lift claim; workflow and evaluation repairs are `.8`. |
| Phase 6 — governance | Mostly Done | Historical G3 machinery exit retained; binding use remains gated; policy/publication/deployment repairs are `.9`. |
| Phase 7 — Internet qualification | Mostly Done | Hardening machinery exists; G6/G7 Internet exposure remains NOT MET. Local repairs and external threat-model, injection and penetration-test evidence remain required. |
| Phase 8 — federation and interoperability | In Progress | Through regional routing historically recorded; `.5.3` store-and-forward, `.5.4` exit export/import and `.6` G8 remain. Shared authority and protocol gaps are prerequisite repairs. |
| Phase 9 — stable release | Not Started | G9 requires sustained operational evidence and the outstanding release decisions. |
| Corrective review | In Progress | `.1`, `.2.1`, `.2.2`, `.3.1` complete; `.3.2.1` site service passes ten live controls and strict lint. `.3.2.2` operator CLI passes 3 live controls plus adjacent service/ownership checks and strict lint; `.3.2.3` HTTP enforcement passes 8 focused controls; the selected 58-test security run and final 12-test registry run pass. `.3.3.1` core subject serialization passes 49 unit + 3 subject controls and 40 live compatibility tests; `.3.3.2` bound evaluation passes 51 core unit + 3 subject tests, 6 evaluator controls, 32 live authority/command API tests and strict lint; all results consumed and the owned cluster removed. `.3.3.3.1` actual-parent command selection passes 37 live tests, six evaluator controls and strict lint. `.3.3.3.2.1` frozen-read eligibility passes 40 live tests, ten pure evaluator controls and strict lint; all results consumed and the owned cluster removed. `.3.3.3.2.2.1` provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all confirmation/shutdown results consumed, owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint; all results and shutdown consumed. Exact scoped receipt readback passes 18 live authority tests and 30 HTTP tests plus strict lint, including final denied-record readback; all results/shutdown consumed. Actual-parent selection and frozen-read audit/readback `.3.3.3` are complete; The `.3.3.4.1` source census/contract is complete (42 named-call locations plus transitive/mutation coverage). `.3.3.4.2` qualifies the guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass, including cancelled-BEGIN replacement and the formerly stale authority executable. Seven acceptance-checker controls also pass, correcting nested-evidence owner selection while preserving real-tree box refusals. All results/shutdown consumed; owned clusters/probes removed. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. Bounded HTTP waits and restart qualification follow; ordinary no-key behavior remains intentionally distinct. |

The shared adapter/region registry design is now explicit site-operator authority.
The separate site service implements that design. HTTP handlers call that service under `.3.2.3`, with live cross-tenant/freeze,
actual-parent, audit/rollback, wire-input and revocation-race controls.

The scheduled pre-push checkpoint inventory is complete under .11.4.3.1.1. Workflow
locality/script coverage, publisher/browser fixture ownership and evidence-based
cleanup have concrete prerequisite owners .3–.6 before full execution .2. This
source/tool census changes no qualification category and claims no fresh full CI.
The shared CI environment prerequisite .11.4.3.1.3.1 now passes eight focused and
eighteen adjacent controls. Scanner setup .11.4.3.1.3.2 passes thirteen controls,
final affected checks, all eight archive identity/layout checks and native version
probes; the original version-format failure and corrected retry remain preserved.
Workflow wiring .11.4.3.1.3.3 now passes YAML/shell routing checks, five omission
controls, real synthetic Gitleaks redaction, fifty Python controls (including live
PostgreSQL ownership) and rendered book checks. Subsequent prerequisite and full
local/remote gate results are tracked separately; categories are unchanged.
Publisher fixture ownership .11.4.3.1.4 now passes five focused controls, independent
helper lifetimes, two concurrent executable runs, strict lint and rendered book
checks. Historical residue is preserved; all results are consumed. Browser and
compiler-artifact prerequisites remain before full execution; categories unchanged.

Browser test ownership .11.4.3.1.5.1 passes eight controls with real rendering,
strict lint and consumed process/source/residue checks. Native transient group
refusal is reproduced; bounded observation still requires actual absence and
preserves the original failed fixture. Production lifetime .5.2 now passes five
unit/thirteen integration controls, strict lint and native/source/book checks:
private runtime storage, owned cancellable launch and consumed process/task shutdown.
All twenty-one final groups are absent; failed startup evidence is retained and its
uncaptured wait mechanism remains .11.2. Parent transport/termination and aggregate
retention/container limits have concrete .7.3.1/.2 owners. Combined .5.3 now passes
fifteen integration controls, strict lint and native/source/book checks with unchanged
production bytes: real root relocation/refusal, gated overlap and exact listener
close receipts. Twenty-six final groups are absent and nine failed fixtures preserved.
Browser prerequisite .5 is complete. Compiler disposition .6 removes 645 obsolete
sessions under verified native locks; exact residue and preserved source/evidence
checks pass. The affected server build and final book checks pass; all results
are consumed and .6 is complete. Full local/remote execution .2 and push remain
pending. Qualification categories are unchanged.

Publication-precondition audit .11.4.3.1.2.1 established the public remote and
preserved the interrupted checkpoint. The director has resolved the question:
README's private instruction was wrong; the project is public and must remain
public. Correction .2.3 synchronizes README, ADR, security/companion guidance and
the book; no remote-setting change is needed. Format/cargo-deny passed; two
redacted history findings remain .2.2 and interrupted/unstarted gates remain
unqualified. Continue repairs and full checkpoint .2 before the authorized public
push. Qualification categories are unchanged. Policy:
docs/decisions/2026-09-09_public-repository-policy.md.

History-scan repair .11.4.3.1.2.2 classifies both original matches as predictable
metadata-only fixture literals. Two exact immutable fingerprints are qualified by
five native controls (2/1/1/0/2 findings), including detection of identical content
in a new commit. The actual pinned scanner passes with an empty report; eleven
control groups are independently absent and all results consumed. No file/rule
suppression or history rewrite. Full checkpoint .2 resumes next; qualification
categories remain unchanged.

The source-7e01097 full checkpoint passes format, strict lint, bins, fifty Python
controls, book/doctrines and both scanners, but workspace execution stops at two
browser timing witnesses. Repair .11.4.3.1.2.4 reproduces those assumptions and
qualifies explicit gated arrivals/overlap with two delayed controls and all sixteen
integration tests on a dedicated testing runtime. Native evidence identifies the
desktop browser's detached updater/crash-report stderr writers; that refusal stays
preserved. Production bytes are unchanged; strict focused lint and process/fixture
checks pass. Pinned local/CI runtime binding .2.5 precedes the next full checkpoint.
PostgreSQL/demo and remote CI are pending; qualification categories are unchanged.

Dedicated runtime prerequisite .11.4.3.1.2.5 is now complete. Local Make and CI pin
the same verified testing browser; four archive layouts and native setup/rendering
are qualified. All 67 Python controls and sixteen browser integration tests pass,
including a reproduced/repaired shared Python zombie-group shutdown race. Final
wiring/book/process/residue checks pass; failed evidence remains and production
Rust is unchanged by that prerequisite. Qualification categories remain unchanged.

The resumed source-b0cddfe checkpoint passes nine gates, including the pinned-browser
workspace run and 67 Python controls, then fails in identity_store fixture cleanup
after thirteen live PostgreSQL suites pass. Certificates left by node_work block
parent-node deletion; twenty-six later commands and the demo are unstarted. The
minimal test-only repair passes its new regression and all four identity tests on
fresh and node-work-populated databases; the production FK and deployment CA remain
intact. Expanded fixture review reproduces MCP-listener, CLI spend-breaker and
incarnation residue failures. Shared cleanup-plan check .11.4.3.1.2.7.1 passes
eight guard tests and strict lint. Fourteen node callers now use it and all cleanup
plans execute; real MCP→identity and breaker→CLI sequences pass. The affected
census has 135 passing assertions and two failures, both reproduced with original
fixtures: participant-removal authorization (.2.8) and a fixed retention-test date
(.2.9). The server target repair .2.8 now passes all six invitation tests, 22
authority and 33 command-API tests plus ten pure evaluator controls and strict
server lint. Refusals preserve domain state and historical denial replay; valid
tenant administrators can remove participants. CLI companion .2.10 now passes
all five real CLI tests: administrative removal requests tenant-wide scope and
ordinary delegated invitation retains its single-thread scope. Actual unchanged-
CLI and deliberately overbroad controls fail as expected. Retention fixture .2.9
now uses observed creation times and passes the focused test and all 31 profiles
tests, with strict boundaries, exact class effects and preserved audit/replay
state. Strict server lint and source/process/book checks pass. All six partial
fixtures under .2.7.3 now use explicit checked dependencies and pass both actual
node-work→consumer pairs and a consecutive consumer run, preserving deployment
CA rows and original assertions. All 25 original plans now use checked cleanup. Final five-plan adoption .2.7.4
passes its 22-test consumer sequence and all 169 distinct tests in the consecutive
affected collection, plus strict server/MCP/CLI lint and final verification.
Both successful databases are removed and prior failures preserved. Full
checkpoint resumption follows the state-lock prerequisite below; authorized public push and remote CI remain pending.
Preserve the stopped failed databases and startup diagnostics under
.11.2. Qualification categories remain unchanged. Evidence:
docs/tasks/artifacts/signoff_review/identity-fixture-cleanup.md and
docs/tasks/artifacts/signoff_review/fixture-cleanup-plan-check.md and
docs/tasks/artifacts/signoff_review/node-fixture-cleanup.md and
docs/tasks/artifacts/signoff_review/participant-removal-authority.md and
docs/tasks/artifacts/signoff_review/cli-removal-delegation.md and
docs/tasks/artifacts/signoff_review/retention-fixture-clock.md and
docs/tasks/artifacts/signoff_review/partial-fixture-cleanup.md and
docs/tasks/artifacts/signoff_review/fixture-plan-coverage.md.

The resumed source-8d1504d checkpoint passes eight gates, then workspace testing
fails one initial state-writer lock acquisition (eleven other writer tests pass).
PostgreSQL/demo never start. Diagnosis .11.4.3.1.2.11.1 proves close-only lock
retention across actual CLI success/error/cancellation when a forked child retains
the descriptor; all three no-child controls and later child-exit acquisitions
succeed. All 341 source hashes match and fifty recorded groups are absent. The
original holder was not captured; its unchanged reruns pass. Explicit-release
repair/permanent controls .2.11.2 precede checkpoint resumption. Qualification
categories remain unchanged. Evidence:
docs/tasks/artifacts/signoff_review/state-writer-lock-lifetime.md.
