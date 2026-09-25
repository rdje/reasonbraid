# LIVE_STATUS.md — authoritative live progress tracker

Rows use only **Done · Mostly Done · In Progress · Not Started**. This is a current
snapshot. Historical implementation and verification records live in the phase
task-trees and git; the pre-review snapshot is `9c2d2ba:LIVE_STATUS.md`.

## Current status

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
| Phase 7 — Internet qualification | Mostly Done | Hardening machinery exists; G6/G7 Internet exposure remains NOT MET. The director greenlit Internet exposure over HTTPS with OAuth on 2026-09-25, security first; owner-only first (the director and their own agents), gated by our own evidence; opening to anyone else still needs an independent threat-model review and penetration test (`docs/runbooks/external-security-review.md`). Local repairs and external threat-model, injection and penetration-test evidence remain required. |
| Phase 8 — federation and interoperability | In Progress | Through regional routing historically recorded; `.5.3` store-and-forward, `.5.4` exit export/import and `.6` G8 remain. Shared authority and protocol gaps are prerequisite repairs. |
| Phase 9 — stable release | Not Started | G9 requires sustained operational evidence and the outstanding release decisions. |
| Corrective review | In Progress | `.1`, `.2.1`, `.2.2`, `.3.1` complete; `.3.2.1` site service passes ten live controls and strict lint. `.3.2.2` operator CLI passes 3 live controls plus adjacent service/ownership checks and strict lint; `.3.2.3` HTTP enforcement passes 8 focused controls; the selected 58-test security run and final 12-test registry run pass. `.3.3.1` core subject serialization passes 49 unit + 3 subject controls and 40 live compatibility tests; `.3.3.2` bound evaluation passes 51 core unit + 3 subject tests, 6 evaluator controls, 32 live authority/command API tests and strict lint; all results consumed and the owned cluster removed. `.3.3.3.1` actual-parent command selection passes 37 live tests, six evaluator controls and strict lint. `.3.3.3.2.1` frozen-read eligibility passes 40 live tests, ten pure evaluator controls and strict lint; all results consumed and the owned cluster removed. `.3.3.3.2.2.1` provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all confirmation/shutdown results consumed, owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint; all results and shutdown consumed. Exact scoped receipt readback passes 18 live authority tests and 30 HTTP tests plus strict lint, including final denied-record readback; all results/shutdown consumed. Actual-parent selection and frozen-read audit/readback `.3.3.3` are complete; The `.3.3.4.1` source census/contract is complete (42 named-call locations plus transitive/mutation coverage). `.3.3.4.2` qualifies the guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass, including cancelled-BEGIN replacement and the formerly stale authority executable. Seven acceptance-checker controls also pass, correcting nested-evidence owner selection while preserving real-tree box refusals. All results/shutdown consumed; owned clusters/probes removed. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. Bounded HTTP waits and restart qualification follow; ordinary no-key behavior remains intentionally distinct. The final administrative effect REPRESENTATION `.3.3.4.7.1` passes 10 representation controls within 68 core tests, strict core lint and the falsified object-only control; its census derives the closed fourteen-operation set from 27 guarded-admission call sites. Its STORAGE `.3.3.4.7.2` then passes 8 live controls (falsified 5/3 against three attributable injections): migration 0058 keyed on the admission with a composite tenant-binding foreign key, a writer on the caller's already-guarded transaction, and a tenant-filtered reader. `.3.3.4.8` is the first producer of an effect record; `.3.3.4.9` is the second, putting spend-breaker arm/reset onto one exclusive-guard transaction — 25 live controls, the affected set 5 suites / 94 tests, falsified 17/8 against the exact pre-`.9` handlers.. Fixture ownership `.11.2.1.1` censuses all 61 `create_dir_all` sites in tracked Rust with 0 unresolved and repairs the 3 that adopted: exclusive per-call creation on the repository volume, falsified against an occupied path, with 89 selected controls across five suites (63 of them `profiles` on a disposable cluster), strict lint for both crates and the 25-check gate green. |

## 2026-09-25 — A prompt can no longer be read as a Codex command-line option (`SIGNOFF-REPAIR.10.1.1`)

`REASONBRAID-REPAIR-0509`.

- 🔴 **Before:** the Codex connection handed the prompt to the Codex program without the usual "end of options" marker (`--`). A prompt starting with `-`, which can come from any participant, could therefore be read by the program as an option, placed right after the setting that keeps it read-only.
- ✅ **Now:** the marker is there, as it already was for Claude. A project check refuses any change that removes it, or that adds a second one (which would quietly switch the read-only setting off).
- ✅ Tested: the check failed on the old code and passes now; a deliberately broken version that first slipped through led to the stricter rule.

## 2026-09-25 — The Claude and Codex connections checked point by point: three hold, four to fix, one deferred (`SIGNOFF-REPAIR.10.1`)

`REASONBRAID-DOC-0179`.

- Checked against the code: the overall output limit, the "never started" failure, and the Claude connection's handling of the prompt all hold.
- 🔴 To fix, in this order: the Codex connection hands the prompt to the Codex program in a way that lets text starting with `-` be read as a program option (new, and the most serious); its output reading has no size limit, can crash on some non-English text, and can stall; finished programs are never cleaned up; and a cancelled request is recorded as a definite failure.
- ⏸️ Deferred: checking the token counts against a real provider receipt, which needs a live run.

## 2026-09-25 — Policy error messages say what actually happened, and policy registration's review is complete (`SIGNOFF-REPAIR.9.1.7`)

`REASONBRAID-REPAIR-0508`.

- 🔴 **Before:** several policy error messages said the opposite of the truth. A policy that did not exist was reported as "not registered … already exists". When the database itself failed, the server answered as if the request were wrong ("not registered"), instead of saying it had a problem. Version numbers were called "semantic" but accepted `1` and refused `1.2.3-rc.1`.
- ✅ **Now:** each refusal has its own accurate message, a database failure is reported as the server's own error, and version numbers follow the real Semantic Versioning standard.
- ✅ **Policy registration and authority (`.9.1`) is closed:** all six problems its review found are fixed; the deeper parts that matter only once policies are used as binding rules are scheduled for then.
- ✅ Tested: 14 problems shown on the old code; 24 deliberately broken versions caught, including simulated database failures.

## 2026-09-25 — The policy resolver's checks now refuse what they should (`SIGNOFF-REPAIR.9.1.6`)

`REASONBRAID-REPAIR-0507`.

- 🔴 **Before:** when several policies are combined for one target, a rule that needs "version 2 of X" was satisfied by version 1, or by an X that does not even apply there. A loop of three policies each claiming priority over the next was accepted, while the server's own explanation said there was no loop. Naming the same policy twice was reported as a conflict with itself, and two versions of one policy could be combined.
- ✅ **Now:** a dependency must apply at the exact version it names, a priority loop of any length is refused and named, a policy named twice is refused as such, and the shapes of these entries are checked when a policy is registered. The manual gained a section that explains, step by step, how policies are combined.
- ⚖️ Decided: a policy's "draft/active" label is set once and never changes, and a policy is put into force by approval and publication. So the label is shown, not acted on. Making labels change over time is deferred until policies are used as binding rules.
- ✅ Tested: all nine problems were shown on the old code; the fix was checked against 42 deliberately broken versions, all caught.

## 2026-09-25 — A mistyped policy selector is refused instead of matching everything (`SIGNOFF-REPAIR.9.1.5`)

`REASONBRAID-REPAIR-0506`.

- 🔴 **Before:** a policy says where it applies (and where it does not) with small "selectors". A selector with a missing or misspelt part was read as "everywhere", so one typo could apply a rule to every target, or switch it off for every target.
- ✅ **Now:** a selector must name both its layer and its target, with `*` written out where "everything" is meant. Anything else is refused when the policy is registered, and an old stored one that does not make sense stops the resolution with a clear message instead of being guessed at.
- ✅ Tested: the new check failed on the old code and passes now; 17 deliberately broken versions and one more by hand were all caught; the full live suite (554 tests) and strict lint pass.

## 2026-09-25 — The policy lock file is written by the server, not by the requester (`SIGNOFF-REPAIR.9.1.4`)

`REASONBRAID-REPAIR-0505`.

- 🔴 **Before:** a published policy bundle comes with a lock file, a receipt listing which rule versions went into it and who stands behind each. The server copied that receipt from whatever the requester typed, so it could list a rule that was never used, a made-up fingerprint, or someone else's authority. Shown live: a forged receipt was published.
- ✅ **Now:** the server writes the receipt itself, one line per rule the request named, read from the registry. A request that tries to supply its own is refused, and a rule whose stored fingerprint no longer matches its text is never put on a receipt.
- ✅ Tested: both new checks failed on the old code and pass now; four deliberately broken versions were caught; the manual's projection examples now work and are run by a test.

## 2026-09-25 — A policy's fingerprint is now computed by the server (`SIGNOFF-REPAIR.9.1.3`)

`REASONBRAID-REPAIR-0504`.

- 🔴 **Before:** every policy version carries a fingerprint (digest) meant to identify its exact text, but the server stored whatever fingerprint the sender typed. Two different texts could share one, and a fingerprint could match nothing at all.
- ✅ **Now:** the server computes the fingerprint from the text itself. A sender may still include one to pin what they mean, and a wrong one is refused. Every read says whether a stored fingerprint still matches its text, so old entries with a typed-in fingerprint show as unverified.
- ✅ Tested: the new check failed on the old code and passes now, with the expected fingerprints computed independently by the test; the related live suites pass.

## 2026-09-25 — The 175 GB build cache is deleted, on the director's word (`SIGNOFF-REPAIR.11.4.3.1.11`)

`REASONBRAID-DOC-0178`.

- The compiler's cache folder had grown back to 175 GB. The director asked for it to be deleted, since it can always be rebuilt. It is gone, with about 173 GB freed on the disk.
- Checked first: nothing was using it, nothing recorded depends on it, and the folders beside it that hold test evidence are untouched (same size, same file count). The next build starts from scratch and takes about an hour.

## 2026-09-25 — A policy's owner must be someone who holds that authority (`SIGNOFF-REPAIR.9.1.2`)

`REASONBRAID-REPAIR-0503`.

- 🔴 **Before:** an operator allowed to add policies could name anyone else's authority, in any tenant, as the owner of a policy that person never wrote.
- ✅ **Now:** the person registering must hold the authority they name. Naming someone else's, or one that does not exist, is refused with the same message, so nothing leaks about other people's authorities. A policy owned by someone else is registered by that owner.
- ✅ Tested: the new check failed on the old code and passes now; two deliberately broken versions were caught; the full live suite (550 tests) and strict lint pass.

## 2026-09-25 — Policy registration checked point by point: four hold, six problems to fix, one deferred (`SIGNOFF-REPAIR.9.1`)

`REASONBRAID-DOC-0177`.

- Checked against the code: which tenant owns what, whether a policy's named owner is a live authority, and write-once storage all hold.
- 🔴 To fix, in this order: an operator can name someone else's authority as a policy's owner; a policy's fingerprint (digest) is never computed from its text; the published lock file lists whatever the caller typed, not what was resolved (new, and the most concrete); a mistyped selector applies a policy everywhere; draft policies bind and some dependency and precedence checks are shallow; and some refusals say false things, such as a missing policy "already exists".
- ⏸️ Deferred until policies are used as binding rules: checking that an owner's authority covers the target, effective dates, charter-based precedence and applying waivers. The book now says so, and flags the two examples that do not work as written.

## 2026-09-25 — Cloud hosting: qualify one provider first, and one gap the fetch guard would have on Azure (`PARTICIPATION.7`, `SIGNOFF-REPAIR.18`)

`REASONBRAID-DOC-0176`.

- The director asked whether ReasonBraid must be tested on several cloud providers. The recorded view: fully qualify one, keep the deployment portable, and smoke-test a second before claiming portability. The director will open the provider accounts when hosting starts, after the owner-only gate.
- 🔴 Found while answering: the check that stops the server fetching internal addresses would let it reach one Azure platform address (`168.63.129.16`), because that address looks public. It matters only on Azure, so it is logged, owned, and scheduled for when a provider is chosen.

## 2026-09-25 — Test databases left by failed runs are cleared, and the cleanup has a dated record (`SIGNOFF-REPAIR.11.4.3.1.10`)

`REASONBRAID-REPAIR-0502`.

- 🔴 **Before:** every failing database test keeps its throwaway database for inspection, and in three days they had grown from 0.2 GB to 11 GB. The daily cleanup kept no record of when it last ran.
- ✅ **Now:** 210 of those databases, none referenced by any record, were removed by the project's own checking tool, freeing about 10.7 GB. `docs/ARTIFACT_CLEANUP.md` holds the date. The 165 GB compiler cache is left in place: the disk is 80% free, and clearing it forces a cold rebuild of over an hour. That choice is raised with the director.

## 2026-09-25 — The timestamp check runs on every change, and its six open cases are judged (`SIGNOFF-REPAIR.11.31.2`)

`REASONBRAID-REPAIR-0501`.

- 🔴 **Before:** a check that guards against a known timing mismatch (the database keeps microseconds, the program can hold nanoseconds; it once let the budget ledger lend money twice) was never switched on. Six new places had gone unexamined, two of them in the budget ledger.
- ✅ **Now:** all six were examined in the code; none can cause the mismatch. The check runs on every change, and it was shown refusing a missing verdict before being trusted.

## 2026-09-25 — On the Internet, the owner comes first (`PARTICIPATION`, `SIGNOFF-REPAIR.14`)

`REASONBRAID-DOC-0175`.

- 📌 **Decided by the director:** the first Internet version will be used only by the director and their own agents. Opening it to other people comes later, once we are both confident, and with extreme care.
- ⚖️ **What gates each step:** the owner-only version needs every check we can run ourselves, a full test programme, and a proven refusal of anyone else who tries to sign in. Opening to others additionally needs the independent security review and penetration test.

## 2026-09-25 — Two safety checks that ran nowhere now run on every change (`SIGNOFF-REPAIR.7.1.6`)

`REASONBRAID-REPAIR-0500`.

- 🔴 **Before:** two checks that watch which parts of the system are shared between organisations had been written but never switched on. One had drifted without anyone noticing, and the book quoted an out-of-date figure from it.
- ✅ **Now:** both run on every change. The drift was reviewed item by item and none of it was a security problem; the book's figures are corrected.
- 🔎 **Found:** a third unswitched check, about timestamps, is failing at six places, two of them in the budget ledger. That is the next item (`SIGNOFF-REPAIR.11.31.2`).

## 2026-09-25 — Internet access greenlit, with an independent security review still required (`PARTICIPATION`, `SIGNOFF-REPAIR.13.1`)

`REASONBRAID-DOC-0173`.

- 📌 **Decided by the director:** the 64-item inbox limit stands; agents will be able to message each other by name; the web console will do everything the command-line tool does; ReasonBraid will go on the Internet over HTTPS with modern sign-in, with security first and no exceptions.
- ⚖️ **What it changes:** the two parked Internet items are reopened, to resume once the current corrective work is finished.
- ⛔ **What it does not change:** the roadmap's Internet gate still needs an independent review of the threat model and an outside penetration test with serious findings fixed. A new guide, `docs/runbooks/external-security-review.md`, lists what the director does and what the project does, step by step.

## 2026-09-25 — The choice of fetcher no longer depends on storage order (`SIGNOFF-REPAIR.7.1.5`)

`REASONBRAID-REPAIR-0499`.

- 🔴 **Before:** when two fetchers were equally fast on paper, which one fetched a page depended on the order the database happened to return them in, so two identical requests could fetch the same page in different ways.
- ✅ **Now:** a tie is broken by the fetcher's name, so the same request always picks the same fetcher. The book now explains how the choice is made.
- ✅ Tested: the new tests failed on the old code and pass now; deliberate faults were each caught.

## 2026-09-25 — Citing a web address no longer reveals that another organisation cited it (`SIGNOFF-REPAIR.7.1.4.1`)

`REASONBRAID-REPAIR-0498`.

- 🔴 **Before:** when an organisation cited a web address for the first time, the reply said "already seen" if any other organisation had cited it before. That told it something about another organisation's research.
- ✅ **Now:** "already seen" means only that this organisation cited it before. The page is still stored once and shared.
- ✅ Tested: the new test failed on the old code and passes now; an older test that had recorded this as a limit that could not be fixed was updated, because it could be.

## 2026-09-25 — The director's requirements for who can take part are recorded (`PARTICIPATION`)

`REASONBRAID-DOC-0172`.

- 📌 **Asked for:** send any request to an agent even while it is offline, delivered when it returns, like email; tell "away" from "gone for good"; production-grade command-line and web clients for people; and ChatGPT, Claude, Gemini, DeepSeek, Kimi, Qwen, GLM, MiniMax and MiMo agents able to take part fully.
- 🔎 **What exists today:** a waiting inbox per node that only the server can fill; a read-only web console; a tool server that local apps can start, but no Internet-facing one with sign-in; runners for the Claude and Codex command-line tools. The book's roadmap page lists it per requirement and per platform.
- ⚖️ **Decided:** two ways in. A chat app plugs ReasonBraid in as a connector (MCP, which ChatGPT and Claude both accept). An agent that works on its own is run by ReasonBraid through an adapter. The inbox comes first, because a chat app only acts when its person asks.
- ⏭️ **When:** after the corrective work's blocking items, because this widens who can reach the server. The first step, measuring, can run earlier.

## 2026-09-25 — Each organisation's citation of a web address is its own (`SIGNOFF-REPAIR.7.1.4`)

`REASONBRAID-REPAIR-0497`.

- 🔴 **Before:** when two organisations cited the same web address, the first one's description of it (how to fetch it, its type, its purpose, its risk) was used for both. So the first organisation could decide how the second one's citation was fetched, and the second could read the first one's private note about why it cited the page.
- ✅ **Now:** each organisation's description is stored separately and used only for its own citation. The page itself is still shared, so it is fetched and stored once.
- ✅ Tested: the new test failed on the old code (the second organisation saw the first one's description and note) and passes now; six deliberate faults in the new code were each caught.
- 🔎 Next: a small leftover the same test area revealed: the reply to a citation still says whether someone else cited the address first (`SIGNOFF-REPAIR.7.1.4.1`).

## 2026-09-25 — Only a site operator can add a resolver (`SIGNOFF-REPAIR.7.1.3.1`)

`REASONBRAID-REPAIR-0496`.

- 🔴 **Before:** any tenant's administrator could add a resolver to the list the whole server shares, so one customer could put an entry into every other customer's acquisition results.
- ✅ **Now:** adding a resolver is a site-operator act, like the other site-wide lists: it needs a site grant for it, carries a reason, and is audited. A tenant administrator without that grant is refused. Re-registering an existing resolver is refused with an audit record naming it.
- ✅ Tested: the new test failed on the old code (a tenant administrator's registration was accepted) and passes now; the operator path is tested alongside it, and deliberate faults in the new code were each caught.
- 🔎 Found on the way and owned: two census checks nothing runs, and two stale figures in the book (`SIGNOFF-REPAIR.7.1.6`).

## 2026-09-25 — Governance charters can be registered again (`SIGNOFF-REPAIR.9.1.1`)

`REASONBRAID-REPAIR-0495`.

- 🔴 **Before:** registering a governance charter needs a site-operator permission, but that permission had been added to the code without the database change that lets it be stored. No one could hold it, so every charter registration was refused. The book listed it as a normal permission.
- ✅ **Now:** the permission can be granted, and an operator holding it can register a charter while anyone else is refused. A new test lists every site permission from the code and checks the database accepts each one, so this cannot slip again.
- ✅ Tested: the new test failed on the old database, naming exactly this permission, and passes now.

## 2026-09-25 — Closing checks finished: nine items reviewed, seven real problems found (`SIGNOFF-REPAIR.12`)

`REASONBRAID-DOC-0163`, `0165`–`0171`. No code changed.

- ✅ **Closed** after checking the evidence: machine-acquisition safety (`.7.2`), bootstrap waits, and two tooling items.
- ⏸️ **Deferred:** the optional browser pack's storage and process limits, which only matter when that pack is switched on.
- 🔴 **Found, and now queued to fix:** four evidence problems, two evaluation problems and one publication problem (listed in the entries below). These are exactly what "every sub-item done" had hidden.
- ⏭️ **Next:** the must-fix items, starting with the ones that cross between organisations.

## 2026-09-25 — Three finished items formally closed (`SIGNOFF-REPAIR.11.4.5`, `.3.3.4.3.3.3.3.2.3`, `.11.4.3.1.7`)

`REASONBRAID-DOC-0167`–`0169`. Closing checks; no code changed.

- ✅ Each had all its work done and verified, and had simply never been marked closed. Each closure names the commits and the tests that prove it.

## 2026-09-25 — Policy publication checked: one problem found (`SIGNOFF-REPAIR.9.2`)

`REASONBRAID-DOC-0166`. A closing check; no code changed.

- 🔴 **Must be fixed:** the step that marks a policy publication as effective or failed checks the current state and then writes without locking, so two at the same moment can overwrite each other, and a live publication can be turned into a failed one (`.9.2.2`).
- ✅ **Holds:** where a publication is written, which approval and policy it is tied to, the Git checks, the safe update of the live pointer, and the reconciliation between the database and Git.

## 2026-09-25 — Evaluation checked: two problems found (`SIGNOFF-REPAIR.8.2`)

`REASONBRAID-DOC-0165`. A closing check; no code changed.

- 🔴 **Must be fixed:** an evaluation trial accepts the same option listed twice, which silently gives it twice the share of cases (`.8.2.6`). And when a thread is created, its routing decision is logged before the request is authorised, so a refused request still leaves a log entry for a thread that never existed (`.8.2.7`).
- ✅ **Holds:** quality gates refuse missing or non-numeric results and are tied to the right test set, calibration ignores runs that do not belong to it, and only site operators can write evaluation records.

## 2026-09-25 — Checking each fix is now about five times faster, with the same rigour (`SIGNOFF-REPAIR.12.1`)

`REASONBRAID-DOC-0164`. A measured decision; no product code changed.

- ✅ **Measured:** the slowest check deliberately breaks the new code in several ways and confirms a test catches each one. The same ten breaks took **77 minutes** before, and now take **14 minutes**, with all ten still caught. The saving comes from rebuilding only the tests that cover the code, instead of about 50 unrelated test programs each time.
- ⚠️ **A wrong turn, kept on record:** the obvious setting made it slower, 2 hours, because the tool ignored it at the build step. The logs showed why.
- ✅ **The rule:** the full live test run happens on any change to shared foundations, otherwise at least every three fixes, and always before publishing.

## 2026-09-25 — Evidence checked: four real problems found (`SIGNOFF-REPAIR.7.4`)

`REASONBRAID-DOC-0163`. A closing check; no code changed.

- 🔴 **Must be fixed:** evidence that was deleted can still be built on, still supports assessments, and is quietly cited again if the same content is fetched again (`.7.4.6`). Anyone can sign an assessment with someone else's name (`.7.4.7`). Resubmitting an assessment with a different quote silently keeps the first one (`.7.4.8`). Fetching evidence again does not renew its "fresh until" date, although the book says it does (`.7.4.9`).
- ⏸️ **Deferred with a trigger:** evidence quarantine (the book already says it does not exist), unchecked derivation labels, and a write order that can leave an unused stored file behind.
- ✅ **Holds:** a quoted excerpt must really appear in the evidence; deletion uses the server's clock; one organisation cannot delete evidence another relies on.

## 2026-09-25 — The repair work now has a finish line (`SIGNOFF-REPAIR.12`)

`REASONBRAID-DOC-0162`. Decisions taken at the director's request; no product code changed.

- 🔴 **Why:** the repair work was meant to end when every finding was closed, but every review finds new ones: yesterday 17 items were closed and 17 opened, so it never got closer. A first count also said 50 open items when the true number is 65, because it missed an older way items record their state.
- ✅ **The finish line:** the repair work ends at a "bug bar", the rule security and reliability teams use to decide what must be fixed before a release. What must be fixed: anything that lets one organisation touch another's data, money or service; anything that silently loses or corrupts data, budgets, evidence or publications; anything the book claims that the code does not do; and any check that gives a false answer. Everything else is deferred with a written trigger that reopens it, and the book states the limit meanwhile.
- ✅ **The triage of all 65:** 8 close after a quick check, 1 duplicate, 25 must be fixed (plus a new one: counting open items reliably), 12 deferred with triggers, 19 group headings that close with their parts. Every open item now says which it is.
- ⏳ **Faster checking, same rigour, being measured:** the slow step, mutation testing, rebuilds about 50 unrelated test programs for every change it tries. My first attempt to narrow it made it slower (2 hours instead of 77 minutes), because the tool ignored the setting at the build step. A second setting is being measured; the result and the rule go in the next entry.
- 📖 The book's qualification chapter gains "When the corrective work ends", and its out-of-date row on machine recovery and budgets now says that work is complete.

## 2026-09-25 — A fake fetcher can no longer silently stop everyone's web fetches (`SIGNOFF-REPAIR.7.1.3`)

`REASONBRAID-REPAIR-0494`.

- 🔴 **Before:** any organisation's administrator could register a "fetcher" advertised as fast. It would be ranked first for every organisation, the server had nothing to run it with, and every fetch then returned an empty answer with no error. Reproduced live: the test's leftover entry even silenced four other tests.
- ✅ **Now:** the server uses the first ranked fetcher it can actually run, and names any it skipped. If none can run, the answer says so by name.
- ⏭️ **Next:** decide who should be allowed to add fetchers to the shared list at all (`.7.1.3.1`).
- ✅ Tested: the new test failed on the old code and passes now; the deliberately broken versions were caught; broad live suite and strict lint pass.

## 2026-09-24 — Resource and resolver ownership reviewed: three real gaps found (`SIGNOFF-REPAIR.7.1`)

`REASONBRAID-DOC-0161`. A review against the code; no code changed.

- 🔴 **Most serious:** any organisation's administrator can register a new "resolver" (the component that fetches a web resource), and the list is shared by everyone. A fake one advertised as fast would outrank the real fetcher. Every organisation's fetches would then silently do nothing, with no error shown. Owned by `.7.1.3`, next.
- 🔴 **Also real:** the first organisation to cite a web address decides how it is fetched for everyone else (`.7.1.4`); and when two fetchers tie, which one is used is left to chance (`.7.1.5`).
- ✅ **Holds:** fetched content is checked against its expected fingerprint, a corrected fetcher description replaces the old one completely, the isolation rules compare in the right direction, and who may write to the shared lists was settled earlier.

## 2026-09-24 — Tenant-owned administration checked and closed (`SIGNOFF-REPAIR.3.5`)

`REASONBRAID-DOC-0160`. A closing check; no code changed.

- ✅ **Closed:** every piece of this review was already done, but it had never been formally closed. Each of its goals was checked against the code and the test that proves it. One organisation's administrator cannot change or read another organisation's machines, queues, enrollment tokens or spending cut-off. A frozen organisation's administrator can still look at its own records, as approved.
- ⏭️ **Next:** ownership of resources and resolvers (`.7.1`), which has not been reviewed yet.

## 2026-09-24 — The server's certificate authority now renews itself, and the node-machine review is complete (`SIGNOFF-REPAIR.4.1.8.2`)

`REASONBRAID-REPAIR-0493`. The second of the two steps decided in `REASONBRAID-DOC-0159`.

- 🔴 **Before:** nothing ever renewed the certificate authority. After a year every machine would have been locked out at once.
- ✅ **Now:** when a third of the authority's life is left (about four months for a one-year authority), the server creates its successor by itself, without a restart, and switches new certificates to it. Machines certified by the old one keep working. With several servers, exactly one creates the successor and the others adopt it. The health check has a new line, `server_ca_renewal`, which only turns red if a due renewal fails.
- ⭐ **The node-machine review is complete:** enrollment, certificates, leases, delivery, recovery and budgets.
- ✅ Tested: the live test shows one successor created, reused on every later check, adopted by a second server, and the old authority's machines still trusted. Every deliberately broken version was caught, some only after I added tests. This one could not be shown failing on the old code, because the feature did not exist there at all; that is recorded. Broad live suite and strict lint pass.

## 2026-09-24 — The server can hold more than one certificate authority (`SIGNOFF-REPAIR.4.1.8.1`)

`REASONBRAID-REPAIR-0492`. The first of the two steps decided in `REASONBRAID-DOC-0159`.

- 🔴 **Before:** the server had exactly one certificate authority, at every level: stored as one row, loaded once, and the only thing it trusted. Even a stored successor would have been ignored.
- ✅ **Now:** the server keeps every authority it has. New machine certificates come from the newest, and a machine certified by an older one keeps working until that authority expires. Nothing changes while there is only one.
- ⏭️ **Next:** the server creates the successor by itself before the current authority runs out, and warns if it ever fails to (`.4.1.8.2`).
- ✅ Tested: the new test failed on the old code; a live test shows a machine from the first authority still connecting after a second one takes over, and a new machine getting a certificate from the second. Every deliberately broken version was caught, two only after I added tests, including one for the exact second an authority expires. Broad live suite and strict lint pass.

## 2026-09-24 — Decided how the server's certificate authority renews itself (`SIGNOFF-REPAIR.4.1.8`)

`REASONBRAID-DOC-0159`. A design decision; no code changed.

- ✅ **Decided:** the server will keep more than one certificate authority. It trusts any that hasn't expired, issues from the newest, and creates a successor by itself once a third of the current one's life remains, with no restart. The health check will warn if that renewal ever fails to happen. This is how widely used systems (SPIFFE/SPIRE, cert-manager, Vault) do it.
- 🔎 **Checked first:** no machine pins the authority; only the server's own checks trust it. So it can change without touching any machine.
- ⏭️ **Next:** build it in two steps: first let the server hold several authorities (`.4.1.8.1`), then the automatic renewal and the warning (`.4.1.8.2`).

## 2026-09-24 — A machine's certificate can no longer outlive the authority that signed it (`SIGNOFF-REPAIR.4.1.7`)

`REASONBRAID-REPAIR-0491`.

- 🔴 **Before:** the server gave each machine a 10-minute certificate without checking when its own certificate authority expires. In the authority's last minutes, it signed certificates that outlived it.
- ✅ **Now:** a machine's certificate ends no later than the authority does. If the authority has 5 minutes or less left (the point at which a machine asks for a new certificate anyway), the server refuses to issue and says clearly that the authority must be renewed. The server and the machines now read that 5-minute figure from one shared place.
- ✅ Tested: the new test failed on the old code and passes now; all the deliberately broken versions that compile were caught (one only after I added a check of the refusal message); broad live suite passes.

## 2026-09-24 — Machine certificates reviewed: one worry cleared, two real gaps owned (`SIGNOFF-REPAIR.4.1`)

`REASONBRAID-DOC-0158`. A check of three open findings against the code; no code changed.

- ✅ **Cleared:** "a machine with only expired certificates never reads as suspended". Revoking a machine revokes all its certificates, expired ones included, so a revoked machine always reads suspended. One whose certificates simply ran out reads offline, which is the truth.
- 🔴 **Real:** the server's certificate authority is made for one year and nothing renews it. The health check does report it once it has expired, but nothing warns beforehand or stops new certificates being issued from it, so after a year every machine would fail at once (`.4.1.8`).
- 🔴 **Real:** a machine's certificate can be issued to outlive the authority that signed it (`.4.1.7`, next).

## 2026-09-24 — An operator's ruling on a lost answer now settles its budget hold, and the budget review is complete (`SIGNOFF-REPAIR.4.5.1.1`)

`REASONBRAID-REPAIR-0490`. The last item of the budget review (`REASONBRAID-DOC-0156`).

- 🔴 **Before:** since this morning, a hold for a lost answer stayed counted, correctly. But nothing ever let it go: when an operator ruled that the call had not been charged, the money stayed held for good.
- ✅ **Now:** when the machine applies the operator's ruling, the hold for exactly that attempt is released ("it did not charge") or charged in full ("it did, amount unknown"). This is keyed by attempt, so a re-run's separate hold is never confused with the original's.
- ⭐ **The budget review is done:** uncertain holds are counted and then settled, totals cannot overflow, simultaneous requests cannot squeeze past a limit, a machine cannot charge another tenant, and the spending cut-off behaves as documented. The outgoing-event queue waits for its first reader.
- ✅ Tested: the new test failed on the old code and passes now; three deliberately broken versions were caught; broad live suite and strict lint pass.

## 2026-09-24 — A spending cut-off that watches nothing is refused (`SIGNOFF-REPAIR.4.5.6.1`)

`REASONBRAID-REPAIR-0489`.

- 🔴 **Before:** an administrator could arm a spending cut-off that named no measure at all. After this morning's fix it would never trip: a silent alarm.
- ✅ **Now:** that request is refused with a clear message listing the four measures, and the refusal is recorded like any other administrative action. A measure set to zero still counts as named.
- ✅ Tested: the new test failed on the old code and passes now; two deliberately broken versions of the rule were caught; broad live suite and strict lint pass.

## 2026-09-24 — A replaced NUL character can now be traced and undone, by anyone reading the thread (`SIGNOFF-REPAIR.4.4.10.3.1`)

`REASONBRAID-REPAIR-0488`. Revisits an earlier decision at the director's request.

- 🔴 **Before:** our database cannot store the NUL character, so a machine swaps it for "�" (U+FFFD) and noted only how many it swapped. The swap could not be undone: a "�" the AI wrote itself looked the same as a swapped one, and the note never reached the thread people read.
- ✅ **Now:** the machine records exactly where each swap happened. The server checks that record against the text and keeps it on the contribution itself, so any reader can see what changed and restore the original exactly. A "�" the AI wrote itself stays unmarked.
- ✅ Tested: all three new or changed tests failed on the old code and pass now. Ten deliberately broken versions were all caught: two only after I added a test for them, and one breaks the server's hand-off of the positions. The live test rebuilds the original text from the positions and checks it matches exactly. Broad live suite and strict lint pass.
- Technical: node `storable_content` → maximal scalar-index runs `nul_positions` (replacing `nul_replaced`); `threads::check_nul_positions` on contribute/revise; event bodies carry `nul_positions` when present; the fold forwards it. Decision: `docs/decisions/2026-09-24_nul-in-provider-output-is-replaced-losslessly.md` (supersedes the count-only record).

## 2026-09-24 — A spending cut-off set on calls no longer trips on the first job (`SIGNOFF-REPAIR.4.5.6`)

`REASONBRAID-REPAIR-0487`.

- 🔴 **Before:** a tenant's spending cut-off set on only some measures, say 100 calls, tripped on the very first job. Every job also asks for tokens and time, and the check treated "not set" as "no room".
- ✅ **Now:** the cut-off watches only the measures it names: set at 100 calls, it trips at the hundred-and-first. The budget limit itself still refuses anything it doesn't measure, which is right for a limit.
- ⚠️ **Next:** a cut-off that names nothing at all can now never trip, and the arm command still accepts one (`.4.5.6.1`).
- ✅ Tested: the new test failed on the old code and passes now; five deliberately broken versions were all caught.
- Technical: core `BudgetDimensions::restricted_to`; `check_spend_breaker_in_tx` compares `threshold.covers(&projected.restricted_to(&threshold))`.

## 2026-09-24 — The server's outgoing-event queue waits for its first reader (`SIGNOFF-REPAIR.4.5.5`)

`REASONBRAID-DOC-0157`. The fifth item from the budget review; no code changed.

- ✅ **Found:** every change the server records also queues an outgoing event, but nothing reads that queue yet, and the "deliver" step only writes to a table nobody reads. The queue grows at the same rate as the event history, which is kept anyway, and it blocks nothing today.
- ⏸️ **Decided:** the queue's worker, its retry limit and its failed-message shelf get built together with the first thing that actually consumes events (the publication worker, a federation export, a webhook), or as soon as anything deletes old events. Recorded in `docs/decisions/`.

## 2026-09-24 — Proven: a machine cannot charge its work to someone else's budget (`SIGNOFF-REPAIR.4.5.4`)

`REASONBRAID-REPAIR-0486`. The fourth item from the budget review (`REASONBRAID-DOC-0156`).

- ✅ **Held, now proven:** a machine's result names a budget hold, but the server ignores that name and settles the hold it recorded itself. Nothing tested this. A new test has one tenant's machine name another tenant's hold, and requires the other tenant's hold to stay untouched.
- ✅ Tested: the test passes on today's code. When I broke the server so that it trusted the machine's name, this test failed (the other tenant was charged a million tokens) and every other test still passed. So it closes a real gap in our checks.
- Technical: `node_work.rs` `a_node_cannot_settle_a_foreign_reservation_by_naming_it`; hand mutant on `api::settle_work_item_reservation`.

## 2026-09-24 — Two requests at once can no longer both squeeze under a limit (`SIGNOFF-REPAIR.4.5.3`)

`REASONBRAID-REPAIR-0485`. The third fix from the budget review (`REASONBRAID-DOC-0156`).

- 🔴 **Before:** the budget limit, the spending cut-off and the invite quota each looked at what was already used and then recorded the new use, with nothing stopping a second request in between. Two requests arriving together could both see room for one and both get in.
- ✅ **Now:** each check locks the record it decides against. A second request waits for the first to finish, then counts it and is refused if there is no room left.
- ✅ Tested: all three new tests failed on the old code and pass now. Each proves its own lock (removing any single lock fails exactly its own test), and each checks that the waiting request then decides correctly. The broad live suite passes.
- 🔴 **Also fixed (`SIGNOFF-REPAIR.11.35`):** that broad run found a test suite (`bootstrap_recovery`) failing on `main` since 23 September. A table was added to the middle of its checklist, which was matched by position, so every later entry shifted. It now matches by table name.
- Technical: `FOR UPDATE` on `budget_ceilings` (`create_reservation_in_tx`), `spend_breakers` (`check_spend_breaker_in_tx`, after the ceiling), `usage_quotas` (`quota::check_in_tx`); controls observe the wait in `pg_stat_activity`.

## 2026-09-24 — Budget totals can no longer overflow (`SIGNOFF-REPAIR.4.5.2`)

`REASONBRAID-REPAIR-0484`. The second fix from the budget review (`REASONBRAID-DOC-0156`).

- 🔴 **Before:** the budget was added up with plain arithmetic, and usage figures come from the machines. Two absurd reports (2⁶³ tokens each) crashed the server's limit check. In a release build they would have wrapped round to a small number and let the limit lend again.
- ✅ **Now:** a total too large to count is a clear refusal that names what overflowed. New work is refused, the usage report answers `ledger_overflow`, and a machine's own budget counts itself spent. Large usage is still recorded in full, because the work did happen.
- ✅ Tested: all four new tests failed on the old code and pass now; nine deliberately broken versions were all caught, two of them only after I strengthened a test; live suites pass; strict lint clean.
- Technical: `BudgetDimensions::add -> Result`, `BudgetError::Overflow`; `budget.rs` held/spend/projection `map_err` to `Unavailable`; `api::ledger_overflow` (`500 ledger_overflow`); `LocalBudget` try_reserve refuses, settle saturates to `u64::MAX` on overflow.

The entries before those above were rotated into reachable Git history at the
**fourteenth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 8c36ee57442e64155de068fd84a78cdf111a281c:LIVE_STATUS.md
```

That snapshot is 52434 bytes and 401 lines, and contains 42 dated
entries; its Git blob is `77b2b6fc9344d95fa728ac668f697a7d6a2efe87` and its SHA-256 is
`02da440a953ae051a2d194aa681c69791d069549ecc7b3e9387815cb97db8683`. It carries the thirteenth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **13 record(s) rotated out, 30 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
