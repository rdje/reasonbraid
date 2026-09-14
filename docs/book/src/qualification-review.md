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
| Authority and administration | Foreign grant/boundary mutation and shared site-registry authority are corrected with matched controls. Core subject serialization is reproduced and corrected with direct/enclosing and live compatibility controls; bound evaluation passes core/evaluator controls, 32 live authority/command API tests and strict lint. Actual-parent command selection passes 37 live tests, six evaluator controls and strict lint; frozen-read eligibility passes 40 live tests, ten pure controls and strict lint. Provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all results consumed and the owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint, including audit/query failure recovery; all results and shutdown consumed. Exact scoped receipt lookup passes 18 live authority tests and 30 HTTP tests with strict lint; all results/shutdown consumed. The `.3.3.4.1` census maps guard/effect integration children. `.3.3.4.2` qualifies the primitive guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass. Cancelled-BEGIN pooling and a stale embedded-migration executable are reproduced and repaired; all results/shutdown consumed and owned clusters/probes removed. Seven checker controls also correct nested-evidence owner selection, with broader doctrine accuracy still open. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. Bounded HTTP waits and restart qualification follow; ordinary no-key behavior remains intentionally distinct. The final administrative effect representation is defined under `.3.3.4.7.1`: a closed set of fourteen operations, each carrying a caller-supplied target that exists at refusal as well as at success, and a closed `applied`/`no_op`/`refused` outcome whose refusal names one of three codes. `.3.3.4.7.3` corrects that last field: `.7.1` typed it against the §9.8 registry, and its first consumer found the registry has no `not_found` — a census showing 10 of the 19 codes the product emits are absent from it, routed out as `.11.7`. The three are what the fourteen administrative handlers actually refuse an admitted operation with. The fourteen were measured from the 27 guarded-admission call sites rather than chosen, and the four other tenant_admin mutations are deliberately outside it because other leaves own their gates. 10 representation controls, 68 core tests and strict core lint pass; the object-only control is falsified against a permissive decoder. `.3.3.4.7.2` then gives it durable storage: migration 0058's `administrative_effects`, keyed by the admission's own id, with a composite foreign key so an effect cannot cite another tenant's admission; a writer that runs on the caller's already-guarded transaction so evidence and mutation share one commit and an evidence failure rolls the protected write back; and an exact own-tenant reader that filters before it decodes. 8 live controls pass, and the fixture-plan blast radius (26 plans) was censused before the constraint was written. `.3.3.4.8` then makes grant and boundary revocation its FIRST producer: one exclusive-guard transaction holding the admission, the tenant-bound target selection, the status change, the epoch bump and the effect record, with the two superseded revocation services deleted rather than left as a second unordered path. 16 live controls pass; falsified 11/5, where restoring the pre-`.8` two-transaction shape lets a revocation APPLY after the caller's own administration ended while it queued (`left: 200, right: 403`). Two documented wire changes: the reason gains a 1 024-byte/control-character contract, and both responses carry an `x-reasonbraid-authorization` receipt. `.3.3.4.9` adds the SECOND family, spend-breaker arm and reset — previously the weakest administrative path, admitting under a shared guard and then mutating **on the pool** with no transaction, no guard and no record. Both verbs now run one exclusive-guard transaction; the exclusive mode is measured rather than copied, because classifying an arm against a row that may not exist cannot be done under a row lock, and because the reservation path holds the shared guard. Status codes, messages and success bodies are byte-unchanged; the one addition is the receipt header. The `409` that has always collapsed "no breaker armed" and "armed but untripped" still does, and the effect record distinguishes them as `refused`/`invalid_transition` against `no_op`. Neither verb takes a reason, so neither invents one, and neither advances the revocation epoch. 25 live controls pass (16 from `.8` plus 9 new); the affected set passes 5 suites / 94 tests. Falsified against the exact pre-`.9` handlers: **17 passed / 8 failed**, where 8 of the 9 new controls go red and an arm APPLIES while another operation holds the tenant's authority guard. ⚠️ That falsification also corrected the controls themselves: an EXCLUSIVE-holder fixture could not discriminate the repair, because the superseded admission took a SHARED guard and waited behind it too — the shared-holder fence is what turns the ordering control red. `.3.3.4.10` then censused the five node administrative routes before splitting them, and measured that **four of the five mutations carry no tenant predicate at all** — three inbox verbs and the certificate revocation select rows by `node_id` alone, the latter's tenant-bound existence probe being a separate pool query outside the mutation's transaction. That finding is annotated at `.3.5`, which owns real target ownership; `.3.3.4.10` owns putting the verification inside the mutating transaction. Its first child `.10.1` puts enrollment-token issuance onto the shape: one transaction under the tenant's **shared** guard — shared rather than exclusive because the refusal is decided atomically by one insert that does nothing on conflict, same-node contention is already settled by the partial unique index, and issuance touches nothing the reservation path reads. 11 live controls pass; the affected set passes 6 suites / 84 tests. Falsified 7/4 against the exact pre-`.10.1` handler, where a token COMMITS although its evidence could not be written (`left: 200, right: 500`). ⚠️ A shared-guard repair cannot be falsified by a lock-holding fixture in either mode, because the superseded admission took the shared guard too; the discriminator is atomicity, and the ordering property is labelled a regression control rather than presented as proof. ⚠️ Reproduced and routed to `.3.5`, not introduced here: the one-unused-token index is global rather than per tenant, so one tenant's outstanding token both reveals itself to, and blocks, another tenant's administrator for the same node identity. `.10.2` then puts node certificate revocation onto the shape, taking the **exclusive** guard — the opposite answer from `.10.1` one commit earlier, and derived: this advances the tenant's revocation epoch, so it is a revocation in the sense the guard contract means. Its superseded shape ran the tenant-bound existence probe as a separate pool query and then mutated on `node_id` alone, so the check and the act read two different snapshots; both are now one transaction with the tenant predicate in the writing statement. The reason is persisted rather than discarded and gains the 1 024-byte/control-character bounds; `revoked_at` is the transaction's own database time; the single 409 now distinguishes a repeat (`no_op`) from a node that never had a certificate (`refused`) in the record only. 30 live controls pass; the affected set passes 5 suites / 74 tests. Falsified 25/5 against the exact pre-`.10.2` handler, where the request COMPLETED while a shared holder held the tenant's guard. `.10.3` closes the parent by putting the three inbox verbs on the shape, and it is the most consequential of the three children: `node_inbox` has carried a `tenant_id` column since migration 0003 and **none of the three verbs used it**. A probe run before the repair measured an administrator of one tenant quarantining (200) and replaying (200) another tenant's inbox row, and pruning its inbox with `{"deleted":2,"before":2,"after":0}` — destroying both of the other tenant's rows. Three of the project's own fixtures depended on that defect, seeding into one tenant and administering from another, and are corrected rather than deleted with every original feature assertion intact. All three verbs now select bound to the admitted tenant under the shared guard, with the prune's before/after counts bound too, and each records its outcome. 7 live controls pass; the affected set passes 6 suites / 72 tests. Falsified 3/4 against the exact pre-`.10.3` handlers, where the foreign row comes back carrying a fresh `quarantined_at` and the prune deletes two rows while reporting success with unwritable evidence. Effect auditing for `.11` and `.12` and other caller/target paths remain. | `.3.1`–`.3.5` |
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
| `GET /v1/nodes/inbox` admitted on the caller's tenant and then selected the inbox by node id alone | **Reproduced and repaired.** A tenant administrator read another tenant's inbox rows — command ids, thread ids, delivery state and payloads — for a node id they could name. The three inbox *mutations* were bound to their tenant under `.3.3.4.10.3`; this read was outside that census's scope. The select now carries the admitted tenant. | `.3.5.3`, closed |
| Three doctrine enforcers write their scratch to an ambient temporary directory | Project-owned temporary data lands off the repository volume, against §13. The storage-locality gate enumerates Rust sources only, so it cannot see them. | `.11.2.2` |

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
| `POST /v1/resolvers` admits on tenant administration and writes a site-global registry | Any tenant's administrator can register — or, through the upsert's single-column key, replace — a resolver row, including a built-in pack's declared schemes, egress class, sandbox level and security evidence. The site-operator repair pinned a closed set of six actions that does not include it. | `.7.1` |
| `POST /v1/workflow-profiles` admits on enrolment alone | Any enrolled principal can register a new version of any profile id, and resolution takes the highest version — so a built-in deliberation profile can be shadowed for the whole site. | `.8.1` |
| The publication reconciler never reads the effective channel ref | Its `effective` case compares only the immutable publication ref, so the §15.8 row "effective / ref missing or moved → freeze deployment" cannot fire for a moved effective channel. A unit test encodes the gap. | `.9.2` |
| A decision-family close consults only the caller's own unresolved list | The durable open-challenge count the engine maintains is never read at close, so omitting a challenge closes the thread with a decision outcome. | `.8.1` |
| `regions::route` reports any database error as an undeclared region | **Reproduced and repaired.** With its reads failing, the routing answered ``the region `dev-local` is undeclared`` for a region the same run proved declared. The decision now returns a typed storage failure that names neither the region nor the word "undeclared", while every existing refusal keeps its exact wording. | `.11.10`, closed |

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
| An unvalidated host claim reaches the certificate library's `expect` | **Reproduced and repaired.** Driven through the supported routes, a non-ASCII host claim was accepted at issuance and panicked at redemption: the node received a dropped connection rather than an answer, no node row was written, and the token was left unconsumed — so that node id could not be enrolled until the token lapsed. The claim is now checked at issuance, where an operator typed it, and both later paths answer with a typed refusal. ⚠️ The accepted set is deliberately unchanged: the check asks the certificate library, and whether a stricter grammar should bind is an open question with a compatibility cost. | `.4.1.6`, closed |
| One certification run asserts three of the six named invariants | The conformance box it writes reads "the six §19.4 invariants passed". Its completion invariant also accepts a failed terminal as completion. | `.10.2` |
| The certificate authority is signed for a year, with no renewal and no check at load | A deployment that has run a year issues leaves from an expired issuer, and a leaf is never compared against its issuer's expiry. | `.4.1` |
| The visibility policy's documented default and its actual default disagree | The documentation says every profile field defaults to self-only; the default publishes eleven of fourteen to the tenant or the network, and a profile that names no policy takes it. | `.5.1` |
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
| Seven of the nine doctrine checks that read the staged file list then open the worktree | Staged evidence can be satisfied by text that was never staged, including for the ownership check. | `.11.2` |
| The document-path check cannot see this repository's own checkout prefix | It matches `/Users` and `/home` only; this checkout is under `/Volumes`, so the one absolute path this project would leak is the one the gate is blind to. | `.11.2` |
| The handoff background-job census reports success when it fails | Both process censuses discard their errors and the script does not exit on error, so an unavailable census renders as "no job running". Its own `ps` arm also covers every user, where its documentation claims one. | `.11.2` |
| The scaffold updater would overwrite the task-tree index | It lists that file as project-neutral while the file holds this project's index of active trees, and its own header promises the opposite. A local donor path is copied whole, build artifacts and caches included. | `.11.2` |
| The template bootstrap treats the presence of one file as proof of a pristine copy | In a copy that still holds it, a named invocation overwrites the resume pointer, the decision index and the tree index. Its in-place edits are also written in a dialect a stock macOS `sed` rejects — mid-way through the destructive sequence. | `.11.2` |
| The demonstration's negative controls pass when the command fails | Each runs a negation over a pipeline in a fresh shell, so a failing CLI and a genuine absence are indistinguishable. Its evidence-capturing requests also ignore HTTP status. | `.11.3` |
| The load harness reports success for a run that issued nothing | A zero or negative command count passes both exit gates having sent no requests, and a positive count is rounded up per worker, so the published total can exceed the one requested. | `.11.3` |
| The one-command development environment builds the caller's repository | It never changes to its own root before building, yet runs its own binary, and outside `make dev` it writes its build cache off the repository volume. Its teardown removes the cluster whether or not the database stopped, and reports success either way. | `.11.3` |

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
| A node's work is looked up by operation id alone, with the node it belongs to absent from the query | The reconciliation the handshake performs returns another node's event id for an operation id it names; a receipt written first under a known event id silently swallows the rightful node's own; and an inbox row is marked consumed by a different node's result carrying the same command id. Three places, two source records, one shape. | `.4.3` |
| The inbox cursor's high-water mark is derived from the rows pruning deletes | Prune everything and the mark returns to zero, so the next item re-uses a cursor the node has already acknowledged and discards as seen. The existing test leaves a partial prefix that keeps the highest cursor, so the case never arises in it. | `.4.3` |
| A review trigger named "repeated waiver" fires on the first waiver, and on waivers that expired long ago | Neither recording a waiver nor scheduling a review consults its expiry, so one bounded exception schedules a review for ever. A test records exactly one waiver and asserts the trigger fires. | `.9.3` |
| A node with no usable certificate does not read as suspended | The presence view treats an unrevoked certificate as active without consulting its expiry, so a single expired leaf holds the node out of the suspended state. | `.4.1` |

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
| All three MCP read tools return data the caller is not entitled to, while the module documents the opposite | The policy-bundle tool ignores both the caller and the tenant — its query has no tenant restriction at all — and returns every tenant's policy documents and clauses, labelled with the caller's own tenant. The inbox tool declares a caller argument and never reads it. The thread tool computes the correct reader class for a foreign tenant and then returns the full projection anyway, reporting that class beside it. The module's opening paragraph states that every read runs the same authorization as the HTTP handlers. | `.6.1` |
| The MCP call-response tool checks the caller against one tenant and the call against none | A caller admitted on their own tenant can record a response on another tenant's recruitment call, and a decline, recommendation or recusal skips the eligibility check entirely. | `.6.1` |
| Registering a policy accepts an expired authority grant | Registration checks the grant's status only, while resolution in the same module also checks its expiry — so a lapsed grant still registers a policy version. | `.9.1` |
| A policy's applicability selector defaults to the wildcard | A missing or malformed selector field is read as "matches everything", which is fail-open in the place the design requires fail-closed. Policies in draft, superseded and deprecated states also remain applicable. | `.9.1` |

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
| Each provider adapter can panic while reporting a failure | The stderr excerpt attached to a failure reason is cut at a byte offset rather than a character boundary, so a child that writes non-ASCII can crash the adapter on the path that was already handling an error. | `.10.1` |
| Each provider adapter keeps every child it has ever started | The map of running children is added to and never removed from, so it grows by one unreaped process for the life of the server. The successful path also returns without waiting for the child or its output drain, while the failing path does both. | `.10.1` |
| A per-attempt deadline is sent and never enforced | The value is computed, placed on the dispatch and carried in the contract; no adapter and no supervisor reads it. | `.4.4` |
| A completed item that reaches the retry gate is quarantined as a dead letter | The gate refuses every state it does not name individually, completion included, and every refusal is reported to the server as a dead letter, which quarantines the row. The report itself never marks its own outgoing record as delivered. | `.4.4` |

⚠️ Two source-record claims were narrowed rather than confirmed: an unknown
provider outcome does leave the worker as an error, but the type documents that
as deliberate and what a caller does next was not measured here.

The fourth group of records opened with a correction to this review's own
published figure. A row recorded that a test fixture accepts an unbound
adjudication digest "in four places"; counted with a command it is seven, across
seven separate tests, and the file has not changed since the row was written. The
row and the leaf that carries it are corrected, and the finding is larger than it
was published as. The first eight of that group added these limitations:

| Limitation | Effect | Owner |
| --- | --- | --- |
| Citing an authority is the same as holding one | Recording a policy correction checks only that the named grant is active and unexpired — never the caller, the action, the selector, the boundary, the publication, or when the grant became valid — behind an endpoint that admits anyone enrolled. Any enrolled principal who knows an active grant's identifier can suspend, retract or waive a publication in its name. The deployment assignment path repeats the shape. | `.9.3` |
| A policy review can only ever happen once per publication and trigger | The review's identifier is derived from the pair and is the table's primary key, so once the first review is marked done, every later attempt collides and the error is discarded. The same discarded error makes a storage failure look like "nothing was due". | `.9.3` |
| Publishing and marking-effective accept declared values bound to nothing | A publication is written to any filesystem path the caller names, under enrolment-only authorization; a deployment assignment accepts any well-formed digest without comparing it to the publication; and marking a publication effective accepts any strings as its Git object identifiers without looking for them. | `.9.2`, `.9.3` |
| The server changes the database before it validates the configuration it refuses to start without | An undeclared secret-store profile refuses the boot — after the migrations have already run. The bind address is also ungated: binding every interface is accepted while the startup line still reports the development profile. | `.11.12` |

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
