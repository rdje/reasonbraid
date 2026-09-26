# CHANGELOG.md

## 2026-09-26 — The demonstration's "this did not happen" checks can no longer pass by accident (`SIGNOFF-REPAIR.11.3.7`)

`REASONBRAID-REPAIR-0537`.

- 🔴 **Before:** three of the demonstration's checks that something did NOT happen would also pass if the tool doing the checking failed, and the responses it keeps as evidence were kept without noticing when a request had failed.
- ✅ **Now:** each of those checks first requires the tool to have worked, a failed request in the evidence is reported as a failure, and the check that the restarted server accepts a node requires a genuine acceptance.

## 2026-09-26 — The load test measures exactly what it was asked to (`SIGNOFF-REPAIR.11.3.4`)

`REASONBRAID-REPAIR-0536`.

- 🔴 **Before:** the load test could run more requests than asked (10 became 16), reported success when asked to run none, could hang forever on one stuck request, and mixed failed requests into its speed figures, so fast failures made the system look faster.
- ✅ **Now:** it runs exactly the number asked (or refuses), gives up on a request after a set time, and reports speed only over requests that succeeded, with failures listed beside them. The speed figures published on 2026-09-08 were not affected.

## 2026-09-26 — The development environment checks its own server and cleans up honestly (`SIGNOFF-REPAIR.11.3.3`)

`REASONBRAID-REPAIR-0535`.

- 🔴 **Before:** if another copy of the development environment was already running, a self-check reported success by talking to the other copy; the cleanup deleted the database folder even when the database had not stopped, and said all was well; and it could not be started from outside the project folder.
- ✅ **Now:** it checks that it is talking to its own server, deletes the database folder only once the database has stopped (and says so plainly if it cannot), and works from any folder.
- 🔎 **Found and tracked:** some of the demonstration's "this did not happen" checks would pass even if the tool running them failed (`.11.3.7`).

## 2026-09-26 — The demonstration checks it is talking to its own server, and its two-host mode works (`SIGNOFF-REPAIR.11.3.5`)

`REASONBRAID-REPAIR-0534`.

- 🔴 **Before:** if another program already used the demonstration's port, the demonstration carried on talking to it without noticing; it copied the database address, user included, into the evidence it produces for sharing; and its documented two-host mode could never have worked (paths were quoted in a way that broke them, the node program was looked for in the wrong folder, and a check looked at the wrong machine).
- ✅ **Now:** the demonstration stops at once if its own server did not start, the evidence shows the address without the user, and the two-host mode passes every check in a same-machine rehearsal (not yet between two real machines).

## 2026-09-26 — A backup is never half-written and never readable by others (`SIGNOFF-REPAIR.11.3.1`)

`REASONBRAID-REPAIR-0533`.

- 🔴 **Before:** an interrupted backup left a partial file under the name of a real one; the backup, a plaintext copy of the whole database, was readable by other users; and the database password was visible on the backup tool's command line.
- ✅ **Now:** a backup appears under its name only once it is complete and never replaces another, only its owner can read it, and the password is passed privately.

## 2026-09-26 — The restore test can no longer overwrite the live database (`SIGNOFF-REPAIR.11.3.2`)

`REASONBRAID-REPAIR-0532`.

- 🔴 **Before:** the restore test wiped and replaced whatever database it was pointed at, including the live one, and passed the database password on its command line, where other users of the machine could read it.
- ✅ **Now:** it refuses the live database by name and any database that already holds data, before touching anything, and it hands the connection details to the database tools privately rather than on the command line.

## 2026-09-26 — The operational scripts reviewed: six problems to fix (`SIGNOFF-REPAIR.11.3`)

`REASONBRAID-DOC-0183`.

- 🔎 **Found:** the restore script will wipe whatever database it is pointed at, the live one included; the database password is visible to other users on the machine while the backup, restore, load and demo scripts run, and the demo copies it into its shareable evidence; a backup that is interrupted leaves a partial file that looks complete; the development and load scripts can report success while talking to a different program on the same port; the load test runs more requests than asked and mixes failures into its speed figures; and one test leaves an empty folder behind on every run (34 so far).
- ✅ **Next:** each script gets its own fix, the restore script first.

## 2026-09-26 — The review page no longer lists problems that were already fixed (`SIGNOFF-REPAIR.11.37`)

`REASONBRAID-REPAIR-0531`.

- 🔴 **Before:** the book's review page, where a reader looks for what is still wrong, listed 24 problems whose fixes had already landed, some of them weeks earlier, and in two places contradicted its own summary table.
- ✅ **Now:** every one of the 24 was checked against the code. 21 now say what was fixed and how things work today, one says it was never a problem, one explains which half was fixed and which was a deliberate decision, and one small remaining gap got its own tracked item. An automatic check now refuses to let the page keep an open problem once all its fixes are done.

## 2026-09-26 — Deployment records no longer blame database failures on the caller (`SIGNOFF-REPAIR.9.3.3.6`)

`REASONBRAID-REPAIR-0530`.

- 🔴 **Before:** when the database failed while registering a target, assigning a policy, or recording drift, a correction or an outcome, the answer said the record "does not exist", the authority "is not active", or the record "already exists". Someone could re-create or abandon a request that was fine. A badly written expiry date was reported as missing.
- ✅ **Now:** a database failure is reported as the server's problem; "already exists" means exactly that; a badly written date is named as such. The deployment review (`.9.3.3`) is closed.

## 2026-09-26 — A drift record compares against what was actually assigned (`SIGNOFF-REPAIR.9.3.3.5`)

`REASONBRAID-REPAIR-0529`.

- 🔴 **Before:** a drift record (the note that a target is not running what was published) stored whatever "should be running" value the person recording it typed, so it could disagree with the assignment it was about.
- ✅ **Now:** that value must be the assignment's own, or the record is refused with a message naming it. What the target was seen running is still the observer's report.

## 2026-09-26 — Only a target's own authority can decide what it runs (`SIGNOFF-REPAIR.9.3.3.7`)

`REASONBRAID-REPAIR-0528`.

- 🔴 **Before:** assigning a published policy to a target only checked that the policy belonged to the person assigning it. Targets are shared across the whole site, so one organization could set what another organization's target should run.
- ✅ **Now:** only whoever holds the target's own authority can assign to it; anyone else is refused as unauthorized.
- 🔎 **Found and fixed:** the review page still listed a problem with policy corrections that was fixed weeks ago. It is corrected, and 10 more entries on that page that may be similarly out of date are tracked (`.11.37`).

## 2026-09-26 — The book no longer implies deployments roll out in stages (`SIGNOFF-REPAIR.9.3.3.4`)

`REASONBRAID-DOC-0182`.

- 🔴 **Before:** the book said a publication is assigned to a target in a "canary wave", which reads as a staged rollout where a failing first group stops the rest. Nothing does that: the wave number is only stored, and the system does not carry out deployments at all yet.
- ✅ **Now:** the book says the wave is a label you choose to group targets, and that nothing holds a later wave back. Rollout ordering is tracked for when the system first carries out a deployment itself.
- 🔎 **Found and tracked:** assigning a publication to a target checks no authority over that target (`.9.3.3.7`, next).

## 2026-09-26 — A target's reports are kept, not overwritten (`SIGNOFF-REPAIR.9.3.3.3`)

`REASONBRAID-REPAIR-0527`.

- 🔴 **Before:** each time a target reported which policy content it was running, the new report replaced the old one, so what it said before, and who said it, was lost.
- ✅ **Now:** every report is kept as its own record that cannot be edited afterwards, the target's current state is always its latest report, and the full history can be read in order. Reports that arrived before this change had already been lost.

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

## 2026-09-24 — A budget hold for a lost answer now stays in place (`SIGNOFF-REPAIR.4.5.1`)

`REASONBRAID-REPAIR-0483`. The first fix from the budget review (`REASONBRAID-DOC-0156`).

- 🔴 **Before:** when a provider's answer was lost, we could not know whether the call was charged, yet its budget hold expired after ten minutes and the money could be lent to other work.
- ✅ **Now:** when a machine reports that it cannot retry such a job without permission, the server marks the job's hold "outcome unknown", and the hold keeps counting after its window. The limit check, the spending cut-off and the usage report share one rule, and the thread's budget page shows the mark.
- 🔴 **Also found:** a spending cut-off set on only some measures (say, calls) trips on the very first job, because every job also asks for tokens and time (`.4.5.6`). Nothing yet releases a marked hold after an operator's ruling (`.4.5.1.1`).
- ✅ Tested: the new test failed on the old code and passes now; five deliberately broken versions were all caught; six live suites pass; strict lint clean. The stall hunt (`.11.26`) ran 120 times with no stall.
- Technical: migration `0107` `budget_reservations.outcome_unknown_at`; `budget::holding!` read by admission, `check_spend_breaker_in_tx` and `admin_usage`; `hold_for_unknown_outcome_in_tx` from the `work_dead_lettered` fold on `RETRY_REQUIRES_AUTHORIZATION`; `GET /v1/threads/{id}/budget` row field `outcome_unknown_at`.

## 2026-09-24 — Budget review: a held allowance for a lost answer quietly expires after ten minutes (`SIGNOFF-REPAIR.4.5`)

`REASONBRAID-DOC-0156`. A review of the budget and quota rules against the code; no code changed.

- 🔴 **Found:** when a provider call's answer is lost, its budget hold is meant to stay in place in case the call did charge. In fact every hold stops counting ten minutes after it was placed, so the allowance can be lent out again. Yesterday's notes and the operator's advice said "the original stays held"; that is only true for ten minutes. The book, the command-line page and the runbook now say so; `.4.5.1` fixes the rule itself.
- 🔴 **Found:** a machine's reported usage is added up without an overflow check, so an absurd report could crash the server or make the budget look nearly empty (`.4.5.2`). Two admissions at the same moment can both pass a limit only one should (`.4.5.3`).
- ✅ **Holds:** a settled hold cannot be settled twice; a machine can only settle the hold the server recorded (a proof test is owed, `.4.5.4`). The server's outbound queue has no worker yet (`.4.5.5`).
- Technical: held query `r.status = 'active' AND r.expires_at > $2`; dispatch hold `Duration::minutes(10)`; `BudgetDimensions::add` plain `u64 +`, no `[profile.release]`; no `FOR UPDATE`/advisory lock in `budget.rs`/`quota.rs`; `claim_ready` has no production caller.

## 2026-09-24 — Operators can re-run a job whose outcome is unknown, from the screen's advice and the command line (`SIGNOFF-REPAIR.4.4.7.2.3`)

`REASONBRAID-REPAIR-0482`. Completes the machine-recovery work found by `REASONBRAID-DOC-0153`.

- ✅ **Now:** the operator's list of recovery options shows "ask again, accepting a possible duplicate charge" as available. It names who takes it (the tenant's administrator) and how. The command-line tool can do it: `rb node replay … --allow-possible-duplicate --reason "…"`. It insists on a reason, and refuses a reason given without the approval.
- ⭐ **The whole machine-recovery review is done:** every gap found on 23 September is closed. That covers lost answers, answers refused or wedged, stuck machines, timeouts, retries, budgets and the operator's views.
- ✅ Tested: the availability check failed on the old code and passes now; the command-line rules are checked, and all seven deliberately broken versions were caught; live suites pass; strict lint clean.

## 2026-09-24 — An administrator can now authorize re-running a job whose outcome is unknown (`SIGNOFF-REPAIR.4.4.7.2.2`)

`REASONBRAID-REPAIR-0481`.

- 🔴 **Before:** when a provider call's outcome was unknown, the machine rightly refused to run the job again on its own. But nothing let anyone approve a re-run while accepting the risk of paying twice, so the job stayed stuck.
- ✅ **Now:** the tenant's administrator can approve the re-run with a stated reason. The job gets its own new budget hold (the original stays reserved in case the first call did charge), the machine runs it again, and the answer lands. The approval is recorded in the audit trail under its own name, applies only to jobs stuck for exactly this reason, and is refused if the conversation's budget has no room.
- ⚠️ Next: the command-line action, and marking the option available on the operator's screen.
- ✅ Tested: a full live run (a lost answer, the approval, the re-run, the answer landing) that could not happen before; the audit vocabulary checks; five further live suites; both deliberately broken versions were caught; strict lint clean.

## 2026-09-24 — A machine can now receive permission to retry a job whose outcome is unknown (`SIGNOFF-REPAIR.4.4.7.2.1`)

`REASONBRAID-REPAIR-0480`. The first part of the "ask again, accepting a possible duplicate" action.

- 🔴 **Before:** once a machine had a job, nothing sent later could change it. That included permission to retry a job whose outcome is unknown, so even an authorized retry could never reach the machine.
- ✅ **Now:** when a job is sent again with a fresh approval, the machine takes two things from it: whether a possible duplicate is allowed, and which budget hold to run under. It keeps everything that says what the job is. A resend can extend what a machine may risk; it can never change what it was asked to do.
- ⚠️ Next: the server-side action that sends such a resend, then the command-line action and a full end-to-end check.
- ✅ Tested: two new checks failed on the old code and pass now; 112 machine tests and two live suites pass; every deliberately broken version was caught; strict lint clean.

## 2026-09-24 — The recovery options now say which ones actually exist (`SIGNOFF-REPAIR.4.4.7.1`)

`REASONBRAID-REPAIR-0479`.

- 🔴 **Before:** the list of recovery options for a job with an unknown outcome included "ask again, accepting a possible duplicate charge", as if it could be done. It could not, and the runbook and the book said the same.
- ✅ **Now:** every option says whether it is available today. That one says it is not, why, and what will change it. The runbook and the book describe what an operator can really do: a revision can be requested again with a new challenge; a first answer cannot be requested again yet.
- ⚠️ Next: building the "ask again, accepting a possible duplicate" action itself.
- ✅ Tested: a new check failed on the old code and passes now; the live suite passes; a deliberately broken version was caught.

## 2026-09-24 — Checked: operators were told to use a recovery action that does not exist (`SIGNOFF-REPAIR.4.4.7`)

`REASONBRAID-DOC-0155`. A review; no code changed.

- 🔴 **Found:** when a job's outcome is unknown, the operator's screen, the runbook and the book all suggest "ask again, accepting the risk of a duplicate charge". Nothing lets anyone do that. For a first answer (as opposed to a revision) there is no way at all to ask again today.
- ⚖️ **Decided:** first make the screen, runbook and book tell the truth about which actions are available now; then build the "ask again, accepting a possible duplicate" action properly, with its own separate budget, because the roadmap names it as one of the four ways to handle an unknown outcome.

## 2026-09-24 — The charter lookup no longer blames the caller for server faults (`SIGNOFF-REPAIR.4.4.10.1.2`)

`REASONBRAID-REPAIR-0478`.

- 🔴 **Before:** looking up a governance charter answered any database problem as "your request is invalid", and quoted the database's internal error message back to the caller. A real server fault was blamed on the user, and internal details leaked.
- ✅ **Now:** a server fault is reported as a server fault, with the details kept in the server's log. A request containing the null character gets the clear "cannot be stored" reply. Publications were checked too: that character cannot reach their storage, because the input is validated first.
- ✅ Tested: a new check (the null character, plus a simulated database fault) failed on the old code and passes now; three live suites pass; both deliberately broken versions were caught; strict lint clean.

## 2026-09-24 — More places now say clearly when input can never be stored (`SIGNOFF-REPAIR.4.4.10.1.1`)

`REASONBRAID-REPAIR-0477`.

- 🔴 **Before:** in seven places (agent profiles, snapshots, derivations, assessments, references, quotas and evaluations), input holding the null character was still answered "internal server error" rather than the clear "cannot be stored" reply.
- ✅ **Now:** all seven give the clear, permanent reply through one shared rule; genuine server faults still report as server faults.
- ⚠️ Tracked next: two remaining places (publications and charters) that lose the information needed to tell the two apart.
- ✅ Tested: a new check failed on the old code (a profile with that character) and passes now; the shared rule is checked against every kind of database error; six live suites pass; strict lint clean.

## 2026-09-24 — Time spent on jobs now counts against a conversation's time budget (`SIGNOFF-REPAIR.4.4.6.1`)

`REASONBRAID-REPAIR-0476`.

- 🔴 **Before:** each conversation has a time budget (10 minutes by default) alongside its call and token budgets. Time was never charged once a job finished, so the budget only limited how many jobs could run at the same moment, never the total. However long the jobs took altogether, the time budget never ran out.
- ✅ **Now:** each machine times every job, rounding up to whole seconds, and reports the time alongside the tokens. The server charges it, so a conversation's time budget runs out like its token budget does.
- ✅ Tested: two checks (one on the machine, one against the real server) failed on the old code and pass now; 199 core and machine tests and three live suites pass. The deliberately broken versions exposed a gap ("always charge one second" went unnoticed because every test job was quick); a longer job was added, and all are now caught. Strict lint clean.

## 2026-09-24 — The web console's inbox panel works (`SIGNOFF-REPAIR.4.4.2.2`)

`REASONBRAID-REPAIR-0475`. A defect found by the previous fix.

- 🔴 **Before:** the console's "Inspect inbox" panel had never worked. It asked the server with the wrong parameter name, so every request was refused, and it tried to show a field that does not exist. The check meant to keep the console honest had itself been written with the wrong name, so it agreed with the mistake.
- ✅ **Now:** the panel works and shows each job's delivery state, whether its answer was refused (and why), and any quarantine. The check now tests the panel against the server's own definitions, so the two cannot drift apart unnoticed again.
- ✅ Tested: the new check failed on the old console, once for each of the two mistakes, and passes now; strict lint clean.

## 2026-09-24 — Operators can now see when a finished job's answer was refused (`SIGNOFF-REPAIR.4.4.2.1`)

`REASONBRAID-REPAIR-0474`.

- 🔴 **Before:** in the operator's inbox view, a job whose answer the server refused (for example because the conversation had closed) looked exactly like one whose answer was accepted. Both showed as "done".
- ✅ **Now:** each job in the view shows whether its answer was refused, and why. Accepted answers show nothing extra. The same information reaches the assistant-facing (MCP) view.
- 🔴 **Found along the way (tracked, fixed next):** the web console's inbox panel has never worked. It asks the server in the wrong way, and it reads a field that does not exist.
- ✅ Tested: two checks failed on the old code and pass now; six live suites pass; a deliberately broken version was caught; strict lint clean.

## 2026-09-24 — An answer containing the null character now gets through, marked (`SIGNOFF-REPAIR.4.4.10.3`)

`REASONBRAID-REPAIR-0473`. The last of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** an answer containing the invisible "null" character could never be stored, so the whole paid answer was lost over one character.
- ✅ **Now:** the machine swaps each null character for the standard "unreadable character" symbol (�), which shows exactly where it was, and the answer records how many were swapped. Nothing else in the answer changes, and answers without the character are untouched.
- ⚖️ **A decision taken for you to review:** the alternative was to reject such an answer outright, keeping the principle that answers pass through unchanged but losing the work. The reasoning is recorded, and switching is a one-line change.
- ✅ Tested: a new check failed on the old code (the answer was lost) and passes now; the real server stores such an answer as a contribution; 108 machine tests pass; all eight deliberately broken versions were caught; strict lint clean.

## 2026-09-24 — A machine no longer locks itself out on an answer the server can never accept (`SIGNOFF-REPAIR.4.4.10.2`)

`REASONBRAID-REPAIR-0472`. The second of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** when the server refused an answer permanently (because of what the answer contained), the machine treated it as a broken connection. It reconnected, resent the same answer, was refused again, and never got back to work.
- ✅ **Now:** the machine records the refusal (the server's reason included), stops offering that answer, and carries on with its other work. Only refusals about the answer's own contents count as permanent. Login problems, outages and version mismatches are still handled by reconnecting.
- ⚠️ Tracked next: the answer itself is still lost when it contains the null character. The last fix decides what the machine does with that character.
- ✅ Tested: two new checks (one on first sending, one on resending after a reconnect) failed on the old code and pass now; 105 machine tests, five live suites and the two-machine demonstration pass; all ten deliberately broken versions were caught; strict lint clean.

## 2026-09-24 — The server now says clearly when an answer can never be stored (`SIGNOFF-REPAIR.4.4.10.1`)

`REASONBRAID-REPAIR-0471`. The first of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** input holding the invisible "null" character, which the database cannot store, was answered "internal server error", a reply that looks like a temporary outage. A machine given that reply for its answer kept retrying for ever.
- ✅ **Now:** such input is refused with a clear, permanent "cannot be stored" reply, and nothing of it is kept. This covers both the machines' channel and the command interface people use. A genuine server fault on normal input still reports as a server fault, so the two are never confused.
- ⚠️ Tracked next: the machine side (stop retrying anything refused permanently); and seven less-used command paths that still give the old reply.
- ✅ Tested: a new check failed on the old code and passes now; eight live suites pass; all five deliberately broken versions of the new rule were caught, after the first round showed a missing check (a real server fault was being confused with bad input), which was added; strict lint clean.

## 2026-09-24 — Measured: a single invisible character in an answer can lock a machine out for good (`SIGNOFF-REPAIR.4.4.10`)

`REASONBRAID-DOC-0154`. A check of a suspected risk; it turned out to be real.

- ✅ **Confirmed safe:** the largest answer a machine can now produce (256 KB, `.4.4.6`) is accepted by the server, even in its most expensive encoding. Double that is refused, so the server's limit really exists.
- 🔴 **Found:** an answer containing one particular invisible character (the "null" character, which the database cannot store) is refused by the server with a message that looks like a temporary outage. The machine keeps retrying the same answer and can never get back to work. A provider can produce that character; the test provider's own sample of garbled output contains it.
- ⚖️ Now three tracked fixes, in order: the server rejects such an answer clearly and permanently; a machine stops retrying anything the server rejects permanently; and a machine decides up front what to do with that character.

## 2026-09-24 — A provider that never answers no longer freezes the machine, and answers have a size limit (`SIGNOFF-REPAIR.4.4.6`)

`REASONBRAID-REPAIR-0470`. The sixth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** every job came with a time limit, but nothing enforced it. A provider that took a job and never answered froze the machine for good. Answers could also be any size, including sizes the server would refuse to accept.
- ✅ **Now:** when the time limit passes, the machine tells the provider to stop and records the job as "outcome unknown", with the reason. It never guesses the job failed or succeeded, because the provider may have done the work. An answer larger than 256 KB is stopped and recorded as failed, naming the limit; a partial answer is never passed off as a whole one.
- ✅ Tested: two checks failed on the old code (a frozen job, and an oversized answer accepted) and pass now, plus four more; 102 machine tests, four live suites and the two-machine demonstration pass. The deliberately broken versions first exposed a gap: nothing checked that the provider was actually told to stop. The checks were strengthened until all were caught. Strict lint clean.
- ⚠️ Tracked next: whether a machine can get stuck on an answer the server refuses outright; and charging a job's elapsed time to its budget.

## 2026-09-24 — A machine that cannot reconnect now waits longer between tries (`SIGNOFF-REPAIR.4.4.5.3`)

`REASONBRAID-REPAIR-0469`. Completes the fifth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** a machine that could not reconnect tried again every second, for ever. A server down for an hour met 3,600 reconnect attempts from each machine.
- ✅ **Now:** the wait doubles after each failure (1, 2, 4 … seconds) up to a minute, and goes back to one second once a reconnect succeeds.
- ✅ Tested: the waiting rule has its own check; 96 machine tests and the two-machine demonstration pass; three deliberately broken versions were all caught; strict lint clean.

## 2026-09-24 — A delivered job now shows as done, and stops counting against the machine's capacity (`SIGNOFF-REPAIR.4.4.9`)

`REASONBRAID-REPAIR-0468`. A defect found while testing the previous fix.

- 🔴 **Before:** a job a real machine had finished and delivered never showed as "done" in the operator's inbox view, only "received". Worse, the same check decides how busy a machine is, so every finished job kept counting as "in progress". A machine allowed N jobs at a time stopped getting new work after its first N, until old records were cleaned out. The tests had not noticed because they built their fake results in a shape no real machine produces.
- ✅ **Now:** the server recognises a finished job by the job the answer names, so it shows as done and frees its capacity slot at once. The tests now use results shaped the way a real machine sends them, plus one end-to-end check with a real machine.
- ✅ Tested: five checks failed on the old code (two of them the capacity checks) and pass now; six live suites and the two-machine demonstration pass; a deliberately broken version was caught; strict lint clean.

## 2026-09-24 — Finished jobs are no longer reported as undeliverable (`SIGNOFF-REPAIR.4.4.8`)

`REASONBRAID-REPAIR-0467`. A defect found while testing the previous fix.

- 🔴 **Before:** one round after a job finished successfully, the machine reported it as "undeliverable", and the server filed it in the dead-letter pile. An operator looking at the inbox saw every successful job marked as failed.
- ✅ **Now:** the machine treats a finished job as finished and says nothing more about it. The server also refuses on its own to file a job as undeliverable once its answer has arrived, whoever sends the report.
- 🔴 **Found along the way (now tracked, fixed next):** a successfully delivered job shows as "received" instead of "done" in the operator's inbox view.
- ✅ Tested: two new checks (one on the machine, one against the real server) failed on the old code and pass now; 185 core and machine tests, six live suites and the two-machine demonstration pass; three deliberately broken versions were all caught; strict lint clean.

## 2026-09-23 — A dropped connection or one unanswerable reply no longer shuts a machine down (`SIGNOFF-REPAIR.4.4.5.2`)

`REASONBRAID-REPAIR-0466`. The second part of the fifth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** three ordinary problems stopped a machine's program entirely: the connection dropping while it sent an answer, a provider reply going missing, and one job with unreadable instructions. The last also left every job after it in the same round undone.
- ✅ **Now:** a dropped connection makes the machine reconnect and resend. A missing reply is recorded as "unknown" and left for a person to decide, as designed. A job with unreadable instructions is reported once as undeliverable, and the other jobs carry on.
- 🔴 **Found along the way (now tracked, fixed next):** every job that finishes successfully is wrongly reported as "undeliverable" one round later.
- ✅ Tested: three new checks failed on the old code and pass now; 94 machine tests, six live suites and the two-machine demonstration pass; four deliberately broken versions were all caught; strict lint clean.

## 2026-09-23 — A machine no longer waits for ever on a server that stops answering (`SIGNOFF-REPAIR.4.4.5.1`)

`REASONBRAID-REPAIR-0465`. The first part of the fifth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** if the server accepted a machine's connection and then went silent, the machine waited for ever. It stalled completely, and because nothing reported an error, it never tried to reconnect. Measured: still waiting after 45 seconds, with no end in sight.
- ✅ **Now:** the machine gives up after 10 seconds trying to connect, or 30 seconds waiting for an answer. It then treats the silence like any other broken connection and reconnects. Measured: it gives up at 30 seconds exactly.
- ✅ Tested: the new checks use a stand-in server that accepts and never replies; 91 machine tests and three live suites pass; two deliberately broken versions were caught (one more could not be built); strict lint clean. One gap is stated in the record: only a 30-second check, run once and not kept, would notice the unbounded client being put back by hand.

## 2026-09-23 — A machine that is not properly connected no longer starts paid work (`SIGNOFF-REPAIR.4.4.4.2.2`)

`REASONBRAID-REPAIR-0464`. Closes the fourth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** a machine whose reconnect had failed still sent work to the provider and paid for it, although it could not yet deliver the answer. If it ever noticed, it shut itself down.
- ✅ **Now:** such a machine refuses to start paid work. The work waits untouched, and the machine reconnects and carries on instead of shutting down. Safety checks that need no connection still run first.
- ✅ Tested: a new check failed on the old code (the machine did the work) and passes now; eleven existing checks now run on properly connected test machines; 89 machine tests, six live suites (91 tests) and the full two-machine demonstration pass; three deliberately broken versions were all caught; strict lint clean.

## 2026-09-23 — The machine's own tests can now see an answer being sent (`SIGNOFF-REPAIR.4.4.4.2.1`)

`REASONBRAID-REPAIR-0463`. Test equipment for the fourth recovery gap; no product behaviour changed.

- 🔴 **Before:** the machine's own tests could never get a machine properly connected, so nothing below the slow full-system tests could see an answer actually being sent. A deliberately broken machine that claimed "sent" while sending nothing passed all 83 of them.
- ✅ **Now:** a small stand-in server lets a test machine connect for real. New checks show an answer sent at once when connected, and an answer kept while disconnected then sent exactly once on reconnect under its original id.
- ✅ Tested: the broken "claims sent, sends nothing" machine is now caught by the machine's own tests; 87 machine tests pass; strict lint clean.

## 2026-09-23 — A finished, paid-for answer can no longer be lost between "done" and "sent" (`SIGNOFF-REPAIR.4.4.4.1`)

`REASONBRAID-REPAIR-0462`. The first half of the fourth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** a machine recorded "the provider finished" and the answer to send as two separate saves. If it died between them, or was merely not yet reconnected, the answer was thrown away. The work was paid for, marked finished, and never sent; nothing could bring it back. Seven existing tests treated that loss as normal.
- ✅ **Now:** the "finished" record and the answer are saved together in one step. If the machine cannot send the answer right away, the answer waits and goes out on the next reconnect, under its original id so it is never counted twice.
- ⚠️ Still to do (tracked, next): a machine that is not yet reconnected should not start paid work at all.
- ✅ Tested: the seven tests, rewritten to demand the waiting answer, failed on the old code (7 of 7) and pass now; two new crash tests; 83 machine tests and six live suites (91 tests) pass; nine deliberately broken versions were tried: four caught, three not buildable, one caught only by the live suites, one hand-made "two separate saves" version caught; strict lint clean.

## 2026-09-23 — The tests guarding the hand-off to the provider can now see whether the provider was reached (`SIGNOFF-REPAIR.4.4.3`)

`REASONBRAID-REPAIR-0461`. The third of the seven recovery gaps found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** one test was meant to prove that a replacement machine refuses outdated work instead of sending it to the provider. Its stand-in provider was set up to refuse by itself, though. The test also passed with the safety check switched off, because the stand-in's own refusals produced the same outcome. Nothing in the tests could count provider calls.
- ✅ **Now:** the stand-in provider counts every call. The tests use one that *would* succeed, then check it was called zero times and that the recorded refusal is the safety check's own. Every step's result is checked, not thrown away, and a second test now looks for the right kind of event (a revision, not a contribution).
- ⭐ **Shown, not assumed:** with the safety check deliberately broken, the OLD test still passed and the new one failed. A second test the review had flagged turned out to catch the break already through an earlier check; only its last line was blind, and that line is fixed too.
- ✅ Tested: 13 + 2 live tests pass; 134 tests in the provider and machine packages pass; strict lint clean. No product behaviour changed.

The entries before those above were rotated into reachable Git history at the
**fifty-third rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show a0a5ca4d5f80a66617cbc409b492b7bb88525afb:CHANGELOG.md
```

That snapshot is 94676 bytes and 821 lines, and contains 96 dated
entries; its Git blob is `7605670483c2178a01229b32f9870336b3ba17de` and its SHA-256 is
`aa13afd931f461769636220f40bafda0e6201bc29400a16c5a4f6d56697682bd`. It carries the fifty-second rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **10 record(s) rotated out, 87 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
