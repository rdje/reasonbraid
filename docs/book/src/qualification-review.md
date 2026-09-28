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

## When the corrective work ends

The corrective tree used to end when *every* census finding was closed or
refuted. That bar could not be met: each area reviewed turns up real defects, and
on 2026-09-24 seventeen leaves were closed while seventeen were opened. It now ends
at a **bug bar** (`docs/decisions/2026-09-25_the-corrective-tree-ends-at-a-bug-bar.md`),
the release rule security and reliability programmes use: a finding's class is set
by its impact on what the project claims today, a dev / trusted-LAN deployment.

A finding **blocks** the return to the roadmap when it lets one tenant read,
change, spend against or deny service to another; when it silently loses or
corrupts stored state, money, evidence or a publication; when the book or a
qualification statement says something the code does not do; or when a gate the
commit or push relies on lies. Anything else is **deferred with a trigger**: it
keeps an owner, a condition that reopens it (switching on a gated pack, claiming
Internet exposure, the first signed release), and a limit stated here.

The roadmap resumes when every blocking leaf is closed with evidence, every
deferred leaf names its trigger, the full CI passes on a pushed commit, and this
chapter matches measured behaviour.

The 65 open leaves on 2026-09-25 triage as follows. A first count said 50; it
missed 16 leaves that record their state in an older form, which is why counting
open leaves now gets its own instrument.

| Class | Count | Examples |
| --- | --- | --- |
| Closed by a closing check: every child done, parent never closed | 8 | evidence integrity, evaluation evidence, policy publication, acquisition safety |
| Closed as a duplicate | 1 | the pre-push stall's twin |
| Blocking | 25 | the shared resolver registry, policy registration authority, publication reviews and deployment digests, decision rules the close cannot count, console rendering, operational scripts, subprocess adapters, a fuzz baseline whose trigger fired, the bootstrap client's recovery, the full CI checkpoint, the review records not yet routed |
| Deferred with a trigger | 12 | the gated browser and credential packs, release certification, the introspection proposal, Internet exposure, the rare pre-push stall, four items already deferred |
| Structural: close when their children do | 19 | the section and chain parents, and the requalification leaf itself |

The rows below give each surface's current state.

## Other findings under repair

These are source observations and test limitations except for the explicitly
measured revocation controls above. Each row has executable repair ownership rather than an inert issue list.

| Surface | Current limitation identified in source | Repair leaves |
| --- | --- | --- |
| Authority and administration | Foreign grant/boundary mutation and shared site-registry authority are corrected with matched controls. Core subject serialization is reproduced and corrected with direct/enclosing and live compatibility controls; bound evaluation passes core/evaluator controls, 32 live authority/command API tests and strict lint. Actual-parent command selection passes 37 live tests, six evaluator controls and strict lint; frozen-read eligibility passes 40 live tests, ten pure controls and strict lint. Provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all results consumed and the owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint, including audit/query failure recovery; all results and shutdown consumed. Exact scoped receipt lookup passes 18 live authority tests and 30 HTTP tests with strict lint; all results/shutdown consumed. The `.3.3.4.1` census maps guard/effect integration children. `.3.3.4.2` qualifies the primitive guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass. Cancelled-BEGIN pooling and a stale embedded-migration executable are reproduced and repaired; all results/shutdown consumed and owned clusters/probes removed. Seven checker controls also correct nested-evidence owner selection, with broader doctrine accuracy still open. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. HTTP waits are bounded and a timed-out bootstrap keeps its key (`.3.3.4.3.3.3.3.2.3.2`). The ordinary writers check the server's reply against their request before saving it (`.3.3.4.3.3.3.3.2.4.1`). The CLI chapters describe this recovery in the present tense, and a test decodes their JSON examples through the real decoders (`.3.3.4.3.3.3.3.2.4.2`). Recovery across a server restart is qualified live (`.3.3.4.3.3.3.3.3.1`); so is a writer killed while a descendant holds its lock, whose successors refuse until the descendant exits (`.3.3.4.3.3.3.3.3.2`), and a real SIGKILL at each point between the CLI's own writes, after which the same command recovers the same request (`.3.3.4.3.3.3.3.3.3`). Power-loss survival is not claimed. Ordinary no-key behavior remains intentionally distinct. The final administrative effect representation is defined under `.3.3.4.7.1`: a closed set of fourteen operations, each carrying a caller-supplied target that exists at refusal as well as at success, and a closed `applied`/`no_op`/`refused` outcome whose refusal names one of three codes. `.3.3.4.7.3` corrects that last field: `.7.1` typed it against the §9.8 registry, and its first consumer found the registry has no `not_found` — a census showing 10 of the 19 codes the product emits are absent from it, routed out as `.11.7`. The three are what the fourteen administrative handlers actually refuse an admitted operation with. The fourteen were measured from the 27 guarded-admission call sites rather than chosen, and the four other tenant_admin mutations are deliberately outside it because other leaves own their gates. 10 representation controls, 68 core tests and strict core lint pass; the object-only control is falsified against a permissive decoder. `.3.3.4.7.2` then gives it durable storage: migration 0058's `administrative_effects`, keyed by the admission's own id, with a composite foreign key so an effect cannot cite another tenant's admission; a writer that runs on the caller's already-guarded transaction so evidence and mutation share one commit and an evidence failure rolls the protected write back; and an exact own-tenant reader that filters before it decodes. 8 live controls pass, and the fixture-plan blast radius (26 plans) was censused before the constraint was written. `.3.3.4.8` then makes grant and boundary revocation its FIRST producer: one exclusive-guard transaction holding the admission, the tenant-bound target selection, the status change, the epoch bump and the effect record, with the two superseded revocation services deleted rather than left as a second unordered path. 16 live controls pass; falsified 11/5, where restoring the pre-`.8` two-transaction shape lets a revocation APPLY after the caller's own administration ended while it queued (`left: 200, right: 403`). Two documented wire changes: the reason gains a 1 024-byte/control-character contract, and both responses carry an `x-reasonbraid-authorization` receipt. `.3.3.4.9` adds the SECOND family, spend-breaker arm and reset — previously the weakest administrative path, admitting under a shared guard and then mutating **on the pool** with no transaction, no guard and no record. Both verbs now run one exclusive-guard transaction; the exclusive mode is measured rather than copied, because classifying an arm against a row that may not exist cannot be done under a row lock, and because the reservation path holds the shared guard. Status codes, messages and success bodies are byte-unchanged; the one addition is the receipt header. The `409` that has always collapsed "no breaker armed" and "armed but untripped" still does, and the effect record distinguishes them as `refused`/`invalid_transition` against `no_op`. Neither verb takes a reason, so neither invents one, and neither advances the revocation epoch. 25 live controls pass (16 from `.8` plus 9 new); the affected set passes 5 suites / 94 tests. Falsified against the exact pre-`.9` handlers: **17 passed / 8 failed**, where 8 of the 9 new controls go red and an arm APPLIES while another operation holds the tenant's authority guard. ⚠️ That falsification also corrected the controls themselves: an EXCLUSIVE-holder fixture could not discriminate the repair, because the superseded admission took a SHARED guard and waited behind it too — the shared-holder fence is what turns the ordering control red. `.3.3.4.10` then censused the five node administrative routes before splitting them, and measured that **four of the five mutations carry no tenant predicate at all** — three inbox verbs and the certificate revocation select rows by `node_id` alone, the latter's tenant-bound existence probe being a separate pool query outside the mutation's transaction. That finding is annotated at `.3.5`, which owns real target ownership; `.3.3.4.10` owns putting the verification inside the mutating transaction. Its first child `.10.1` puts enrollment-token issuance onto the shape: one transaction under the tenant's **shared** guard — shared rather than exclusive because the refusal is decided atomically by one insert that does nothing on conflict, same-node contention is already settled by the partial unique index, and issuance touches nothing the reservation path reads. 11 live controls pass; the affected set passes 6 suites / 84 tests. Falsified 7/4 against the exact pre-`.10.1` handler, where a token COMMITS although its evidence could not be written (`left: 200, right: 500`). ⚠️ A shared-guard repair cannot be falsified by a lock-holding fixture in either mode, because the superseded admission took the shared guard too; the discriminator is atomicity, and the ordering property is labelled a regression control rather than presented as proof. ⚠️ Reproduced and routed to `.3.5`, not introduced here: the one-unused-token index is global rather than per tenant, so one tenant's outstanding token both reveals itself to, and blocks, another tenant's administrator for the same node identity. `.10.2` then puts node certificate revocation onto the shape, taking the **exclusive** guard — the opposite answer from `.10.1` one commit earlier, and derived: this advances the tenant's revocation epoch, so it is a revocation in the sense the guard contract means. Its superseded shape ran the tenant-bound existence probe as a separate pool query and then mutated on `node_id` alone, so the check and the act read two different snapshots; both are now one transaction with the tenant predicate in the writing statement. The reason is persisted rather than discarded and gains the 1 024-byte/control-character bounds; `revoked_at` is the transaction's own database time; the single 409 now distinguishes a repeat (`no_op`) from a node that never had a certificate (`refused`) in the record only. 30 live controls pass; the affected set passes 5 suites / 74 tests. Falsified 25/5 against the exact pre-`.10.2` handler, where the request COMPLETED while a shared holder held the tenant's guard. `.10.3` closes the parent by putting the three inbox verbs on the shape, and it is the most consequential of the three children: `node_inbox` has carried a `tenant_id` column since migration 0003 and **none of the three verbs used it**. A probe run before the repair measured an administrator of one tenant quarantining (200) and replaying (200) another tenant's inbox row, and pruning its inbox with `{"deleted":2,"before":2,"after":0}` — destroying both of the other tenant's rows. Three of the project's own fixtures depended on that defect, seeding into one tenant and administering from another, and are corrected rather than deleted with every original feature assertion intact. All three verbs now select bound to the admitted tenant under the shared guard, with the prune's before/after counts bound too, and each records its outcome. 7 live controls pass; the affected set passes 6 suites / 72 tests. Falsified 3/4 against the exact pre-`.10.3` handlers, where the foreign row comes back carrying a fresh `quarantined_at` and the prune deletes two rows while reporting success with unwritable evidence. Effect auditing for `.11` and `.12` and other caller/target paths remain. Deferred with a trigger (`.11.38`): the CLI's text output prints a list that a malformed server reply left out as an empty one (for example `tenant …'s grants (0):`), where it should refuse the reply; `--json` prints the reply as received. It becomes a defect to fix once the CLI is claimed to work against a server of another version, or a text-mode output is documented as machine-readable. | `.3.1`–`.3.5`, `.11.38` |
| Node recovery and budgets | **Reopened** on 2026-09-28 by the review-record census (tranche 5a); it was complete on 2026-09-24. 🔴 Open and blocking: after a total machine loss the lost node's in-flight work is replayed without the authorization a possible duplicate needs, because the fact that its outcome was unknown died with the machine (`.11.52`); a node's result is linked to the role's current incarnation rather than the one it was dispatched to, and a result the server cannot use is still acknowledged (`.11.48`). Deferred with a trigger: every call over a quota writes a denial row that nothing prunes, and the quota window has no upper bound in time (`.11.49`; the trigger is admitting a principal the owner does not control). What held on 2026-09-24: a result survives a crash between being journaled and being sent; channel calls time out and back off; an attempt's deadline and output size are enforced; a retry whose outcome is unknown needs an administrator's authorization, and an operator's verdict settles its budget hold. The budget ledger cannot overflow, admission serializes on the ceiling, quota and breaker it decides against, a machine cannot settle another tenant's hold, and a breaker watches only what it names. Machine certificates never outlive their authority, and the authority renews itself with an overlapping trust set. Each repair was observed failing first and falsified by deliberate mutants. Deferred with a trigger: the server outbox's runner, until its first consumer (`docs/decisions/2026-09-24_the-server-outbox-has-a-producer-and-no-consumer.md`). | `.4.1`–`.4.6`, `.11.48`, `.11.49`, `.11.52` |
| Directory and recruitment | **Reopened** on 2026-09-28 by the review-record census (tranche 5a). 🔴 Open and blocking: an imported agent card keeps whatever confidence its claims state, and nothing authenticates its origin beyond a digest of its own bytes (`.11.45`); the view another tenant reads, which this book calls the network pseudonym, carries the role's own id (`.11.46`); the initiator closes its call without its invitation authority being read again (`.11.47`). Before `.5.1`–`.5.3`, candidates were classified by the reader's own class, a call could name a thread that was not a real one of its tenant, and automatic creation reused one key per role and tenant. Now each candidate is judged by the reader's relation to its tenant, a call must name a real thread of its own tenant (`404 scope_hidden` otherwise), and each automatic initiation carries its own key under a per-initiator quota. ⚠️ Signed agreement bundles stay deferred beyond v1 (`.5.3`). | `.5.1`–`.5.3`, `.11.45`–`.11.47` |
| MCP and A2A | ✅ **Complete** except the deferred A2A build (2026-09-26). MCP reads run the HTTP handlers' own authorization, because they call it (`.6.1.1`). The write seam takes the authority the handler needs rather than enrolment (`.6.1.2`) and indexes a typed body (`.6.1.3`), the quota's admission is tested by a control that exercises it (`.6.1.4`), the policy registry's scope is decided and its lifecycle reads are bound over HTTP and MCP alike (`.6.1.5`), and listen deduplication keeps the right window and cursor (`.6.2`). The A2A facade records every semantic loss and its response keeps the peer's task id (`.6.3.1`). A2A qualification demonstrates serialization rather than an independent transport peer: nothing in the product accepts an A2A message yet, so no local grant authorizes one. Deferred with a trigger until a surface does (`.6.3.2`). | `.6.1`–`.6.3`, `.6.3.2` |
| Resource acquisition | A resolver the server cannot execute no longer silences a resolution: the first executable ranked resolver acts, and the others are named (`.7.1.3`). Adding a row to the site-global resolver registry takes the `resolver_register` site capability, so a tenant administrator can no longer add one (`.7.1.3.1`). Each tenant's citation of a URL is read and resolved by what that tenant declared, not by the first tenant to cite it (`.7.1.4`), and a tenant's first citation no longer tells it that another tenant cited the same URL (`.7.1.4.1`). Still open and blocking: deterministic ranking on a tie (`.7.1.5`), and the ungated R2 extraction bounds (`.7.3`). Deferred with a trigger: the browser and credential packs, which are off unless `RB_ENABLE_R5R3RX=1` and are not qualified. The browser pack's Chrome also connects to Google services on its own (time, accounts, the component updater, messaging registration), measured on 2026-09-25; those connections are not the page's, so they are not on the render receipt and no destination class is applied to them (`.7.3.7`). 🔴 Open and blocking (`.11.50`): the IPv6 destination check refuses a list of ranges and admits everything else, so several reserved ranges pass, and the git pack does not classify a redirect to a literal IPv6 address. Deferred with a trigger (`.11.51`): the git pack's walk of its own downloaded files skips a file it cannot read, and counts a directory it cannot read as empty. | `.7.1`–`.7.3`, `.11.50`, `.11.51` |
| Evidence and evaluation | Checked clause by clause on 2026-09-25 (`.7.4`). ✅ Repaired: a deleted (tombstoned) snapshot no longer accepts derivations or supports assessments, each refused with the tombstone's reason, and acquiring the same content again makes a new snapshot instead of silently re-citing the deleted one (`.7.4.6`). A tombstone retires one acquisition, not the content: the same bytes acquired later are live evidence until that row is tombstoned too, and blocking content wherever it appears would be the quarantine gate's job, which does not exist. ✅ Repaired: an assessment's author is the principal that submitted it; a body naming an author or a verifier is refused (`.7.4.7`). Deferred: a verifier, which the roadmap describes as a second party's act, is not recorded, because nothing performs that act yet (`.7.4.13`). ✅ Repaired: resubmitting an assessment with a different excerpt, selector, rationale or quality indicator is refused, naming what differs, where it used to be reported as stored and silently dropped; an identical resubmission still returns the same assessment (`.7.4.8`). ✅ Repaired: a re-acquisition now refreshes the freshness horizon, and the horizon is each citing tenant's own, kept on its citation. It used to sit on the shared row, where a replay's horizon was discarded and the first tenant to acquire the bytes decided every other tenant's stale list (`.7.4.9`). Holding: an excerpt must appear in the snapshot's bytes, and its presence does not prove the claim; expiry uses the server's clock and a tenant cannot delete evidence another relies on. Evidence quarantine does not exist (deferred, `.7.4.10`). Evaluation (`.8.2`, checked the same day): gates refuse a missing or non-numeric measurement, bind their corpus, and calibrations refuse ineligible runs, all behind the site-operator gate; ✅ repaired, a trial refuses a repeated arm or case id, where a repeated arm used to bias its seeded assignment silently, and the harness's refusals say what is wrong instead of reading as a digest error (`.8.2.6`); deferred, a stored gate baseline that were corrupt would be reported as a refusal of the caller's request rather than a server fault, unreachable while baselines are validated on write (`.8.2.8`); ✅ repaired, thread creation records its routing audit row inside the create command, after authorization, so a refused create or a replay leaves no row; before, the row was written first, whatever happened next (`.8.2.7`). | `.7.4`, `.8.2` |
| Deliberation and governance | Policy publication checked clause by clause on 2026-09-25 (`.9.2`): its filesystem target, approval and projection binding, Git object checks, compare-and-swap, reference verification, reconciliation and reproducible commit all hold; ✅ Repaired: a publication's final transition now writes only while the publication is still staged, so two at once cannot both succeed and an effective publication cannot be flipped to failed; the loser is told the stage it found (`.9.2.2`). ✅ Repaired: the proposal, decision, approval and staging routes answer a database failure as the server's (`500`), where fifteen lookups answered it as a missing record or an invalid proof (`.9.2.3`). Policy registration and resolution were checked the same way the same day (`.9.1`). Tenant scoping, the owning authority's liveness and action, and write-once rows all hold. ✅ Repaired: a registrar must now hold the grant it names as a policy's owner, where a site operator could name any other principal's (`.9.1.2`). ✅ Repaired: the server now computes a policy's digest from the document, where it used to store whatever the caller declared, and every read says whether a stored digest still matches (`.9.1.3`). ✅ Repaired: a projection's `policy.lock` is now written by the server from the registry, one line per policy the request named, where it used to publish whatever rows the caller wrote; the projection examples in the policy lifecycle chapter now run and are sent by a control (`.9.1.4`). ✅ Repaired: a selector is now exactly a layer and a target, and a malformed one is refused at registration and fails a resolution closed, where it used to match every target (`.9.1.5`). ✅ Repaired: a dependency must apply at the exact version it names, a precedence cycle of any length is refused, and a set naming a policy twice is refused as such (`.9.1.6`). Decided rather than changed: a policy's lifecycle label is its registrar's one-time declaration, a policy is put in force by approval and publication, and the label is shown in each resolved clause rather than acted on. Label transitions are deferred with binding use (`.9.1.8`). ✅ Repaired: the registry's refusals say what happened, a store that cannot answer is a `500` rather than a refusal, and a version is SemVer 2.0.0 (`.9.1.7`). ✅ **`.9.1` is closed**: every blocking clause is repaired, and the rest is deferred with a trigger. Deferred until binding policy use is claimed (`.9.1.8`): resolution checks that each owner's grant is live but not that it covers the target, models no effective interval, takes precedence from what each policy declares about itself rather than from the charter, names a requested waiver without applying it, and has no lifecycle transitions (a version's label is set once at registration). Repeated challenge resolution and supplied attribution need correction; policy authority, lifecycle and Git reconciliation need stronger binding. The publish verb no longer takes its repository location from the request body (`.9.2.1.1`): the deployment declares one root, the caller names a location inside it, and `..` and a symlink out of it are refused by the same canonicalized containment test. An undeclared root closes the verb (`503 publication_repository_unconfigured`) and an unusable declared root refuses the boot before the migrations run. Five live legs and seven offline arms pass, each observed red first. `.9.2.1.3` then closes the second half: `mark_effective` recorded whatever Git object ids the caller declared and nothing opened a repository, and **three of this project's own fixtures drove the transition with ids that resolve to nothing** — the suite meant to qualify it was proving it with `abc123`. Every declared id must now name an object that exists in the named repository, checked per id in the core both publish verbs go through; the three fixtures are re-seeded with ids read back from a real repository rather than relaxed. `repo_path` becomes required on the effective verb, which is a change to a shipped request shape. Four offline arms and two live legs pass. ⚠️ An object is proved to EXIST, not to be this publication's own — binding the ids to its refs needs the repository recorded with the row. `.9.2.1.2` closes the last of the three: both verbs admitted any enrolled principal, and now require an `owning_authority` the caller HOLDS, checked by one definition both call before the path is resolved or the publication loaded. 🔴 **It was not the last of the three verbs, only of the three findings, and the difference cost a live hole**: a publication has a THIRD transition, `/failed`, which kept admitting on enrolment alone until `.9.2.1.2.1` — while the server comment beside the repair asserted three call sites and there were two. That transition is terminal, so it is the one that permanently takes a publication off the table. A shared helper binds the callers that call it, never the sibling that never did. Six live legs over two tenants pass, including naming another principal's real active grant — refused — beside the matched pair where only the holder differs and the request is admitted. ⚠️ **A held grant is effectively tenant-wide for these verbs**: no grant action and no target selector can name a publication, so holding is stronger than enrolment and is the best the model expresses; `.9.3.4` owns the narrowing. Deferred with a trigger (`.8.1.1.4.1`): `role_weighted` and `human_committee` are refused at thread creation until they are designed, a schema for the charter's weights and a §13.3 bar for the committee; a roadmap phase or a tenant asking to decide under one picks the design up. Deferred with a trigger (`.8.1.1.6.1`): recusal, role replacement, incarnation attribution and amendment after voting starts are not built. A counted rule takes the narrowest reading, and each reading is what the code does: no member is recused, a ballot belongs to the principal that cast it (under delegation, the caster), and a ballot is final. | `.8.1`, `.8.1.1.4.1`, `.8.1.1.6.1`, `.9.1`–`.9.3` |
| What this manual covers, measured | ✅ **Complete** (`.11.4.6.3`–`.11.4.6.8`). Every surface the product ships is listed and judged one by one, rather than described in a sentence. There are **48**: 34 families of HTTP route, 3 routes the web console serves for itself, and 11 programs. **45 are covered** by a chapter, **3 are deliberately internal** and say why, and **none is a gap**. When first measured there were 43, 17 of them gaps, each handed to a repair leaf; the largest were the evaluation harness, whose seven endpoints appeared nowhere, and the policy lifecycle. ⚠️ The earlier, simpler measure was too kind: asking only *does some chapter mention this?* made 21 of 30 route families look covered, but asking whether the manual actually documents **each endpoint** of a family leaves 14. Seven families the simple measure called covered are missing endpoints — usually the reads are written down and the actions an operator performs are not, including three revocations and the node quarantine, replay and revoke verbs. ⭐ The measure was also too harsh in one place, which is why a judgement is recorded rather than a count: the console page at `/` scores as a bare mention and in fact has a whole chapter. A check runs on every commit and refuses the build if a surface is added that nobody has judged, if a judgement outlives the thing it judged, or if what a judgement was based on has since changed. It does not claim any judgement is *right* — only that it is still about the facts it was made from | `.11.4.6.3`–`.11.4.6.8` |
| Adapters and console | The Claude and Codex adapters were checked clause by clause on 2026-09-25 (`.10.1`). Holding: the supervisor bounds an attempt's output, a failed spawn is reported before any dispatch, and the Claude adapter ends its options before the prompt. ✅ Repaired: the Codex adapter now ends its options with `--` before the prompt, where a prompt starting with `-` used to reach the provider CLI's option parser, and a doctrine pins exactly one separator for both adapters (`.10.1.1`). ✅ Repaired: both adapters read their streams as bytes, with a 2 MiB line bound tied to the supervisor's at compile time, a stderr drain that keeps the last 8 KiB and never stops early, and a tail cut on a character boundary (`.10.1.2`). The old drain stopped at invalid UTF-8, and the provider was then killed by `SIGPIPE`; an earlier reading had called that a stall. ✅ Repaired: every terminal path settles and reaps the child, and dropping an attempt releases and stops it, where the adapter used to keep every child and the success path reaped none (`.10.1.3`). ✅ Repaired: an attempt ended by a signal, a cancel included, now has no verdict (`outcome_unknown`), where it used to be reported as a known provider failure (`.10.1.4`). ✅ **`.10.1` is closed.** Deferred with a trigger: the usage mapping has been checked against the providers' documentation only, never a real receipt (`.10.1.5`). Certification evidence has source-review findings. The console was checked the same day (`.11.1`): no view can run data as script. ✅ Repaired: the Timeline view failed for every thread that had events, because a number reached a function that only accepted text or page elements; any value now renders as text (`.11.1.1`). A browser check now runs the console in the pinned Chrome over a real server and database, compares the Timeline and Audit cell by cell with the server's answer, and confirms that markup in thread content stays text and runs nothing; it failed on the old page with the exact error an operator saw. ✅ Repaired: a slow answer no longer lands in a later view; each view draws into its own space, and the presence and inbox panels keep only the latest click's answer (`.11.1.2`). Four browser runs hold one answer back on purpose and check that it changes nothing when it arrives. ✅ **`.11.1` is closed.** | `.10.1`–`.10.2`, `.11.1` |
| Verification and operations | The supported launcher and disposable runner now localize stores and validate test connections. All five core live documents are now bounded, each refusing oversize input by naming its own file. The registry that routes overflow away from the landing page is now checked for what its rows CLAIM, not only for whether they are well-formed. It had been possible for a row to state a control that nothing performed, and one did for months. A census first established what could be checked at all — fourteen of the twenty rows make a claim a machine can evaluate and six are narrative — and each of those rows now carries its claim in a form a checker reads, beside the sentence a person reads. Every commit evaluates them and refuses a false one by name. The three faults the census found are repaired rather than recorded: the task-tree index named a governing rule that does not exist under that name, a frozen companion claimed it had not changed since a moment nothing could resolve and had in fact changed three days after it, and the decision-record directory's rule was true only by care. The claims are not read out of the prose, which was measured and found wrong more than a quarter of the time in both directions; a row states its terms deliberately, and the check refuses both a term quietly dropped and a term invented that the sentence never made. Measured against the external standard this discipline was adopted from, the contract is now met: a reader can load a small bounded current view, follow one exact pointer that is itself checked, retrieve any retired history deterministically, and run one gate that rejects every undeclared or oversize path — without loading a monolith. One requirement of that standard is still unmet and owned: two of the bounded surfaces refuse at their limit with no earlier warning, where the three rotating ones warn while there is still room to act. ✅ Repaired (`.11.40`): the check that every code change is owned by a task-tree leaf could not see a commit that only deleted or renamed code, or only changed a database migration or a git hook, so any of those could land with no leaf. It now counts every kind of staged change, shares one definition of code with the acceptance check, reads that definition from what the commit carries, and has no bypass; no earlier commit relied on the gap. Remaining direct-entrypoint storage, artifact cleanup, script checks and historical claim accuracy require correction. | `.11.2`–`.11.4`, `.11.40`; completed prerequisite evidence in `.2.1`–`.2.2` |

Full records: `docs/tasks/artifacts/signoff_review/INDEX.md`. The corrective tree
must give every finding a reproducible fixed or refuted disposition before its
requalification leaf closes. The next roadmap features remain store-and-forward,
verified export/import and the independent G8 implementation exercise.


### The census records, reconciled clause by clause

The startup source review produced 131 records under
`docs/tasks/artifacts/signoff_review/`. Each names one or more candidate repair
leaves. `SIGNOFF-REPAIR.11.9` measured that **114 of them are cited by none of
those leaves** and rejected a gate over that population — 114 of 131 is a backlog,
not a gate. `SIGNOFF-REPAIR.11.9.1` then censused the backlog: roughly **491
clauses**, drawn from a vocabulary of only **33 distinct candidate leaves**, one of
which (`.11.4`) is named by **99 of the 131 records**.

That last figure changed the plan. `.11.9.1` had been told to work the records
that name a single leaf first, on the reasoning that a narrow routing was a real
assignment. Measured, the opposite holds: every single-candidate record points at
one of those broad containers, and the clearest case — `R-6-27-1` — carries **no
finding at all** and still received one. The backlog is now ordered by each
record's *narrowest* candidate instead, and the classification lives in
`docs/tasks/artifacts/signoff_review/RECONCILIATION.md`, one row per clause, with
a closed vocabulary an instrument re-reads.

Reconciling the first seven records found two limitations that were not otherwise
tracked, both now owned:

| Limitation | Effect | Owner |
| --- | --- | --- |
| ✅ **Repaired** (`.3.5.3`): `GET /v1/nodes/inbox` admitted on the caller's tenant and then selected the inbox by node id alone | **Reproduced and repaired.** A tenant administrator read another tenant's inbox rows — command ids, thread ids, delivery state and payloads — for a node id they could name. The three inbox *mutations* were bound to their tenant under `.3.3.4.10.3`; this read was outside that census's scope. The select now carries the admitted tenant. | `.3.5.3`, closed |
| ✅ **Repaired** (`.11.2.2`): three doctrine enforcers wrote their scratch to an ambient temporary directory | Project-owned temporary data landed off the repository volume, against §13, and the storage-locality gate enumerated Rust sources only, so it could not see them. The scripts now write under the repository's `target/`, and the gate reads shell and Python sources as well as Rust. | `.11.2.2`, closed |

Neither changes a documented product behaviour today; both are recorded here
because this chapter is where the manual states what is *not* yet qualified.

Tranche 2 was **2.8 times** tranche 1 by record text, so `SIGNOFF-REPAIR.11.9.1.1`
split it into three before classifying anything — on the same narrowest-candidate
measurement that formed the tranche, so each part covers one surface. Its first
part reconciled six records (27 clauses) whose narrowest candidate is the
site-registry authority leaf, and it found four further limitations plus one
correction to the backlog's own headline number:

| Limitation | Effect | Owner |
| --- | --- | --- |
| ✅ **Repaired** (`.7.1.3.1`): `POST /v1/resolvers` admitted on tenant administration and wrote a site-global registry | **Reproduced and repaired** (`.7.1.3.1`): a tenant administrator registered a row, answered `200`, before the change; registering is now the `resolver_register` site act, and the same caller receives `403`. What was found: any tenant's administrator could register a resolver row on a site-global table that has no tenant column. The site-operator repair pinned a closed set of six actions that does not include it. **Narrowed by `.7.3.6.2`:** the verb no longer REPLACES — it refuses an already-registered id by name, so a built-in pack's declared schemes, egress class, sandbox level and security evidence can no longer be rewritten through it. Creating a new site-global resolver on a tenant-admin grant was untouched by that narrowing, and `.7.1.3.1` closed it. | `.7.1.3.1`, closed |
| ✅ **Repaired** (`.7.1.2.1`): `POST /v1/workflow-profiles` admitted on enrolment alone | Registering a workflow profile is the `workflow_register` site act. What was found: any enrolled principal could register a new version of any profile id, and resolution takes the highest version, so a built-in deliberation profile could be shadowed for the whole site. | `.7.1.2.1`, closed |
| The publication reconciler never reads the effective channel ref | Its `effective` case compares only the immutable publication ref, so the §15.8 row "effective / ref missing or moved → freeze deployment" cannot fire for a moved effective channel. A unit test encodes the gap. | `.9.2` |
| A decision-family close consults only the caller's own unresolved list | The durable open-challenge count the engine maintains is never read at close, so omitting a challenge closes the thread with a decision outcome. | `.8.1` |
| ✅ **Repaired** (`.11.10`): `regions::route` reported any database error as an undeclared region | **Reproduced and repaired.** With its reads failing, the routing answered ``the region `dev-local` is undeclared`` for a region the same run proved declared. The decision now returns a typed storage failure that names neither the region nor the word "undeclared", while every existing refusal keeps its exact wording. | `.11.10`, closed |

The correction concerns the backlog's headline: **eight** of the uncited records
*are* named in the task tree, in text that belongs to no leaf — six of them in one
historical dispositions table written by the leaf that did the work. "114 uncited"
therefore means "cited by none of their candidate leaves' own sections", not "114
that nobody re-read". No published figure changes; what changes is what may be
concluded from it.

The same reconciliation found a defect in its own instrument. Three clauses
classified as `attach` — the state meaning *a leaf owns this surface but its text
does not mention the clause, so its next census will drop it* — had been recorded
and never attached. All are now written into the leaves that own them, and the
ledger requires the attachment in the commit that classifies it.

The tranche's second part read four more records and added another five
limitations, plus the first clause the ledger has *declined*:

| Limitation | Effect | Owner |
| --- | --- | --- |
| ✅ **Repaired** (`.4.1.6`): an unvalidated host claim reached the certificate library's `expect` | **Reproduced and repaired.** Driven through the supported routes, a non-ASCII host claim was accepted at issuance and panicked at redemption: the node received a dropped connection rather than an answer, no node row was written, and the token was left unconsumed — so that node id could not be enrolled until the token lapsed. The claim is now checked at issuance, where an operator typed it, and both later paths answer with a typed refusal. ⚠️ The accepted set is deliberately unchanged: the check asks the certificate library, and whether a stricter grammar should bind was decided rather than left open (`docs/decisions/2026-09-14_host-claim-checked-at-issuance.md`), because it has a compatibility cost. | `.4.1.6`, closed |
| One certification run asserts three of the six named invariants | The conformance box it writes reads "the six §19.4 invariants passed". Its completion invariant also accepts a failed terminal as completion. | `.10.2` |
| ✅ **Repaired** (`.4.1.7`, `.4.1.8.1`, `.4.1.8.2`): the certificate authority was signed for a year, with no renewal and no check at load | A deployment that had run a year would issue leaves from an expired issuer, and a leaf was never compared against its issuer's expiry. A leaf is now capped at its issuer's expiry, an issuer at or near its end issues nothing, and the authority renews itself when a third of its life remains, with the old and new generations trusted together; the server's health prober runs the renewal. The one-year lifetime is kept by design. | `.4.1.7`, `.4.1.8.1`, `.4.1.8.2`, closed |
| ✅ **Repaired** (`.5.1.5`): the visibility policy's documented default and its actual default disagreed | The type's documentation said every profile field defaults to self-only, while the default publishes eleven of fourteen fields to the tenant or the network, and a profile that names no policy takes it. The tiered default was kept by decision; the code's documentation and [Profiles](profiles.md) now state it field by field (four to the network, seven to the tenant, three to the profile's owner). | `.5.1.5`, closed |
| A publication commit embeds the wall clock | Re-publishing byte-identical content a second later writes a different commit, while the step's own documentation calls it an idempotent re-write. | `.9.2` |

The declined clause is recorded rather than dropped: an unset optional profile
field is serialised as `null` rather than omitted, which is true, but the
documented promise it was said to contradict concerns *hidden* fields and still
holds — hidden means absent, unset means `null`, and a reader can still tell them
apart.

The tranche's third and last part read the remaining four records — the ones
about this project's own scripts and gates — and closed tranche 2 at fourteen
records and eighty-four clauses. It opened no new repair leaf, because every
clause reached a leaf that already owned its surface. It did add nine
limitations, and one of them is about the manual you are reading:

| Limitation | Effect | Owner |
| --- | --- | --- |
| The table-arity gate disagrees with the renderer that publishes this book | A pipe inside an inline code span is treated as part of the cell, and the renderer splits on it and discards whatever will not fit the header. Two tracked table rows lose content today while the gate reports none — including the doctrine registry's own row for the tree-index check, whose published third cell is a bare em dash instead of the script that runs it. The gate's self-test asserts the wrong behaviour, so it cannot catch this. | `.11.2` |
| Doctrine checks that read the staged file list then open the worktree | Staged evidence can be satisfied by text that was never staged. Seven of nine was the count first published, and the ownership check was one of the seven in error: it reads only the names of the staged files, and since `.11.40` the definition of code it shares with the acceptance check is read from what the commit carries too. | `.11.2` |
| The document-path check cannot see this repository's own checkout prefix | It matches `/Users` and `/home` only; this checkout is under `/Volumes`, so the one absolute path this project would leak is the one the gate is blind to. | `.11.2` |
| The handoff background-job census reports success when it fails | Both process censuses discard their errors and the script does not exit on error, so an unavailable census renders as "no job running". Its own `ps` arm also covers every user, where its documentation claims one. | `.11.2` |
| The scaffold updater would overwrite the task-tree index | It lists that file as project-neutral while the file holds this project's index of active trees, and its own header promises the opposite. A local donor path is copied whole, build artifacts and caches included. | `.11.2` |
| The template bootstrap treats the presence of one file as proof of a pristine copy | In a copy that still holds it, a named invocation overwrites the resume pointer, the decision index and the tree index. Its in-place edits are also written in a dialect a stock macOS `sed` rejects — mid-way through the destructive sequence. | `.11.2` |
| ✅ **Repaired** (`.11.3.7`): the demonstration's negative controls passed when the command failed | Each ran a negation over a pipeline, so a failing CLI and a genuine absence were indistinguishable, and its evidence-capturing requests ignored HTTP status. Each negative check now requires its command to succeed with output first, a capture that does not answer `2xx` is a recorded failure, and the authenticated-poll probe requires a `200`. | `.11.3.7`, closed |
| ✅ **Repaired** (`.11.3.4`): the load harness reported success for a run that issued nothing | A zero or negative command count passed both exit gates having sent no requests; a positive count was rounded up per worker, so 10 commands at 8 workers ran 16; and its latency percentiles included the failed requests, so a fast refusal pulled the published figure down. It now runs exactly the requested count or refuses, bounds every request, counts every worker's exit, takes percentiles over the committed requests with the failures beside them, and waits for its own server. The one published run (200 commands at 8 workers, 0 failures, 2026-09-08) is unaffected by either defect. | `.11.3.4`, closed |
| ✅ **Repaired** (`.11.3.3`): the one-command development environment built the caller's repository | It never changed to its own root before building, yet ran its own binary, and outside `make dev` it wrote its build cache off the repository volume. Its teardown removed the cluster whether or not the database stopped, reporting success either way, and its check accepted any server on its port. It now builds its own repository through the project environment, removes the cluster only once no database runs on it, and waits for its own server. | `.11.3.3`, closed |

Eleven of this part's thirty-eight clauses were `attach` — more than the other
three parts together. The cause is measurable and worth stating: these leaves'
goal lines are written as short lists of *named mechanisms*, so a finding inside
the surface but outside the list is invisible to a census driven by the goal
line. The earlier parts' leaves state *properties*, which generalise. All eleven
are now written into the leaves that own them.

The reconciliation's own instrument gained a gate. One of its six clause
states, `attach`, means *a leaf owns this surface but its text does not mention
the clause, so its next census will drop it* — and it is the only state whose
required next action is to write a sentence into a leaf the classifier does not
own. Every other property of a ledger row can be checked from the row. That one
could not, and tranche 1 recorded three such clauses and attached none of them.
A refusal now blocks any commit whose `attach` row is not named by the leaf that
owns it. Before registering it, the rule was run against each tranche's closing
commit: it fires on all three historical instances and on none of the
twenty-six rows standing today.

The next group of records — the first five of tranche 3 — added four
limitations, three of which are one missing predicate seen from three places:

| Limitation | Effect | Owner |
| --- | --- | --- |
| ✅ **Repaired** (`.4.3.1`, `.4.3.3`): a node's work was looked up by operation id alone, with the node it belongs to absent from the query | The reconciliation the handshake performs returns another node's event id for an operation id it names; a receipt written first under a known event id silently swallows the rightful node's own; and an inbox row is marked consumed by a different node's result carrying the same command id. Three places, two source records, one shape. ✅ The reconciliation lookup now takes the node and the receipt key names it (migration `0102`); the `consumed` rung joins this node's own receipt and the result fold's idempotency key names the node (migration `0104`). Each was reproduced first with two nodes. | `.4.3.1`, `.4.3.3`, closed |
| ✅ **Repaired** (`.4.3.2`): the inbox cursor's high-water mark was derived from the rows pruning deletes | Prune everything and the mark returns to zero, so the next item re-uses a cursor the node has already acknowledged and discards as seen. The existing test leaves a partial prefix that keeps the highest cursor, so the case never arises in it. ✅ **Reproduced first, as the clause asked:** a prune-everything-then-reconnect control failed on the old code (the reconnect was refused `cursor_ahead`). The mark is now one durable row per node (migration `0103`) that every writer bumps in the statement that reads it; pruning removes rows, never numbers. | `.4.3.2`, closed |
| ✅ **Repaired** (`.9.3.2`): a review trigger named "repeated waiver" fired on the first waiver, and on waivers that expired long ago | Neither recording a waiver nor scheduling a review consulted its expiry, so one bounded exception scheduled a review for ever, and a test asserted exactly that. It now takes two waivers in force, recorded within 90 days. | `.9.3.2` |
| ✅ **Refuted** (`.4.1`, DOC-0158): a node with no usable certificate did not read as suspended | The presence view reads whether a certificate is revoked, not whether it has expired, and that is correct rather than a defect: revoking a node revokes every certificate it holds, expired ones included, so a revoked node always reads suspended, and a node whose certificates have merely expired reads offline, which is its true state. | `.4.1`, closed |

⚠️ One clause was *declined*: two tests hardcode a correction expiry of
2026-09-15, and the record asked whether that makes them time-brittle already.
Measured on both sides the day before, it does not — neither writing a
correction nor scheduling a review compares that field to the current time. The
brittleness is real but conditional on the missing check being added, which is
what the repair will do.

The next five records concerned the MCP surface and the policy registry, and
produced the sharpest read finding this review has recorded:

| Limitation | Effect | Owner |
| --- | --- | --- |
| ✅ **Repaired** (`.6.1.1`): all three MCP read tools returned data the caller was not entitled to, while the module documented the opposite | The inbox tool declared a caller argument and never read it. The thread tool computed the correct reader class for a foreign tenant and returned the full projection anyway, reporting that class beside it; measured live, a caller in one tenant received another tenant's entire thread projection. The policy-bundle tool read neither its caller nor its tenant. ⚠️ **One part of this finding was corrected by measurement**: the policy registry has no tenant column and no site filters by one, so the bundle was not crossing a tenant boundary — it is a site-wide registry, and the HTTP surface returns the same set to any enrolled caller. What the tool owed was that surface's enrolment check, and what it got wrong besides was labelling a site-wide bundle with the caller's tenant. Whether the registry should be tenant-scoped was `.6.1.5`'s question, and it decided (DOC-0071) that the policy library is shared by design while its lifecycle records belong to their tenants. | `.6.1.1`, `.6.1.5`, closed |
| ✅ **Repaired** (`.6.1.2`): the call-response path checked the caller against one tenant and the call against none | A caller admitted on their own tenant could record a response on another tenant's recruitment call. The tenant binding is now in the shared path; a decline or recusal still skips the eligibility check, within the call's own tenant, by design. ⚠️ **The finding named the MCP tool and the plain HTTP verb was equally affected** — it takes no tenant at all and shares the same code — so the check was added to the shared path rather than to the tool, and the control exercises both. | `.6.1.2`, closed |
| ✅ **Repaired** (`.9.3.1`, `.9.1.2`): registering a policy accepted an expired authority grant | Registration checked the grant's status only, while resolution in the same module also checked its expiry, so a lapsed grant still registered a policy version. Both now ask one shared predicate, which is also the first time either consulted the grant's start date. Whether a policy's owner must be a grant the registrar personally holds was a separate question; `.9.1.2` answered it, and registration now requires a live grant the registrar holds. | `.9.3.1`, `.9.1.2`, closed |
| A policy's applicability selector defaults to the wildcard — **selectors repaired, `.9.1.5`** | A missing or malformed selector field was read as "matches everything", which is fail-open in the place the design requires fail-closed. ✅ A selector is now exactly a layer and a target; registration refuses any other shape, and a stored one that does not parse fails the resolution closed. The lifecycle half is decided, not changed (`.9.1.6`): the label is a one-time declaration and never transitions, so `draft`, `superseded` and `deprecated` versions resolve, with the label shown in each clause's path. Transitions are deferred with binding use. | `.9.1.5`, `.9.1.6` closed; `.9.1.8` |

⚠️ Two of the source record's claims were *narrowed* by measurement rather than
confirmed, and both are recorded that way: a panic on a malformed payload is
real but not reachable through the typed tool, and the quota's behaviour on a
retried call is stated exactly as it behaves in the module's own header.

The last five records of that group concern the processes the product spawns —
the browser worker and the two provider adapters — and they close the third
group at fifteen records and sixty-nine clauses:

| Limitation | Effect | Owner |
| --- | --- | --- |
| The browser worker waits for its child to finish before reading what the child wrote | The request allows four megabytes of output while an operating-system pipe holds at most sixty-four kilobytes, so any larger response blocks the child's own write, the child never finishes, and the wait runs to its deadline. The failure is reported as a timeout. | `.7.3` |
| The browser worker's error and timeout paths leave processes behind | A failure writing the request abandons a running child without killing or waiting for it, and the timeout kills the worker but not the browser it started. | `.7.3` |
| ✅ **Repaired** (`.10.1.2`): each provider adapter could panic while reporting a failure | The stderr excerpt attached to a failure reason was cut at a byte offset rather than a character boundary, so a child that wrote non-ASCII could crash the adapter on the path that was already handling an error. One shared reader now reads both streams as bytes, keeps the last 8 KiB, and cuts the excerpt on a character boundary. | `.10.1.2`, closed |
| ✅ **Repaired** (`.10.1.3`): each provider adapter kept every child it had ever started | The map of running children was added to and never removed from, and the successful path returned without waiting for the child or its output drain. Every terminal path now settles and reaps the child within a bound, and dropping a handle removes its own entry, and only its own. | `.10.1.3`, closed |
| ✅ **Repaired** (`.4.4.6`, `.4.4.6.1`): a per-attempt deadline was sent and never enforced | The value was computed, placed on the dispatch and carried in the contract, and nothing read it. The node's supervisor now bounds the provider's acknowledgement and every stream read by it, charging elapsed wall-clock time, cancels on expiry and records `outcome_unknown` naming the deadline. The adapters still do not read the field; the supervisor enforces it for them. | `.4.4.6`, `.4.4.6.1`, closed |
| ✅ **Repaired** (`.4.4.8`): a completed item that reached the retry gate was quarantined as a dead letter | The gate refused every state it did not name individually, completion included, and every refusal was reported to the server as a dead letter, which quarantined the row. A completed item is now settled at the gate, and the server quarantines only a command that no result from that node names. | `.4.4.8`, closed |
| A node never marks its own dead-letter report delivered | The node records the report in its outgoing journal and sends it, and never acknowledges it there, as it does a result. The record stays pending until the next reconciliation, at start-up or after a channel error, which sends it again under its original id or finds it already known. | `.4.4.11` |

⚠️ Two source-record claims were narrowed rather than confirmed: an unknown
provider outcome does leave the worker as an error, but the type documents that
as deliberate and what a caller does next was not measured here.

## Can this project still state its own release-gate position?

Five gate records exist — for G1–G2, G3, G4, G5 and G6–G7 — and between them they
count **35 verdict claims and 19 named deferrals**. Every one of those counts was
taken *before* the full source review.

⛔ **The records are not dishonest.** Each claims the evidence its test suites
carried at the time, and that is true. What none of them could know is whether
those suites *covered* the paths a later reading would find. That is the same
defect as a test whose name promises more than it exercises — one level up, at a
release gate.

The clearest case is G6–G7's line (2), *authenticated enrollment, rotation,
revocation and tenant-isolation tests*. Since it was counted as shipped, the
review has reproduced — live, against supported surfaces — five separate
cross-tenant defects inside that exact subject. All are repaired. The line has
not been re-counted.

**All five records have now been re-derived**, line by line, each verdict
carrying the command that produces it — G1–G2, G3, G4, G5 and G6–G7 — and
blocker C2 is closed. ⛔ No record's conclusion changed: G6–G7 remains *not met*
for Internet exposure and G3 remains blocked as binding use.

Of G6–G7's seven shipped lines, **three must be re-earned, two are narrowed and
two stand** — the [blockers page](blockers.md) carries the table, and the reasoning
is in `docs/decisions/2026-09-15_g6g7-shipped-lines-re-derived.md`. G3's seven
claims came out **three standing, one narrowed and three re-earned**; G4 must be
re-earned and G5's three stand. G4's last un-discharged strand — the R3 pack
advertising a `vm_container` sandbox it does not provide — is closed by the
correction described under [what a pack advertises](deployment.md#what-a-pack-advertises-and-what-is-actually-behind-each-line).

⛔ One of the seven turned out to be an **open defect** rather than a repaired
one. The product has two resource-acquisition packs, and they follow HTTP
redirects in opposite ways. The web fetcher refuses automatic redirects and
re-checks the destination at every hop. The Git pack checks the **first**
destination before it connects — including one written as a bare IP address — and
then follows up to five redirects automatically, with only a name-resolution hook
behind them. That hook never runs for a bare IP address, which the underlying
library states plainly in its own source.

So the gap was the **redirect hops alone**: a request the caller makes directly is
checked, and a hop that an origin sends it to was not. The exposure was therefore a
Git origin that redirects into a private address range, rather than a caller
naming one. The first published description of it overstated the gap by omitting
the pre-flight check, and was corrected the same day.

✅ **This one is repaired.** The Git pack's redirect decision now applies the same
destination policy the pre-flight applies, so a hop written as a bare IP address
is classified before anything connects, and a refused hop is named — the address
and the class it belongs to — instead of failing as a generic transport error. A
hop the policy allows is still followed, and the five-hop cap moved into the same
decision rather than being lost with the policy it replaced.

⚠️ **It was reproduced before it was repaired, and the reproduction is the part
worth trusting.** A local origin redirects to `0.0.0.0`, which classifies as a
reserved address and which the operating system routes to the local host — so the
same origin answers the redirected request and counts the hit. Against the
unrepaired client that counter read **one**: the refused destination was dialed and
replied. Against the repaired one it reads zero and the caller gets a named
refusal. A control that only checked for an error could not tell a classification
apart from a connection that happened to fail.

The same repair gave the name-resolution hook a voice: a host whose every address
is refused now says so, rather than returning an empty address list that surfaced
as an ordinary connection failure. That was never a safety gap — it failed closed —
but an operator could not tell it from an origin being down.

**A gate record's "shipped" count is a claim about the evidence available on its
date, not a current statement.** That was written while four of the five were
still un-re-derived; it stays because it remains the right way to read a dated
record, and because the re-derivations confirmed it — across the five, more
lines had to be re-earned or narrowed than stood unchanged.

### Two things the census found before re-deriving anything

| Finding | Effect |
| --- | --- |
| The **G1–G2 record undercounts its own deferrals** | It states *five named deferrals* twice; its own table lists **six**, and the sixth row shipped in the same commit as the sentence — so the number was wrong when written rather than overtaken. A deferral is a named limitation with a revisit trigger, so an undercount is one limitation not being carried forward. This manual repeated the smaller number and is now corrected, with a note on the [roadmap page](roadmap.md). The record itself is a dated decision and is superseded rather than edited |
| A **gate cannot see the claim it was built to stop** | The check that guards against the superseded visibility instruction anchors all four of its sentence shapes on the full word *repository*. Two live sentences use the abbreviation instead and are invisible to it. Both are correct history — written before the visibility correction — which is exactly why nothing ever failed and the gap survived. The risk is a *new* sentence written the same way. ⭐ The check then proved precise by refusing **this page** while it was being written, twice: it fires correctly on every shape it knows |

The fourth group's second part read eight more records — the seven whose
narrowest candidate is the delegation leaf, plus one folded singleton — and its
finding is about that leaf rather than about the records. `SIGNOFF-REPAIR.3.4` is
open, all five of its sub-tasks are finished, and **two of the things its own
description promises have no sub-task at all**: whether a delegated subject
consents, and binding a command's replay key to the thread it acts on. The
description names both in so many words. What hid them is arithmetic nobody
checked — the sentence recording the split says it drew "five children along the
four mechanisms the goal line names", and the goal line names five. Both now have
owners, and the rule the case earns is written where the next split will meet it:
count what the description promises against what the split delivered, and make
the sentence recording it reconcile.

That reading added five limitations, and the last of them is about this manual:

| Limitation | Effect | Owner |
| --- | --- | --- |
| ✅ **Repaired** (`.5.2`): a role could start an automatic thread exactly **once per tenant, for ever** | The node-initiated creation derived its idempotency key from the role and the tenant alone, so a second automatic initiation replayed the first thread or was refused as a key conflict, and the covering test never reached a second legitimate initiation. Each initiation now carries its own required key, and a per-initiator quota bounds repeats. | `.5.2`, closed |
| ✅ **Repaired** (`.7.2.4`): the Git acquisition opened its repository with the library's **default** permissions, over an untrusted remote | The operator's global and system Git configuration and the Git environment variables were honoured while fetching a caller-supplied URL, against §12.5's default refusal of hooks, filters and alternates. The acquisition now opens its repository in the library's isolated mode, so none of them reaches it. Separate from the repaired question of where the acquisition WRITES. | `.7.2.4`, closed |
| Evidence quarantine **does not exist**, in either direction | The snapshot table has carried a quarantine column since it was created, and no code anywhere reads or writes it. The reason code that would report a quarantined snapshot, `evidence_quarantined`, is likewise never emitted — it is already listed on the [errors page](errors.md) among the registered codes this build never sends, and nothing had connected the two facts. | `.7.4` |
| ✅ **Repaired** (`.7.4.6`): a citation could be validated against a **tombstoned** snapshot | Deleting a snapshot stamps the row rather than removing it, which is the intended honesty. The citation check and the derivation parent check both joined that table without excluding stamped rows, so a deleted snapshot still supported a new assessment, and a re-acquisition of its bytes replayed onto it. Both checks now refuse a tombstoned row by name, and a re-acquisition is a new row. | `.7.4.6` |
| ✅ **Repaired** (`.3.3.4.12.2`): three superseded federation services remained, and **this manual contradicted itself about them** | The three direction verbs had moved onto the guarded, audited shape and the unguarded originals were left in place with no caller, while the [authority chapter](authority.md) said both that a card import is fenced by a revocation from either side and that nothing here could fence it. The originals are deleted and the chapter says one thing. | `.3.3.4.12.2`, closed |

Two further claims in the records were checked against the state of the code
when the review was written, and both turn out to have been **accurate then and
overtaken since** — a distinction worth keeping, because it separates a stale
finding from a mistaken one. A third was narrowed rather than confirmed: a test
named for expired *and revoked* grants exercises only expiry, which is exactly
true, but the revoked case is covered elsewhere, so what is wrong is the name
rather than the coverage.

The fourth group of records opened with a correction to this review's own
published figure. A row recorded that a test fixture accepts an unbound
adjudication digest "in four places"; counted with a command it is seven, across
seven separate tests, and the file has not changed since the row was written. The
row and the leaf that carries it are corrected, and the finding is larger than it
was published as. The first eight of that group added these limitations:

| Limitation | Effect | Owner |
| --- | --- | --- |
| ✅ **Repaired** (`.9.3.1`, `.9.3.4`, `.9.3.3.7`): citing an authority was the same as holding one | Recording a policy correction checked only that the named grant existed and was active, so anyone enrolled who knew a grant's identifier could suspend, retract or waive a publication in its name; and assigning a publication to a target checked no authority over the target at all, so one organization could set what another's target should run. A correction now requires a live grant the caller holds, from its valid-from time, that covers `policy_correction_record`; an assignment requires holding the target's own authority, and anyone else is refused `403`. ⚠️ A held grant still reaches every publication of its tenant, because no selector can name one publication (accepted, `.9.3.4`). | `.9.3.1`, `.9.3.4`, `.9.3.3.7` |
| ✅ **Repaired** (`.9.3.2`): a policy review could only ever happen once per publication and trigger | The review's identifier was derived from the pair and was the table's primary key, so once the first review was marked done, every later attempt collided and the error was discarded; the same discard made a storage failure look like "nothing was due". Reviews now have their own ids, recur on each new occurrence, and a failure is a `500`. | `.9.3.2` |
| ✅ **Repaired** (`.9.2.1.1`–`.9.2.1.3`, `.9.3.3.1`): publishing, marking-effective and deploying accepted declared values bound to nothing | A publication was written to any filesystem path the caller named, under enrolment-only authorization; marking a publication effective accepted any strings as its Git object identifiers without looking for them; and a deployment assignment accepted any well-formed digest and any ref without comparing either to the publication. The publish location is now inside one declared root, both publication verbs require an authority the caller holds, each declared object id must exist in the named repository, and an assignment's digest must be its publication's projection digest and its ref one of the ids the publication recorded, refused by name otherwise. ⚠️ A recorded id is proved to exist, not to be this publication's own. | `.9.2.1`, `.9.3.3.1` |
| ✅ **Repaired** (`.9.3.3.2`): any principal of the owning tenant could file a target's receipt | The receipt is what a target says it runs, and the drift comparison's input, and any enrolled principal of the publication's tenant could write it, including the operator who assigned what it should run. A target now names one reporter when it is registered, and only that principal files its receipts. ⚠️ A target registered before this takes no receipt, and a target's reporter cannot yet be named or replaced after registration (deferred, `.9.3.3.2.1`). | `.9.3.3.2` |
| ✅ **Repaired** (`.9.3.3.3`): each receipt erased the one before it | A target's reported state was two columns overwritten in place, so what a target said last week, and who said it, was gone. Every receipt is now its own row, naming its reporter, never rewritten, written together with the assignment's current state, and readable as a history. ⚠️ Receipts filed before this change had already been overwritten and have no history. | `.9.3.3.3` |
| ✅ **Repaired** (`.9.3.3.5`): a drift record's "should be running" half was whatever the caller typed | A drift record says a target is not running what was published, and its desired digest was stored as given, so it could disagree with the assignment it named. It must now be that assignment's desired digest; the observed half remains the observer's report. | `.9.3.3.5` |
| ✅ **Repaired** (`.9.3.3.6`): the deployment and drift records blamed database failures on the caller | Registering a target, assigning a publication, and recording drift, a correction or an outcome answered a database failure as a missing record, an inactive grant or a duplicate, so a caller could re-create or abandon a request that was fine. A database failure is now the server's `500`, only a real uniqueness conflict is a duplicate, and a unit test over both modules, run with the test suite and in CI, refuses the code shapes that discard a database error. | `.9.3.3.6` |
| A deployment wave orders nothing | An assignment's `wave` is a label the caller chooses; nothing holds a later wave back until an earlier one succeeds, and nothing pauses a rollout on failures, because the server does not yet carry out a deployment at all. The book called it a *canary wave* and now says it is a label (`.9.3.3.4`, decided). | `.9.3.3.4.1` |
| ✅ **Repaired and decided** (`.11.12`): the server changed the database before it validated the configuration it refuses to start without | An undeclared secret-store profile refused the boot only after the migrations had run, and binding every interface was accepted while the startup line reported the development profile. The secret-store profile and the bind address are now checked before the database is touched. Binding every interface is still accepted, by decision (`docs/decisions/2026-09-16_rb-server-bind-exposure.md`), for a trusted local network, and the startup line says it is reachable from every interface; gating exposure to the Internet belongs to G6. | `.11.12`, closed |

Every limitation listed above now has a leaf that can be finished, which was
not true when they were first recorded. Reviewing that claim directly: of the ten
leaves the earlier entries named as owner, eight had no acceptance criteria of
their own and six had no sub-tasks — they named the right area and left the work
undescribed. Eight of them have since been broken into eleven bounded leaves,
each stating how to reproduce the defect, what it owns, and what must be
observed — including a control that must be seen to fail against the current
code before any repair is accepted.

Two further repairs have landed since. The first: answering a recruitment call
was not bound to the call's own tenant, so a participant enrolled in one
organisation could decline or recuse themselves from another organisation's
call. The finding named the tool surface; the plain HTTP route turned out to
share the same code and to be equally open, which is why the check went into
the shared path — a fix at the tool would have left the route open while every
test in view still passed. The same change corrected a description that had
claimed all three write tools carry a per-action permission check and an audit
record: one of them does, and the sentence has been replaced by a per-tool
account of what each actually brings.

The second: Naming an authority was treated as holding
one: any enrolled principal who could name an active grant could record a policy
retraction under it, register a deployment target owned by it, or file an
approval as another person — and grant identifiers are derived from principal
identifiers, so naming one takes nothing but an identifier the caller has seen.
Taking the whole family in one search found five places asking that question in
four different spellings, a third affected surface the repair's own leaf had not
named, and one thing none of the five did: check whether the grant had begun. A
grant scheduled to start tomorrow authorised everything today. All five now ask
one shared predicate, and the three surfaces where a caller cites an authority
require that caller to hold it. One part of that repair's stated goal could not
be met and says so: the system has no way to name a publication or a deployment
target as the thing an authority is *over*, so "this grant covers this target"
has nothing to compare. That design question is now `SIGNOFF-REPAIR.9.3.4`.

The work is ordered by severity rather than by the order it was found. The
cross-tenant read on the MCP tools came first and is repaired: the three read
tools now call the HTTP handlers' own authorization instead of a private copy
that omitted it, and a fourteen-leg live control — observed failing on eight
legs against the unrepaired code, and made to fail again one gate at a time
afterwards — holds it there. Next is the authority check that accepts any grant
identifier a caller can name, then the call-response path that binds the caller
to one tenant and the call to none, then the table gate that disagrees with the
renderer, then the publish verbs that accept a filesystem path and a set of
object identifiers from the caller.

⛔ One consequence is stated plainly because it is visible on this page's own
neighbour: the doctrine reference still publishes one table row whose last cell
is lost, because that row is the only real-world example left to test the
corrected parser against. The leaf that owns the parser owns repairing the row in
the same change.

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
supply-chain scanners pass as real gates. This is a **local** qualification: the
remote gate has not observed this source.

⚠️ **Two clauses of that sentence were stale, and are corrected here rather than
deleted.** It read *"remote CI has never run … name clearance and the license
decision remain open"*. Both halves stopped being true while this paragraph
stood:

| The clause | What is true |
| --- | --- |
| *remote CI has never run* | **False, and was never true.** 41 runs, 29 success / 12 failure, all three workflows green at `c17841c` — which is `origin/main`. The real limit is the commits sitting beyond the last remotely-gated one, inside the documented push cadence. [Blockers](blockers.md) carries the measurement |
| *the license decision remains open* | **Cleared 2026-09-15.** `LICENSE-MIT` and `LICENSE-APACHE` ship at the repository root, carrying the texts the 13 manifests already declared |

What genuinely remains open is **G6/G7 Internet exposure**, **name clearance**
(ADR-001), and the historical phase closures under corrective review. See
`docs/tasks/artifacts/signoff_review/checkpoint-7233122.md` — the dated artifact
is left byte-unchanged, because it records what was measured on its date.
