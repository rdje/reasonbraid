# DEV_NOTES.md

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
- Technical: `AMBIGUOUS_ATTEMPT_ACTIONS` re-ask → available, `who` = the tenant's administrator, `effect` = the verb; CLI `rb node replay --allow-possible-duplicate --reason` via `possible_duplicate_reason` + `replay_request_body` (unit-tested; mutants 7/7). Closes `.4.4.7.2`, `.4.4.7`, `.4.4`.

## 2026-09-24 — An administrator can now authorize re-running a job whose outcome is unknown (`SIGNOFF-REPAIR.4.4.7.2.2`)

`REASONBRAID-REPAIR-0481`.

- 🔴 **Before:** when a provider call's outcome was unknown, the machine rightly refused to run the job again on its own. But nothing let anyone approve a re-run while accepting the risk of paying twice, so the job stayed stuck.
- ✅ **Now:** the tenant's administrator can approve the re-run with a stated reason. The job gets its own new budget hold (the original stays reserved in case the first call did charge), the machine runs it again, and the answer lands. The approval is recorded in the audit trail under its own name, applies only to jobs stuck for exactly this reason, and is refused if the conversation's budget has no room.
- ⚠️ Next: the command-line action, and marking the option available on the operator's screen.
- ✅ Tested: a full live run (a lost answer, the approval, the re-run, the answer landing) that could not happen before; the audit vocabulary checks; five further live suites; both deliberately broken versions were caught; strict lint clean.
- Technical: core `AdministrativeOperation::NodeCommandReplayPossibleDuplicate` + `RETRY_REQUIRES_AUTHORIZATION`; `migrations/0106` (sixteen kinds); `authority::replay_command_with_possible_duplicate_in_one_transaction` (precondition `dead-lettered: retry_requires_authorization`, fresh reservation via `create_reservation_in_tx`, `jsonb_set` flag + reservation, re-sequence); `POST /v1/nodes/replay` `allow_possible_duplicate` + `reason` → `reservation_id`. Control `node_work::a_possible_duplicate_replay_re_runs_an_unknown_outcome`.

## 2026-09-24 — A machine can now receive permission to retry a job whose outcome is unknown (`SIGNOFF-REPAIR.4.4.7.2.1`)

`REASONBRAID-REPAIR-0480`. The first part of the "ask again, accepting a possible duplicate" action.

- 🔴 **Before:** once a machine had a job, nothing sent later could change it. That included permission to retry a job whose outcome is unknown, so even an authorized retry could never reach the machine.
- ✅ **Now:** when a job is sent again with a fresh approval, the machine takes two things from it: whether a possible duplicate is allowed, and which budget hold to run under. It keeps everything that says what the job is. A resend can extend what a machine may risk; it can never change what it was asked to do.
- ⚠️ Next: the server-side action that sends such a resend, then the command-line action and a full end-to-end check.
- ✅ Tested: two new checks failed on the old code and pass now; 112 machine tests and two live suites pass; every deliberately broken version was caught; strict lint clean.
- Technical: `journal.rs` `with_refreshed_authorization(held, redelivered)` refreshes only `allow_possible_duplicate` and `reservation` in `record_command`'s fresh-decision branch. Controls in `worker_retry_policy`. Mutants 1/1 + hand 2/2.

## 2026-09-24 — The recovery options now say which ones actually exist (`SIGNOFF-REPAIR.4.4.7.1`)

`REASONBRAID-REPAIR-0479`.

- 🔴 **Before:** the list of recovery options for a job with an unknown outcome included "ask again, accepting a possible duplicate charge", as if it could be done. It could not, and the runbook and the book said the same.
- ✅ **Now:** every option says whether it is available today. That one says it is not, why, and what will change it. The runbook and the book describe what an operator can really do: a revision can be requested again with a new challenge; a first answer cannot be requested again yet.
- ⚠️ Next: building the "ask again, accepting a possible duplicate" action itself.
- ✅ Tested: a new check failed on the old code and passes now; the live suite passes; a deliberately broken version was caught.
- Technical: `AMBIGUOUS_ATTEMPT_ACTIONS: [AmbiguousAttemptAction; 4]` with `unavailable_because`; response fields `available` / `unavailable_because`; `node_channel` control extended; runbook and `node-channel.md` corrected.

## 2026-09-24 — Checked: operators were told to use a recovery action that does not exist (`SIGNOFF-REPAIR.4.4.7`)

`REASONBRAID-DOC-0155`. A review; no code changed.

- 🔴 **Found:** when a job's outcome is unknown, the operator's screen, the runbook and the book all suggest "ask again, accepting the risk of a duplicate charge". Nothing lets anyone do that. For a first answer (as opposed to a revision) there is no way at all to ask again today.
- ⚖️ **Decided:** first make the screen, runbook and book tell the truth about which actions are available now; then build the "ask again, accepting a possible duplicate" action properly, with its own separate budget, because the roadmap names it as one of the four ways to handle an unknown outcome.

## 2026-09-24 — The charter lookup no longer blames the caller for server faults (`SIGNOFF-REPAIR.4.4.10.1.2`)

`REASONBRAID-REPAIR-0478`.

- 🔴 **Before:** looking up a governance charter answered any database problem as "your request is invalid", and quoted the database's internal error message back to the caller. A real server fault was blamed on the user, and internal details leaked.
- ✅ **Now:** a server fault is reported as a server fault, with the details kept in the server's log. A request containing the null character gets the clear "cannot be stored" reply. Publications were checked too: that character cannot reach their storage, because the input is validated first.
- ✅ Tested: a new check (the null character, plus a simulated database fault) failed on the old code and passes now; three live suites pass; both deliberately broken versions were caught; strict lint clean.
- Technical: `CharterError::UnrepresentableInput` / `PublicationError::UnrepresentableInput` built by classifying `storage` constructors; the charter GET maps `Storage` → logged 500 and the variant → `400 unrepresentable_input` (`api::unrepresentable()`). Control `command_api::the_charter_read_tells_the_callers_input_from_the_stores_fault` (NUL digest; table renamed away for one request).

## 2026-09-24 — More places now say clearly when input can never be stored (`SIGNOFF-REPAIR.4.4.10.1.1`)

`REASONBRAID-REPAIR-0477`.

- 🔴 **Before:** in seven places (agent profiles, snapshots, derivations, assessments, references, quotas and evaluations), input holding the null character was still answered "internal server error" rather than the clear "cannot be stored" reply.
- ✅ **Now:** all seven give the clear, permanent reply through one shared rule; genuine server faults still report as server faults.
- ⚠️ Tracked next: two remaining places (publications and charters) that lose the information needed to tell the two apart.
- ✅ Tested: a new check failed on the old code (a profile with that character) and passes now; the shared rule is checked against every kind of database error; six live suites pass; strict lint clean.
- Technical: `api::storage_failure(cause, context)` routes `EvaluationError`, both `QuotaError`, `AssessmentError`, `DerivationError`, `SnapshotError`, `ReferenceError` storage arms and `profile_error` through `unrepresentable_input`. Unit `storage_failures::only_unrepresentable_input_is_the_callers` (stand-in `DatabaseError`); live `profiles::a_profile_holding_nul_is_refused_as_the_callers`.

## 2026-09-24 — Time spent on jobs now counts against a conversation's time budget (`SIGNOFF-REPAIR.4.4.6.1`)

`REASONBRAID-REPAIR-0476`.

- 🔴 **Before:** each conversation has a time budget (10 minutes by default) alongside its call and token budgets. Time was never charged once a job finished, so the budget only limited how many jobs could run at the same moment, never the total. However long the jobs took altogether, the time budget never ran out.
- ✅ **Now:** each machine times every job, rounding up to whole seconds, and reports the time alongside the tokens. The server charges it, so a conversation's time budget runs out like its token budget does.
- ✅ Tested: two checks (one on the machine, one against the real server) failed on the old code and pass now; 199 core and machine tests and three live suites pass. The deliberately broken versions exposed a gap ("always charge one second" went unnoticed because every test job was quick); a longer job was added, and all are now caught. Strict lint clean.
- Technical: `BudgetDimensions::attempt_usage(input, output, wall_clock_seconds)`; supervisor `seconds_since(dispatched_at)` (ceil, min 1) at every settlement; `ExecutionReport.wall_clock_seconds`; worker adds `usage.wall_clock_seconds`; `api::settle_work_item_reservation` reads it. Controls: two in `supervisor_bounds`, one live assertion in `node_work`.

## 2026-09-24 — The web console's inbox panel works (`SIGNOFF-REPAIR.4.4.2.2`)

`REASONBRAID-REPAIR-0475`. A defect found by the previous fix.

- 🔴 **Before:** the console's "Inspect inbox" panel had never worked. It asked the server with the wrong parameter name, so every request was refused, and it tried to show a field that does not exist. The check meant to keep the console honest had itself been written with the wrong name, so it agreed with the mistake.
- ✅ **Now:** the panel works and shows each job's delivery state, whether its answer was refused (and why), and any quarantine. The check now tests the panel against the server's own definitions, so the two cannot drift apart unnoticed again.
- ✅ Tested: the new check failed on the old console, once for each of the two mistakes, and passes now; strict lint clean.
- Technical: `web/app.js` inbox panel → `node_id`, `delivery_state`, `result_refusal`; `ui.rs` `the_inbox_panel_speaks_the_servers_contract` parses the panel's query with `axum::extract::Query::<InboxInspectionParams>::try_from_uri` and checks every `r.<field>` against a serialized `InboxRow`; `web-ui.md` corrected.

## 2026-09-24 — Operators can now see when a finished job's answer was refused (`SIGNOFF-REPAIR.4.4.2.1`)

`REASONBRAID-REPAIR-0474`.

- 🔴 **Before:** in the operator's inbox view, a job whose answer the server refused (for example because the conversation had closed) looked exactly like one whose answer was accepted. Both showed as "done".
- ✅ **Now:** each job in the view shows whether its answer was refused, and why. Accepted answers show nothing extra. The same information reaches the assistant-facing (MCP) view.
- 🔴 **Found along the way (tracked, fixed next):** the web console's inbox panel has never worked. It asks the server in the wrong way, and it reads a field that does not exist.
- ✅ Tested: two checks failed on the old code and pass now; six live suites pass; a deliberately broken version was caught; strict lint clean.
- Technical: `api::inbox_inspection` LEFT JOIN `idempotency` on `(tenant_id, node_id || ':' || command_id)` → `InboxRow.result_refusal` = `response_result->'error'` when `ok = 'false'`. Controls in `node_work` (refused → code; applied → null). Hand mutant (pre-0104 key) caught.

## 2026-09-24 — An answer containing the null character now gets through, marked (`SIGNOFF-REPAIR.4.4.10.3`)

`REASONBRAID-REPAIR-0473`. The last of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** an answer containing the invisible "null" character could never be stored, so the whole paid answer was lost over one character.
- ✅ **Now:** the machine swaps each null character for the standard "unreadable character" symbol (�), which shows exactly where it was, and the answer records how many were swapped. Nothing else in the answer changes, and answers without the character are untouched.
- ⚖️ **A decision taken for you to review:** the alternative was to reject such an answer outright, keeping the principle that answers pass through unchanged but losing the work. The reasoning is recorded, and switching is a one-line change.
- ✅ Tested: a new check failed on the old code (the answer was lost) and passes now; the real server stores such an answer as a contribution; 108 machine tests pass; all eight deliberately broken versions were caught; strict lint clean.
- Technical: `worker.rs` `storable_content(&chunks) -> (String, usize)`: U+0000 → U+FFFD, count in the result's `nul_replaced` (absent when 0). Decision `docs/decisions/2026-09-24_nul-in-provider-output-is-replaced-visibly.md`. Controls: unit, two `worker_refusals`, live `provider_output_holding_nul_lands_as_a_contribution`. Mutants 8/8.

## 2026-09-24 — A machine no longer locks itself out on an answer the server can never accept (`SIGNOFF-REPAIR.4.4.10.2`)

`REASONBRAID-REPAIR-0472`. The second of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** when the server refused an answer permanently (because of what the answer contained), the machine treated it as a broken connection. It reconnected, resent the same answer, was refused again, and never got back to work.
- ✅ **Now:** the machine records the refusal (the server's reason included), stops offering that answer, and carries on with its other work. Only refusals about the answer's own contents count as permanent. Login problems, outages and version mismatches are still handled by reconnecting.
- ⚠️ Tracked next: the answer itself is still lost when it contains the null character. The last fix decides what the machine does with that character.
- ✅ Tested: two new checks (one on first sending, one on resending after a reconnect) failed on the old code and pass now; 105 machine tests, five live suites and the two-machine demonstration pass; all ten deliberately broken versions were caught; strict lint clean.
- Technical: `ChannelError::permanent_refusal()` (400/413/422, not `protocol_incompatible`; unit-pinned); `Node::deliver_at` is the one send path for `emit_event`, `deliver_journaled_event` and the reconcile's re-emission; a permanent refusal → `record_event_refusal` + `acknowledge_event` → `EventDelivery::Refused`. Stub `start_refusing(epochs, marker)`; `tests/worker_refusals.rs` 2 controls. Mutants 8/8 + hand 2/2.

## 2026-09-24 — The server now says clearly when an answer can never be stored (`SIGNOFF-REPAIR.4.4.10.1`)

`REASONBRAID-REPAIR-0471`. The first of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** input holding the invisible "null" character, which the database cannot store, was answered "internal server error", a reply that looks like a temporary outage. A machine given that reply for its answer kept retrying for ever.
- ✅ **Now:** such input is refused with a clear, permanent "cannot be stored" reply, and nothing of it is kept. This covers both the machines' channel and the command interface people use. A genuine server fault on normal input still reports as a server fault, so the two are never confused.
- ⚠️ Tracked next: the machine side (stop retrying anything refused permanently); and seven less-used command paths that still give the old reply.
- ✅ Tested: a new check failed on the old code and passes now; eight live suites pass; all five deliberately broken versions of the new rule were caught, after the first round showed a missing check (a real server fault was being confused with bad input), which was added; strict lint clean.
- Technical: `api::unrepresentable_input` (SQLSTATE `22P05`/`22021`) → `400 unrepresentable_input` in `From<sqlx::Error>` for the node channel's `ApiError` and `ControlApiError`; `ApplyError::Sql` and unrepresentable `EvaluationError::Storage` routed through it; `status_for_code` maps it. Control `node_work::input_the_store_cannot_hold_is_refused_as_the_callers` (jsonb, text, command API, and an injected store fault staying 500).

## 2026-09-24 — Measured: a single invisible character in an answer can lock a machine out for good (`SIGNOFF-REPAIR.4.4.10`)

`REASONBRAID-DOC-0154`. A check of a suspected risk; it turned out to be real.

- ✅ **Confirmed safe:** the largest answer a machine can now produce (256 KB, `.4.4.6`) is accepted by the server, even in its most expensive encoding. Double that is refused, so the server's limit really exists.
- 🔴 **Found:** an answer containing one particular invisible character (the "null" character, which the database cannot store) is refused by the server with a message that looks like a temporary outage. The machine keeps retrying the same answer and can never get back to work. A provider can produce that character; the test provider's own sample of garbled output contains it.
- ⚖️ Now three tracked fixes, in order: the server rejects such an answer clearly and permanently; a machine stops retrying anything the server rejects permanently; and a machine decides up front what to do with that character.
- Technical: census of `node_channel::events`' error statuses (domain refusals are 200 + `refused`); `node_work::the_largest_result_a_node_can_produce_is_accepted` (256 KiB of U+0001 accepted; 512 KiB → 413). U+0000 → `jsonb` "unsupported Unicode escape sequence" → `500 dependency_unavailable` → `calls_for_reconcile` → re-emission → wedge. Split `.4.4.10.1`–`.3`.

## 2026-09-24 — A provider that never answers no longer freezes the machine, and answers have a size limit (`SIGNOFF-REPAIR.4.4.6`)

`REASONBRAID-REPAIR-0470`. The sixth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** every job came with a time limit, but nothing enforced it. A provider that took a job and never answered froze the machine for good. Answers could also be any size, including sizes the server would refuse to accept.
- ✅ **Now:** when the time limit passes, the machine tells the provider to stop and records the job as "outcome unknown", with the reason. It never guesses the job failed or succeeded, because the provider may have done the work. An answer larger than 256 KB is stopped and recorded as failed, naming the limit; a partial answer is never passed off as a whole one.
- ✅ Tested: two checks failed on the old code (a frozen job, and an oversized answer accepted) and pass now, plus four more; 102 machine tests, four live suites and the two-machine demonstration pass. The deliberately broken versions first exposed a gap: nothing checked that the provider was actually told to stop. The checks were strengthened until all were caught. Strict lint clean.
- ⚠️ Tracked next: whether a machine can get stuck on an answer the server refuses outright; and charging a job's elapsed time to its budget.
- Technical: supervisor `within` (`tokio::time::timeout_at`) over `invoke` and every `next()`; `cancel_within_grace` (5 s); `Terminal::Lost(reason)` → extracted `settle_unknown(UnknownOutcome{…}, …)`; `MAX_OUTPUT_BYTES = 256 KiB` → `Terminal::FailedKnown`; `tokio` `time` feature. `tests/supervisor_bounds.rs` 6 controls (`SlowCancel`, `SilentInvoke`). Mutants: first pass 3 missed in `cancel_within_grace`, strengthened to 0 missed; hand mutant on the `invoke` wait caught.

## 2026-09-24 — A machine that cannot reconnect now waits longer between tries (`SIGNOFF-REPAIR.4.4.5.3`)

`REASONBRAID-REPAIR-0469`. Completes the fifth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** a machine that could not reconnect tried again every second, for ever. A server down for an hour met 3,600 reconnect attempts from each machine.
- ✅ **Now:** the wait doubles after each failure (1, 2, 4 … seconds) up to a minute, and goes back to one second once a reconnect succeeds.
- ✅ Tested: the waiting rule has its own check; 96 machine tests and the two-machine demonstration pass; three deliberately broken versions were all caught; strict lint clean.
- Technical: `reasonbraid_node::reconcile_backoff(n)` = `1s.saturating_mul(2.saturating_pow(n)).min(60s)`; `rb-node` loops `while let Err(e) = node.reconcile()` with a per-recovery failure count. No jitter (decided; trigger: a fleet profile).

## 2026-09-24 — A delivered job now shows as done, and stops counting against the machine's capacity (`SIGNOFF-REPAIR.4.4.9`)

`REASONBRAID-REPAIR-0468`. A defect found while testing the previous fix.

- 🔴 **Before:** a job a real machine had finished and delivered never showed as "done" in the operator's inbox view, only "received". Worse, the same check decides how busy a machine is, so every finished job kept counting as "in progress". A machine allowed N jobs at a time stopped getting new work after its first N, until old records were cleaned out. The tests had not noticed because they built their fake results in a shape no real machine produces.
- ✅ **Now:** the server recognises a finished job by the job the answer names, so it shows as done and frees its capacity slot at once. The tests now use results shaped the way a real machine sends them, plus one end-to-end check with a real machine.
- ✅ Tested: five checks failed on the old code (two of them the capacity checks) and pass now; six live suites and the two-machine demonstration pass; a deliberately broken version was caught; strict lint clean.
- Technical: `migrations/0105_node_inbox_consumed_by_command.sql` redefines `node_inbox_state`'s `consumed` rung as `e.payload->>'kind' = 'work_result' AND e.payload->>'command_id' = i.command_id` (was `e.operation_id = i.command_id` since `0021`); four `node_channel.rs` fixtures re-shaped to `op_…` ids; `node_work::a_completed_item_is_never_dead_lettered` asserts `consumed`.

The entries before those above were rotated into reachable Git history at the
**fourteenth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 27f4a670ffb40f9243c7159e5390a04c628d611b:DEV_NOTES.md
```

That snapshot is 73662 bytes and 562 lines, and contains 61 dated
entries; its Git blob is `91090c74446fce54b3d1de21874000579b2d3005` and its SHA-256 is
`1c0b6daf8cc9ff71872ae286590b6bf06faa10c25d300a41b13cf2ad584cd164`. It carries the thirteenth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **9 record(s) rotated out, 53 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
