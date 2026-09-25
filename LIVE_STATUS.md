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
| Phase 1 — LAN vertical slice | Mostly Done | Historical demonstration retained in `PHASE-1`; the console review (`.11.1`) is closed with a real-browser control; current authority and demo-script findings remain open. |
| Phase 2 — identity, delivery and recovery | Mostly Done | Historical machinery retained in `PHASE-2`; revocation, fencing, budget and recovery repairs are `.3`–`.4`. |
| Phase 3 — directory and recruitment | Mostly Done | Historical machinery retained in `PHASE-3`; tenant visibility, recruitment and automatic initiation repairs are `.5`. |
| Phase 4 — resources and evidence | Mostly Done | Historical G4 record retained in `PHASE-4`; acquisition isolation, evidence integrity and retention repairs are `.7`. |
| Phase 5 — deliberation and evaluation | Mostly Done | Historical G5 subtraction gate withdrew the quality-lift claim; workflow and evaluation repairs are `.8`. |
| Phase 6 — governance | Mostly Done | Historical G3 machinery exit retained; binding use remains gated; policy/publication/deployment repairs are `.9`. |
| Phase 7 — Internet qualification | Mostly Done | Hardening machinery exists; G6/G7 Internet exposure remains NOT MET. The director greenlit Internet exposure over HTTPS with OAuth on 2026-09-25, security first; owner-only first (the director and their own agents), gated by our own evidence; opening to anyone else still needs an independent threat-model review and penetration test (`docs/runbooks/external-security-review.md`). Local repairs and external threat-model, injection and penetration-test evidence remain required. |
| Phase 8 — federation and interoperability | In Progress | Through regional routing historically recorded; `.5.3` store-and-forward, `.5.4` exit export/import and `.6` G8 remain. Shared authority and protocol gaps are prerequisite repairs. |
| Phase 9 — stable release | Not Started | G9 requires sustained operational evidence and the outstanding release decisions. |
| Corrective review | In Progress | `.1`, `.2.1`, `.2.2`, `.3.1` complete; `.3.2.1` site service passes ten live controls and strict lint. `.3.2.2` operator CLI passes 3 live controls plus adjacent service/ownership checks and strict lint; `.3.2.3` HTTP enforcement passes 8 focused controls; the selected 58-test security run and final 12-test registry run pass. `.3.3.1` core subject serialization passes 49 unit + 3 subject controls and 40 live compatibility tests; `.3.3.2` bound evaluation passes 51 core unit + 3 subject tests, 6 evaluator controls, 32 live authority/command API tests and strict lint; all results consumed and the owned cluster removed. `.3.3.3.1` actual-parent command selection passes 37 live tests, six evaluator controls and strict lint. `.3.3.3.2.1` frozen-read eligibility passes 40 live tests, ten pure evaluator controls and strict lint; all results consumed and the owned cluster removed. `.3.3.3.2.2.1` provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all confirmation/shutdown results consumed, owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint; all results and shutdown consumed. Exact scoped receipt readback passes 18 live authority tests and 30 HTTP tests plus strict lint, including final denied-record readback; all results/shutdown consumed. Actual-parent selection and frozen-read audit/readback `.3.3.3` are complete; The `.3.3.4.1` source census/contract is complete (42 named-call locations plus transitive/mutation coverage). `.3.3.4.2` qualifies the guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass, including cancelled-BEGIN replacement and the formerly stale authority executable. Seven acceptance-checker controls also pass, correcting nested-evidence owner selection while preserving real-tree box refusals. All results/shutdown consumed; owned clusters/probes removed. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. Bounded HTTP waits and restart qualification follow; ordinary no-key behavior remains intentionally distinct. The final administrative effect REPRESENTATION `.3.3.4.7.1` passes 10 representation controls within 68 core tests, strict core lint and the falsified object-only control; its census derives the closed fourteen-operation set from 27 guarded-admission call sites. Its STORAGE `.3.3.4.7.2` then passes 8 live controls (falsified 5/3 against three attributable injections): migration 0058 keyed on the admission with a composite tenant-binding foreign key, a writer on the caller's already-guarded transaction, and a tenant-filtered reader. `.3.3.4.8` is the first producer of an effect record; `.3.3.4.9` is the second, putting spend-breaker arm/reset onto one exclusive-guard transaction — 25 live controls, the affected set 5 suites / 94 tests, falsified 17/8 against the exact pre-`.9` handlers.. Fixture ownership `.11.2.1.1` censuses all 61 `create_dir_all` sites in tracked Rust with 0 unresolved and repairs the 3 that adopted: exclusive per-call creation on the repository volume, falsified against an occupied path, with 89 selected controls across five suites (63 of them `profiles` on a disposable cluster), strict lint for both crates and the 25-check gate green. |

## 2026-09-25 — Only the principal a target names can report what it runs (`SIGNOFF-REPAIR.9.3.3.2`)

`REASONBRAID-REPAIR-0526`.

- 🔴 **Before:** a receipt, the report of which policy content a target is actually running, could be filed by anyone in the owning organization, including the operator who had just said what it *should* run. The comparison that detects a target running the wrong thing was only as honest as whoever posted last.
- ✅ **Now:** a target names one reporter when it is registered (a person or an agent role), and only that principal can file its receipts; anyone else is refused with a message saying so. A target registered before this change takes no receipts until it is given a reporter, which is not possible yet and is tracked for when it is needed.

## 2026-09-25 — A deployment must deploy what its publication published (`SIGNOFF-REPAIR.9.3.3.1`)

`REASONBRAID-REPAIR-0525`.

- 🔴 **Before:** when someone assigned a published policy to a target, they also typed in which version and which content fingerprint the target should run, and the system only checked that the fingerprint looked like one. A target could be told to run content the publication never had, and later comparisons of "what should be running" against "what is running" used that made-up value.
- ✅ **Now:** the fingerprint must be the publication's own and the version one the publication recorded; anything else is refused with a message naming which one is wrong. The book shows where to read both.
- 🔎 **Found and tracked:** a drift record (the note that a target is not running what was published) still accepts a typed-in fingerprint (`.9.3.3.5`), and the deployment and drift records report a database failure as "does not exist" or "already exists" in 14 places (`.9.3.3.6`).

## 2026-09-25 — A commit check no longer mistakes a setting's name for a waiver (`SIGNOFF-REPAIR.11.2.10`)

`REASONBRAID-REPAIR-0524`.

- 🔴 **Before:** one of the automatic checks that runs on every commit treated the name of a setting, `REPEATED_WAIVER_THRESHOLD`, as if someone had written "this check doesn't apply to me", and refused a correct commit.
- ✅ **Now:** the check only reacts to the words it was built for, and it tests itself on every commit against thirteen known examples, so the same kind of mistake cannot come back unnoticed.

## 2026-09-25 — A policy can be reviewed more than once (`SIGNOFF-REPAIR.9.3.2`)

`REASONBRAID-REPAIR-0523`.

- 🔴 **Before:** once a policy had been reviewed for a given reason, it could never be reviewed for that reason again: the system silently failed to create the next review. The same silent failure made a database problem look like "nothing needs review", and a single exception granted to a policy (even an expired one) counted as a "repeated" exception.
- ✅ **Now:** each new problem after a completed review schedules a new review; only one review per policy and reason is open at a time; "repeated exceptions" means at least two still in force within 90 days; and a failure to create a review is reported as an error.

## 2026-09-25 — A database failure is no longer reported as "that record does not exist" (`SIGNOFF-REPAIR.9.2.3`)

`REASONBRAID-REPAIR-0522`.

- 🔴 **Before:** when the database failed while the system was recording a policy proposal, decision, approval or publication, the answer said the record the request named "does not exist", or that the approval's proof was invalid. Someone reading that could re-create the record or give up on it, when nothing was wrong with their request.
- ✅ **Now:** a database failure is reported as the server's problem on all of these, and a missing decision is called a decision (it said "proposal"). The policy publication review (`.9.2`) is closed.

## 2026-09-25 — A policy publication can only be finished once (`SIGNOFF-REPAIR.9.2.2`)

`REASONBRAID-REPAIR-0521`.

- 🔴 **Before:** a policy publication ends either "in force" or "failed". If both answers arrived at the same moment, both were accepted, the second overwrote the first, and the record could end up "in force" while still carrying the failure's reason. Both senders were told they had succeeded.
- ✅ **Now:** the first to arrive wins and the other is refused with a message saying what state it found. A missing publication is now called a publication in the error (it said "proposal"), and a database failure is reported as the server's problem rather than as "does not exist".
- 🔎 **Found and tracked:** the same "database failure reported as does not exist" mistake in 14 places of the proposal, decision and approval records (`.9.2.3`, next).

## 2026-09-25 — A refused conversation leaves no trace in the routing log (`SIGNOFF-REPAIR.8.2.7`)

`REASONBRAID-REPAIR-0520`.

- 🔴 **Before:** when a new conversation was routed to a working style automatically, the routing log was written before the system checked whether the request was allowed. A refused request left a log entry for a conversation that never existed, and re-sending a successful request added a duplicate entry.
- ✅ **Now:** the entry is written as part of creating the conversation, after the permission check, so it exists exactly when the conversation does. The evaluation and routing review (`.8.2`) is closed.

## 2026-09-25 — An experiment can no longer list the same option twice, and its errors say what is wrong (`SIGNOFF-REPAIR.8.2.6`)

`REASONBRAID-REPAIR-0519`.

- 🔴 **Before:** a routing experiment that listed an option twice gave that option a double share of the test cases, silently; a test case listed twice was stored twice. Separately, most of the evaluation tool's error messages were garbled into a sentence about a malformed fingerprint ("digest `the arms are empty` is not a 64-hex string").
- ✅ **Now:** both repeats are refused with a message naming the repeated item, and every refusal of the evaluation tool says plainly what is wrong.
- 🔎 **Tracked, not urgent:** if a stored baseline were ever damaged, the error would be reported as the user's mistake instead of a server fault; this cannot happen today because baselines are checked when saved (`.8.2.8`).

## 2026-09-25 — Changing an assessment is refused instead of silently ignored (`SIGNOFF-REPAIR.7.4.8`)

`REASONBRAID-REPAIR-0518`.

- 🔴 **Before:** sending an assessment again with a different quote or explanation was answered "accepted", with the first one's id, and the change was thrown away. In a conversation, the record even showed the new quote while the stored assessment kept the old one.
- ✅ **Now:** sending the same assessment again unchanged still returns the same one; sending it with anything changed is refused, and the message lists exactly what differs. Nothing is half-applied.

## 2026-09-25 — An assessment's author is whoever submitted it (`SIGNOFF-REPAIR.7.4.7`)

`REASONBRAID-REPAIR-0517`.

- 🔴 **Before:** when someone recorded that a piece of evidence supports (or contradicts) a claim, the request itself said who the author was, and who had verified it. Anyone could put someone else's name on it.
- ✅ **Now:** the server records the person who actually sent it as the author, and a request that tries to name an author or a verifier is refused with a message saying which field is not allowed.
- 🔎 **Found on the way:** a second person verifying an assessment, which the roadmap describes, does not exist yet (tracked, `.7.4.13`); and most routes answer a badly formed request without the error code the book promises (tracked and stated in the book, `.11.36`).

## 2026-09-25 — The two-host demonstration passes again (`SIGNOFF-REPAIR.4.4.2.2.1`)

`REASONBRAID-REPAIR-0516`.

- 🔴 **Before:** since the web console's inbox panel was fixed on 2026-09-24, the demonstration's check of the console still looked for the panel's old, broken query and failed. Nothing noticed because everyday test runs skip the demonstration, but the next publication to GitHub would have failed its automated checks on it.
- ✅ **Now:** the check looks for the corrected query and says what it actually checks, and the whole demonstration passes. The book's description of that step was corrected the same way.

## 2026-09-25 — Deleted evidence can no longer be built on, and fetching it again makes fresh evidence (`SIGNOFF-REPAIR.7.4.6`)

`REASONBRAID-REPAIR-0515`.

- 🔴 **Before:** when an operator or the retention sweep deleted a piece of evidence (it is kept, marked deleted, with a reason), three things still used it: new summaries could be filed against it, assessments could be checked against its text (in a conversation too), and fetching the same content again quietly pointed back at the deleted record.
- ✅ **Now:** the first two are refused with a message naming the deletion and its reason, and fetching the same content again creates a fresh record with its own history and retention period. The deleted record stays readable, as before.
- ⚖️ **Decided and recorded:** a deletion retires one fetched copy, not the content itself. Otherwise a routine expiry would become a permanent ban. Blocking content wherever it appears is a separate feature that does not exist yet (`.7.4.10`), and the book says so.

## 2026-09-25 — The web console no longer shows a late answer under the wrong view (`SIGNOFF-REPAIR.11.1.2`)

`REASONBRAID-REPAIR-0514`.

- 🔴 **Before:** each view cleared the page, fetched, then drew wherever the page was by then. On a slow server, clicking Timeline and then Audit could put the timeline's rows under the Audit heading, and saving a new identity could show the previous tenant's conversations under it.
- ✅ **Now:** each view draws into its own space, which a later click replaces, so a late answer is dropped. The presence and inbox panels keep only the answer to the latest click. Four new browser checks hold one answer back on purpose and confirm it changes nothing when it arrives; each failed on the old page, and six deliberately broken versions of the repair are each caught.
- ✅ **The console review (`.11.1`) is closed.**

## 2026-09-25 — The web console's Timeline works again, and a real browser now checks the console (`SIGNOFF-REPAIR.11.1.1`)

`REASONBRAID-REPAIR-0513`.

- 🔴 **Before:** the Timeline view showed only an error for every conversation that had any events. A number reached a piece of the page that only accepted text, and no test ever opened the console in a browser, so nothing noticed.
- ✅ **Now:** every value on the page is shown as text. A new check opens the console in the pinned test browser against a real server and database, clicks through it the way an operator does, and compares what the page shows with what the server returned. It also puts HTML in a conversation's title and confirms it stays text and never runs. The check failed on the old page with the exact error an operator saw, and three deliberately broken versions of the repair each fail it.
- 🔎 **Found on the way:** the browser used by the web-page reading tool (off unless switched on) contacts Google services by itself: time, accounts, updates and a messaging registration. The book now says so, and `.7.3.7` owns the fix.

## 2026-09-25 — The web console checked point by point: no security hole, but one view is broken (`SIGNOFF-REPAIR.11.1`)

`REASONBRAID-DOC-0180`.

- ✅ The feared problem, other people's data running as code in an administrator's browser, is not possible with how the console builds its pages.
- 🔴 To fix: the Timeline view fails for every conversation that has any events (a number reaches a piece of code that only accepts text), and no test ever opens the console in a real browser, which is why nobody noticed. Also, a slow view can appear under the heading of the view you switched to.

## 2026-09-25 — A cancelled Claude or Codex request is no longer recorded as a definite failure (`SIGNOFF-REPAIR.10.1.4`)

`REASONBRAID-REPAIR-0512`.

- 🔴 **Before:** cancelling a request stops its program, and the connection then recorded "the provider failed", which nobody knows: the provider may well have finished the work.
- ✅ **Now:** a program stopped from outside (a cancel, or any other forced stop) leaves the result marked "unknown", which is what the system uses to require care before retrying. A program that reports its own failure is still recorded as a failure.
- ✅ **The Claude and Codex connections review (`.10.1`) is closed:** all four problems it found are fixed; checking token counts against a real receipt waits for a live run.

## 2026-09-25 — Finished or abandoned Claude and Codex programs are cleaned up (`SIGNOFF-REPAIR.10.1.3`)

`REASONBRAID-REPAIR-0511`.

- 🔴 **Before:** the Claude and Codex connections kept a hold on every program they had ever started, so a request that was given up on left its program running for ever. And a successful answer was reported before its program had even finished.
- ✅ **Now:** every way a request can end waits (for at most a few seconds) for the program to finish and cleans it up, stopping it if it lingers. Giving up on a request stops its program.
- ✅ Tested: both problems shown on the old code by looking at the real processes; every deliberately broken version caught.

## 2026-09-25 — The Claude and Codex connections read their output safely (`SIGNOFF-REPAIR.10.1.2`)

`REASONBRAID-REPAIR-0510`.

- 🔴 **Before:** when reading what the Claude or Codex program printed, the connection could crash on some non-English error text, held a line of any size in memory before checking it, and kept the start of the error log instead of the end, where the actual error is. Worse, one garbled error line made it stop listening, and the program was then killed the next time it wrote an error.
- ✅ **Now:** both connections share one careful reader: every line has a size limit (tied to the overall output limit, checked when the software is built), error text is always read to the end and its last part kept, and non-English text is cut safely.
- ✅ Tested: all five problems shown on the old code (one turned out to be a crash, not the stall first suspected); every deliberately broken version caught or explained.

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

The entries before those above were rotated into reachable Git history at the
**sixteenth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show a69491d3b43e79b801dc4aaff4420060a7e33249:LIVE_STATUS.md
```

That snapshot is 52774 bytes and 432 lines, and contains 49 dated
entries; its Git blob is `2ddb7b5f8d4d3f553c617e2d470fac28a4f2252b` and its SHA-256 is
`7006aaa3ee262e4c4e8b42bfde61d41bb46ea66993c5f1fb5f067f1ff8da5b53`. It carries the fifteenth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **14 record(s) rotated out, 36 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
