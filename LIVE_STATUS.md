# LIVE_STATUS.md — authoritative live progress tracker

Rows use only **Done · Mostly Done · In Progress · Not Started**. This is a current
snapshot. Historical implementation and verification records live in the phase
task-trees and git; the pre-review snapshot is `9c2d2ba:LIVE_STATUS.md`.

## Qualification correction
🔴 **THE PAIR WAS GATED AT TWO CADENCES, AND THE LOOSER HALF IS NOW A PURE FUNCTION (`.7.3.6.3`, REPAIR-0268).**

The R3 pack advertises two deny-policies and enforces them; nothing derives one from the other. Both sides were already gated — by two earlier leaves that did not know about each other — **at very different strengths**.

- 🔴 A moved ADVERTISEMENT is refused **every commit** (the doctrine gate). A moved BEHAVIOUR was caught only by the real-browser suite, which runs in CI **at push** — cadence ~300 commits, tree **237 ahead**. One half of one claim, guarded hundreds of commits more loosely, invisible because both were green.
- ✅ `refusing_policy` is now a pure function: no network, no browser, no filesystem, so ordinary `cargo test` covers it — **8 → 12 tests, 0.08 s**. ⛔ The end-to-end control is NOT replaced; a pure function cannot show the decision is wired to Chrome's `Fetch` domain.
- ⭐ **Both directions observed RED.** Advertisement flipped with the worker untouched → the census refuses by name. Worker neutralized two ways → `9 passed; 3 failed` (the original defect) and `8 passed; 4 failed` (the almost-fix). The blackout arm fires on the second and not the first, so the suite separates a policy from a prohibition.
- ⚠️ **Rejected:** plumbing the advertised word into the worker. It would make one value decide whether the pack isolates anything, and `resolver_capabilities` has no tenant column — **a one-source design that puts the source where an attacker can reach it is worse than two sources with a gate between them.**
- ✅ **VERIFIED:** browse bins **12/12**; `browser_roundtrip` **18 passed, 0 failed** in 31.20 s; census `--check` rc=0; clippy rc=0; fmt rc=0.

🔴 **A REPLACE IS COMPLETE, AND THE HTTP VERB DOES NOT PERFORM ONE (`.7.3.6.2`, REPAIR-0267).**

`resolvers::register` INSERTed 18 columns and updated **6** on conflict, so a corrected advertisement never reached an existing row. Reproduced at runtime, both halves: the boot sync left drift in place (`left: "allow"`, `right: "deny"`), and the verb answered a narrowing re-registration `{"registered":true}` — **`left: 200`, `right: 409`**.

- ⚠️ **The re-read the acceptance demanded changed the repair.** `.11.9.1.1.1` had already measured that `resolver_capabilities` has **no tenant column**, so any tenant administrator addresses any row including `r0-https-fetcher`'s. ⛔ Writing all 18 columns would have handed that unbound principal eleven more on a site-global row — rejected.
- ✅ **Split by caller:** `sync_gated_entries` at boot replaces **completely** (`registered_at` excepted); `POST /v1/resolvers` **refuses** an already-registered id by name (409). ⭐ This NARROWS `.7.1` and does not discharge it — creating a new site-global resolver on a tenant-admin grant is untouched, and that is the larger half.
- ✅ **VERIFIED:** `profiles` **58 passed, 0 failed**, all 56 pre-existing unchanged; clippy `-D warnings` rc=0; fmt rc=0.
- 🔎 **After a debug-tree retirement, run `cargo build --workspace --bins --locked` before the pg suites** — two R2 controls refuse to report without `target/debug/reasonbraid-extract`, which is correct of them and is a consequence DOC-0060 did not name.
- ⚠️ **Citation corrected at five sites:** the *in the same act* promise is `resolvers.rs:211`'s doc comment, not the 2026-09-12 decision record. The finding is unchanged.

🔴 **THREE OF THE THIRTY-SIX ADVERTISED POLICY LINES ARE ENFORCED (`.7.3.6.1`, REPAIR-0266).**

A resolver pack publishes six policy fields to every caller that reads the §12.2 registry; six packs ship, so **36 lines**. Adjudicated: **`enforced` 3 · `unverified` 5 · `vacuous` 14 · `misdescribed` 7 · `undefined` 7** — every verdict with its evidence in `.doctrine/advertised_policy_verdicts.tsv`. Re-derive with `python3 -B scripts/census_advertised_policies.py`; never read the numbers from this line.

- 🔴 **Four of the six fields are consumed by NOTHING** — `redirect_policy`, `archive_policy`, `subresource_policy`, `javascript_policy`: **34 field mentions on 32 lines, 0 reads**, every one a declaration, a write or a comment. ⭐ Enumerated in BOTH directions — an occurrence no rule explains defaults to `read`, so the finding can only be under-stated. ⇒ `.7.3.5` repaired the BEHAVIOUR of two lines without connecting the advertisement to the enforcement.
- ⚠️ **`vacuous` is 14 and is not a synonym for safe.** R0 advertises `javascript_policy: "deny"` and runs no script engine: true today, guarded by nothing, false the day R0 gains one — exactly how `.7.3.5`'s two lines behaved until the pack started executing pages.
- 🔴 **`archive_policy` is defined nowhere in the repository**, and R2 advertises it `deny` two fields above a `media_types` list containing `application/zip` and `application/x-tar` that its own worker expands.
- 🔴 **The instrument carried the defect it was built to find.** Keyed by pack and field alone, flipping R3's `subresource_policy` `deny` → `allow` in the producer left the census GREEN at `enforced — .7.3.5`. The advertised VALUE is now part of the verdict's key; measured in situ, restored byte-identical, replayed against the repair.
- ✅ Registered as a doctrine gate — 0.044 s, self-test 26/26 in 0.067 s, `make gate` 21 checks green. ⚠️ Calibrated over all **594** commits, not a 300 window that returns a survivorship 0: **4 (0.7%)**, all four commits that ADDED a pack.
- ⛔ **No product code changed.** Four defects routed OUT with their own acceptance — `.7.3.6.2` (the upsert writes 6 of 18 columns), `.7.3.6.3` (advertisement and enforcement are two constants a comment holds together), `.7.3.6.4` (the egress ladder admits a wider pack than the caller asked for), `.7.3.6.5` (the undefined and misdescribed terms, **G4's open strand among them**).

✅ **THE DEBUG BUILD TREE IS RETIRED, AND `cargo clean` WOULD HAVE TAKEN THE EVIDENCE WITH IT (`.11.4.3.1.8`, DOC-0060).**

Option **(b)**, taken by the director on 2026-09-19 — the rebuild cost accepted explicitly, which forecloses the leaf's third outcome ("measured and not worth acting on"). **≈174 GiB recovered by `df`** (666,182,692 → 483,578,156 KiB used).

- 🔴 **The finding is the blast radius, not the bytes.** The leaf wrote option (b) as *"a whole-tree `cargo clean`"*, and `cargo clean` removes the entire `CARGO_TARGET_DIR` — including `target/pg-tests` (**26** tracked citations), `target/ci-browser` (**7**) and `target/browser-lifetime-controls` (**5**). ⭐ The census that protects those from the fixture reaper does not protect them from cargo's own broom. Scoped to `target/debug`.
- ✅ Frozen manifest → retirement → residue census: **182,302,640 KiB / 1,826,894 files** before; `target/debug` absent after, all three evidence directories byte- and file-identical, `run-9_ueev0t` present.
- ⚠️ `du` is logical bytes and does not establish physical recovery on a cloning filesystem — the published figure is the volume's own `df`.
- ⛔ **Not claimed: faster builds.** No experiment was run, and the measured cause is a HOST property (`syspolicyd` at 44% CPU, swap 6,151/7,168 MB during a 68m 16s rebuild).

✅ **BLOCKER C2 IS CLOSED — ALL FIVE GATE RECORDS RE-DERIVE (`.11.4.7`, REPAIR-0265).**

G1–G2, G3, G4, G5 and G6–G7, line by line, each verdict from `stands`/`narrowed`/`must be re-earned` with the command that produces it. ⛔ **No record's conclusion changed**: G6–G7 stays NOT MET for Internet exposure and G3 stays blocked as binding use.

- 🔴 **G4 must be re-earned and is NOT fully discharged.** Three defects landed inside its own clause (REPAIR-0224, REPAIR-0227, `.11.14.3.12`), all repaired. ⚠️ **One strand stays open**: the R3 `vm_container` deferral is a claim *about the deployment*, not an enforced property, and `.7.3.6` owned it — since split by `.7.3.6.1`, the strand is `.7.3.6.5`.
- 🔴 **4 of 6 evidence pointers moved** (`profiles.rs` 23→56; `profiles 23` was G4's only citation). ⛔ The numbering instrument was wrong first — it counted helpers — and was rebuilt with a self-check before any conclusion.
- ✅ **G5's three stand**, on a positive-controlled census: the first pattern returned 0 and was wrong; corrected it finds 3 hits, all withdrawals. H1 null preserved, no new benchmark evidence.
- `profiles` 56/56 · `evaluation` 3/3 · `routing` 2/2 — **61 tests, 0 failures**.

✅ **G3'S SEVEN CLAIMS RE-DERIVED — AND ITS EVIDENCE POINTERS NO LONGER RESOLVE (`.11.4.7.3`, REPAIR-0264).**

**3 stand · 1 narrows · 3 had to be re-earned**, against **38 tests, 0 failures** across every suite the record cites (`policy` 14/14, `invitations` 6/6, `compiler` 8/8, `publisher` 7/7, `reconciler` 3/3).

- 🔴 **G3 cites its evidence by test POSITION.** `policy.rs` went **11 → 14** tests; **2 of 10** pointers now name a different test, including `policy 10` — the **correction clause's only evidence** — which today resolves to a publication-authority test. ⭐ Three more are unresolvable from the document: the suite's count equalled the highest index cited, so `publisher 2` is ambiguous, and `publisher` now holds **7**.
- 🔴 **AUTHORITY had to be re-earned and its own citations could never have caught the defect**: `authority_holds(pool, grant_id)` asked whether a grant EXISTS, never whether the caller HOLDS it, at 5 sites in 4 spellings. ⚠️ Residual OPEN: `GrantAction` cannot express a publication or correction target.
- ⛔ **A 'cited evidence no gate can run' concern was RAISED AND REFUTED** — all six suites absent from `SERVER_SUITES` are offline and CI runs them.
- ⚠️ **Blocker C2's cell was stale by two** (*four remain* where three of five were re-derived); corrected, with the command to re-derive it. **`.11.4.7.4` is the last open child — closing it closes C2.**
- ⛔ The 2026-09-07 record and Demonstration B stay byte-unchanged; no gate proposed.

✅ **THE LEASE WAS WRITTEN BY ONE CLOCK AND READ BY ANOTHER, SO THE PUBLISHED 60 s TTL WAS NOMINAL (`.4.2.3.1`, REPAIR-0263).**

🔴 **`node_leases.lease_expires_at` was written `Utc::now() + LEASE_TTL` by the server process and compared against `now()` by the database**, so the real duration was `60 s ± S` for whatever `S` the two clocks disagreed by. Driven ten minutes ahead at the writer's own parameter, the unrepaired product granted **660.0 s of lease on the database clock** — both writers, **38 passed / 2 failed**.

- ⭐ **The deciding measurement was in the wire:** one `HandshakeResponse` carried `lease_expires_at` (process clock) beside `server_time` = `clock_timestamp()` (database) — and `server_time` is what the node corrects every server instant against.
- ✅ **The database clock owns the column** at all five sites. ⛔ One clock is not one function: `now()` where the statement IS its transaction, `clock_timestamp()` where it is not — `verify_fencing_in_tx` runs inside a caller transaction that may block on the tenant guard, and `now()` there would re-admit a lapsed lease.
- 🔴 **Three writers, not two** — `.4.2.3`'s census was right when run; `.4.1.5` added the replacement's afterwards.
- ⚠️ **Three of the five sites cannot be falsified by any control here** and say so in the leaf, the book and the record rather than carrying a control that would be green either way.
- ✅ **VERIFIED:** `bash scripts/run_pg_tests.sh node_channel` → **40 passed, 0 failed** in 53.08 s; both controls print **`60.0 s of lease left on the DATABASE clock`**, and the three controls the acceptance named pass unchanged.
- 🔎 Opened: `.11.22.1` — the frontier named one unfinished leaf twice while its caption said the duplicate was gone; **1 across all 15 trees**.

✅ **THE ACCEPTANCE GATE READ ONE CHECKLIST PER FILE, AND HAD DRIFTED OUT OF ITS OWN CORPUS (`.11.2.6`, REPAIR-0262).**

🔴 **`check_task_acceptance.sh` stopped at the first matching box and exited. This tree carries 194 `ROOT CAUSE` boxes; it read one, at line 388, from a leaf closed long before — on every code commit for ~200 commits.** ⭐ One cause, two defects: while inert, its vocabulary drifted — it blocked on `ADDRESSED` (14) and `ROOT CAUSE` (13) against the corpus's `NO REGRESSION` (203), `FIX / LOCKSTEP` (192), `REPRODUCE / ISSUE` (142).

- ✅ Scope is now the **deepest leaf whose `Status:` turns `done` in the staged diff** — from the hunk headers, never the commit message (a pre-commit hook runs before the message exists) and from the staged blob, never the working tree.
- ✅ Label families are a census-derived project seam, `.doctrine/acceptance_labels.txt`. Calibrated over 300 commits: **57.7% → 20.6%**.
- ✅ The gate gained the `--self-test` it never had (six arms; the negative arm is the extractor it replaced). `scripts/tests/test_task_acceptance.py` rewritten — **13 tests, OK**.
- 🔴 **The in-situ falsification caught the first repair passing with its own checklist deleted** — the leaf *discusses* `NO REGRESSION`, so prose mentioning it answered the question. Fixed by anchoring the family as the bullet's **lead-in**; 17.5 % → **20.6 %** refused, three points paid for correctness.
- ⛔ **`--debt` counts 73 of 281 closed leaves; they are NOT backfilled.** Writing the missing bullets in would manufacture evidence after the fact — the exact offence the gate guards against. ⚠️ Not a quality claim about those changes: most were verified, the record of it is what is missing. Decision: `docs/decisions/2026-09-19_acceptance-debt-is-counted-not-backfilled.md`.

⚠️ **THE ACCEPTANCE GATE ENFORCES A VOCABULARY THIS PROJECT DOES NOT USE (`.11.2.6`, calibration — NOT yet repaired).**

🔴 **`check_task_acceptance.sh` validates ONE checklist per staged tree file; this tree has 194 `ROOT CAUSE` boxes and it reads line 388 on every code commit.** Calibrating the repair found a larger second defect: the census returns `NO REGRESSION` **203**, `REPRODUCE / ISSUE` **142**, `FIX / LOCKSTEP` **192** against the gate's hard-gated `ADDRESSED` **14** and `ROOT CAUSE` **13**.

- ✅ Four candidates priced over 300 commits: **23.8% · 25.5% · 57.7%** rejected; **deepest closed leaf + corpus-derived vocabulary at 20.9%** is the candidate.
- ⭐ **20.9% is past drift, not a forecast** — the gate only examines the leaf being closed now. Four of five sampled refusals carry no regression evidence anywhere: real omissions an inert gate never asked about.
- ⛔ **Not repaired yet.** The leaf stays `pending` with the design settled by measurement; the historical debt is named for an explicit decision rather than silently inherited.

✅ **ONE NODE PRESENCE READ ASKED NOBODY WHO WAS CALLING (`.3.5.5`, REPAIR-0261).**

🔴 **`GET /v1/nodes/presence` took no `HeaderMap`** — no principal, no authorization — then `SELECT … FROM node_presence WHERE node_id = $1` on a view carrying `tenant_id`, while `GET /v1/admin/nodes/presence` gated the SAME view behind a `tenant_admin` grant, on one port in one process.

- ⚠️ **Reproduced at 200 against 401**, with the other 37 tests passing so the control discriminates exactly this repair. Because the handler read no headers, that path was **every** caller's — the cross-tenant case is the same execution, not an extrapolation.
- ✅ **Authenticate, then DERIVE the tenant from the principal** — no wire change, since a principal belongs to exactly one tenant structurally. The parser is shared, not copied. ⛔ A foreign node answers `unknown_node`, the SAME answer an absent node gives: the existence oracle is closed (§9.8 `scope_hidden`).
- ⭐ **Cost measured at ZERO for the only caller:** `web/app.js` already sends the header on every GET and its comment claimed the gates applied. No node client calls the route.
- ✅ `node_channel` **38/38**, `node_replacement` **2/2**, clippy `-D warnings` clean, book rebuilt. Decision: `docs/decisions/2026-09-18_node-presence-is-read-by-its-own-tenant.md`.
- ⛔ **NOT claimed:** that the node channel's authentication is settled generally — `presence` was its only GET; the POST verbs use fencing/enrolment tokens, unaudited here.

✅ **THE READ CENSUS COULD NOT SEE 34 OF ITS OWN ROUTES (`.3.5.4`, REPAIR-0260).**

🔴 **`.3.5.3` published "24 `get(…)` routes in `api.rs`"; its own commit had 54, and the binary merges three routers for 58.** The leaf was opened to follow 10 delegating handlers and found the population wrong by more than half. ⛔ The `24` had **no producer** — prose in a leaf, nothing deriving it, so nothing could contradict it.

- ✅ `scripts/census_get_route_binding.py` enumerates all 58 across the three mounted routers, reports what the caller supplies, whether the handler authenticates, which authorization primitive it reaches **transitively**, and follows each `crate::module::fn` delegate one level. All 58 classified: **24 authorize · 6 derive the tenant and bind it · 24 site-global BY SCHEMA · 4 unauthenticated**.
- ⭐ **The 24 site-global routes are the false-positive class.** Every one of their 24 tables has no tenant column, so there is nothing to bind — a column-grep would have reported 24 defects. The schema, not the predicate, is what separates a design from a leak.
- 🔴 **One product defect → `.3.5.5`: `GET /v1/nodes/presence` asks nobody who is calling** — no principal, no authorization, no tenant predicate, on a `node_presence` view that carries `tenant_id`, while the same view is gated at `/v1/admin/nodes/presence`. ⚠️ Bounded: a read, observability payload, needs a known unguessable node id; but it needs **no principal at all** and leaks existence. Source-measured; runtime reproduction is `.3.5.5`'s first act.
- 🔴 **The instrument found five defects in itself first**, including two on the dangerous axes — a false negative that read 11 gated admin routes as ungated, and a false positive that reported a gate on a route with none. All five are now two-sided self-test arms.
- 🔎 **Second finding → `.11.2.6`:** `check_task_acceptance.sh` validates **one** checklist per staged tree file; this tree has **194** `ROOT CAUSE` boxes and the gate reads line 388 every time.

✅ **THE SCAFFOLD SHIPPED AN ENFORCER REGISTERING CHECKS IT DID NOT CARRY (`.11.23`, REPAIR-0259).**

⚠️ **CORRECTED (DOC-0055): the original claim overstated this.** A project is instantiated from a GitHub **template** — the whole repository is copied — then cloned and bootstrapped, so a fresh project receives every script and commits fine. Confirmed in this repo's own history (`b932c05 "Initial commit"` then `823c2bc "bootstrapped from bedrock"`) and its remotes (**reasonbraid's set contains no bedrock entry**; bedrock has its own `origin` for maintaining the template — what is absent is a link from an instantiated project back to it). The bare-repo reproduction modelled a creation path nobody uses. **The real hazard is at UPDATE time:** `update_scaffold.sh` only copies listed files, so syncing into a project built from an older snapshot installs a registry naming checks that never arrive. The repair stands on that basis. `update_scaffold.sh` carried the registry of 18 doctrines and not the 7 scripts it names; a bare `git init` reproduction printed seven `?? (missing/not executable)` rows and a blocked commit.

- ⭐ Fourth instance of *a second copy nothing derives*: `NEUTRAL` is the scaffold's copy of the registry. All seven were EARNED here, and that list is the only mechanism that moves a doctrine anywhere — so the spine's improvements could not leave the instance.
- ✅ Fixed, `SCAFFOLD-COVERAGE` registered at **0.02 s**, bare-repo re-run now **zero** `??` rows. 🔴 The gate failed its own reproduction (a project need not carry the sync script — NOT CHECKED, not a breach), and falsification found its `$EXEMPT` guard DEAD because the parser could not see the conditional registration form.
- ⛔ Backflow to `rdje/bedrock` remains MANUAL: the scaffold pulls downstream only and bedrock is the root template. The seven are now transportable; carrying them up is a `BEDROCK-MAINTENANCE` leaf there.

✅ **A GATE BLIND TO THE ABBREVIATION OF THE VERY NOUN IT IS BUILT ON (`.11.2.4`, REPAIR-0258).**

🔴 **`check_visibility_policy.sh` exists because the superseded private-repository instruction leaked past two reviews — and all four of its clauses were anchored on the full word, so two live instances in the abbreviated form sat in the corpus with the gate returning rc=0.**

- ✅ Widening DERIVED from a ~60-line census, not invented: the abbreviation is the only uncovered spelling that *states* a visibility, while `private-repository instruction` lines put `private` before the noun and are corrections. Delta **3 → 11**, none lost.
- 🔴 **Six of the eight new matches were the project documenting its own blind spot** — the self-reference trap reached from the documentation side.
- ✅ Dispositioned by rule: **except what must stay verbatim (3), reword what is prose (5)**, so the changeable surfaces stay policed. Falsified four ways, including a genuinely new sentence refused by file and line.
- ⭐ `FRONTIER-STATUS`, registered one commit earlier, caught this leaf leaving a stale frontier row — firing on its author within the hour.

⭐ **DIRECTION RECORDED, UNSCHEDULED (2026-09-18): small-swarm orchestration is the EXISTING objective, not a pivot.**

The director asked for 2–5 agents coordinated on any given problem, scaling later. `README.md` already states that objective, and the shipped step vocabulary carries no programming-specific verb — `solicit`, `blind_solicit`, `critique`, `revise`, `synthesize`, `assess`, `vote`, `adjudicate`, `decide`, … A profile is versioned CONFIGURATION over existing verbs, and real `claude.rs`/`codex.rs` adapters already dispatch.

- ⛔ **The binding constraint is the ACCEPTOR, not orchestration.** Model output stays untrusted until DETERMINISTIC rules accept it; programming has acceptors, "any problem" often has none.
- ⚠️ 2–5 agents is in range; **thousands is a re-architecture**, not an extrapolation — the design serialises administrative acts on one guard row.
- ⛔ **It does not displace the LAN bar and no leaf is opened.** Much of it is inside that bar anyway: `blind_solicit`, rounds, quorum and adjudication are G3/G5 surfaces, and G5 is withdrawn.
- Record: `docs/decisions/2026-09-18_multi-agent-orchestration-is-the-objective-not-a-pivot.md`.

⛔ **INTERNET EXPOSURE IS DEFERRED — director instruction, 2026-09-18.**

> *"The internet exposure is not high priority right now. It needs to fully work on the local network first."*

- B1, B2 and B3 become **`yes (deferred)`**: the in-repo work is still owed and `.14` still owns it, but it leaves the frontier and no commit is spent on it until the LAN path is complete. ⛔ **`yes (deferred)` is not `no`** — a `no` row has nothing left for this repository to do; these three do.
- ⛔ **Priority changed, claims did not.** Phase 7 G6/G7 remains **NOT MET** and `.14`'s standing prohibition against turning the exposure profile on is untouched. Deferring the work moves the claim further away, not closer.
- ✅ **The trigger is DEFINED** (director, 2026-09-18): the LAN bar is **G0–G5 genuinely met plus G7 earned on the LAN**, G6 out of scope. The roadmap's gate table partitions exactly there — G6 is the only gate whose *unlocks* column names a transport. ⛔ The earlier phase-arithmetic candidate is WITHDRAWN as wrong in kind: repairs closing and behaviour matching can diverge in both directions.
- ⛔ **G7 splits.** Structural legs (restore, survival of induced failure, instrumentation) are earned once on the LAN; **load and SLO NUMBERS are anchored to a LAN** and must be re-derived under the Internet posture rather than carried across.
- ⚠️ **G7 is the least-built LAN gate, measured before the bar was accepted:** backup/restore real, metrics surface without objectives, load dev-scale only, **chaos/game-day absent — 0 hits across `crates scripts docs/book/src`**. ⚠️ And G5 is *withdrawn*, not merely unverified, so re-earning it is inside the bar.

✅ **THE FRONTIER TABLE GAINS A ROW PER STATE CHANGE AND NEVER RETIRES THE OLD ONE (`.11.22`, REPAIR-0257).**

🔴 **Censusing DOC-0049's stale pointer found a structural cause: 66 rows naming 49 leaves — 13 leaves with 2–4 rows each, and 10 status columns contradicting their leaf.**

- ⭐ It caught its author: `.11.21` carried a `done` row and a `pending` row, both written two commits earlier in the same session. A third historical firing surfaced too — `.11.15` records `.7.4.1` sitting at row 2 for seven commits after closing.
- ✅ Rule: every row's status column equals its leaf's own status, and row 1's leaf is not finished. ⛔ **A `done` row kept as history stays legal** — the rule refuses a row that lies, not a finished leaf, and a self-test arm pins that.
- ✅ 28 of 350 leaves carry no `Status:` line (27 use an `Opened:` bullet), so both forms are read; `active` is a real third value and is not finished.
- ✅ 9 stale rows removed, 1 corrected. `FRONTIER-STATUS` registered at **0.06 s**, falsified five ways and against the real defect.

⚠️ **A FRONTIER ROW POINTED AT A LEAF CLOSED FORTY COMMITS EARLIER (`.11.22`, DOC-0049).**

🔴 **Row 1 of the frontier — the single row a fresh session acts on — named `.7.4.5`, `done` since REPAIR-0216.** Found only by opening the leaf to work on it. Two days earlier REPAIR-0253 had reconciled **five** rows whose status column said `pending` while their leaves said `done`.

- **Censused, not asserted:** 10 tracked scripts mention a frontier and **0** relate a row's status column to its leaf's `Status:`. Both apparent hits were false positives of a loose key (a self-test fixture; a heredoc) — the `.11.8.1` shape, one commit later.
- ⛔ Three gates look adjacent and all miss it: `INDEX-FRONTIER` compares the two FILES to each other (both agreed, about a closed leaf); `TASK-STATUS` reads leaves, never tables; `TABLE-ARITY-RATCHET` reads cell counts, never cell meaning. The defect is in the seam between three checks each doing its own job correctly.
- ⚠️ A row deliberately naming a `done` leaf as HISTORY is correct and common, so the rule wanted is about row 1 and about the status column's AGREEMENT — not about closed leaves appearing in the table. Owned, measured and decided by `.11.22`; the pointer itself is corrected to `.11.2.4`.

✅ **UNFORMATTED RUST REACHED `main`, BECAUSE NOTHING CHECKED PER COMMIT (`.11.21`, REPAIR-0256).**

🔴 **`crates/reasonbraid-server/src/git.rs` failed `cargo fmt --all -- --check` at two sites from REPAIR-0251/0252** — found only because a LATER leaf happened to run the check while verifying a file it had not touched.

- ⭐ **A policy gap, not a lapse.** §16 routes full CI to pre-push and the per-commit gate to the enforcer, whose 18 checks had no formatting check; `cargo fmt --check` sat only in `COMMIT.md`'s pre-push list, and pushes go out in batches of ~300 commits.
- ✅ **Measured before proposing (`.11.6`)**: `cargo fmt --all -- --check` at **0.66 s** against an enforcer at **12.6 s** — about 5 %. ⚠️ That enforcer figure supersedes the anchored `3.15 s` of `.11.4.3.1.7.2`; the registry has grown since.
- ⛔ **Leaving it pre-push was rejected on evidence**: that is the arrangement that let the defect ship, and a check running once per ~300 commits cannot say which commit broke formatting.
- ✅ `RUST-FORMATTING` is doctrine 16 of 19, **seen RED** against injected unformatted code and green again. Its self-test reaches the tool, not just the verdict, and FAILS rather than skips when `rustfmt` is absent.
- 🔴 The registry's own `DOCTRINE-REGISTRY` guard refused my first entry: backticks in a bash-quoted description are command substitution the driver would EXECUTE on every commit.

✅ **A ROUTE KEY THAT WAS TOO LOOSE AND TOO TIGHT IN ONE EXPRESSION (`.11.8.1`, REPAIR-0255).**

🔴 **`stem()` DELETED path parameters, so the route census was wrong in both directions — and its own self-test ASSERTED the defect.**

- ⛔ Too loose: `/v1/snapshots/{id}` and `/v1/snapshots` collapsed onto one key, so the ITEM route was counted `described` by the SUBMIT route's contract line. Re-derived by hand: `/v1/calls` was credited by `GET /v1/calls/{call_id}` (`authority.md:833`) and no bare contract line for it exists.
- ⛔ Too tight: a mid-path parameter produced `/v1/snapshots/derivations`, which appears nowhere, while the book writes `GET /v1/snapshots/{id}/derivations`.
- ✅ **21 of 104 misclassified** — 18 wrongly `absent`, 3 wrongly credited. Corrected: **104 routes — 49 described, 3 mentioned, 52 absent**; and at `.11.8`'s own commit, **30 / 3 / 70** against its published **22 / 1 / 80**.
- ⛔ No left match edge, and that is measured: the hazard it guards is **0** on this surface and it would cost `$RB_URL/v1/admin/grants`. `tail_collisions()` prints the count every run, so the assumption is checked rather than commented.
- 🔴 The old self-test asserted `stem(…) == "/v1/calls/respond"` — a control that demanded the defect. Falsified five ways.

✅ **A CITATION IS WITHDRAWN BY ITS TENANT; A SHARED ROW IS TOMBSTONED BY THE SITE (`.7.4.4`, REPAIR-0254).**

🔴 **One verb carried two acts that do not share an authority.** A snapshot two tenants cite is ONE row, and `.11.14.1`'s binding to a *citing* tenant cannot separate two citers — so A's delete removed B's evidence and stamped B's receipt with A's reason, reproduced RED with that exact string on B's read.

- ✅ `DELETE /v1/snapshots/{id}` withdraws the caller's citation; `POST /v1/snapshots/{id}/tombstone` is a site act under the existing `evidence_expire` grant, the same authority the retention sweep needs.
- ⛔ *Refuse when another tenant cites* and *tombstone on the last withdrawal* were BOTH rejected: each makes the caller's outcome depend on whether a stranger cites the row, which is the §9.8 leak.
- ✅ The withdrawal is RECORDED not deleted (§12.9) and re-citing restores it; the tombstone stays irreversible, now behind an audited operator. Falsified four ways, each red by name. `profiles` 56/0, `evaluation` 3, `command_api` 39, `migration_upgrade` 4.
- 🔎 Routed out: `.11.8.1` (the route census misclassifies in both directions) and `.11.21` (`git.rs` committed unformatted).

✅ **A RETENTION RULE WITH NO RETIREMENT, ON BOTH POPULATIONS AT ONCE (`.7.3.2.1`, REPAIR-0253).**

🔴 **Two suites retain a workspace on failure — deliberately, because it is the evidence — and nothing had ever retired one. 1,896,248,619 bytes across 37 fixtures, of which 99.93 % is reproducible payload.** The premise had got worse while the leaf sat pending: `target/pg-tests` went from 12 clusters / 599 MiB to **25 / 1,372,355,705 bytes** in two days.

- ⭐ **What a fixture must keep was measured, not chosen.** `owner.json` and `completion.json` — the worker's own `browser_group`/`cleanup_confirmed` receipts — live INSIDE the payload directory, so dropping `.project-data` wholesale would have destroyed exactly what `.7.3.2` is about.
- ✅ **One rule, both populations: keep the receipt, drop the reproducible payload.** `scripts/census_retained_fixtures.py` never deletes a fixture and never signals a process; an id in use keeps a fixture, so a recycled id costs disk, never evidence.
- ✅ **34 of 37 reduced, 1,824,989,121 bytes dropped, 0 deleted**, residue green and `du` agreeing independently (384 KiB and 70,464 KiB remain). ⭐ The citation guard fired in PRODUCTION, keeping `run-9_ueev0t` whole.
- 🔴 **Mutation found three of the instrument's own controls vacuous** — the EPERM branch no self-owned pid can reach, a guard that only fires when the code is wrong, and a re-check nothing drove. All six mutants now red. Promoted: `a-guard-your-own-process-cannot-reach`.

✅ **THREE REF CLAUSES, AND A FOURTH DEFECT THAT MADE ALL THREE UNREACHABLE (`.7.2.8`, REPAIR-0252).**

🔴 **`acquire_into` passed the URL's fragment to `remote_at`, so every selector-bearing acquisition failed at the remote.** It survived because nothing ever drove the feature: 13 `acquire_local` call sites, none with a fragment.

- 🔎 A careful reading found three real defects in unreachable code; the FIRST control found why it was unreachable.
- ✅ An annotated tag resolves via `Ref::Peeled` (decided, not defaulted); an ambiguous short name is refused BY NAME listing both refs, and only when the commits differ; `refs/heads/..` is refused.
- ✅ Two falsification passes, because the fragment neutralization would have masked the other three. 21 tests pass, clippy rc=0.
- ⭐ **`ACQUISITION-KIND-DOC`, shipped one leaf earlier, refused the change** by command, the moment the wire mapping was added — before any commit was attempted. `.7.2`'s mechanism table is now fully discharged.

✅ **A SAFETY DEFAULT DISABLED BY OUR OWN CORRECTNESS (`.7.2.9`, REPAIR-0251).**

🔴 **`gix` ships a 16 MiB per-object allocation brake; this project had it switched off.** The default applies only at `Trust::Reduced`, and `init_opts` hardcodes `Trust::Full` — so every acquisition repository, fully trusted BECAUSE the server created it, let a remote-supplied pack entry declare any size.

- ⭐ **The defect lives in the seam between two correct decisions in two codebases** — full trust is right for config isolation and is exactly what disables the limit.
- ✅ **The repair refuses nothing that would have succeeded:** the limit is `max_bytes`, the ceiling the acquisition already declares. gix's 16 MiB was rejected for sitting *below* this project's own limits.
- ⛔ No crafted pack — reading the dependency established both the bound and the condition disabling it. One arm red by name; the negative arm labelled. 17 tests, clippy rc=0, book table updated.

✅ **A SECOND REFUSAL VOCABULARY, DOCUMENTED NOWHERE (`.7.2.10`, REPAIR-0250).**

🔴 **An acquisition refusal travels in `acquisition_error.kind`, not in `code`: 41 strings a client branches on, and the book documented none.**

- ⭐ **Not a `REASON-CODE-DOC` gap** — the two vocabularies share **0** strings, so the census keying on `code:` was right and the gap it left was real.
- ⭐ **Three keys were wrong before the fourth was right** (a missed `GitError` site, a regex catching `derived_kind:`/`actor_kind:`, a window overrunning into a `#[serde]` attribute) — all pinned as arms with their own reds.
- ✅ Gated as `ACQUISITION-KIND-DOC`, **0 of 200** commits, standing population discharged to 0, fired red end to end. The book table was asserted equal to the producers before it was written.
- ⚠️ The set is **OPEN at 5 sites** forwarding a worker-chosen kind, and the page says so.

✅ **A REFUSAL WIRED END TO END AND CONSTRUCTED BY NOTHING (`.7.2.7`, REPAIR-0249).**

🔴 **`GitError::TimedOut` was declared, given a Display message and mapped to a wire string — with no construction site. `GitLimits::max_time` was read by no code path, and `gix` was handed an interrupt flag nobody could raise.**

- ⭐ **The repair bounds the TRANSFER, not the caller's wait**, through the flag `gix` already polls — traced in the pinned dependency to the reader that fails once it is set.
- ✅ ***Before allocation* narrowed with its measurement:** all five other ceilings trip after `receive`, so consumption is bounded **in seconds, not bytes**. Published as a ceiling table in `deployment.md` and read back out of the rendered page.
- ⛔ **Decode brake and overflow guard both DECLINED on measurements**, with triggers stated: `gix-pack` has no ratio guard but allocates fallibly, and `max_time` is never operator-supplied.
- ✅ One arm red by name; two labelled. Routed: `.7.2.9`, `.7.2.10`.

✅ **A RIGHT CONCLUSION ON A FALSE PREMISE, GRADED SEPARATELY (`.3.4.3.1.1.1`, DOC-0047).**

🔴 **A signed-off leaf closed on *"nothing enforces the stored value"*. Three production sites enforce it. Its conclusion still stands — for reasons it did not give.**

- ⭐ Read from the PRE-REPAIR source at `9446034`: `rotate` sampled `now` AFTER signing so its stored value is never earlier; `enroll` sampled it 11 awaits and 16 database calls before, so it is earlier whenever a second boundary falls in between.
- ✅ **The refusal window is real and UNREACHABLE** — the final sub-second of a 600 s certificate against a rotate trigger at the halfway point.
- ⭐ **§4.1 on all three axes:** PROSE holds (re-derived), NAMED INSTANCE false and **withdrawn**. The control is named by its ABSENCE — 0 pre-repair, and the one the repair added prevents the divergence rather than observing the window.

✅ **A LEAF DISCHARGED ITS OWN GAP CLAIMS BY HAVING BEEN VERIFIED (`.11.2.5`, REPAIR-0248).**

🔴 **`GAP-CLAIM-CENSUS` asked "does this section contain a command?", and `TASK-ACCEPTANCE` puts one in every closed leaf by construction: 103 claim lines, 0 undischarged — inert everywhere.**

- ✅ **Six discharge tests priced against the same 103** (65.0%, 59.2%, 37.9%, 14.6%, 9.7%, **1.0%**). The 9.7% variant's extra nine were classified and **6 were false positives** — a claim inside a box discharged by a sibling box. **1 of 200 commits, 1 true positive, 0 false positives.**
- ⭐ The predicate shipped with a redundant clause and the falsification sweep deleted it — found by a degenerate arm that should have failed and did not.
- 🔴 **Its first real catch was a false claim inside a signed-off leaf:** `.3.4.3.1.1`'s *"nothing enforces the stored value"*, against three production sites. The conclusion survives for a different reason (the mTLS handshake enforces the signed expiry first); the direction that argument misses is routed to `.3.4.3.1.1.1`. Standing population **0 of 104**.

✅ **A REPLAY OVER POLICED HISTORY MEASURES DETERRENCE, NOT COST (`.11.18.2`, REPAIR-0247).**

🔴 **Two calibrations of the same candidate returned 0 of 559 and only one meant anything.** Once a rule is enforced, every commit that landed had already been edited until it passed.

- ⚠️ **Neighbour of the window trap, not the same one:** widening the window fixes that one and makes this one worse.
- ⭐ **ABSORBED into the window note**, needing a third clause beyond `.11.20.2`'s criterion — same activity, different failure mode, decided by the host already claiming the reader's question.
- ✅ **Census, not judgement:** **2 of 3** calibrating instruments replay over a corpus their own `--check` gates; detection costs **17 ms**; the distortion grows to 100% exactly when a mature gate is re-argued. Shipped as a DISCLOSURE, not a refusal.
- 🔴 Adding the arms caught a banner printing `23 arms` over **21** — the second instance of one hazard in two leaves.

✅ **THE ADVISORY AND THE BLOCKER NEVER GOVERNED THE SAME CORPUS (`.11.18.1`, REPAIR-0246).**

🔴 **`GAP-CLAIM-CENSUS` advised on 69 files and enforced on 15, for the whole of its life, because the script spelled its population twice.**

- **Measured:** `--all` iterates `git ls-files 'docs/tasks/*.md'` (69 — git's `*` crosses `/`); the blocker filtered `^docs/tasks/[^/]*\.md$` (15). **102 claims across 6 files advised, 97 across 5 enforced**, 54 nested artifacts invisible.
- ⛔ **The cause is the two spellings.** One `governed_filter` now serves both paths and the arms run that filter itself, so the agreement is structural.
- ✅ The *dated record, not a living leaf* defence was refuted by measurement (**66 of 559 commits** edit one), and the extension replays at **0 of 559** — `REASON-CODE-DOC`'s shape. ⛔ Full history, because the top-level replay's identical 0 is survivorship (`.11.18.2`).
- ✅ Five wrong rules, five arms red by name; the gate proved end-to-end on a real staged nested artifact — rc 1 after, rc 0 before.

✅ **A CONTROL THAT PASSES FOR AN UNRELATED REASON, PROMOTED (`.11.20.2`, REPAIR-0245).**

🔴 **A lesson that had fired in three consecutive leaves was reachable only from the tree it was written in.**

- ⭐ **EXTRACTED, not absorbed, and the criterion is the deliverable:** *absorb when the host note's THESIS is your rule; extract when your rule is a TOOL it merely uses* — the named difference from `.11.17.2`'s opposite ruling one commit earlier. The host note got SHORTER.
- ⭐ **New beyond relocation:** a NEGATIVE arm cannot be falsified against the old code, and needs a DEGENERATE implementation of the new rule; running a suite against broken code is the only time your controls are tested as reporters.
- 🔴 **The acceptance's own last clause was unsatisfiable** — a precedence chain let a leaf citation SHADOW the method anchor, so a correct pointer could not be confirmed. Both anchors are reported now, falsified from both sides. `MEMORY.md` 3,883 → 3,659 bytes.

✅ **THE MEMORY CENSUS SAW TWO WARNINGS WHERE EIGHT STOOD (`.11.20.1`, REPAIR-0244).**

🔴 **47% of `MEMORY.md` by weight was invisible to the instrument that exists to guard its weight.** Its model was one bullet; the template the file had been conformed to uses several.

- **Measured before the model was touched:** 2 visible, 8 present, **1,801 of 3,809 bytes** in four bullets never read. ⭐ The new model is parsed from `MEMORY_ARCHITECTURE.md` §6, never from the measured file. After: 8 warnings, **2,312 of 3,809 bytes (61%)**, reported per bullet with the cap and headroom.
- ⛔ **Two falsifications were needed.** The negative arms have no red against the old model, so a DEGENERATE rule was run to fire them by name — a negative arm falsified only against the old code is untested. 🔴 That exercise exposed an arm raising `IndexError`: a RED naming nothing.
- ✅ **Re-deciding `UNCITED` changed its answer** — 3 of 4 recorded, **1 not**: an environment fact living only in layer A, written there the same session, now `docs/decisions/2026-09-18_a-cargo-process-is-not-evidence-of-this-repo.md`. UNCITED **4 → 2**.

✅ **A SLASH IS NOT A RESOLUTION (`.11.17.2`, REPAIR-0243).**

🔴 **`POSITIONAL-REF` decided a citation was resolvable from the SHAPE of the string. Of 251 such occurrences, 39 named no tracked file and one was a suffix of THREE — the ambiguity the gate exists to refuse, passed by it.**

- **The defect.** `classify` read `kind = "pathed" if "/" in ref`. The branch for a reference WITHOUT a slash consulted the tracked tree; the branch WITH one did not. ⭐ The check existed and was wired to one branch — `docs/knowledge/trust-comes-from-the-check-not-the-shape.md`, extended with this as a second instance.
- **The population, pinned:** `python3 -B scripts/census_positional_refs.py --at 6f91897`. ⚠️ The leaf's own opening table said 28 partial / 5 unresolved where the tracked instrument says 27/6 — totals and the headline 251/212/39 reproduce exactly; the split came from an unrecorded pipeline.
- ✅ **The repair is the OLD question asked of both branches**, which produced the classes rather than arguing them, and `dependency` gains `Cargo.lock` as an oracle — earning `dependency-stale`, a failure mode nothing had before. ⛔ A suffix must fall on a SEGMENT boundary or the founding 93-false-positive hyphen bug returns as a false NEGATIVE.
- ✅ **Calibrated at 21 of 200 commits (10.5%)** against the 9.5% that argued this gate in. Refusing `partial` too was priced at **4.5%** and DECLINED on SHAPE rather than cost. Standing population **8 → 0**; 11 of 12 new arms falsified by name in situ, the two passing both ways LABELLED, and the gate itself fired red three times in the real enforcer.
- ⛔ **Verifying a dependency's IN-CRATE path is declined** — it needs the vendored registry a cold clone lacks, so the check would be green here and red there.

✅ **THE INSTRUMENT GUARDING `MEMORY.md` HAD BEEN DEAD SINCE THE COMMIT THAT RESHAPED `MEMORY.md` (`.11.20`, REPAIR-0242).**

🔴 **A census refused on every run for dozens of commits while its own `--self-test` reported 16 controls passing — and the defect it prevents recurred in the meantime.**

- **The defect.** It keys on a literal bullet name; `6199f43` (`.11.4.2.3`) renamed that bullet while conforming the file to its template and did not update the reader. ⛔ The commit that reshaped the file is the commit that blinded its guard.
- 🔴 **`SELF-TEST` stayed green** — all 16 controls are fixtures carrying the old name. The blind spot was introduced LATER, by a commit to a DIFFERENT file, which no care at authoring time covers.
- 🔴 **The defect recurred:** `.11.4.2.2` cleared this file to 1,395 bytes; one session took it to **6,123** of 7,168.
- ⭐ **The recurrence is `.11.16`'s shape:** **65% of the file was six standing-lesson bullets restating notes already durable in `docs/knowledge/`**. ✅ **6,123 → 3,825 bytes with nothing lost** — every lesson is a named pointer, all nine targets verified tracked BEFORE eviction.
- ⭐ **A live-corpus arm now reads the real file**, falsified by name against the pre-fix key.
- ⛔ **The general gate DECLINED on COST:** 1 of 12 then, 0 of 12 now — `REASON-CODE-DOC`'s shape — but 3.9 s on an 11.6 s enforcer, against the 1.01 s on 3.15 s that argued `SELF-TEST` in.

✅ **`init.rs` WAS NEVER IN THIS REPOSITORY (`.11.17.1`, REPAIR-0241).**

🔴 **Eight published citations named "a file that no longer exists". It was a dependency's source, and every cited line is exact at the pinned version.**

- **The premise was wrong, by command:** `git log --all --diff-filter=A --name-only -- '*init.rs'` returns NOTHING. The citations name `gix-0.87.1/src/config/cache/init.rs`, and all seven cited lines resolve to the code their prose quotes.
- ⛔ **17 citations qualified, not the 8 the leaf counted** — its key missed the partially-pathed ones in the same tables.
- ⭐ **DECIDED: a dependency citation carries its crate and version.** It resolves AND dates itself; a line number into a dependency moves on every upgrade.
- ✅ **The example-versus-citation obstacle is answered by NOT telling them apart** — no instrument can, so an example may not be written in the positional form, and the registry row is reworded. The fourth time this doctrine has policed its own description.
- ✅ **`unresolved` 9 → 0**, arm calibrated at **2 of 200 commits (1.0%)**, both the instances discharged. 🔴 It then flagged this leaf's own closing prose.
- 🔎 **Routed:** `pathed` is assigned on a slash alone — **39 of 251 do not resolve**, and one is a suffix of **three** tracked files. `.11.17.2`.

✅ **A TABLE SWALLOWS THE BLOCK THAT ABUTS IT (`.11.19.2`, REPAIR-0240).**

🔴 **`PHASE-3`'s eleven-line closing statement rendered as eleven table rows, under a header that had no rows of its own.**

- **The defect — the MIRROR of `.11.19`.** A GFM table body continues across any non-blank line, so a block abutting a table with no blank line is ABSORBED, one row per line, each padded to the header's width.
- ⭐ **The repair is the corpus's own convention:** the table had no data rows at all, and `PHASE-1` and `PHASE-2` already close with prose and no table. The rowless header/delimiter pair is removed rather than a blank line inserted.
- ✅ **Verified by the RENDERER:** data rows **59 → 48** (−11 exactly), tables **3 → 2**, the paragraph now a `<p>`.
- ⭐ **Calibrated over the FULL 551-commit history, because the 200-commit window could not see the arm at all** (the instance is older): **19 blocked (3.4%), and all 19 introduced a defect this work has repaired.** ⛔ The blocked set and the defect set are the SAME set.
- ⛔ **The first key over-counted by a third** — 15 against 11 — because it read a bullet list as absorbed. A list, heading, blockquote and HTML block END a table; plain prose does not. Not uniform, not in any specification, established one construct at a time against the renderer.
- ⭐ `BROKEN-TABLE` now gates BOTH directions of the boundary: **23 arms, ten of them negatives.**

✅ **TWELVE VERIFICATION-LOG ROWS HAD LOST THEIR FIRST TWO CELLS (`.11.19.1`, REPAIR-0239).**

🔴 **An insert-at-top edit re-emitted the row it displaced without its date and leaf id — twelve times, over twelve commits.**

- **The defect.** Twelve rows of `docs/tasks/PHASE-2.md`'s Verification Log render with their columns shifted left and padded with two empty cells. `e7a829c` added the `PHASE-2.4.3` row well-formed; `bb42f65` dropped its first two cells while prepending a new row above it.
- ✅ **All 12 recovered unambiguously** by suffix-matching every well-formed row in the file's history. ⭐ The recovered leaf ids come out strictly descending — a property the match never uses, so it is independent confirmation.
- ✅ **Verified by the RENDERER on a discriminating property:** rows whose first cell is a date **21 → 33**, rows ending in two empty cells **12 → 0**, total rows **76 both ways**. ⛔ That invariance is the point — a row-count check would have reported success before the repair.
- 🔴 **Two defects found in `BROKEN-TABLE`, one commit old.** mdbook says a table body continues across ANY non-blank line; the scanner ended one at the first pipe-less line — a FALSE POSITIVE flagging two adjacent tables separated by a blank (ordinary Markdown) and a FALSE NEGATIVE missing a later blank line. Both fixed, falsified in situ, **18 arms**.
- ⭐ **The terminators are not uniform:** a list, heading, blockquote and HTML block END a table with no blank line; plain prose and indented continuation do NOT. Established one construct at a time against the renderer.
- 🔴 **CORRECTION:** REPAIR-0238 published that one of the twelve "renders outside the table entirely". It does not — all twelve are rows. The probe's key matched an unrelated occurrence 1,889 lines away. Promoted as `a-key-too-loose-returns-the-wrong-instance`.
- Routed: **11 lines of `docs/tasks/PHASE-3.md` are a closing paragraph ABSORBED into its frontier table** — the mirror of `.11.19`. `.11.19.2`.

✅ **A BLANK LINE ENDS A MARKDOWN TABLE, AND ONE SAT INSIDE THE TREE'S OWN FRONTIER (`.11.19`, REPAIR-0238).**

🔴 **52 table rows across 3 tracked files rendered as paragraphs of literal pipe-text — including all 46 rows of the active tree's Current Frontier, row 1 among them.**

- **The defect.** A blank line TERMINATES a GFM table; every row after it comes back as one paragraph. ⛔ The SOURCE looks fine, which is why it survived every review that read the file rather than the page.
- ⚠️ **The narrow key found 1, the wide key found 51** — `a-census-is-as-wide-as-its-key` a third time in one session, and the widest miss yet.
- ⭐ **Asked of the RENDERER, which corrected two rules the probe had wrong:** a delimiter row whose cell count differs from its header is NOT a table, and four spaces of indent is a code block while three is still a table.
- 🔎 **`TABLE-ARITY-RATCHET` governs all three files and cannot see it** — it compares a row's cells against its header's, and an orphaned row has no header. `BOOK-LINKS`' founding shape a third time.
- ✅ **All 52 discharged, verified BY THE RENDERER:** `<tr><td>` 157→203, 53→56, 29→32 — **+52 exactly**, matching the census by a different route.
- ⭐ **`BROKEN-TABLE` SHIPS** — calibrated across 200 commits first: **1 blocked (0.5%)**, and that commit is the one that introduced the defect. Zero false positives. Three of its twelve arms are NEGATIVES, without which the rule degenerates.
- 🔴 **The leaf published a `0` that measured `12`.** Twelve verification-log rows in `docs/tasks/PHASE-2.md` LOST their first two cells, so the renderer emits them with their columns shifted left and padded by two empty cells, and both table gates are blind to the shape. The superseded claim is kept rather than edited into agreement. `.11.19.1`.

✅ **A RESTATED NUMBER NEEDS A PRODUCER, NOT A RULE (`.11.16`, REPAIR-0237).**

🔴 **The file that documents this project's gates calls itself the registry's human mirror, and four of the numbers it mirrored had gone stale with nothing deriving them.**

- **The defect.** The `SELF-TEST` row said *"all 17 pass"* and *"11 of the 28 check/census scripts"* — measured, **29** and **9 of 38**. The `FILE-TERMINATION` row said *"642 files scan"* — measured, **773**. ⛔ Each was stale in the instrument's OWN header too, so the registry mirrored a mirror and neither copy had a producer.
- ⚠️ **The leaf's own first pass was off by a factor of seventeen** — 6 numerals, because it keyed on **bold**. The real population is **103 across 19 rows** (`scripts/census_mirror_numbers.py`, 12 self-test arms).
- ⛔ **All three candidate gates DECLINED, each priced before it was proposed** (`.11.6`): **66%**, **50%**, and staged-diff-scoped **10 of the 14 commits in 200 that add a numeral (71%)** — against `.11.9`'s rejected 87%, `.11.15`'s 93% and `POSITIONAL-REF`'s accepted 9.5%. A per-numeral allowlist is refused by `VISIBILITY-POLICY`'s own *"thirty entries teaches bypass"* against 103.
- ⭐ **The measurement redirected the work:** the four stale numerals all name a POPULATION SIZE the instrument enumerates on every run and never printed. `--census` on two scripts fixed them; a number derived on every run cannot drift.
- ✅ **`REASON-CODE-DOC` discharged by DELETION, not correction** — it had drifted and become true again BY ACCIDENT, and correcting it would have taught the next reader it never drifted.
- **ADDRESSED:** `bare` numerals **51 → 37**. ⚠️ The remaining 37 are unreviewed by any instrument — read once by hand and judged frozen. That is the declined gate's cost, named rather than hidden.
- 🔴 **Routed — a blank line ENDS a Markdown table, and one sits inside the tree's own Current Frontier**: 45 rows render as literal pipe-text, **51 across 3 files** once the key widens from "after the delimiter row" to "anywhere in the body". ⛔ `TABLE-ARITY-RATCHET` governs these files and cannot see it. `.11.19`.

✅ **A POSITIONAL REFERENCE IS EXACT ONLY IF A READER CAN RESOLVE IT (`.11.17`, REPAIR-0236).**

🔴 **29 published source citations named two files each, and `DOCPATH` could not see any of them.**

- **The defect.** §4.1 grades a NAMED INSTANCE as exact with no tolerance band; a bare `profiles.rs` with a five-digit line names a 606-line source AND an 11,154-line suite. ⭐ `DOCPATH` wants repo-root-relative references and a bare basename SATISFIES it while naming nothing — `BOOK-LINKS`' founding shape.
- ✅ **Leg 3 closed first:** `scripts/census_positional_refs.py` is the tracked producer (9 self-test controls); the ad-hoc pipeline is retired. **499 occurrences, 321 distinct — pathed 250, unique 240, ambiguous 0, unresolved 9.**
- ⭐ **The leaf's 29 reconciled exactly once the UNIT was named** — 29 distinct, 47 occurrences — and **all 47 are discharged to 0**.
- ⭐ **`POSITIONAL-REF` ships calibrated across 200 commits BEFORE it was proposed** (`.11.6`): **48 of 415 added references ambiguous (11.6%), 19 of 200 commits blocked (9.5%)** — against the **87%** and **93%** that got `.11.9`'s and `.11.15`'s candidates rejected for teaching bypass.
- ⛔ **It gates AMBIGUITY, not DRIFT.** Five references had already drifted and were confirmed by grepping the SYMBOL, not the line. Which FILE is fixable and stays fixed; which LINE moves with every insertion above it.
- 🔴 **It flagged its own registry row, then its own leaf prose** — reworded both times rather than excluded, so it polices its own description.
- 🔴 **The falsification destroyed three of my own discharges:** `git checkout --` restored `LIVE_STATUS.md` to HEAD, taking three uncommitted repairs with it, silently. The knowledge note that recommended that restore is CORRECTED by the failure it caused.
- Routed: 9 references name no tracked file — 8 `init.rs` plus the registry row's own illustrative placeholder, which is the finding: gating that class means telling an EXAMPLE from a CITATION. `.11.17.1`.

✅ **THE SECTION SCANNER DID NOT KNOW WHAT MARKDOWN IS (`.11.18`, REPAIR-0235).**

🔴 **A doctrine gate called every `#` line a heading — including shell comments inside the fenced censuses it exists to require — and the leaf's own claim about which way that failed was wrong.**

- **The defect.** `GAP-CLAIM-CENSUS` wants a claim's census in its own heading section; a fenced `# PINNED at …` opened a pseudo-section, so the claim above it lost the census three lines below. That is how it BLOCKED `.11.14.3.2`.
- 🔴 **THE LEAF'S DIRECTION CLAIM IS REFUTED.** It asserted the dangerous direction was a SILENT mirror. A pseudo-section starts later than its real section and ends no later — a strict SUBSET — so it can only WITHHOLD a discharge. ⛔ **False positives only.** Measured: the fixture built for the asserted silent direction classifies identically before and after, and is pinned as the gate's DECLARED limit rather than as a fence bug.
- ⚠️ **The population is 6, not 7, and only 3 are governed** — the leaf's probe keyed on `^```` while the corpus holds 20 indented fences, and 3 of the 6 sit in a nested artifact the gate's filter never reads. Its two pinned line numbers had already moved (`.11.17`'s shape, inside one session).
- ⭐ **A second instance fixed with the first** (an indented ATX heading, one in the corpus): measured before it was taken, because widening a heading test only REMOVES discharges — **82 → 82, 0 newly blocked**.
- ⚠️ **An unbalanced fence now REFUSES rather than guessing**, since otherwise later claims silently inherit the last real section's discharges. Every governed file is balanced today, so it ships inert.
- **Population unchanged: 94 claims, 0 unbacked, before and after** — a prevention, not a correction.
- **FALSIFIED IN SITU:** pre-fix classifier back in the current script → `--self-test` 12/17, failing arms 10, 12, 13, 14, 15 by name. 🔴 And it caught one of MY arms discriminating on indentation rather than on the fence — the session's third control passing for an unrelated reason.

✅ **A CREDENTIAL BINDING IS TENANT-BOUND (`.11.14.3.10`, REPAIR-0234).**

🔴 **A credential SELECTOR lived on a content-addressed row, and a second tenant drove an authenticated acquisition with the first tenant's credential.**

- **The defect.** `resource_references` is keyed `UNIQUE (original_locator, expected_digest)` — identity by CONTENT. `credential_binding_ref` is not content: the R5 arm hands it to `broker.resolve`. A second tenant replaying the pair inherited a binding it never named, and its resolve attached the OWNER's credential. ⛔ ROADMAP §16.3 invariant 5.
- **REPRODUCED RED FIRST** (gate open, one registered binding, two tenants, one locator): the stranger answered `resolvers: ["r5-credential-broker"]` → `destination_refused` — the **loopback pre-flight**, reached only once a credential RESOLVED.
- ⭐ **The discriminator is the RESOLVER, not the error kind.** Both paths end at the same loopback refusal; only `resolvers` says which ran. ⛔ My first draft asserted `credential_unavailable` and would have FAILED against the repaired product.
- ⭐ **DECIDED: the selector moves to `reference_registrations`; `migrations/0069` DROPS the column.** The shared row keeps CONTENT, the tenant-bound row keeps the DECISION — the **fourth** instance of one shape in this family, now promoted as `a-shared-row-may-not-hold-a-tenant-decision`.
- ⭐ **Wider than a denial, and measured rather than designed:** the binding is a RANKING input, so a tenant that named none is **not routed to the credential pack at all**.
- ⭐ **A limit closes in the OTHER direction too, unstated until now:** the pair key meant two tenants citing one URL could not hold two DIFFERENT bindings. Both may now.
- ⭐ **Structural, not conventional:** the unbound read has no column to read a selector from. Only `get_for_tenant` supplies one, and only the asking tenant's own.
- ⚠️ **The backfill is EXACT** (`registered_by` and `submitted_by` are literally the same value); every other tenant gets NULL — §16.4's fail-closed rule for secret access.
- 🔴 **A defect of my own, caught by the neighbours:** the control left the opt-in gate open. ⛔ Closing it at the END of the test was NOT the fix — a PANICKING test never reaches its own end, which is how it was found. Normalised in `pool()`.
- **FALSIFIED** (stash verified landed; RED at exactly the defect arm; restored). **No regression:** `profiles` 55/55, `migration_upgrade` 4/4, `backup_restore` 1/1, `--lib` 113/113, clippy clean.
- ⚠️ **NOT closed, and owned:** the broker's namespace is GLOBAL — naming a binding an operator made for someone else is still enough. `GrantAction` is thread-scoped (10 variants, 0 about a resource), so it is a §16.4 authorization surface rather than a wiring change. `.11.14.3.10.1`; measured latent (3 `.register(` sites, all tests; 0 in `src/bin`).

✅ **THE PLAN CHECKER NOW HAS A COMMIT-TIME TRIGGER (`.11.14.1.2`, REPAIR-0233).**

⭐ **The failure class that bit three times today is now mechanical — and building the gate produced a fourth instance of it, caught.**

- **The rule is not new and the checker is not wrong.** The shared runtime checker already refuses a cleanup plan whose declared tables omit a real foreign-key child. ⛔ **What was missing is a TRIGGER**: it only runs when a suite runs, so a plan nobody executes is never checked. Six suites were refused for ~20 commits, and the two omissions came from `migrations/0062` and `migrations/0067` — the second in the session that wrote the sweep-key rule down.
- ⭐ **`.11.6` is satisfied rather than waived.** It forbids proposing a rule before its population is measured; `.11.14.1.1` measured it in the PREVIOUS commit — **16 parent tables, 45 inline foreign keys, 29 declared plans, 6 refused** — and the population is now **0 refused of 29**. The gate ships GREEN, and its value is preventing the class rather than finding a present defect. Said plainly so a green run is not read as evidence of one.
- **It needs no database**, which is what makes it affordable in a pre-commit hook: `ALTER TABLE … ADD CONSTRAINT … FOREIGN KEY` appears **0** times, so every key is inline in a `CREATE TABLE` and the graph is readable from the migrations.
- ⚠️ **IT REFUSES RATHER THAN UNDER-REPORTS.** An unmodelled key form is an ERROR naming that reason; an unreadable `delete_tables(` call shape is an ERROR naming the file. Only two files are excluded — the helper's own definition and its 15 negative-control tests — **by path, with the reason**.
- 🔴 **That defence was earned, not anticipated.** The first draft parsed **29** plans from **45** call sites, and I nearly published the difference as coverage. The 16 skipped were the right exclusions, reached **by accident** because a regex happened not to match them. A control now asserts the exclusion is by path rather than by luck.
- **FALSIFIED at both layers.** Against the instrument: the pre-repair `quota` plan restored → RED naming both omissions → byte-identical restore → green. Against the REGISTRATION: two lines removed from `cards.rs` → `make gate` answers `1 doctrine breach(es) — commit blocked`, naming the file and both tables.
- 🔴 **And the registration's first falsification passed for the WRONG reason.** The scripted edit's indentation did not match, so **nothing was injected** and the gate stayed green — a green run proving nothing, exactly like the `pg_stat` instrument `.11.14.3.7` discarded, one layer out. ⭐ **An injection must be shown to land**: `git diff --stat` before reading the result. The same attempt also wrote its backup to a temporary directory this environment refuses, so the restore never ran; `git checkout --` is the restore that cannot fail that way. Promoted as `docs/knowledge/an-injection-must-be-shown-to-land.md`.
- **Self-test: 8 controls**, two-sided, and the SELF-TEST harness discovers it (30 instruments now carry one).

🔴 **SIX SUITES COULD NOT START SINCE REPAIR-0213, AND THE SECOND MISSING CHILD WAS MINE (`.11.14.1.1`, REPAIR-0232).**

🔴 **Six suites could not start, since REPAIR-0213 — and the second missing dependency was mine, added in the session that promoted the rule against exactly this.**

- **Found because a broad verification run stopped dead** at a suite this session had not run before. Attributed by command rather than by reading: `evidence_citations` landed in `b4d6201` (REPAIR-0213); the `quota` plan was last touched in the **earlier** `40155d1` (REPAIR-0154); and with this session's working changes stashed, the suite fails identically at committed HEAD.
- **The mechanism.** The shared checker validates a plan BEFORE the first deletion and requires dependents to precede parents, cascading ones included. `evidence_citations` declares `tenant_id … REFERENCES tenants ON DELETE CASCADE`, and six plans delete `tenants` without naming it: `cards`, `classification`, `federation`, `mcp_listen`, `quarantine`, `quota`.
- 🔴 **AND I DID THE SAME THING, TODAY.** `migrations/0067` added `reference_registrations` with foreign keys to **both** `resource_references` and `tenants`. I swept the 21 plans naming the first — the table I was thinking about — and not the plans naming the second. So all six were missing **two** children and the second was mine, added in the very session in which I wrote `docs/knowledge/a-census-is-as-wide-as-its-key.md`. ⛔ **Knowing the rule is not applying it: the sweep must be keyed on the NEW TABLE'S OWN constraints, every one of them, computed rather than recalled.**
- ⚠️ **"SIX" is a correction to this leaf's own first number, made inside one session.** It opened saying **TEN**, from a loose census — `grep -l '"tenants"'` matches a row-count tuple in `authority_transaction.rs:346`, a bare list entry in `migration_upgrade.rs:118`, and files with no cleanup plan at all. The tightened instrument parses each `delete_tables(…, &[…])` array: **16 tables carry a direct FK to `tenants`, 27 declared plans delete it, 6 were refused.** ⭐ Catching the wrong number inside the session is the point of the audit; publishing it in a committed leaf first is what it cost.
- ⭐ **The rule, derived before the edit.** The runtime checker already computes the right thing. What is missing is not a rule but a **trigger** — it only runs when a suite runs, so a plan nobody executes is never checked. The census above is that trigger made cheap: it needs no database, because the constraint it depends on is declared in the migrations.
- ⛔ **Whether it becomes a GATE is decided: not yet, and `.11.6` is the reason.** Its population was measured for the first time in this commit, and a rule proposed in the same commit that first measured its population is a rule proposed before the population settles.
- ⚠️ **What this means for a published claim:** the full-checkpoint record's *"40 of 40 database suites, 291 tests, no failures"* predates `migrations/0062`. Six of those suites could not run today. `.11.4.7` re-derives the gate records and now has one more reason to distrust a count taken before the source review.
- **Verification:** all six RUN and pass — **14 tests / 0 failed**, rc=0; the census re-derives to **0 refused of 27**.

✅ **THE ACQUISITION PATH TAKES THE QUOTA (`.11.14.3.14`, REPAIR-0231) — the director's delegated call.**

⭐ **The director delegated this call on 2026-09-17. Taken in full, and the finding is why the two scopes had sat unwired since `0047`.**

- **The measurement.** `resolver` and `destination` ship in `SCOPE_KINDS`, in `migrations/0047`'s CHECK constraint, and in **nothing else**. Meanwhile `POST /v1/resources/{id}/resolve` carries **no quota, no storm control and no breaker** and performs a real network acquisition per call — which is exactly what §16.11 names as "resolver abuse" and "scraping". ⚠️ A raw `grep -c 'pub const SCOPE_'` returns **5** because it matches the `SCOPE_KINDS` array itself; the array's own type is `[&str; 4]`.
- **(1) The gap §16.11 names is the ACQUISITION path, not the thread verbs.** ⛔ The other eleven thread operations do not gain quotas: `.11.14.3.7` measured that the citation amplification is inside ONE request, so a per-hour ceiling bounds arrivals and not per-request work, and bounding the rest would be a rule without a population (`.11.6`).
- 🔴 **(2) WHY THEY SAT UNWIRED — and the deferral never said it.** `check_in_tx` is fail-closed, and `0047` is right that the bound must exist before the surface is usable. That works for the two WIRED scopes because of a property these two lack: **their members are created by a path the server controls, so the bound is seeded at creation** — a tenant by the enroll transaction, a principal by enrolment and card import. The RESOLVER space grows at runtime through `POST /v1/resolvers`; the DESTINATION space is the open internet. ⛔ **A fail-closed bound over a space you cannot enumerate is not a bound, it is an outage.**
- ⭐ **(3) DECIDED: the fail-closed CONTRACT is kept and the MECHANISM changes.** A per-tenant DEFAULT row at the wildcard scope id `*` — seeded by `insert_defaults_in_tx`, backfilled by `migrations/0068` — with a specific row overriding it, and **the absence of BOTH still the typed refusal**. `check_open_scope_in_tx` resolves most-specific-wins with one existence probe and reaches `check_in_tx` unchanged, so the window arithmetic and the recorded `use`/`denial` are the ones already qualified. The property fail-closed exists for — *there is always a bound* — is untouched; the default is now a **row** rather than an **absence**.
- **(4) It counts ATTEMPTS**, after the ranking and before the pack executes, on the ranked resolver and the locator's host. An attempt is what a caller repeats and what reaches the network — the control shows the bound consumed by an attempt the destination policy then refuses. ⚠️ A specific row starts its OWN count, because `quota_events` is keyed by `quota_id`: a narrowed bound is a new bound, not a continuation.
- ⛔ **The ceilings are dev-profile defaults, labelled at the constant.** No acquisition volume has ever been measured, and `.11.6` forbids proposing a threshold before its population. **What was decided is the shape of the bound, not its number**, and no gate may read the ceiling as evidence that abuse is bounded at any level.
- **Verified:** the enrol transaction seeds **one default row per open scope, not one per member**; one resolution records **one use per scope**; a host-specific ceiling **overrides the default** and refuses `429` with a **recorded denial**; removing **both** rows is still the typed fail-closed `503`.
- ⚠️ **The first broad run was BLOCKED by a pre-existing defect and that is recorded rather than worked around**: the `quota` suite — and **nine** others — cannot start, because `migrations/0062` added `evidence_citations` with a foreign key to `tenants` and did not sweep the fixture plans that name `tenants`. Attributed by command (`REPAIR-0213` added the table; the plan was last touched in the earlier `REPAIR-0154`) and reproduced at committed HEAD with this session's changes stashed. `.11.14.1.1` owns it.

🔴 **THE BLOCKER REGISTER ASKED A QUESTION ABOUT ITS READER (`.13.5`, DOC-0043).**

🔴 **The blocker register asked a question about its reader, and the director had to point it out.**

- **The defect, named rather than apologised for.** `.13` gave the register an `Ack?` column meaning *"has the director engaged with THIS row"*, and a rule that every stopping-point reply re-surface every row whose value was `no`. ⛔ That is **a field whose value is a fact about the READER, in the maintainer's own register** — the register cannot observe it and the maintainer cannot set it, so the only mechanism available was to repeat the row until the reader reacted.
- **Measured, not felt:** `git log -S"Ack? = yes"` over both copies returns **nothing**. From 2026-09-15 to 2026-09-17 the column was `no` on every open row and never once anything else. ⭐ A column whose value has never changed in its lifetime carries no information — the same shape as a control never seen RED, which this tree refuses everywhere else.
- ⛔ **And it inverted a delegation.** The director had delegated blocker disposition; the column made his attention the precondition for a row to stop being repeated at him. An instrument for *surfacing* had become one for *nagging*, which trains a reader to skip the section the register exists to be read.
- ⭐ **The replacement is a different question, not a renamed column: `Owed here?`** — *is there anything left that THIS REPOSITORY can do about this row?* Set by the maintainer from the owning leaf, beside the named next action. A `yes` row is **work** and belongs in the frontier; a `no` row is surfaced **once per session** with its external party and trigger.
- 🔎 **It paid for itself immediately, and that is the finding.** Under `Ack?`, B1–B3 were reported to the director as *"OUTSIDE"* with nothing owed. Under `Owed here?` all three are **yes** — they share one in-repo prerequisite, `SIGNOFF-REPAIR.14`'s frozen exposure candidate, which `docs/book/src/blockers.md` **had already named**. ⛔ The old column was hiding the maintainer's own work from the maintainer, and the same reply that called those rows external also linked the page that said they were not.
- **The corrected values:** B1 **yes**, B2 **yes**, B3 **yes**, B4 **no** (the only row with nothing owed here), C1 **no** and relabelled *not a blocker — a limit on what may be CLAIMED*, C2 **yes** (four gate records remain).
- ⚠️ **No blocker's substance moves.** B1, B2 and B4 still need outside parties; `.14` remains under its standing prohibition against turning the exposure profile on; G6/G7 remains NOT MET. What changes is which rows the maintainer treats as work.

🔴 **THE SNAPSHOT CENSUS WAS EIGHT, NOT SEVEN — A PUBLISHED NUMBER WAS FALSE AND THE DEFECT IT HID WAS LIVE (`.11.14.3.15`, REPAIR-0230).**

🔴 **A published census was FALSE, and the defect it hid was live. It was found because the maintainer asked whether the findings hold.**

- **What was published.** `.11.14.3.8` (REPAIR-0222) published *"seven surfaces name a `snapshot_id`, six citation-bound, exactly one not. That is the enumeration the finding is made of, not an impression"* — in its leaf, its decision record, `deployment.md`, `CHANGELOG.md` and `LIVE_STATUS.md`.
- 🔴 **What is true.** Re-derived on the **identifier** rather than on the route table and the `cited_snapshot` call sites: **eight surfaces, six bound, TWO not.** The eighth is `POST /v1/derivations`, which names its parent as `parent_snapshot_id` in the request **BODY** — invisible to a census built from `Path(snapshot_id)` extractors.
- ⛔ **Graded on all three axes** (`CLAIM_VERIFICATION.md` §4.1) and none survives: the **PROSE** is false rather than imprecise, the **NUMBER** moved 7→8 and 1→2, and the **NAMED INSTANCE** — an enumerated table — omitted a row. ⚠️ Its acceptance test — *"when asked whether you stand by it, the answer is yes, immediately, with no keyboard"* — was **failed**.
- ⭐ **This is the FIRST instance of the blind spot `docs/knowledge/a-census-is-as-wide-as-its-key.md` describes, and that note was written from the SECOND and never applied backwards.** A rule earned from one instance is worth almost nothing until it is run over the instances that came before it. The note now says so, and carries all three.
- 🔴 **The defect the false number hid.** `submit_derivation` admitted on enrolment alone; `derivations::submit` checked `SELECT EXISTS (SELECT 1 FROM evidence_snapshots WHERE snapshot_id = $1)` with **no tenant predicate**, and took no tenant at all. Falsified against the exact unrepaired store: a second tenant's derivation against a snapshot it had never cited **SUCCEEDED** — `200 {"derivation_id":"drv_01a0b0c6c1367e22abcd461ab1bf3001"}` — while an absent id was refused.
- ⚠️ **Width, before it is inflated:** an ORACLE requiring the caller to hold an unguessable `snp_` id, and the written half was bounded by the read — the foreign tenant could not read its own derivation back. ⛔ That bounds the harm and does not excuse it: an invisible write into another tenant's evidence graph is worse evidence than a visible one.
- **FIX** — the citation binding in the store, as the two before it. ⛔ No new error variant: a parent the caller did not cite answers the `ParentMissing` an absent one gets. ⚠️ The derivation GRAPH stays shared and the control asserts it — once both tenants cite the parent, **both read both children**, which is `.11.14.2`'s disposition. This binds the write without narrowing the read.
- ⚠️ **A falsification note worth carrying:** reverting the store alone left the call sites passing an argument that no longer existed, so the run failed to COMPILE rather than going red — a falsification that proves nothing. Both files must be reverted together.
- ⭐ **Every other census this session published was re-derived under the same audit and HOLDS** — the single `WHERE reference_id` site, "nothing reads `evidence_snapshots.original_locator` for a decision", 2 `check_in_tx` callers / 4 scope kinds / 12 `OP_` constants, `axum-core-0.5.6`'s `DEFAULT_LIMIT = 2_097_152`, "no production path registers a broker binding" (3 sites, all `#[cfg(test)]`), `.13.4`'s 13 hits with none bare, and 21 of 21 fixture plans. ⚠️ One instrument artefact: `grep -c 'pub const SCOPE_'` returns **5** because it matches the `SCOPE_KINDS` array itself; the array's type is `[&str; 4]`.
- **Verification:** eight live suites, **121 tests / 0 failed**, rc=0 (`profiles` 53, `command_api` 39, `node_work` 8, `mcp` 6, `mcp_write` 5, `cli_end_to_end` 5, `evaluation` 3, `routing` 2).

✅ **WHAT A CITATION LIST COSTS (`.11.14.3.7`, REPAIR-0229) — AND AN INSTRUMENT DISCARDED FOR REPORTING THE SAME VALUE EITHER WAY.**

⭐ **Three answers, and the most transferable one is about an instrument I threw away.**

- **The amplification, measured.** `.11.14.3.2` made each citation register a §12.1 reference **inside the thread's aggregate transaction**, which holds `FOR UPDATE` on the thread's row. **64 distinct citations register 64 references**, asserted by a live control. ⚠️ The leaf's acceptance asked for a before→after on transaction DURATION; the row count is substituted deliberately and the substitution is stated — duration is the symptom, the count is the thing, and a wall-clock assertion over ~60 round trips measures the machine.
- ⭐ **(1) A windowed quota is the WRONG instrument for this defect**, measured rather than argued: the cost is inside ONE request, so a per-hour ceiling on `thread.contribute` bounds how many requests arrive and says nothing about how long any one of them holds the row. The dimension is request SIZE, not call rate.
- **(2) What bounds a request today**, read from the vendored source rather than recalled: `axum-core-0.5.6/src/ext_traits/request.rs:319`'s `DEFAULT_LIMIT = 2_097_152` — **2 MiB, inherited from a dependency**, since `grep -rn 'DefaultBodyLimit'` returns 0. ⛔ Declaring it explicitly was NOT taken here: one global limit also governs `POST /v1/snapshots`, which legitimately carries base64 evidence bytes, so it is a PER-ROUTE decision and taking it inside this leaf would be the unmeasured policy the leaf exists to avoid.
- **(3) The de-duplication ships**, DERIVED rather than chosen — the `(uri, digest)` pair IS the reference's identity, so a repeat can only fetch back what the first citation wrote. ⛔ It de-duplicates the **work**, never the **record**, and that is structural: the decision is a pure function returning one answer per citation, so the event still carries all 48 repeats in order with their own notes. ⚠️ It is **not a bound** — N distinct citations cost the same as before.
- 🔴 **HOW IT IS VERIFIED, and what that cost.** The de-duplication is **not observable through any product surface**: the row count is **1 either way**, because the pair replay already returns the existing row. ⛔ A `pg_stat_user_tables` scan-counter instrument was written for it and **DISCARDED** — run against the unrepaired handler it reported **the same value**, and the control passed both times. **A control that passes identically either way measures nothing**, and shipping it would have converted an unverified change into one that looks verified. The decision was extracted into a pure function and falsified directly instead: a locator-keyed implementation gives `[0, 0, 0, 0]` against the correct `[0, 1, 0, 3]`. Promoted as `docs/knowledge/a-change-no-surface-can-see-needs-a-seam.md`.
- ⚠️ **A correction to the leaf's own census:** it said *"the one `quota::check_in_tx` call is `OP_INVITE`'s"*. Re-derived, there are **2** — `OP_INVITE` on the tenant scope and the MCP write gate on the principal scope.
- ⚠️ **Two control defects of my own**, both caught by running it and corrected rather than relaxed: an assertion keyed on `len() == REPEATS` matched the DISTINCT contribution's event because both arms used 64, and the events payload is `{"events": […]}` rather than a bare array. Neither was a product defect.
- **Verification:** `profiles` **52 passed / 0 failed**, plus the pure function's unit test falsified by injection.

🔴 **A REFERENCE'S `scheme` IS A CAPABILITY SELECTOR — AND I SHIPPED A WRONG REPAIR INTO A RED/GREEN CYCLE BEFORE THE PRODUCT'S OWN CONTROLS CAUGHT IT (`.11.14.3.5`, REPAIR-0228).**

🔴 **Three mechanisms in one goal line. ONE was a real defect, one was a design I had misread, one was already decided — and the misreading was caught by the product's own controls after I had taken it through a RED/GREEN cycle.**

- 🔴 **(1) `scheme` — REFUTED, not repaired, and the route there is the main deliverable.** The premise came from a test helper's comment (*"caller-supplied and NOT validated against the locator"*) beside the facts that §12.2 ranks resolvers on the field and that `resources::scheme_of` exists with one caller. ⛔ I wrote the check, reproduced RED (`ftp://…` declared `https` registered, `200`), went GREEN on my own control — and **two of this suite's own controls refused it**: `the_r1_resolver_resolves_git_…` and `the_gated_packs_resolve_only_while_the_gate_is_open`.
- ⭐ **Measured instead of inferred.** The consumer is one predicate — `resolvers::resolve`'s `WHERE schemes @> $1::jsonb` — and **two SHIPPED packs pair a non-URI scheme with an `https://*` locator pattern**: `r1-git-fetcher` advertises `["git"]`, the R3 browser pack advertises `["web+render"]`. A Git repository and a rendered page are both reached over HTTPS. **The field is how a caller asks for a CAPABILITY.**
- ⭐ **(1) DECIDED: no check; the contract is STATED** — at the type, in the book, and in a control arm that pins the refutation. The helper comment that produced the wrong premise is corrected **at its source**, because a sentence that produced one wrong repair will produce another. ⚠️ Nothing else validates it either, deliberately (§3.7: accepting a reference is not a promise the core can resolve it), and a second arm pins that. ⛔ A shape check was considered and rejected under `.11.6` — no census of malformed scheme tokens exists, and inventing a rule before its population is how this section went wrong the first time.
- 🔎 **What caught it, because that is the transferable part:** the two refusing controls are not about references — they exercise the PACKS, and **the packs are the consumer**. A repair is falsified by the code that USES the thing, which is the same place its contract lives. Running the affected suites BROADLY rather than only this leaf's own control is what surfaced it; the narrow run was green. Promoted as `docs/knowledge/a-fields-name-is-not-its-contract.md`.
- 🔴 **(2) The dead column defaults — a real defect, repaired.** `migrations/0023` declares `NOT NULL DEFAULT 'network'` / `'low'`; `#[serde(default)]` is `String::default()` — the **empty string** — and the store binds it explicitly, so the column default never applied and `''` is not a risk class. The citation path wrote the declared values, so **the two writers produced different rows for the same omission**. The serde default is now the schema's, and the control asserts it by COMPARING the two writers' rows.
- ⚠️ `low` is the permissive direction and it is **adopted rather than chosen** — it is what the migration already recorded. ⛔ Nothing reads either column for a decision; §12.2's risk filter owns whether `low` may be a default at all, and that is stated at the field.
- ⭐ **(3) The fragment stays in the locator** — three spellings, three references, and the arm pins the behaviour rather than a repair. Splitting IS canonicalization, which §12.1 defers as scheme-specific and which would erase a distinction that section protects. ⭐ The same boundary `.11.14.3.13` drew from the other side, so the two leaves agree and the line is stated rather than felt.
- ⚠️ **The leaf therefore closes with ONE repair and TWO stated contracts**, which is the honest count rather than "three findings, three fixes".

✅ **A SNAPSHOT NAMES THE LOCATOR ITS REFERENCE NAMES (`.11.14.3.13`, REPAIR-0227).**

⭐ **The census decided the disposition, and the new check found a defect in this session's own test data on its first run.**

- **The census, first because the leaf asked for it.** `grep -rn "original_locator" crates/reasonbraid-server/src/*.rs`: every hit outside `resources.rs` is either `reference.original_locator` — the REFERENCE's field, which the resolvers read to fetch — or the snapshot column being written, mapped and listed. ⭐ **Nothing reads `evidence_snapshots.original_locator` to make a decision**: no resolver, no gate, no routing rule. So this is an evidence-integrity defect rather than a routing one, and a refusal is the whole repair — there is no downstream behaviour to correct, only a record that could be false.
- 🔴 **RED, falsified against the exact unrepaired store.** A submission naming `https://example.org/some-other-document`, filed against a reference registered as `https://example.org/the-registered-report`, was **stored**: `200 {"replay":false,"snapshot_id":"snp_01a0af0ef6fe7f028d0e2014a444655b"}`; `49 passed; 1 failed`.
- ⭐ **DECIDED: byte equality with the reference's locator; `final_locator` untouched** (`docs/decisions/2026-09-17_a-snapshot-names-the-locator-its-reference-names.md`, three alternatives rejected).
- ⛔ **The leaf's warning is ANSWERED rather than obeyed.** *"An equality check is a canonicalization decision wearing a different name"* is true of a **normalising** check — one that lower-cases a host or strips a trailing slash to decide two spellings are the same — and false of a **strict** one, which normalises nothing and decides nothing. §12.1's immutable locator is exactly what makes the reference's stored string the identity to compare against.
- ⭐ **`final_locator` stays free, and the control asserts it.** A redirect legitimately ends somewhere else, which is why §12.6 records both. A repair that compared them too would have been WRONG, not merely stricter.
- **The refusal names BOTH values.** It runs after `.11.14.3.11`'s registration predicate, so the caller has proved it registered the reference and may read that locator — quoting it discloses nothing it does not hold, and the diagnosis is worth more than the symmetry.
- ⚠️ **NO REGRESSION, and it corrected a control of my own.** `.11.14.3.6`'s pin control passed the pinned report's locator for BOTH of its references — simply wrong about which document the unpinned snapshot was of, and nothing checked it. It now passes each reference's own locator, with every original assertion intact.

✅ **A SNAPSHOT IS FILED AGAINST A REFERENCE ITS OWN TENANT REGISTERED (`.11.14.3.11`, REPAIR-0226).**

🔴 **Not merely an oracle — a WRITE. And it was missed by my own census one commit earlier.**

- **REPRODUCE, RED first.** A second tenant's submission against a reference it had never registered, beside an absent id:

```text
foreign: 200 {"replay":false,"snapshot_id":"snp_01a0aee3660d7d32906d5524d6423f98"}
absent:  400 {"code":"invalid_command","message":"the reference does not exist"}
```

  The foreign submission **succeeded** — a snapshot was attached to another tenant's reference. `48 passed; 1 failed`.
- ⛔ **And nothing could see it.** `grep -rn "WHERE reference_id" crates/reasonbraid-server/src/*.rs` returns **1** — the replay lookup inside `submit` itself. No route lists a reference's snapshots, so the attachment is invisible to the reference's own registrants. That bounds the harm and does not excuse it: an invisible write is worse evidence than a visible one.
- ⭐ **How it was missed.** `.11.14.3.4`'s census, one commit earlier, was `grep -n '"/v1/resources' api.rs` → three routes, two unbound, both bound. `POST /v1/snapshots` names a `reference_id` in its **BODY**, so a route-prefix enumeration cannot see it. The census was not wrong; it was **silent**, which is the more dangerous failure. Rule promoted: **enumerate on the identifier, not on the address shape** (`docs/knowledge/a-census-is-as-wide-as-its-key.md`) — second instance of the shape, after `.3.5.3`.
- ⭐ **DECIDED: the write takes the read's registration binding** (`docs/decisions/2026-09-17_the-snapshot-write-is-bound-to-the-reference-it-names.md`, three alternatives rejected). A predicate in the SAME statement, and ⛔ **no new error variant**: a foreign reference answers the `ReferenceMissing` an absent id gets, so there is nothing for the distinction to leak through.
- ⭐ **The leaf's warning is ANSWERED rather than obeyed.** It said the asymmetry that made the snapshot replay safe — a submission carries the BYTES — cuts the other way here. It does, and it is still not a reason to leave the write open: a submission also carries `original_locator`, so a caller that can make one can register the pair, receive the SAME reference id and file. That is a control arm, not an assertion.
- **ADDRESSED** — foreign and absent are refused in the same words with **0** rows attached; the registering tenant files normally; the second tenant registers the same locator, replays to the same reference, files, and the submission replays to **one** shared snapshot row carrying **2** citations.
- ⚠️ **Routed with its measurement — `.11.14.3.13`:** a snapshot still records an `original_locator` its reference need not carry. ⛔ Not an obvious equality check — §12.1 keeps canonicalization separate and scheme-specific, so an equality check is a canonicalization decision wearing a different name.
- **Verification:** six live suites, **104 tests / 0 failed**, rc=0, cluster removed: `profiles` 49, `command_api` 39, `evaluation` 3, `routing` 2, `mcp_write` 5, `mcp` 6.

✅ **A RESOLUTION SAYS WHEN ITS EVIDENCE WAS NOT PERSISTED (`.11.14.3.12`, REPAIR-0225).**

⭐ **The leaf named three candidate dispositions. Reading the neighbour's call site showed the decision had already been taken.**

- **The finding, and it is not the one the leaf was opened on.** `SIGNOFF-REPAIR.7.4.2` decided this for the R2 arm and wrote the reason AT THE CALL SITE: *"A failed snapshot is NOT a successful acquisition. This path used to discard the error and still set `outcome.acquisition`, so a caller was told the document had been acquired while no evidence row and no derivation existed."* It sets `acquisition_error { kind: "evidence_unstored" }` and returns without an `acquisition`. ⛔ So this was a **shipped decision with two unconverted call sites**, not a design choice — a different and much cheaper thing to find, and found by measuring the neighbour rather than reasoning from the leaf's framing.
- 🔴 **RED, falsified against the exact unconverted arms** (`git stash push -- api.rs`, control run, file restored). A reference pinned to bytes the origin does not serve, resolved through R0:

```json
{"acquisition":{"byte_count":412,"digest":"sha256:561688a4…",
  "chain":["http://127.0.0.1:56360/feed.xml","http://127.0.0.1:56360/feed.xml"],
  "final_url":"http://127.0.0.1:56360/feed.xml","sniffed":"text"},
 "resolvers":["r0-https-fetcher"],"unresolvable_now":false}
```

  A complete receipt — digest, byte count, redirect chain — with **0** snapshots stored and **no** `acquisition_error` at all. `47 passed; 1 failed`.
- **FIX** — the R0 and R5 arms take the R2 shape verbatim. `grep -n "let _ = crate::snapshots::submit" crates/reasonbraid-server/src/*.rs` now returns **nothing**. ⛔ No new field and no wire change: a client already handling `evidence_unstored` from R2 handles it from R0 and R5 unchanged.
- **ADDRESSED** — the pinned reference's resolution answers `evidence_unstored` with NO receipt and 0 snapshots; the **unpinned** reference through the SAME deployment acquires, persists **1** snapshot and names no error. That second arm is the bound a repair reporting `evidence_unstored` for everything would fail.
- ⚠️ The acquisition still happened and the server logs it. What the caller no longer receives is a receipt implying evidence that is not there.
- **Verification:** `RB_DEMO=0 bash scripts/run_pg_tests.sh profiles` → **48 passed / 0 failed**.

✅ **A REFERENCE'S `expected_digest` NAMES ITS BYTES (`.11.14.3.6`, REPAIR-0224).**

🔴 **A §12.1 field a caller supplied to say "these are the bytes I expect" constrained nothing, and enforcing it is what makes the pair key mean something.**

- **The census, re-derived on the working tree rather than quoted:** `git grep -n expected_digest -- crates/reasonbraid-server/src` → **20** hits (13 at the pinned commit; the growth is the registration path WRITING the column). `snapshots::submit` asked `SELECT EXISTS (SELECT 1 FROM resource_references WHERE resource_id = $1)`. **Zero sites read a STORED pin.**
- 🔴 **RED, falsified against the exact unrepaired store** — the production file reverted with `git stash push`, the control run, the file restored. A reference pinned to `sha256(the audited figure is 41.2 per cent)` accepted a snapshot of `the audited figure is 62.8 per cent`: `200 {"replay":false,"snapshot_id":"snp_01a0aec59be97d93af711df98b146e90"}`; `46 passed; 1 failed`.
- ⭐ **DECIDED: a pinned reference accepts only the bytes it names; an unpinned one is unchanged** (`docs/decisions/2026-09-17_a-pin-names-its-bytes.md`, three alternatives rejected). ⭐ **Enforcement is what makes `.11.14.3.2`'s pair key MEAN something**: that leaf made `(locator, digest)` the reference's identity so §12.6's changed page would be a SECOND reference rather than an erased distinction — and with no checkpoint both rows accepted any bytes, so the distinction the key was created to preserve was preserved nowhere.
- ⚠️ **The leaf's own worry resolves rather than binds.** It warned that *"a pin enforced at acquisition would forbid exactly that [plural]"*. The plural belongs to the UNPINNED reference: `evidence_snapshots` replays on `(reference_id, raw_digest)`, and the control asserts one unpinned reference holding **2** versions.
- ⛔ **The refusal carries NEITHER digest.** The caller already holds the actual one — it hashed the bytes it sent — and the pinned one belongs to a reference this route does not check the caller may read, so quoting it would be an oracle over a `res_…` id. The message carries the next step instead: *register the locator at the new digest and acquire against that.* In the STORE rather than at the resolver, for `.11.14.3.8`'s reason — four call sites reach it.
- ⛔ **CONSEQUENCE, stated rather than discovered.** `api.rs`'s R0 and R5 arms call `let _ = crate::snapshots::submit(…)`, so a pinned reference whose page drifted now returns an acquisition receipt and no snapshot, **silently**. The discard predates the pin (its own comment says so); what changed is that it now has a likely, caller-meaningful cause. Censused to **two** sites — the R2 arm captures its result. `.11.14.3.12` owns it.
- 🔎 **And a gap in MY OWN census one commit earlier — `.11.14.3.11`.** `.11.14.3.4` enumerated the routes under `/v1/resources` and bound the two that read a reference; `POST /v1/snapshots` names a `reference_id` in its **BODY**, so a route-prefix census cannot see it. ⭐ That is `.3.5.3`'s shape exactly — a census correct about its own scope and silent about what fell outside it — committed by the session that had just written the rule down.
- **Verification:** `RB_DEMO=0 bash scripts/run_pg_tests.sh profiles` → **47 passed / 0 failed**; the RED baseline 46/1.

✅ **THE §12.1 REFERENCE DETAIL READ IS BOUND TO ITS REGISTRANTS (`.11.14.3.4`, REPAIR-0223).**

🔴 **The leaf expected a locator confirmation. RED returned the whole §12.1 row.**

- **REPRODUCE, RED first.** A second tenant holding a `res_…` id it had never registered, at `GET /v1/resources/{id}`:

```json
{"reference":{"original_locator":"https://internal.example.org/q3-reserve-review",
  "expected_digest":"sha256:aaaa…","credential_binding_ref":"the-owner-binding",
  "purpose":"the reserve review the owner is running","visibility_scope":"tenant",
  "risk_class":"high"}, "submitted_by":"agt_c06e5644-…", …}
```

  `45 passed; 1 failed`. ⭐ **Read `visibility_scope` in that payload**: the row declares `"tenant"` and the read ignored it — §12.1 lists the field and nothing consumes it. `POST /v1/resources/{id}/resolve` shares the same lookup and the same gate.
- **The census, both directions:** 3 routes name a resource (register/replay, detail, resolve), 2 unbound callers of the store's read, **no list verb** — so the last two need the id in hand. An ORACLE, materially weaker than `.11.14.1`'s `GET /v1/snapshots/stale`.
- ⭐ **DECIDED, and it is TWO answers rather than one** (`docs/decisions/2026-09-17_the-reference-read-is-bound-to-its-registrants.md`, four alternatives rejected). The detail read and the resolve verb are **bound**; the pair replay is **kept and published**.
- ⛔ **The leaf's own warning was about the WRONG MECHANISM.** *"A per-tenant reference set would break the dedupe the pair key exists for"* is true of a tenant COLUMN and false of a SET — which is exactly why `0062` is a separate table. `migrations/0067` adds `reference_registrations` with the pair key untouched; one shared row still serves every tenant that names it, and the control asserts **2 registrations on one row**.
- ⭐ **The difference from the snapshot ruling is NAMED rather than inherited**, which is the test `CLAIM_VERIFICATION.md` §3 leg 2 sets: confirming a SNAPSHOT means presenting its **bytes**, so `.11.14.1` closed both halves at once; confirming a REFERENCE means presenting a **locator**, which anyone can type. So the existence half is structural, and `POST /v1/resources` still answers `replayed: true` — published with its width (the caller must already know the locator AND the digest).
- ⭐ **Binding the detail read costs nothing, measured** by the rule `.11.14.3.8` earned one commit earlier: every route that yields a `res_…` id goes through the pair replay, and the replay now RECORDS the caller's registration. What changes is the **price of admission** — a bare opaque handle used to be the whole predicate, and the locator is now required, which is the very thing the row would have disclosed.
- **NO BACKFILL**, the fourth time in this family and for `0062`'s measured reason: `submitted_by` is a one-way `Uuid::new_v5` that joins to no identity table and the replay leaves it naming the FIRST registrant for ever. Registering the pair again restores the read, exactly as re-acquiring restores a snapshot's.
- **Blast radius:** 21 fixture cleanup plans name `resource_references` and every one gains `reference_registrations` before it, because the shared checker validates the whole declared plan before the first deletion.
- ⚠️ **Routed with its width MEASURED, not assumed — `.11.14.3.10`.** `credential_binding_ref` is a caller-supplied field that SELECTS a credential, the broker's store carries **no tenant at all**, and the pair key makes the field shared. ⛔ But `grep -rn "\.register(" crates/ --include=*.rs | grep -i broker` finds **3 hits, all tests**, `rb-server` builds `Broker::default()` — an empty store — and `RB_ENABLE_R5R3RX` is off by default. So it is **latent, not live**, and publishing it as a live cross-tenant credential path would have been a claim derived by reading.

🔴 **A CORRECTED CLAIM WAS CORRECTED IN THE REGISTER AND LEFT STANDING IN THE CORPUS (`.13.4`, DOC-0042).** Two blockers were measured false — remote CI on 2026-09-17, the licence on 2026-09-15 — and the book's qualification chapter kept asserting both. Census over the live documents: **11 hits, 4 stale, 7 sound**; all four corrected in place, naming what they used to say. The dated checkpoint artifact is left byte-unchanged. ⛔ Root cause: `.13.3`'s acceptance scoped itself to *this tree* and nothing asked what else in the corpus restated the claim — the shape `.11.16` owns, now with a second instance in a second claim kind. ⚠️ Whether the sweep should be MECHANICAL is routed there and deliberately unanswered.

✅ **THE STANDALONE ASSESSMENT ROUTE IS BOUND TO THE CITING TENANT (`.11.14.3.8`, REPAIR-0222).**

✅ **The leaf predicted a two-answer existence oracle. The control measured THREE answers, and the third is a content probe.**

- 🔴 **REPRODUCE, RED first.** A second tenant holding an `snp_` id it had never cited, driven through `POST /v1/assessments`:

```text
(400, "the cited snapshot does not exist")
(400, "the excerpt does not appear in the snapshot's bytes — the citation is refused")
(200, {"assessment_id": "asn_01a0ae3c81f57622ab8ce416cc1f4099"})
```

  The first two separate *exists* from *does not exist*. ⭐ The third says a **chosen substring appears in bytes the caller was never allowed to read** — and it stored a row. `44 passed; 1 failed`.
- **ROOT CAUSE.** `claims::submit` selects `snapshot_objects.bytes` joined to `evidence_snapshots` on `snapshot_id` alone. ⛔ It cannot carry a tenant predicate: `evidence_snapshots` has no tenant column BY DESIGN — one row serves every tenant that acquired the same bytes — so the binding has to be the separate `evidence_citations` question. `threads.rs`'s `assess` step asked it, with the comment *"without this, an assessment would be a way to learn that a snapshot exists"*. The route did not, and both call the same function.
- **The census, in BOTH directions:** every surface naming a `snapshot_id` — the snapshot read, its delete, its derivations, its assessments, the staleness list, the `assess` step, and `POST /v1/assessments`. **Seven, six citation-bound, exactly one not.**
- ⭐ **The compatibility objection was MEASURED, not accepted.** *"A principal legitimately assessing evidence another team acquired would start being refused"* is what made this a decision rather than a fix — and the census refutes it: every READ of that snapshot already answers a non-citing tenant `404`, so that workflow **cannot function today**. The excerpt check was running over bytes the caller can neither see, list nor delete. The gate removes no working path.
- **DECIDED: the route takes the citation gate** (`docs/decisions/2026-09-17_the-standalone-assessment-is-citation-bound.md`, three alternatives rejected — a uniform refusal leaves the content probe, which is the stronger leg; leaving it loses to the six prior rulings of the same shape unless the difference is named; a tenant predicate in the select is impossible by construction).
- **FIX: the gate is `claims::submit`'s, not the handler's**, because the store is what both writers reach and the finding located the defect in its SQL. New `AssessmentError::SnapshotNotCited`, checked **before anything about the snapshot is read**, with a message carrying no identifier and no fact about the snapshot — so an absent id and an uncited one are the same bytes. ⭐ The `assess` step keeps its own check: that one is the NAMED refusal, this one is the INVARIANT, and for the deliberation path it should never fire.
- **ADDRESSED.** All three probes → `400`, one identical message, **0 rows written**. The citing tenant still assesses (`200`) and still receives the excerpt diagnosis by name. A second tenant that acquires the same bytes replays to the SAME `snapshot_id`, records its citation, assesses it and holds a **separate** row — the bound that makes this a binding rather than a blackout.
- **NO REGRESSION.** One existing control changed and is corrected rather than relaxed: `the_two_assessment_writers_are_two_namespaces`'s stranger arm reached the excerpt check over a snapshot it had never cited, which **was** this gap. The stranger now acquires the bytes first, and every original assertion is intact. ⭐ The gate does not subsume `.11.14.3.3`: two tenants that both cite one shared row still both reach it, which is what keeps `authored_by_tenant` load-bearing.
- ⚠️ **Open and stated:** two principals inside ONE tenant still alias on this route (`.7.4`); `SnapshotMissing` and `ExcerptAbsent` are deliberately kept distinguishable for a caller that HAS cited the snapshot, because the diagnosis is owed to a caller entitled to the bytes.
- **Verification:** `RB_DEMO=0 bash scripts/run_pg_tests.sh profiles` → **45 passed / 0 failed** (RED baseline 44/1); `cargo clippy -p reasonbraid-server --all-targets --locked -- -D warnings` clean; `cargo fmt --all -- --check` clean.

✅ **THE ADAPTER LADDER IS AHEAD OF ITS CALLER — The adapter ladder is ahead of its caller, and the ceiling permitted what nothing declares (`.13.1.1`, REPAIR-0221).**

✅ **A security control measured crate-wide rather than at the two items already suspected — and the census found a third.**

- **The census, in a tracked instrument** (`scripts/census_adapter_public_api.py`, two-sided `--self-test`): **102 `pub` items, 17 with no non-test caller.** The leaf named two; the answer is **three** — `AllowedCapabilities` joins `verify_ladder` and `capabilities_within`.
- 🔴 **All three have exactly ONE non-test mention outside their file, and it is the same `pub use` line.** ⭐ A re-export is not a caller — and it is the mechanism that blinds `dead_code`: without it the items would be crate-private, unused, and the compiler would have said so on the first build. ⚠️ A mention census over-counts by one per re-export, and **one is the most dangerous answer**, because it reads as a caller where a zero would prompt a second look.
- **The other 14 are CLASSIFIED, not published as defects**: 1 test-support, 13 over-exposed helpers used only within their own file. Publishing "17 with no caller" would have been wrong in 14 cases.
- **(a) Stated, not wired.** No adapter load path exists — `git grep -nE "libloading|dlopen|Library::new|load_adapter|from_path"` returns nothing. Wiring a check into a path that never runs creates a second false assurance; the fact is recorded **at the definition** so a reader cannot repeat the mistake.
- **(b) `AllowedCapabilities::dev()` now declares `tool_support: false`** — it was `true`, in the crate's only ceiling, whose own doc says it describes "the shipped adapters' shapes" while every shipped adapter declares `false`. ⚠️ **No runtime behaviour changes and none is claimed**; what changes is the direction the ladder fails if ever wired — closed rather than open.
- **(c) Already satisfied, verified rather than assumed**: the book cites the ladder **0** times; `.13.1.2` had corrected it. Nothing edited.
- 🔎 **The self-test caught a defect in its own assertion** — the mirror of `.13.1.2`, where a weak assertion passed a broken parser. Here a wrong assertion failed a working stripper: it matched the bare name, which the probe's own definition line contains.
- ⛔ **B3 is NOT advanced.** It remains an external blocker; the boundary is still held declaration-side, pinned by `ACTION-BOUNDARY`.
- **Verification:** 14 adapter lib tests (incl. a new two-sided ceiling control) + 38 across the crate's other targets, 0 failed; clippy exit 0 zero diagnostics; `make gate` 18/18; `make book` rc=0.

✅ **THE ASSESSMENT NAMESPACE IS PART OF THE ROW — AND TAKING THAT DECISION FOUND A LIVE ALIASING DEFECT (`.11.14.3.3`, REPAIR-0219).**

- **The population was measured TWO ways, because "this project has no production data" is an assumption and only one of the two survives the project acquiring some.** **Writes:** `git grep -n "INSERT INTO claim_assessments" -- .` returns **1** statement (`claims.rs:152`), reached by exactly **2** callers; `git grep -ln "claim_assessments" -- . ':!crates/**/tests/**' ':!docs/**'` returns only the two defining migrations, the two sources, the MCP fixture-purge table LIST and live docs — **no migration, seed, fixture or deployment artifact inserts a row**; and every invented identifier (`clm_budget`, `clm_beta`, `clm_g4`) lives only inside `profiles.rs`. **Durability — and the correction that made it honest.** The first census ran `find . -name PG_VERSION -not -path "./target/*"` and concluded no cluster exists anywhere in the repository. ⛔ **The exclusion is precisely why that check could not fail**: `run_pg_tests.py` destroys its cluster on success and **RETAINS it under `target/` as failure evidence** — the very path excluded. Run without it: **32 PG data dirs and 3 retained clusters**, one of which (`target/pg-tests/run-9_ueev0t`) is this leaf's own RED run. Rows exist right now. ⭐ The defensible statement is narrower and is carried by the WRITE census rather than by `find`: **no row is carried in TRACKED state** — `target/` is gitignored (`.gitignore:2`), and the GREEN run's own line reads `pg-tests: stopped and removed target/pg-tests/run-wowo355p`.
- 🔴 **RED was not the defect the leaf was opened on.** `claim_assessments_replay_idx` was `(claim_id, snapshot_id, assessment, author)`, and `claims::submit` pre-checks on exactly those four columns before inserting. On the standalone route **`author` is a caller-supplied label** — `.11.14.2` added `authored_by_tenant` precisely *because* `author` cannot be trusted for authorization — so a caller naming a real thread's minted claim digest, that thread's snapshot, its kind and its author matched the whole key, and the pre-check returned **the deliberation's own `assessment_id`**: `assertion \`left != right\` failed … left: "asn_01a0ac9b3bbb7d3293dcd08c20f9fc07" right: "asn_01a0ac9b3bbb7d3293dcd08c20f9fc07"`. `43 passed; 1 failed`.
- 🔴 **The width I first published was WRONG, and auditing it against `docs/CLAIM_VERIFICATION.md` found a SECOND defect.** The first version said the oracle was "bounded to ONE tenant by `.11.14.2`'s authoring gate". ⛔ The authoring gate binds the two READS; the replay pre-check carries **no tenant predicate at all** — and that width had been published by READING the SQL, which leg 2 forbids exactly because reading errs in the flattering direction. Measured afterwards, with `claim_namespace` ALREADY in place: `assertion left != right failed: a SECOND TENANT was handed the first tenant's assessment_id`, `asn_01a0ace7727c7b31bea32938bf807721` on both sides, `43 passed; 1 failed`. ⭐ `authored_by_tenant` therefore joins the replay key and the pre-check, which is what makes `migrations/0064`'s OWN sentence true: it justified its design by saying the key "carries the AUTHOR, so two tenants asserting the same thing already hold two separate rows", and stated **eight lines later in the same file** that `author` is an unauthenticated caller string. ⚠️ And the existing two-tenant control could never have caught it — both tenants submit their own principal as `author`, so they separate naturally; it ILLUSTRATED the property rather than testing it. ⚠️ Left open and stated: two principals inside ONE tenant can still alias each other, which is a deduplication question rather than a disclosure, because the authoring gate already admits both of them to the row. It is **not** an enumeration, and materially weaker than `.11.14.1`'s `GET /v1/snapshots/stale`.
- ⭐ **The decision:** the namespace is part of the row's **IDENTITY** (`docs/decisions/2026-09-17_the-assessment-namespace-is-part-of-the-row.md`, three alternatives rejected). `migrations/0066` adds `claim_namespace` (`thread` | `external`, NULL for pre-existing) and rebuilds the replay index to carry it, `NULLS NOT DISTINCT` so the new key is not WEAKER than the one it replaces. The namespace comes from the CALL SITE, never the submission. ⛔ **It joins the PRE-CHECK as well as the index, and that is not a detail** — a uniqueness constraint constrains the write, not a read standing in front of the write, so an index alone would have left the aliasing exactly where it was.
- **The ROWS: left, no backfill — the third time this family has taken that route** (`.11.14.1`, `.11.14.2`) and the third time it is published rather than assumed. ⭐ The reason that would still hold if the table were full: a pre-existing row's namespace is unrecoverable **in principle**, because the only evidence would be the SHAPE of its `claim_id` and a caller can always type the minted shape — which is the collision the column exists to record. NULL means "unattributed", the disposition `0064` already took on this same table.
- ⭐ **Minting a digest on the standalone route is the plausible WRONG answer, and rejecting it is the substance of the decision.** What makes the thread identifier trustworthy is `claim_exists_in_thread` — the MEMBERSHIP CHECK — not the hashing. A standalone route has no thread to be a member of, so a digest there is a hash of a caller's own string: exactly as invented as the label, but **indistinguishable from a trustworthy one by construction**. ⛔ Filtering the claim read to `thread` rows was rejected too: it makes the standalone route write-only — a route that writes rows nothing can read — which is worse than the removal already declined.
- **Why DISCLOSURE is the whole repair, measured rather than asserted:** nothing in the product reads `claim_assessments` to make a decision — `git grep` finds the two list routes and nothing else, and "evidence gate" appears in this area only in doc comments. An ungated row cannot change an outcome; it can only mislead a reader, and a labelled row does not.
- 🔎 **One residual OWNED rather than reported** (`.11.14.3.8`): `POST /v1/assessments` applies **no citation gate** — `claims::submit` reads `snapshot_objects.bytes` for any `snapshot_id` with no tenant predicate and reports whether an excerpt appears in it, while the `assess` step refuses a snapshot the tenant never cited using `.11.14.3.1`'s own stated reason. Both writers call the SAME function.
- **Verification:** `profiles` 44, `command_api` 39, `migration_upgrade` 4 (the new `0066` applies), `mcp_write` 5, `cli_end_to_end` 5 — **97 / 0**, every cluster stopped and removed. `cargo clippy -p reasonbraid-server --all-targets --locked -- -D warnings` exit 0 with zero diagnostics; `cargo fmt --all` rc=0; `make gate` 18/18; `make book` rc=0.
- ⚠️ **The block below this one is now a PRE-REPAIR description, left standing rather than edited.** `.11.14.3.1`'s block closes by saying `POST /v1/assessments` "stays, with a free-text `claim_id`" and that "two writers produc[e] two kinds of identifier into one column". Both were true when written; the route still stays and the identifiers are still two kinds, but the column now records which. Each block states what was true at its own commit.

✅ **A CONTRIBUTION'S CITATION REGISTERS THE EVIDENCE REFERENCE IT NAMES — §13.2 STEP 2 IS WIRED, AND TAKING IT FOUND A CROSS-TENANT DENIAL (`.11.14.3.2`, REPAIR-0218).**

- **RED, and the two failures are the two defects.** `RB_DEMO=0 bash scripts/run_pg_tests.sh profiles` with the sources stashed: **`41 passed; 2 failed`** in 35.36 s. `the_contribution_citation_registers_a_resource_reference` — *"a contribution's `EvidenceRef` resolves to nothing — the citation is a string beside an evidence store holding the row it describes: []"*, **0** rows where 1 was required. `a_reference_submits_typed_and_the_locator_digest_pair_is_the_key` — the second defect in the product's own words: `{"code":"locator_digest_conflict","message":"the locator's digest is immutable — the same locator with a different digest conflicts"}`, **409** where 200 was required.
- **The decision:** register on citation, and the identity of a reference is the `(original_locator, expected_digest)` **PAIR** — `docs/decisions/2026-09-16_a-citation-registers-the-reference-it-names.md`, five alternatives rejected. §13.2 *registers* at step 2 and *acquires* at step 6, so refusing a citation for naming something unacquired inverts the flow the leaf exists to build.
- 🔴 **The repair could not ship without a second one, and that second one is the more serious finding.** `resources::submit` pre-checked on the **locator alone** and refused a differing digest as `locator_digest_conflict`. Put on the contribution path that becomes a **cross-tenant denial**: the first principal to pin a locator makes it uncitable by every other tenant *in any form* — a digest-less citation takes the same branch, because `None != Some(D)`. On a public network, pinning the popular URLs is one cheap loop.
- ⛔ **And the refusal was already a §9.8 breach before this leaf touched it:** *"cross-tenant existence is not leaked"*. A 409 on `(L, D2)` told the caller that somebody else had registered `L` at a digest they were never shown.
- ⭐ **Three clauses of the frozen roadmap all say the pair, and the schema agreed with them before the code did.** §12.1 — canonicalization "must not erase security-relevant distinctions"; §12.6 — "a live Web page or branch can change"; §9.8. `migrations/0023:21` has declared `UNIQUE (original_locator, expected_digest)` since the table was created, and the pre-check made it unreachable for a differing digest.
- **Migration `0065`** rebuilds that constraint `NULLS NOT DISTINCT` (PostgreSQL 15+; this project pins 16), so a digest-less citation is ONE row rather than one per citation. No backfill and none possible to need: under the old pre-check a locator could hold only one digest, so no duplicate pair exists.
- **Two validations a citation never had:** the scheme is parsed from the locator itself (RFC 3986 §3.1) rather than claimed beside it, and the digest is the ADR-011 `sha256:<64 hex>` scheme. ⭐ The shipped `command_api` control's own fixture cited `sha256:abc123` — six characters where ADR-011 asks for sixty-four — which is what a field nothing reads looks like after two phases.
- **Four residual findings are OWNED rather than reported** (`.11.14.3.4`–`.7`): the reference read is unbound and `.11.14`'s per-table verdict for it answered the column question, not the read one; `POST /v1/resources` still takes `scheme` unvalidated, its omitted fields bypass the column defaults, and a locator's fragment stays in the locator; `expected_digest` is a pin nothing enforces at acquisition; and registering turns an unbounded citation list from unbounded INPUT into O(n) work inside the transaction holding the thread's `FOR UPDATE` row — `thread.contribute` carries no quota at all, and a cap was deliberately not invented.
- 🔎 **One more, outside this family:** `DOCTRINE_ENFORCEMENT.md`'s `REASON-CODE-DOC` row restated a census that had drifted since `.9.2.1.1`, and retiring a code made it true again **by accident**. The book's introduction had the same shape — "rotated four times" against a footer saying nineteen, two paragraphs above its own warning not to copy such facts. The book instance is fixed here; the doctrine one is `.11.16`'s evidence.
- ⚠️ **Two sentences lower down this page are now PRE-REPAIR descriptions, left standing rather than edited.** `.11.14.1`'s block says *"`resources::submit` replays on the **locator alone**"* — true when measured, and the very property this leaf retired; `.11.14.3`'s block says the seam was *"Split into `.11.14.3.1`–`.3`"*, which grew to `.1`–`.7` as executing `.2` found four more. ⭐ Each block states what was true at its own commit, which is the only way a stacked snapshot stays honest; the correction is here, at the top, where a reader meets it first.
- **Verification:** `profiles` 43, `command_api` 39, `authority` 22, `policy` 14, `node_inbox` 8, `invitations` 6, `cli_end_to_end` 5, `mcp_write` 5, `escalation` 4, `migration_upgrade` 4 — **150 / 0**, every cluster stopped and removed. Strict focused clippy and `cargo fmt --all --check` rc=0; `make gate`; `make book`.

✅ **THE `assess` STEP RECORDS AN ASSESSMENT AGAINST THE THREAD — THE DELIBERATION AND THE EVIDENCE CHAIN NOW MEET (`.11.14.3.1`, REPAIR-0217).**

- **RED was the error message itself:** `unknown field \`assessment\`, expected one of \`tenant_id\`, \`content\`, \`kind\`, \`evidence_refs\`, \`claims\`, \`target_claim_digest\`, \`verdict\`, \`ref_event_id\`, \`synthesis\``. ⭐ The contribution body carries a payload for `verdict` and one for `synthesis` — the other two step-bound kinds — and **none for an assessment**. `41 passed; 1 failed`; after the repair **42 / 0**, plus ten regression suites.
- **The control drives the SHIPPED `evidence_review` profile**, not a fixture, so the RED is a tenant's real path: create the thread, contribute a claim on `solicit`, advance twice to `assess`, and try to record what the step is named for.
- **Accepted path checked three ways rather than once:** the stored row (`claim_id` = the digest, the snapshot, `supports`, `authored_by_tenant`), the claim-keyed HTTP read, and the timeline event's `assessment` object.
- ⭐ **Every gate already existed; the leaf composed them.** `claim_exists_in_thread` (the check an evidence request already gets), `snapshots::is_cited_by` (`.11.14.1`), `claims::submit`'s §12.7 vocabulary and excerpt validation, `.11.14.2`'s authoring tenant. **No migration** — the schema was already right, which is the evidence that DOC-0037 was correct to call this wiring rather than design.
- ⭐ **What needed new code was ATOMICITY.** `claims::submit` and `is_cited_by` took a `&PgPool` and could not join the thread's transaction, so an assessment could have been written while its contribution rolled back. Both are now executor-generic on the house's `E: DerefMut` bound; the `assess` path passes `&mut *tx`, and the event and the row commit together. A storage fault maps to `CorruptState`, never `InvalidCommand` (`.7.4.2`).
- **Six refusals, each naming its gate:** wrong step, a digest that is not a claim of this thread, a snapshot this tenant never cited, an excerpt absent from the acquired bytes, the payload on another kind, and an assessment word outside the five.
- 🔎 **The diagnostic was then run over the WHOLE vocabulary, and classified rather than counted.** Of thirteen `STEP_KINDS`: 6 gate a kind, 3 are terminals, 2 appear only in the default profile's list, and **2 return zero** — `critique` and `retrospect`. ⛔ Those two are NOT claimed as defects: a step with no dedicated kind is still a real phase, and what made `assess` different was the complete STORE sitting unreachable behind its zero with §13.2 naming the step that should reach it.
- ⛔ **`POST /v1/assessments` stays**, with a free-text `claim_id` — removing a shipped route is a breaking change §13.2 does not ask for. ⚠️ That leaves two writers producing two kinds of identifier into one column, so `.11.14.3.3` is widened to own the remaining writer as well as the legacy rows, and the book publishes the split rather than implying uniformity.

🔎 **A CLAIM I PUBLISHED TO THE DIRECTOR DID NOT SURVIVE ITS OWN FALSIFICATION (`.11.2.5`).**

- **The claim:** "the enforcement layer is doing real work, not ceremony", offered on the strength of two gates catching me in one session. Falsifying it rather than restating it: **two of three sub-claims held, one did not.**
- ✅ `INDEX-FRONTIER` — verified by a DISCRIMINATING test, which is the part that matters: reconstructing the stale cell returns rc=**1** with the two leaf ids named, and restoring it returns rc=**0**. The gate is neither always-red nor always-green.
- ✅ `GAP-CLAIM-CENSUS` refused an unmeasured claim of mine — that happened, and re-deriving it showed exactly WHY it happened.
- 🔴 **And why it happened is the defect.** Two probes, identical claim line, one variable: a section that also contains `` `cargo clippy …` `` is **not flagged**; the same claim alone **is**. `check_gap_claims.sh:71` discharges a claim when ANY line of its section matches `CENSUS_RE` (`:60`), which includes `cargo `, `make ` and `git grep`. ⛔ **Every closed leaf's `[x] ADDRESSED` box names one by construction** — `TASK-ACCEPTANCE` requires tool output in each box — so the census gate is INERT for exactly the leaves that are complete. It caught me because `.7.4.5` was still being opened. The catch was real; the mechanism was luck.
- 🔎 **And my own CORRECTION was over-approximated in the other direction.** It cited "17 hits across 6 files" as evidence that denials are checked. Classifying rather than counting: **9 across 3 files** genuinely assert a site denial left an audit row; the other 8 are aggregate counters, fixture rows and one TENANT audit. `.7.4.5`'s census now says so.
- ⚠️ Not an argument for widening the pattern — the script's own `:17` explains that firing on 81 pre-existing claims "teaches bypass". The scoping is sound; the DISCHARGE test is the defect: it asks whether a section contains a command, not whether it contains a command *for this claim*.

✅ **THE SITE AUTHORIZATION SKELETON IS ONE COPY AGAIN (`.7.4.5`, REPAIR-0216).**

- `.7.4.3` created the second copy deliberately — refactoring the control flow of an authorization path inside the commit that repairs a hole in it is how a second hole ships — and opened this leaf in the same breath. It is closed two commits later.
- **One `site_authority::authorized(pool, subject, action, target, reason, effect)` owns the transaction:** take the guard, read the clock AFTER the lock wait, evaluate the grant, then either commit an audited denial or run the effect and audit what it produced. Both call sites become an action, a target, a reason and a closure.
- ⭐ **What a reader must check that the compiler cannot, each tied to a NAMED existing control rather than to inspection:** a denial still commits its own record (the denial controls assert a `denied` row and an `audit_id`); a domain refusal keeps its grant and boundary and its own 400 (`undeclared_region`); an audit failure rolls back an allowed effect (`audit_failure_rolls_back_the_registry_effect` injects a raising trigger and requires `Err(Sql(_))` with 0 registry rows and 0 audit rows); the effect runs only after a live grant and on the transaction's own connection; and the clock is the post-lock `clock_timestamp()`.
- **Verification: 11 + 8 + 3 + 3 + 2 + 41 = 68 passed, 0 failed** — every count identical to the pre-refactor run. No migration, route, wire shape or qualification category moved.
- ⚠️ One non-behavioural change recorded so it is not a puzzle later: `execute` now clones its `RegistryCommand` into the closure, because a boxed effect future quantified over the connection's lifetime cannot capture a reference to it.

✅ **DECIDED — THE DELIBERATION FLOW OWNS THE EVIDENCE CHAIN, AND THE HOLD THAT PRECEDED IT WAS WRONG (`.11.14.3`, DOC-0037).**

- **The census, in both directions, over nine modules.** Evidence-store symbols inside `threads.rs`, `agg.rs`, `projections.rs`, `workflows.rs`, `matching.rs` → **0, 0, 0, 0, 0**. Deliberation symbols inside `claims.rs`, `snapshots.rs`, `derivations.rs`, `resources.rs` → **0, 0, 0, 0**. No call, no shared key, no shared type.
- 🔴 **The sharper half is the citation end, because the key already matches.** A contribution cites evidence as `EvidenceRef { uri, digest, note }` (`threads.rs:329`), written verbatim into the event body at `:1527` and resolved by nothing. `resource_references` is `UNIQUE (original_locator, expected_digest)` — **the same key**. A contributor naming a URL at a digest is naming exactly the row the evidence store would hold, and the two are never joined. A deliberation's citations are strings; a complete acquisition → snapshot → derivation → assessment pipeline sits beside them holding the rows those strings describe.
- **The other half:** `claim_assessments.claim_id` is caller-supplied `TEXT` while `threads.rs:401` mints a SERVER-COMPUTED claim digest that `threads.rs:1605` membership-checks. An assessment can cite a claim no contribution ever made.
- ⚠️ **This is not a disclosure path** — `.11.14.1` and `.11.14.2` closed those — **and not a defect in either subsystem alone.** Both work, are tested, and meet their own acceptance. It is a seam no leaf ever owned, because every leaf owned one side of it.
- 🔎 **This block first said the decision was HELD because the frozen roadmap "settles it neither way". That was wrong, and the correction is the finding.** It was written from §12.7 (evidence chapter, names no thread) and §5.2.2 (defines claims without naming the evidence store). **§13.2 was not read.** Its "Rigorous deliberation reference flow" is step 2 *"register context and resource references"*, step 5 *"normalize claims … and requested evidence"*, step 6 *"acquire/assess evidence within the allowed plan"*. The canonical flow CONTAINS both halves. ⛔ A "the specification is silent" claim is a census over the specification, and that one covered two sections of a twenty-five-section document.
- 🔴 **And the product already encodes §13.2, which is what makes it unambiguous.** `crates/reasonbraid-server/src/workflows.rs:17`'s `STEP_KINDS` carries `evidence_request` AND `assess`; `migrations/0032` seeds eight built-ins and **two declare `assess`** — `evidence_review` and `policy_proposal`. Then `git grep -n '"assess"' -- crates/reasonbraid-server/src` returns **one hit: the vocabulary constant itself.** Nothing gates on it, no contribution kind maps to it, nothing writes through it — while `evidence_request` IS wired. **A tenant running the shipped `evidence_review` profile advances onto a step at which no assessment can be recorded.**
- ✅ **The decision:** the deliberation flow owns the evidence chain, joined at the two points §13.2 names — `assess` records a `claim_assessments` row keyed by a membership-checked claim digest of that thread over a snapshot its tenant CITED, and a contribution's `EvidenceRef` resolves to a `resource_references` row (the same key). Four alternatives rejected, including a `thread_id` column on the content-addressed tables. ⭐ Every part the join needs already exists — the claim digest and its membership check, `.11.14.1`'s citation gate, `.11.14.2`'s authoring binding, `claims::submit`'s excerpt validation — which is itself the evidence that this is WIRING, not design. Split into `.11.14.3.1`–`.3`, checked against four mechanisms.

🔴 **AN ASSESSMENT IS READ BY THE TENANT THAT AUTHORED IT — AND `.11.14`'s OWN DECISION RECORD WAS WRONG ABOUT THIS TABLE (`.11.14.2`, REPAIR-0215).**

- **RED:** `tenant A read tenant B's position on evidence A never cited: [3 assessment ids]`. Tenant A, reading a claim identifier it GUESSED, received all three assessments — including tenant B's position on evidence A never touched. The identifier is `clm_budget`, which is the shipped control's own: the namespace is guessable because nothing mints it. `40 passed; 1 failed`. After the repair: **98 / 0** across six suites.
- ⛔ **This was an ORACLE, which `.11.14.1`'s enumeration explicitly was not.** `GET /v1/claims/{claim_id}/assessments` is keyed on caller-supplied `TEXT` with no key and no `claims` table behind it, so a guessed identifier was a complete answer.
- 🔎 **The record this family rests on grouped the table wrongly, and the SCHEMA is what shows it.** `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md` puts `claim_assessments` with `evidence_snapshots` and `derivations` under "site-wide row, tenant-bound read", reasoning from content-addressing: a shared row cannot carry an owner. But `derivations_replay_idx` is `(parent_snapshot_id, derived_kind, derived_digest)` — genuinely shared — while `claim_assessments_replay_idx` is `(claim_id, snapshot_id, assessment, author)` and **carries the author**. Two tenants asserting the same thing already hold two rows. ⭐ An assessment is an AUTHORED OPINION, not a shared receipt, so the argument that correctly forbids a column on the other two never reached it. The record now carries a **Correction** section and one superseded verdict; the other eleven stand.
- ⭐ **And binding on the author closed a residual `.11.14.1` measured and could not close.** That leaf gated the child reads on the PARENT snapshot's citation — so two tenants citing one shared snapshot each read the other's assessments of it, because the gate admitted on the very citation they both held. The control cites ONE shared snapshot from both tenants and requires that neither sees the other's position. ⛔ Still open for `derivations`, which are content-addressed and shared by exactly that construction; the limit is in the book rather than implied.
- ⛔ **`author` is untouched and the authorization never reads it.** It remains an unauthenticated caller label; making it trustworthy changes the replay key and is a forgery question, already `.7.4`'s attached clause, deferred there BY NAME rather than given a second owner.
- 🔴 **The measurement opened a bigger defect than the one it closed, and it is owned rather than reported.** The product HAS a real claim identity — `threads.rs:401`'s `ClaimRecord` with a **server-computed** ADR-011 digest, membership-checked at `threads.rs:1605` so a forged digest fails — and the evidence graph does not use it: `grep -c "ClaimRecord\|claim_digest\|threads::" crates/reasonbraid-server/src/claims.rs` returns **0**. An assessment may cite a claim no contribution ever made, and nothing notices. That is `.11.14.3`, and it is now the frontier's first row.

🔴 **THE RETENTION SWEEP IS A SITE-OPERATOR ACT ON THE SERVER'S OWN CLOCK (`.7.4.3`, REPAIR-0214).**

- **RED, and the response body counts the damage.** `an enrolled principal without site authority swept the site: {"tombstoned":2}` — HTTP **200**. Tenant A, holding nothing but an enrolment and naming `"at": "3000-01-01T00:00:00Z"`, tombstoned BOTH tenants' live rows in one request. `39 passed; 1 failed`. After the repair the six site-touching suites pass **67 / 0**.
- **Two independent defects on one route, either sufficient alone.** The gate was `reader_tenant(…).is_some()` — enrolment — over a sweep with no tenant predicate; and the cutoff was `body.get("at")` with no bound, so the caller decided which rows were "due". The clock is what turns a site-wide sweep into an immediate total one; the missing authority is what lets any tenant reach it.
- ⛔ **It cannot be a tenant verb, and that is schema rather than preference.** `retention_class` is a column on the SHARED row — which `.11.14.1`'s citation table is exactly what made explicit — so *which rows are due* is a site-wide fact no citation owns. A per-tenant sweep would first have to move retention onto the citation, a different decision about a different table. The sweep takes the new `evidence_expire` capability on `2026-09-09_site-operator-authority.md`'s existing shape, with the tombstones and their audit committing as one guarded transaction.
- ⛔ **The caller's clock is REMOVED, not bounded.** `at <= now` would have closed the amplification and left the wrong shape: a row stamped `"the retention expired"` is a factual claim, and a caller who picks the clock can make it false. `deny_unknown_fields` makes `at` a **400**, so the original attack is no longer expressible rather than merely denied — the control's first leg asserts 400 where it observed 200.
- ⭐ **The `at` parameter was a test affordance on a production route and said so in its own doc comment.** The TTL-boundary controls that needed it now drive `snapshots::expire_due` directly, keeping every assertion — the ±1µs boundaries at one and thirty days, the repeated-sweep no-op, the untouched `audit` class, the replay's preserved creation time. Those are assertions about a pure function; what the ROUTE owes is what the new control measures.
- 🔎 **It found a defect in REPAIR-0213's own control, one commit old, and fixed it.** That leg presented `hum_0000…` and asserted **401** for the enrolment gate. `hum_…` is not a principal shape, so `resolve_principal` refused it as `unauthenticated` before the gate ran: the leg passed while measuring the header parser. The gate answers **403** `unauthorized` — and the BOOK carried the wrong number too. Both are corrected, both controls now present a well-formed stranger, and the evidence-binding control asserts the two refusals separately so they cannot be conflated again.
- ⚠️ **Two things this leaf refused to do quietly.** It NARROWED itself before writing code — the shared-row deletion question it was opened carrying is a decision of comparable size, so it is `.7.4.4` with its own acceptance and a derived candidate answer, not a clause ticked in passing. And it created a second copy of the site authorization skeleton deliberately, because refactoring the control flow of an authorization path inside the commit that repairs a hole in it is how a second hole ships; the duplication is named and owned as `.7.4.5` rather than left to be found.

🔴 **THE EVIDENCE READS ARE BOUND TO THE CITING TENANT, AND THE DERIVATION PATH THE LEAF PREDICTED DID NOT EXIST (`.11.14.1`, REPAIR-0213).**

- **RED first, against a two-tenant fixture.** With the control added and no repair present, `the_evidence_reads_are_bound_to_the_citing_tenant` failed at `crates/reasonbraid-server/tests/profiles.rs:4790`: `tenant A enumerated tenant B's evidence trail: ["snp_01a0aa2feeab76c1961e5a65b40fd259", "snp_01a0aa2feead7e00b99ee7a07a2b644e"]` — two rows on tenant A's staleness surface, one of them B's. `38 passed; 1 failed`. After the repair: **39 passed, 0 failed**, cluster `stopped and removed`.
- 🔎 **The leaf's own prediction was wrong, and the measurement is what corrected it.** `.11.14.1` opened saying the binding means "deriving the citing tenant through the reference rather than storing it on the receipt". The reference cannot carry it either: `resource_references`'s only actor column is `submitted_by`, filled from `actor_handle_for_subject(…)` = `Uuid::new_v5(NAMESPACE_OID, subject.describe())` — a one-way value that joins to neither `human_principals` nor `agent_roles` — and `resources::submit` replays on the **locator alone**, so it names the FIRST citer whatever happens afterwards. `git grep -ln snapshot_id -- migrations` returns **3** files, all three the evidence tables: nothing else in the schema links a snapshot to anything.
- ⭐ **So the answer was the fourth possibility the leaf left room for — "a join this schema does not yet carry".** A citation is many-to-many by construction (one shared row, N citing tenants) and no scalar can hold it. `migrations/0062_evidence_citations.sql` records it, written by `snapshots::submit` on the fresh insert **and on the replay**.
- **The replay write is the part that matters.** It is what stops `.6.1.5`'s trap: a tenant that submits a snapshot another tenant acquired first would otherwise have been refused its own evidence. The control proves the opposite — the shared row replays to one id, appears on BOTH tenants' surfaces, and carries exactly **2** citation rows.
- **Five surfaces bound**, the fifth beyond the leaf's four and named as such: `stale`, `GET /v1/snapshots/{id}`, `/derivations`, `/assessments` and `DELETE /v1/snapshots/{id}` — which shares a route with the read, so leaving it would have let a tenant DELETE what it had just been stopped from READING. A foreign tenant gets **404**, not 403: a refusal that separated them would confirm an identifier exists. ⚠️ **Corrected by `.7.4.3` one commit later:** this block first said an unenrolled principal receives 401. The enrolment gate answers **403** `unauthorized`; 401 `unauthenticated` is the HEADER PARSER's answer to an id of the wrong shape, which is what the original control was actually measuring.
- ⛔ **`stale` stays a TENANT read**, settled on a census rather than on how the name sounds: `git grep -n "snapshots::stale" -- crates` and `git grep -n "snapshots/stale" -- crates` return the route, the function, its doc lines and one test — no CLI verb, no MCP tool, no worker.
- ⚠️ **Two limits published, not implied.** No backfill is possible by the same measurement that killed candidate (b), so a pre-migration snapshot is read by no tenant until it is cited again; and a shared row can still be tombstoned by any one of its citers.
- 🔴 **Two findings measured here are NOT repaired here, and both are owned.** `POST /v1/snapshots/expire-due` is enrolment-gated with an **unbounded caller-supplied `at`** and no tenant predicate, so one request naming the year 3000 tombstones every tenant's live `standard` and `temporary` evidence — irreversibly, since all three `SET deleted_at` statements set it and nothing clears it (`.7.4.3`, filed under the goal line that already said "restrict expiry clocks/scope" rather than beside the leaf that found it). And `GET /v1/claims/{claim_id}/assessments` is enrolment-gated over a namespace the server never mints — `git grep -n "CREATE TABLE claims" -- migrations` returns **zero** — which makes it an oracle over guessable ids (`.11.14.2`).

⛔ **THE POINTER REWRITE DID NOT FOLLOW THE SPECIFICATION THAT GOVERNS IT, AND IS CORRECTED (`.11.4.2.3`, DOC-0033).**

- 🔴 DOC-0032 rewrote layer A from a one-line director instruction and `COMMIT.md`, and did not re-read `MEMORY_ARCHITECTURE.md` §6 — which ends with a fenced block introduced as "the entire contents of a demoted `MEMORY.md`". `CLAUDE.md` lists that document as MANDATORY reading.
- **Five deviations, enumerated.** The title dropped its cap reminder; `How to resume` did not name `MEMORY_ARCHITECTURE.md`, the template's first line; the `Current state` block used free-form bullets instead of the five named fields and **omitted `in_flight_uncommitted`** — the field that tells a resuming session whether work is stranded; the block heading dropped "(OVERWRITE this block each update — do not append)"; and three sections were invented.
- ⛔ **The last one was a LAYER violation, not a shape one.** The "traps" it added were environment quirks, which §4's write path assigns to layer C by name — and they were already there: `2026-09-09_repository-local-command-environment.md` carries `project_env.py` at lines 21 and 63–70. ⭐ A commit that removed a log of durable facts from the pointer put two more durable facts back into it.
- **Three measurements:** **26 lines / 7,168 bytes** (the log) → **35 / 3,583** (invented structure) → **16 / 1,395** (the template), with 34 lines and 5,773 bytes free. No cap raised at any point.
- ⭐ **What DOC-0032 got right stands:** the partition was correct and the census still reports 0 standing warnings. What it got wrong is deriving the target shape from a conversation rather than from the file that defines it.
- ⚠️ **Routed rather than absorbed:** §6 also says "prefer derived over hand-written". Four of the five fields are mechanically derivable and one is a judgement sentence — the inverse of `.11.4.5.3`, which rejected its generator because 13 of 14 rows were curated prose. The census is still owed; new owner `.11.4.2.4`.

✅ **THE RESUME POINTER STOPS BEING A LOG, AND `.11.4.2.1`'s OWN DATA HAD THE ANSWER ELEVEN COMMITS EARLIER (`.11.4.2.2`, DOC-0032).**

- **The measurement:** `wc -c MEMORY.md` returned **7168** against a 7,168-byte cap — zero headroom — and every one of this session's thirteen commits had to evict or compress a standing warning to land.
- **The cause was SHAPE, not size.** The file's `- **Next action:**` bullet had become a **~6,000-byte single line** carrying sixteen lessons, one line of twenty-six. An append-only log was living inside one bullet of a pointer, which is why the byte cap bound while the line cap never did.
- 🔴 **`.11.4.2.1` had already measured the answer and the framing hid it.** That census asked "would anything be LOST if a warning were evicted?" and found **0 of 26 orphans**, concluded "the worry is refuted", and the evictions continued. The question it did not ask is the one that mattered: *if all 26 are durable elsewhere, why is any of them in the pointer at all?* ⭐ **A census answers the question it was given — the framing is the part to falsify.**
- **The partition, with the count on each side: 14 leave, 2 stay.** Each leaver is recorded in its own leaf, `docs/knowledge/`, `TOOLBOX.md`, `docs/decisions/` or is gate-enforced (`SECRET-SCAN`). The two that stay are pointer-shaped rather than lesson-shaped — the `docs/knowledge/` pointer, and the `project_env.py` trap that costs a fresh session its first command.
- ⛔ **The instrument proves no substance was lost, not the author**: `scripts/census_memory_warnings.py` now reports **0** standing warnings with **0** uncited, against 16 before.
- **Headroom, named:** **35 lines of 50** and **3,583 bytes of 7,168** — over half the byte cap unused, with **no cap raised**. ⚠️ The headroom is not the repair either: what keeps it is the rule now at the top of the file — overwritten, never appended, and a fact worth keeping goes to a durable layer.
- ⚠️ **The eviction ritual is DISSOLVED, not improved.** Three successive sharpenings of the selection rule were refinements of a choice that should not have existed. The leaf's own diagnosis was right: a rule that needs re-deriving every time it is applied is a sign the population wants partitioning — and the population was the whole log.
- ✅ **Also in this commit:** the vulnerability channel is LIVE. GitHub private vulnerability reporting was enabled by the repository owner on 2026-09-16, so `SECURITY.md` states the channel plainly and names no fallback route — naming two channels is how a report reaches the one nobody watches.

⭐ **THE FOUR B1–B4 DECISIONS ARE TAKEN, AND THREE OF THE FOUR OUTSIDE BLOCKERS SHARE ONE IN-REPO PREREQUISITE (DOC-0030, lane `SIGNOFF-REPAIR.14`).**

- ⭐ **The structural finding.** `2026-09-08_phase7-subtraction-record.md` has S-1 (the Internet exposure) not built because "the qualification gate is incomplete", while **B2** waits on "the exposure profile's candidate freeze" and **B3** on "its first action-bearing surface". That is a circular wait — and it is an artefact of S-1's stated REASON, not of the gate. ROADMAP §16.12 blocks Internet-capable **deployment**; §25.1 blocks **exposing** remote enrollment. Neither forbids BUILDING.
- **Decision 1 — build the candidate, un-deployed.** `SIGNOFF-REPAIR.14` owns it under a standing prohibition that no leaf may deploy it or claim any part of G6/G7. ⭐ It starts closer than the record suggests: `mtls.rs` builds a complete `rustls::ServerConfig` with a `WebPkiClientVerifier`, and `grep -rn "mtls::" -- crates` finds its only callers at `tests/mtls.rs:42` and `:72` while `grep -c mtls` over `rb-server.rs` returns **0**. A finished module with no production caller — `.13.1.1`'s shape.
- **Decision 2 — an OSS audit programme first, commercial fallback TWELVE WEEKS after the candidate freeze.** §2.6 requires *independence*, not procurement; the repository is public and the software unreleased, which is the profile those programmes exist for; and B2's own trigger is the freeze, so the application window costs nothing. ⛔ The fallback is dated because "we applied" is not a plan.
- **Decision 3 — GitHub private vulnerability reporting.** `SECURITY.md` said in its own words that "this policy does not assert a dedicated reporting endpoint exists", which also closed off every independent-findings route into B1/B2. The channel publishes no personal address and creates a draft advisory — the evidence-preservation and coordinated-disclosure machinery §16.12's last line asks for. ⚠️ One owner-side switch remains and the policy marks the channel PENDING rather than promising a route that does not yet accept reports.
- **Decision 4 — clearance covers the United States, the European Union and the holder's jurisdiction.** ⭐ Deferrable because the rename cost is MEASURED, not feared: **67 source files, 191 lines** — 144 crate-path identifiers, 18 prose, 64 hyphenated — and every shipped binary already neutral (`rb`, `rb-server`, `rb-site`, `rb-node`, `rb-journal`, `rb-release-manifest`).
- ⛔ **None of this advances the gate.** G6/G7 remains **NOT MET**; B1, B2, B3 and B4 remain open. What changed is that three of them now have a named next action inside this repository instead of a wait.

🔴 **TAKING `.11.14`'s DECISION FOUND A LIVE CROSS-TENANT ENUMERATION PATH (DOC-0029; the defect is `.11.14.1`).**

- ⭐ **The measurement changed the question.** `migrations/0023_resource_references.sql:21` declares `UNIQUE (original_locator, expected_digest)` — two tenants citing the same URL at the same digest **share one row by construction**, and `snapshot_objects` is keyed by `digest` alone. The evidence chain is content-addressed BY DESIGN (ADR-011); a `tenant_id` column would break the constraint or duplicate identical bytes, making the store worse.
- ⭐ **And ROADMAP §16.8 does not ask for one**: it says tenant id is part of every *aggregate key* and every *authorization DECISION*. A snapshot is not an aggregate — it is an immutable receipt over shared bytes — and what is missing is the decision to disclose it. ⛔ Reading the sentence as "every table" would have produced twelve unnecessary migrations.
- **The verdict: none of the twelve gains a column.** Three gain a tenant-bound READ (`.11.14.1`); the six `evaluation_*` gain a site-operator grant gate on the shape `2026-09-09_site-operator-authority.md` already describes; `deployment_targets` is operator infrastructure; `policy_publications` and `deployment_assignments` DEFER to `.6.1.5` by name.
- 🔴 **The live path, measured while deciding:** `snapshots::stale` is `SELECT … FROM evidence_snapshots WHERE deleted_at IS NULL AND fresh_until IS NOT NULL AND fresh_until < $1 ORDER BY fresh_until` — no tenant predicate, no principal, no limit — and `GET /v1/snapshots/stale` admits on enrolment alone. A row carries `original_locator`, `auth_class` and `provider_receipt`, so tenant A learns which documents B acquired, when and under which credential class.
- ⚠️ **Real width:** a DISCLOSURE path, not a write path, and an ENUMERATION rather than an oracle for a known id — which makes it worse than `.3.5`'s node-token finding, where the ids were unguessable. ⛔ Whether the acquired BYTES are reachable by another route is NOT measured and is not claimed.
- ⛔ **The repair is not a column** — and the second half of this line was REFUTED by `.11.14.1`'s measurement, corrected here rather than left standing: the reference cannot carry the citing tenant either. What holds is the rest of it — `.6.1.5`'s trap applies in full, so the read binding and the write record are one repair, which is exactly why the citation is written on the replay.

🔴 **A SPLIT IS DRAWN FROM A GOAL LINE AND NOTHING CHECKED IT COVERED ONE — MEASURED, GATE DECLINED, AND THE ONE TABLE WRITTEN FOUND TWO LIVE DEFECTS (`.11.15`, REPAIR-0212).**

- **The census**, `python3 -B scripts/census_split_coverage.py`: of **299** leaves, **14** declare a split, and **1** carried a mechanism-to-child mapping. Two of the fourteen had already dropped a mechanism their goal line names — `.3.4` two, found eleven commits later; `.7.2` one, found three commits later, and it was a live credential leak.
- ⛔ **BOTH candidate gates are measured unsound.** Requiring the mapping fires on **13 of 14** — `.11.9`'s rejected shape, "a backlog wearing a gate's clothes". Comparing a prose child-count to the real one produces false positives by construction: `.11.9`'s own section says "three children" and has one, because the sentence is about another leaf's three. ⭐ Counting the mechanisms IN a goal line is not mechanizable at all — the goal lines use semicolons, slashes and "and" interchangeably.
- ⭐ **What ships is `.11.8`'s answer, not `.11.9`'s**: the population is 13, small enough to track. A tracked, self-tested instrument inside `SELF-TEST`'s population, plus a `TOOLBOX.md` statement — never a gate.
- 🔴 **The method paid on first use, which is the whole argument.** Writing ONE of the thirteen tables — `.7.2`'s — found `max_time` declared in `GitLimits`, defaulted to 120 s and **read by nothing**, while R0's identically-named field is enforced at `fetcher.rs:648`; and three ref-verification clauses owned by the lane with no child. New owners `.7.2.7` and `.7.2.8`.
- 🔴 **The instrument was wrong on its first run and a second route caught it.** It reported 0 of 14; `grep -c` returned 1, because `.3.4`'s table is INDENTED and the pattern anchored at the line start. ⚠️ A second number was a substring artefact the same way — `grep -c "ratio" git.rs` returned 10, all `configu-ratio-n`; word-bounded it is 0. Both are kept rather than edited away.
- ⚠️ **And the practice was living in the wrong layer**: the enumeration that found `.3.4`'s gap is in `LIVE_STATUS.md`, where no instrument derives it and no reader of the leaf meets it. The table belongs in the leaf.

✅ **EVERY `rb-server` CONFIGURATION REFUSAL NOW HAPPENS BEFORE THE FIRST MUTATION, AND THE BIND IS A DOCUMENTED CHOICE REPORTED AT BOOT (`.11.12`, REPAIR-0211).** The leaf's two halves were decided SEPARATELY, as its own text required, and they did not come out the same way.

- **Half 1 — the ordering — was never a decision.** `sqlx::migrate!` ran before `SecretStore::resolve` and before the address parse, both pure functions of the arguments. A typo'd profile against the wrong database left that database migrated and no service running: a refusal that has already acted is not a refusal.
- ⭐ **The instrument needs no PostgreSQL.** Point the server at a port nothing serves: a boot that reaches the connection reports `PoolTimedOut`, a boot that refuses first reports the configuration it refused. One knob — which argument is wrong — decides which message appears, with no schema read-back and no port bound. ⚠️ Stated at its real width: this proves the refusal precedes the CONNECT, strictly earlier than the migration the finding named, and the test module's own header says so.
- 🔴 **The first draft of those controls was weaker than it read, and only the neutralization found it.** They asserted the output did not contain `127.0.0.1:1`; the unrepaired boot reports `Error: PoolTimedOut`, which carries no address, so the bind-address control PASSED against the defect it exists to catch. ⛔ **A control's discriminator has to be the string the failure actually prints.**
- **Half 2 — the bind — is REPORTED, not refused.** `docs/book/src/deployment.md` documents `rb-server --host 0.0.0.0` as the supported trusted-LAN profile, walked by `deploy/`'s runbook and the two-host demonstration; a gate would refuse a shipped capability. ⛔ And it would be the wrong instrument: what bounds Internet exposure is G6/G7 and blockers **B1–B3**, none of which is a property of an argument.
- ⭐ **What WAS wrong is that the startup line could not tell the two apart** — `(Phase 0 dev profile)` for every bind. It now names its reach (`loopback` / `one named interface` / `every interface`), classified by a pure function with a control over six addresses including both IPv6 forms. Decision, three rejected alternatives and non-claims: `docs/decisions/2026-09-16_rb-server-bind-exposure.md`.
- ⚠️ **A third defect the leaf did not name, repaired because the leaf is about refusals being useful:** a typo'd `--host` reported `Error: AddrParseError(Socket)` — the debug form of `Box<dyn Error>`, with no argument and no value. It now names both.
- **Verified:** 3 passed / 0 failed in the new boot suite; **112 passed / 0 failed / 1 ignored** in the server lib; clippy `-D warnings`, fmt, gate, book and links all rc=0. **Falsified** self-reversing, restored byte-identical.

🔴 **THE R3 PACK ADVERTISED TWO DENY-POLICIES IT DID NOT ENFORCE, AND A REAL BROWSER PROVED IT (`.7.3.5`, REPAIR-0210).**

- ⛔ **This was never "a control is missing".** `resolvers.rs::gated_advertises` publishes the R3 pack to every caller with `redirect_policy: "deny"` and `subresource_policy: "deny"` — two lines a caller reads when CHOOSING a pack — and the worker enforced neither. ⭐ `a-claim-of-sameness-is-worth-its-call-graph` in a REGISTRY rather than a module header, which is worse: a header is read by maintainers, an advertisement by callers.
- 🔴 **Reproduced under the pinned Chrome for Testing runtime, on the ORIGIN's own counter**: the page's `<img>` named `0.0.0.0`, the browser dialed it and the origin served it — `left: 1, right: 0`.
- ⭐ **The instrument is `0.0.0.0` again, its second use in this tree.** It classifies `reserved` and the kernel routes it at the LOCAL host, so the same origin answers the subresource and its counter reports whether anything reached a SOCKET. ⛔ A control asserting on the network log would have recorded the browser's INTENT — that log is written from `EventRequestWillBeSent`, which fires whether or not a dial happens.
- **The repair enforces exactly what is advertised**, at the CDP `Fetch` domain: a `Document` request for a URL the caller asked to navigate to continues, everything else fails with `BlockedByClient`. A non-document request IS a subresource; a document request for a URL nobody asked for IS a redirect.
- **Every refusal is NAMED on the receipt** — `refused_requests` carries the URL, the policy and Chrome's resource type — while the network log still records the attempt, so the disclosure and the refusal stay two separate facts.
- 🔴 **The behavioural consequence is stated rather than discovered:** a page that assembles its visible text from an external stylesheet or script now renders LESS text than it would in a desktop browser. That is what "deny" means, and it is the posture the pack advertises for untrusted content. ⚠️ Changing the ADVERTISEMENT to `allow` was the alternative and is rejected — it would publish a weaker guarantee than §12.3–12.4's posture.
- **The interception task is owned like every other**: `BrowserOwner::intercept` joins under the same deadline as `handler` and `network`, feeds `cleanup_confirmed` and is aborted in `Drop` — `.7.3.1`/`.7.3.2` own exactly this class of escape.
- ⭐ **Falsified twice with a real browser.** Removing the subresource arm reproduces the red; the blackout almost-fix, refusing EVERY request including the navigation, fails with `net::ERR_BLOCKED_BY_CLIENT` — so the control discriminates a policy from a prohibition.
- **Verified:** **18 passed / 0 failed** across the whole browser suite with every pre-existing control unchanged; 111 passed / 0 failed / 1 ignored in the server lib; 8 passed in the worker's own units. Clippy `-D warnings`, fmt, gate, book and links all rc=0.

🔴 **A CREDENTIAL BOUND TO ONE ORIGIN WAS BEING DELIVERED TO ANOTHER, AND THE REPAIR LANDS ONE COMMIT AFTER THE CLASSIFICATION FOUND IT (`.7.2.6`, REPAIR-0209).**

- 🔴 **Reproduced at runtime, and the secret ARRIVED** — this stopped being a source measurement. The origin's own record against the unrepaired loop is `[("fetch.test", true), ("second.test", true)]`: the `Authorization` header was delivered to a host the caller never named and the broker never bound.
- ⭐ **The instrument is two host names on one listener.** An origin is (scheme, host, port), so `fetch.test` and `second.test` both resolving to `127.0.0.1:{port}` give a genuine cross-origin hop with no second socket and nothing else changed between the two controls. ⛔ The log records the `Host` header and whether `Authorization` was PRESENT — never its value.
- **The rule, decided rather than inherited:** the credential is bound to the origin the CALLER named and is dropped on a hop that leaves it, while the hop is still FOLLOWED. It has already been re-hardened and re-classified, so it is a legitimate address; what must not cross is the secret. An acquisition that redirects to a signed URL on a CDN still succeeds, carrying none — which is how browsers and `curl --location` behave. ⛔ Refusing the hop outright was rejected: it fails closed the same way and breaks the shape the pack exists for. ⚠️ A second origin that DOES need the credential now answers `401`, a named status rather than a silence.
- **The disclosure became a measurement.** `FetchedDocument::credential_hosts` records the distinct hosts the header was attached to, at the moment of attachment; the snapshot's disclosure policy carries the list. It used to name `final_url.host_str()` — whichever host the redirect chain happened to end on.
- ⭐ **Falsified twice, and the second neutralization is the one that said something.** Removing the guard reproduces the red; the ALMOST-FIX `chain.len() == 1` — "only the first dial carries it", headline satisfied, anchor wrong — passes the cross-origin control and FAILS the same-origin one. Complementary failures, so the two controls bound different halves. `docs/knowledge/neutralize-into-the-fix-you-almost-wrote.md` was written two commits earlier and paid here.
- ⚠️ **Not claimed:** the rule covers the one header `fetch_authenticated` supplies. `fetch`, `fetch_head` and `fetch_admitting` pass none, and R1's Git transport has its own client this repair does not touch.
- **Verified:** **111 passed / 0 failed / 1 ignored**; the fetcher suite 21 passed with every pre-existing R0 control unchanged. Clippy `-D warnings`, `cargo fmt --check`, `make gate`, `mdbook build` and the link check all rc=0.

🔴 **TRANCHE 4C RECONCILED, AND IT CORRECTS THIS SESSION'S OWN CLOSURE CLAIM (`.11.9.1.3.3`, DOC-0027).** 26 clauses across six records: 9 `handled`, 10 `owned`, 5 `attach` (all attached in the same commit), 2 `unowned` (both given a leaf).

- ⛔ **The superseded claim, named rather than edited away.** REPAIR-0208 published "`SIGNOFF-REPAIR.7.2` is now closed on every child". That is true of the three children DOC-0026's split produced and FALSE of the goal line they were drawn from, which reads "Validate destinations **and credential forwarding** at every redirect and dial". The split carried a redirect policy, a content predicate and a repository open. It did not carry credential forwarding.
- 🔴 **Tracing the dropped mechanism found a live defect: a credential bound to one origin follows the redirect to another.** `fetcher.rs::fetch_with` runs a MANUAL redirect loop and re-attaches `extra_header` on every iteration; `fetch_authenticated`'s only production caller is the R5 arm at `api.rs:2182`, which passes a broker-resolved credential. So the secret is sent to whatever host the origin redirects to — and the disclosure is written against `final_url.host_str()`, the LAST hop, so a credential sent to three hosts is disclosed as having reached one. ⚠️ **Not an SSRF**: the hop IS re-hardened and re-classified, so the destination is a legitimate public address. What crosses is the credential. ⚠️ Source-measured; runtime reproduction PENDING, and the new leaf `.7.2.6` says reproduce first.
- ⭐ **The lesson is this project's own standing warning — count a goal line's mechanisms against the children a split produces — and it cost ONE COMMIT to relearn**, on the lane whose parent records it. `.3.4` dropped two mechanisms and tranche 4b caught it; `.7.2` dropped one and tranche 4c caught it three commits after the split. ⛔ Nothing mechanical compares a goal line's conjuncts to a split's children, and `.11.11` already owns the observation that an `attach` row's next action is the one thing the ledger cannot check.
- 🔴 **`.7.3` carries the same shape twice.** Its goal line names "enforce browser redirect/subresource isolation **and credential origin binding**" and its four children carry neither. The first is live: the R3 pack pre-flights ONE url and hands the page to a browser that subscribes to every request purely to LOG it — no `Fetch.enable`, no `setBlockedURLs`, no classification — while R0 and R1 classify every hop. ⭐ R3 is the only pack that executes the page's own JavaScript, so it is the one whose destinations the caller does not choose. New owner `.7.3.5`. ⛔ The second is a population of ZERO — the browser carries no credential at all — and deliberately gets NO leaf.
- 🔴 **The site-global data model is a design position nobody has taken.** Twelve of twelve policy, evaluation, deployment and evidence tables carry zero `tenant_id` columns. `.6.1.5` owned exactly one of those questions and owns it as a DECISION rather than a filter; nothing owned the other eleven. New owner `.11.14`, whose acceptance is one record covering all twelve — answering them one surface at a time is how a data model gets decided by accident.
- ⭐ **`R-36-39-2` is wholly `handled` and `git show` at the review baseline is what proved it STALE rather than FALSE**: `9c2d2ba` loads the boundary `WHERE tenant_id = $1 AND status = 'active'` and the grant by `ORDER BY valid_from DESC LIMIT 1`, both verbatim as the record says. At HEAD the boundary is the grant's own and the loader pages every candidate.
- ⚠️ **One clause narrowed, one requirement met, one hedge preserved.** The calibration's caller-supplied Brier IS range-checked — the defect is that nothing derives it. `R-86-1`'s "new operator CLI must never echo credential URLs" is MET by `rb-site` (`hide_env_values = true`, no URL in any print) while `scripts/backup.sh:18` still echoes `$DATABASE_URL`. And the evidence cascade is a LATENT invariant: `git grep` finds no delete verb, exactly as the record hedged.

⛔ **[CORRECTED BY DOC-0027 — `.7.2` IS *NOT* CLOSED: its goal line names credential forwarding and the split dropped it; see the entry above and `.7.2.6`.]** ✅ **THE ACQUISITION LANE'S THREE SPLIT CHILDREN ARE CLOSED, AND THE LAST ONE WAS THE SUITE ITSELF (`.7.2.5`, REPAIR-0208).** `.7.2.4` removed `Permissions::all()` from the production open; the FIXTURES had kept it, so the controls that prove the repair built their own inputs out of the host's system, application and user git configuration.

- ⭐ **The reproduction cost one line**, because `.7.2.4` left an instrument behind. The same child process printed `acquisition HEAD: ref: refs/heads/main` beside `fixture HEAD: ref: refs/heads/smuggled` — one process, one environment, production refusing what the suite accepted.
- ⚠️ **No assertion had moved, and the leaf says so rather than inflating it.** The fixtures assert commit ids, file counts, depths and refusal names, and `acquire_local` resolves through the advertised `HEAD`. What was wrong is that the suite's INPUTS varied with the host — a reproducibility defect, not a security one.
- **Fix:** one `init_fixture_repo` with `gix::open::Options::isolated()`, called by all four fixture sites.
- ⛔ **One open stays deliberately un-isolated and its exemption is in the source, not only in the tree**: the probe's own `gix::init_bare` is the matched pair's other half, and a control whose every open refuses the setting cannot tell a refusal from a setting that never arrived. The reader who would "fix" it is reading the file.
- **Verified:** **109 passed / 0 failed / 1 ignored**, the acquisition suite 12 passed / 1 ignored. **Falsified** by reverting ONE of the four sites — `source_repo` — which fails the control, so it is driven by the fixture the suites use rather than by the helper's existence. Restored byte-identical.

✅ **THE ACQUIRED REPOSITORY IS OPENED WITH GIX'S ISOLATED PERMISSIONS, AND `.7.2` IS NOW CLOSED ON ALL THREE OF ITS SPLIT CHILDREN (`.7.2.4`, REPAIR-0207).** R1 called `gix::init_bare`, which is `ThreadSafeRepository::init(…, open::Options::default_for_level(Trust::Full))` — `Permissions::all()` — while fetching a CALLER-SUPPLIED URL.

- **The census was owed before the fix, and it came out of gix 0.87.1's own source.** `Permissions::all()` admitted the system config (`$(prefix)/etc/gitconfig`), the application config (`$XDG_CONFIG_HOME/git/config`, else `$HOME/.config/git/config`), the user config (`~/.gitconfig`), environment-sourced config (`GIT_CONFIG_COUNT`/`GIT_CONFIG_KEY_n`), `include`/`includeIf` directives reaching outside the repository, the system and application `gitattributes`, and every `GIT_*`/`SSH_*` category. ⛔ The `git` binary's own configuration was off in BOTH modes and is not claimed as something the repair turned off.
- 🔴 **Reproduced at runtime first.** With the opener reverted, the probe printed `default-permissions HEAD: ref: refs/heads/smuggled` AND `acquisition HEAD: ref: refs/heads/smuggled`: a git setting supplied purely through the server process's ENVIRONMENT reached the repository the acquisition creates.
- ⭐ **The instrument is a child process, and choosing it is the design decision.** `GIT_CONFIG_*` cannot be set in-process — `libtest` runs the binary's tests in parallel threads, which is why Rust makes `set_var` unsafe — so the control re-executes its own test binary with the setting in `Command::env`. The probe is `#[ignore]`d with a reason naming its parent, so the skip explains itself instead of reading like a pass.
- ⭐ **It is a matched pair in one process**: one repository opened the superseded way, one the way production does, under the same environment. The first must say `smuggled` — proving the setting really arrives on this host — and the second `main`. ⛔ Without that half, a repair that broke the environment plumbing would look identical to one that refused it.
- ⛔ **This is what `.7.2.1` could not reach.** That leaf owned the workspace DIRECTORY; owning where gix WRITES says nothing about what gix READS, and one record sentence carried both.
- ⚠️ **Not claimed:** the transport is a custom `gix_transport` client, so `http.*` configuration never reached the dial and this repair does not change that.
- 🔎 **Routed rather than absorbed:** the acquisition FIXTURES still build their inputs with `gix::init` under `Permissions::all()`, so the suite's own inputs vary with the operator's `~/.gitconfig`. ⛔ A reproducibility defect, not a security one — new owner `.7.2.5`, with acceptance.
- **Verified:** **109 passed / 0 failed / 1 ignored**, the twelve acquisition controls unchanged; clippy `-D warnings`, `cargo fmt --check`, `make gate`, `mdbook build` and the link check all rc=0. **Falsified** self-reversing, restored byte-identical by `cmp -s`.

✅ **THE GIT LFS GATE IS ANCHORED TO THE POINTER'S OWN GRAMMAR, AND §12.5's "EXPLICIT GIT LFS POLICY" IS NOW STATED (`.7.2.3`, REPAIR-0206).** The gate refused a blob when the ten-byte run `version ht` appeared ANYWHERE in its first 64 bytes. A real pointer carries `version https://git-lfs.github.com/spec/v1` at offset **0**, so the predicate was both too broad and unanchored — and a repository that merely DOCUMENTED Git LFS could not be acquired at all.

- 🔴 **Reproduced before the repair, with two controls red.** A documentation file whose first line quotes the version line in backticks, and a manifest for an unrelated specification (`version https://example.org/manifest/v1`), were both `Refused { what: "Git LFS" }` — `test result: FAILED. 9 passed; 2 failed`.
- ⚠️ **The direction of the defect is stated rather than dressed up:** it OVER-refused. An availability and correctness fault in the acquisition, never a way past the gate, and no bypass claim is made here because none was measured.
- **The repair** is a 42-byte prefix test at offset 0 which must end at a newline — or at the end of a blob carrying nothing else — because the specification makes the `version` line the pointer's FIRST line. ⛔ `oid` and `size` are deliberately NOT required: a truncated pointer is still not the content, and requiring them would break the clause demanding the bare version line refuse.
- **The policy, decided rather than inherited: REFUSE.** A pointer's bytes were never transferred by the clone, so they were never classified by the destination policy and never counted against the object, file or decompressed-size budgets; accepting one would report a stand-in as content. The rejected alternative — a declared passthrough — loses because §12.5's manifest is of *included/excluded* content and a pointer accepted as a file is neither.
- ⭐ **Falsified in two directions, because the repair has two halves.** Restoring the search fails both new controls (`9 passed; 2 failed`); replacing it with an anchored-but-not-spec-matched `starts_with(b"version ht")` passes the prose control and fails the other (`10 passed; 1 failed`). A repair that got only the anchoring right does not survive.
- ⚠️ **The acceptance's own number was wrong and is corrected, not quietly used:** it said "the exact 41-byte pointer prefix"; the version line measures **42** bytes, 43 with its newline.
- ⚠️ **Two residuals recorded rather than absorbed:** git-lfs's legacy `hawser`/`git-media` version URLs are not matched, so a pointer from a pre-2015 client would pass through as a text file — ⛔ a question to MEASURE against git-lfs's source, since widening a predicate without measuring is how the over-refusal arrived; and the refusal detail names the file rather than its path.
- **Verified:** `cargo test -p reasonbraid-server --lib --locked` → **108 passed / 0 failed**, with the pre-existing pointer fixture unchanged; clippy `-D warnings`, `cargo fmt --check`, `make gate`, `mdbook build` and the book link check all rc=0. The policy is in `docs/book/src/deployment.md`.

✅ **THE LAST §16.12 LINE THAT WAS STILL AN OPEN DEFECT IS REPAIRED, AND IT WAS REPRODUCED FIRST (`.7.2.2`, REPAIR-0205).** R1 built its client with `Policy::limited(5)` and a DNS belt, and hyper-util 0.1.20 skips the resolver when the host is already an IP address — in its own source — while `GitFetcher::classify` runs ONCE, on the initial URL. So an IP-literal redirect hop reached no destination control at all.

- 🔴 **The unrepaired client did not merely fail to refuse — it DIALED and was ANSWERED.** With `Policy::limited(5)` restored the control fails with `the IP-literal hop must be refused by name: Ok(200)`: the redirect to `http://0.0.0.0:{port}/private` was followed and the refused destination served the request.
- ⭐ **The instrument is `0.0.0.0`, and choosing it is most of the work.** It classifies as `reserved`, and the kernel routes a connection to it at the LOCAL HOST — measured with a two-socket probe before the control was written — so the same origin answers and its hit counter reports whether the hop was actually dialed. ⛔ A control that only asserted "an error happened" could not tell a classification apart from a connection that failed. ⚠️ `127.0.0.2` was the obvious first choice and does **not** bind on this macOS host (`Errno 49`), measured rather than assumed.
- **The repair** puts the pre-flight's own policy inside the redirect decision, re-states the five-hop cap that `Policy::limited` used to own, and carries a typed refusal out of band — reqwest's policy can answer only *follow*, *stop* or *error*. The slot is CLEARED before each request, because it belongs to a client that outlives one.
- **The belt can now say what it refused:** `ClassifiedDns::resolve` used `addrs.retain(...)`, so a host whose every address is refused produced a bare connect failure. ⚠️ Never a safety gap — it failed closed — and it is written up as diagnosability, not as a vulnerability.
- ✅ **The R1 module header is TRUE again** and now names the two mechanisms that make it so, quoting hyper-util's own sentence. It had claimed R0's property while its call graph did not perform it.
- ⚠️ **Two residuals recorded rather than absorbed:** the POST path is protected by the same policy but has no error channel for a refusal, and an `https → http` redirect hop is still unrefused.
- **Verified:** `cargo test -p reasonbraid-server --lib --locked` → **105 passed / 0 failed**, every pre-existing R0/R1 destination control unchanged. **FALSIFIED** self-reversing, restored byte-for-byte by `cmp -s`. ⭐ A second control, `an_allowed_redirect_hop_is_still_followed`, keeps this a classification rather than a prohibition.

🔴 **THE ACQUISITION LANE WAS CARRYING THREE DIFFERENT REPAIRS UNDER ONE GOAL LINE, AND IS NOW SPLIT (`.7.2`, DOC-0026).** `.7.2` held one `- Status: pending.`, no `- Acceptance:` of its own, and three repairs that share nothing but a subsystem: a client's redirect policy, a content predicate over a blob's first 64 bytes, and how a repository is opened. Two of the three are `attach` clauses the goal line does not make visible at all. ⛔ `.11.13`'s rule is that a leaf without acceptance is not an owner, so each child carries one.

- **`.7.2.2` — classify every redirect hop, including an IP literal.** `Policy::limited(5)` auto-follows up to five hops with only the DNS belt behind them, and hyper-util 0.1.20 skips the resolver when the host is already an IP address, in its own comment. The pre-flight classifies the FIRST destination only.
- **`.7.2.3` — the LFS gate refuses on a ten-byte run, not on a pointer.** `head.windows(10).any(|w| w == b"version ht")` over the first 64 bytes, where a real pointer is that line at offset 0. ⚠️ It OVER-refuses: an availability defect, not a bypass. ✅ **DONE (REPAIR-0206)** — anchored at offset 0, reproduced first, policy stated in the book.
- **`.7.2.4` — the repository is opened with gix's DEFAULT permissions over an untrusted remote**, while gix 0.87.1 ships the named remedy it does not use, `open::Options::isolated()`. ✅ **DONE (REPAIR-0207)** — censused against gix's own source, reproduced with a child-process matched pair, and the fixture-side twin routed to `.7.2.5`.
- ⭐ **The census NARROWED the published finding**: a hostname redirect hop IS covered, because hyper-util resolves it through `ClassifiedDns`, whose `resolve` retains only policy-allowed addresses. The uncovered destination is precisely an IP-LITERAL hop, and the leaf says so rather than keeping the wider claim.
- ⚠️ **And it turned up one nobody had recorded:** `ClassifiedDns::resolve` uses `addrs.retain(...)` rather than erroring, so a hostname resolving only into a refused range yields an EMPTY address list and a generic connect failure instead of a named `DestinationRefused`. It fails closed — diagnosability, not safety — and is owned at `.7.2.2`, whose repair touches the same belt.

✅ **THE DELEGATION LANE IS CLOSED, AND THREE OF ITS SIX VERDICTS ARE NOT "DONE" (`.3.4`, DOC-0025).** Eight children, reconciled against the leaf's own GOAL LINE rather than against their statuses — the discipline `.3.4.3.1` had to learn one commit earlier, where a third of its acceptance turned out to be unmeetable.

| Mechanism the goal line names | Carried by | Verdict |
| --- | --- | --- |
| enforce delegability | `.3.4.1` | ⛔ **REFUSED as a gate** — the flag governs grant chains that have no producer |
| enforce bounded depth | `.3.4.1` | ⛔ **Deliberately unenforced** over a population of zero, and said so in the book |
| enforce participation | `.3.4.1` | ✅ Enforced, by a gate the census had not looked for |
| enforce consent | `.3.4.7` | ⛔ **Not a requirement** — §16.3 states six invariants and none is consent |
| bind the hash to the authority context | `.3.4.2` | ✅ Bound |
| bind the hash to the target | `.3.4.6` | ✅ Bound |
| preserve committed-replay semantics | `.3.4.2` + `.3.4.6` | ✅ Asserted in the SAME control as each binding |
| make cached expiry and future-clock behaviour explicit | `.3.4.3` + `.3.4.3.1` | ✅ Explicit; one clause recorded UNMET and superseded |

- ⚠️ **Three residuals carried forward by name rather than absorbed into a closure:** delegation CHAINS and their depth bound have no producer and the wire cannot carry one (⛔ do not "fix" `delegable`/`max_delegation_depth`); `lease_expires_at` is received on the handshake and the heartbeat and **never read**; and a delegation's ATTRIBUTION claim names a subject that never agreed in an authorization record §16.9 makes high-impact evidence.
- ⛔ **The split bullet's wrong count is kept verbatim.** It says "five children along the **four** mechanisms the goal line names"; the goal line names five. That sentence is what hid two mechanisms for eleven commits, and correcting it in place would hide the hiding.

⛔ **A CONTAINER LEAF'S CLOSURE IS ITS OWN ACCEPTANCE, CLAUSE BY CLAUSE — AND ONE OF THESE COULD NOT BE MET (`.3.4.3.1`, DOC-0024).** The leaf read `active` while all three children read `done` (REPAIR-0136 / 0138 / 0139), which is `.11.4.5.3`'s finding running the other way: the tree over-reporting remaining work. ⛔ Closing it was never bookkeeping. Its clause *"the `.3.4.3` backward-skew controls pass unchanged"* is **UNMET**: both controls assert the receipt-anchoring clamp that `.3.4.3.1.2` removes, and both drove a state production cannot reach — a known revocation epoch with no clock offset, when the two arrive in the same response. That supersession lived inside the child and is now recorded at the parent whose acceptance it belongs to.

- ⭐ **The withdrawn property is STRONGER now**, which is the only thing that makes the supersession honest: "one TTL of real time whatever this node's clock says" is asserted in BOTH skew directions, and the clamp could only do it for one.
- **Clause 2's margin is stated rather than implied:** `a_node_clock_ahead_of_the_server_still_dispatches` drives the node **600 s** ahead against a **60 s** TTL — ten times the threshold that refused every dispatch — and the gate's allow is proved by a CHANNEL error rather than by a success, because a refusal in that path returns `Ok` and asserting "no error" would have passed on the defect itself.
- ⚠️ **One residual carried forward, not closed:** `lease_expires_at` is received on the handshake and the heartbeat and **never read** — a latent third cross-clock comparison the day someone uses it.
- **Verified:** `cargo test -p reasonbraid-node --locked --no-fail-fast` → **14 suites, 78 passed, 0 failed, 2 ignored**, rc=0, with all four clock controls passing by name. ⭐ `.3.4` is now closable on its own acceptance — every child `done`, and both mechanisms its split had dropped carried.

⛔ **CONSENT WAS NEVER A DELEGATION REQUIREMENT IN THIS PROJECT, AND THE LEAF THAT ASKED FOUND A SHARPER ANSWER THAN THE QUESTION (`.3.4.7`, DOC-0023).** `.3.4`'s goal line names "actor/subject participation **and consent**" and no child carried it. The leaf was opened reading that as a gap ADR-009 had left. Measuring the SPECIFICATION instead of the code: `grep -ic consent ROADMAP.md` returns **8**, and `awk 'NR>=1754 && NR<=1769' ROADMAP.md | grep -ic consent` returns **0** — §16.3, where the delegation invariants live, states **six** of them and subject consent is not among them. ⭐ The roadmap's consent is **§4.4's** — an ENROLLMENT and mandate-domain act carried on the `EnrollmentAuthorityBoundary` as `target_disclosure_and_acknowledgement`, shown to the target owner before enrollment completes. So ADR-009 was silent because there was nothing to decide, not because a decision was skipped.

- **The answer, in one sentence, now in ADR-009 and the book:** a delegation here is **trusted impersonation inside one tenant**, the subject is not asked and cannot refuse, and the consequence is bounded to **attribution**.
- **The bound is RE-DERIVED at HEAD rather than inherited** (`.9.3.4`'s ruling — verify the premise a decision turns on): `authority.rs::authorize_in_tx` builds a caller authz with `delegation: None` and the SAME action and target, and a denial there replaces the outcome with `the caller's own authority failed`. ⭐ Each of the four gates is pinned by a NAMED control, so **no new control was added for appearance** and the leaf says so.
- ⚠️ **The cost is accepted, not dismissed:** an authorization record — §16.9 evidence — names a principal as the authority behind an act it never agreed to. The alternative is the capability-token machinery ADR-009 already subtracted, because a subject cannot express consent without something to issue and something to verify.
- **The revisit trigger is mechanical** (`.13.1.2`'s lesson): a delegation crossing a tenant boundary, a subject that is not an enrolled principal of the same tenant, or any relaxation of the four gates — each a failing control rather than a judgement call.
- ⭐ **Both mechanisms `.3.4`'s own split dropped are now carried**, and its split bullet still says "five children along the four mechanisms" while the goal line names five. The wrong count is KEPT rather than corrected in place, because the sentence is what hid the gap. ⚠️ `.3.4` still cannot close: `.3.4.3.1` reads `active` with all three children `done` — the same shape, running the other way — and closing it is that leaf's commit, now frontier row 1a.

🔴 **THE REPLAY HASH DID NOT BIND THE COMMAND'S TARGET, AND THE REPRODUCTION IS WORSE THAN THE PROSE (`.3.4.6`, REPAIR-0204).** `request_hash` covered the operation, the actor, the body and the authority context. A thread command's thread arrives as a PATH segment and no typed body carries it, while `idempotency` is keyed on `(tenant_id, idempotency_key)` — tenant-wide. ⛔ Measured before repairing: the same actor, body and key against a **different thread in the same tenant** answered `200` with `"replayed":true` and returned the FIRST thread's `thread_id` and `event_id`. **A caller addressed one thread and was handed another thread's event as its own result**, with thread two never looked at, because the idempotency claim is step 1 and authorization is step 2.

- **Fix:** `request_hash` takes a `target`, appended as `\ntarget={id}` when present; all **eleven** operations of `POST /v1/threads/{thread_id}/commands` pass the path's thread.
- ⭐ **The target is bound exactly where a caller can vary it independently of the key.** Three callers pass none, each with a structural reason rather than an omission: a **creation** has no thread and its target is the tenant, already the idempotency primary key's first column; the **MCP respond tool** derives its key as `mcp_respond_{thread}_{hash}`, so the thread is fixed inside the key — and binding it there would change the KEY, turning an old call's replay into a **duplicate contribution**; a **node result** is keyed by the server's `command_id`, which belongs to one thread.
- ⚠️ **The migration answer is NOT `.3.4.2`'s, and the leaf says which keys break.** Every historical key for those eleven operations now CONFLICTS instead of replaying — the safe direction, a refusal rather than a result decided for a different target. The other three surfaces hash byte-identically and keep replaying.
- **Verified**: `command_api command_ordering escalation mcp_write atomic_transaction invitations` → **6 suites, 66 tests, zero failures** (39 + 7 + 4 + 5 + 5 + 6), rc=0. **FALSIFIED** against the exact unrepaired sources at `HEAD`: `38 passed / 1 failed`. ⭐ The same control asserts the committed-replay contract, so a hash that merely stopped matching would fail it.

✅ **THE WORKSPACE IS GREEN IN ONE COMPLETED RUN (`.11.4.8`, REPAIR-0203), AND THE ONE RED TEST WAS NEVER A PRODUCT DEFECT (`.11.4.7.2.4`, REPAIR-0202).** The single failure in 94 test binaries — `a_real_navigation_deadline_stops_the_browser_and_origin` — was a control qualified against the PINNED Chrome for Testing runtime being evaluated against whatever browser the host had installed. 🔴 **The cause is one line of test support**: `browser_binary()` carried a discovery list, so any run that did not go through `scripts/ci_browser.py` — which is exactly what `cargo test --all` is — silently selected `/Applications/Google Chrome.app/…`. ⭐ **A pin a second path can bypass is not a pin.** ⛔ The book had already said *"Direct Cargo commands bypass this setup and do not establish pinned-runtime coverage by themselves"*; the sentence was correct and toothless, and the fallback made a bypassed run look identical to a qualified one.

- **Measured, one variable, same machine and the same competing load**: desktop Google Chrome 152.0.7977.83 → cleanup unconfirmed, **failed 4 of 4**, worker elapsed **41.07 s**; pinned Chrome for Testing 153.0.8010.36 → cleanup confirmed, **passed 2 of 2**, elapsed **31.13 s**. ⭐ The ten-second difference is the worker's entire 10 s cleanup budget spent waiting for an end-of-file that cannot arrive.
- **PINPOINTED at the file descriptor:** `lsof` prints the worker's fd 11 and `chrome_crashpad_handler`'s fd 2 as the two ends of one pipe, with that handler at `PPID 1` in a process group the worker never owned — it left the owned group and kept the inherited stderr write end, outliving the worker by **6.24 s**.
- 🔴 **The pinned runtime escapes IDENTICALLY** — two handlers, `PPID 1`, foreign process groups, the same pipe — and differs only in exit latency. ⛔ So this control's green is a latency property of a third-party process, **not** containment evidence; annotated at `.7.3.2`, which owns detached-descendant containment, with a number: two escaped processes per launch, on both runtimes.
- ⛔ **NO PRODUCT CODE CHANGED, deliberately.** The worker was telling the truth on every run, and `a_render_refusal_survives_an_unconfirmed_cleanup` pins that reading with an injected `setsid` writer. Loosening it would have meant claiming a termination nobody observed — what REPAIR-0089 refused by name.
- ⚠️ **A skip must never read like a pass, and this one nearly did.** `libtest` captures `println!`/`eprintln!` and replays them only for FAILING tests. Measured with a one-test `rustc --test` probe: a write through the `std::io::stderr()` HANDLE escapes the capture. The six skips now print on an ordinary captured run, naming what went unqualified and the command that qualifies it.
- ✅ **AND THE COMPLETED RUN NOW EXISTS (`.11.4.8`, REPAIR-0203): 103 suites, 816 passed, 0 failed, 3 ignored**, across 94 test binaries and 9 doc-test targets, rc=0, `real 30m15.757s`, with **zero** skips. ⭐ Against `.11.4.7.2.2`'s 815 passed / 1 failed on the same binaries, **exactly one test moved and nothing else did**. 🔴 The first attempt was CUT OFF at 78 of 94 by the harness's own 3600 s cap — the first timeout in 16 receipts — because that deadline was bounding a COMPILE. `make test` and `rust.yml` now run `cargo test --all --locked --no-run` before entering the harness and pass `--timeout 5400`. ⛔ **The cache was REFUSED on the number**: acquisition is 55.27 s of a 1,872 s run (**3.0%**, 93% of it the download), and a per-call hash-verified fetch is not worth trading for 52 s. ⚠️ The cold path is not re-measured; the instrument that settles it is the remote runner, which is blocker **C1**.
- ⭐ **`--no-fail-fast` is adopted** in `make test` and `.github/workflows/rust.yml` on `.11.4.7.2.2`'s census — 11 of 94 binaries against 94, at 319.3 s — because a command that answers *is the workspace green* cannot answer it from 11 binaries. A crate-scoped development run keeps its early stop. Record: `docs/decisions/2026-09-15_test-browser-runtime-is-named-not-discovered.md`; lesson: `docs/knowledge/a-pin-a-second-path-can-bypass-is-not-a-pin.md`.

🔴 **THE ONE BLOCKER HELD FOR THE DIRECTOR IS DECIDED, AND MEASUREMENT DISSOLVED IT (`SIGNOFF-REPAIR.3.5.2.1`, REPAIR-0196).** `.3.5.2.1` had been held two days over three shapes for auditing `GET /v1/admin/metrics`, each with a real cost: a required `?tenant_id=` breaks every caller, an optional one starts failing when a caller gains a second grant, and deriving the tenant from the admitting grant needs a new selection path. ⛔ **All three shared a false premise — that the route must NAME a tenant.** `migrations/0007_identity_store.sql` declares `human_principals.principal_id` and `agent_roles.role_id` as **PRIMARY KEY**, each with one `tenant_id`, so a principal belongs to exactly one tenant structurally and the tenant is DERIVED from the authenticated caller.

- **The fourth shape costs none of what the other three cost**: no wire change, no new selection path, no conditional behaviour — and **no stored-format change**, because the route uses the ORDINARY `authorize_guarded` and records `boundary_checked`, so `TenantAdminInspection` gains no variant. ⭐ That also honours `.3.5.2`'s prohibition BY CONSTRUCTION: `authorize_tenant_admin_inspection` would have applied the frozen-boundary carve-out, which exists for authority state and not for process counters.
- ⚠️ **One declared narrowing**: "any `tenant_admin` grant in ANY tenant" becomes "an administrator of your OWN tenant". Nothing binds a grant's tenant to its subject's — `R-85-1` clause 2's shape, owned at `.3.3` — but both producers create the principal in the grant's own tenant, so no reachable caller loses access. What an admin SEES is unchanged and still process-wide.
- ⭐ Deleting the hand-rolled gate removes a **seventh** spelling of the grant-liveness question `.9.3.1` unified, and the loosest: it omitted `subject_kind`, and its `expires_at IS NULL` branch was dead against a `NOT NULL` column.
- **Verified**: `command_api` 38 passed, `authority` 22 passed, rc=0, cluster removed. **FALSIFIED** against the exact superseded handler: **37 passed / 1 failed**, the one failure being the new control at `the admitted metrics read carries its admission receipt`. ⚠️ The neutralization was made SELF-REVERSING — the restore chained to the job consuming its result — because a neutralized tree is the one state a handoff must never be left in.

✅ **B5 IS CLOSED — THE LICENCE IS GRANTED (`.13.2`, REPAIR-0197).** `LICENSE-MIT` and `LICENSE-APACHE` now sit at the repository root, and the director supplied the one input that was genuinely missing: the copyright holder is **Richard DJE**. ⛔ The EXPRESSION is untouched — **13 tracked manifests** (5 literal, 8 `license.workspace = true`) already declared `MIT OR Apache-2.0`, and narrowing or broadening it here would have been a relicensing act wearing the clothes of a completion. ⚠️ **“Ten manifests”, published here and in three other places, was never measured; the census says 13.** The figure is named rather than swapped, and it survived precisely because a wrong count that changes no conclusion is the kind nobody re-checks. ⭐ **Neither text was typed.** `LICENSE-APACHE` is byte-identical (`cmp -s`) to the copy **104 crates** in this workspace's own registry ship; `LICENSE-MIT`'s body is whitespace-identical to the copy **127 crates** ship, with only the copyright line differing. A licence is a legal instrument whose operative content is its exact words, and a *plausible* paraphrase is the dangerous kind — nothing in a fluent reconstruction signals which clause drifted. ⛔ The Apache appendix keeps its `[yyyy] [name of copyright owner]` placeholder: that is part of the canonical document, not a blank for the licensor, and filling it would modify the licence. ⛔ **B4 is untouched** — a licence grants permissions in the work and says nothing about the name on it. Record: `docs/decisions/2026-09-15_licence-granted-mit-or-apache-2.md`.

🔴 **THE GAP WAS INVISIBLE TO EVERY GATE, SO THE COMMIT ADDS ONE: `LICENCE-GRANT`.** ⚠️ It is the **9th project-specific check**, not a 19th doctrine — the enforcer still prints `18 checks`, because project checks run nested inside `PROJECT-SPECIFIC`. Measured, after I first wrote “19th” from the same habit that produced “ten manifests” in the paragraph above. **487 commits of project history passed under every gate set this project has ever had, and not one of those sets contained a licence check** — `git log --oneline -- 'scripts/check_licence*'` returns nothing before this commit, and the current 18-doctrine set has only been in force for 67 of them (since `86dd272`). They governed documents, code, tables and claims, and none governed the grant. ⚠️ I first wrote “128 commits with all 18 gates green”, which is wrong twice — the branch is 127 ahead and the 18-gate set is 67 commits old. Third unmeasured number in one commit; all three are named rather than swapped. `scripts/check_licence_grant.sh` closes both directions: a declared identifier must have its text, **and a `LICENSE-*` file must be named by the declared expression**, because a file left behind after an expression changes still reads as an offer. ⭐ It fires on **zero** breaches today and would have fired on every commit before this one — catching the next drift rather than presenting a backlog.

🔴 **FALSIFYING THE NEW GATE FOUND TWO DEFECTS IN IT, AND THE SELF-TEST WAS GREEN THROUGH BOTH.** (a) Its sentinels were literal substrings, and the real MIT text is hard-wrapped at ~55 columns, so `WITHOUT WARRANTY OF ANY KIND` spans a line break — the gate **failed a perfectly valid licence**. (b) All four Apache sentinels sat in the **first five lines**, so `head -5 LICENSE-APACHE` passed while granting nothing. ⛔ **The cause of both is one thing: the fixtures were hand-written, so they were tidier than the shipped files** — nobody hand-wraps a fixture at 55 columns, and a five-line fixture cannot express the difference between a document and its title block. Fixtures are now the REAL files, mutated; sentinels span opening, numbered body and closing line, backed by a length floor; and both holes are PINNED as named self-test cases that say what deleting them restores. Seven falsification arms now fire against the working tree, each self-reversing and each proved restored by `cmp -s`. Promoted: `docs/knowledge/a-self-test-cannot-be-tidier-than-the-real-input.md`.

⭐ **A1 CLOSES IN THE SAME PASS, AND WITH IT THE LAST DIRECTOR-HELD ROW.** Its register row still read “the only item genuinely awaiting a director decision” after REPAIR-0196 had already dissolved the question — a stale row on the very table built to stop blockers going stale. ⚠️ The instructive part is not that two held rows cleared in one day; it is that **A1 sat two days with nobody testing its premise**, because “held for the director” reads like a settled state rather than like work still owed.

⭐ **B3'S DEFERRAL IS NOW MEASURED RATHER THAN INHERITED (`.13.1`).** `threads::work_payload` carries `kind`, `agent_role`, `subject`, `objective` and reservation fields — **no acquired bytes, no evidence, no derivations**; the adapters run `--tools ''` / `--sandbox read-only`; acquisition terminates in stored evidence rather than in a dispatched run. So §16.6's action boundary genuinely has nothing to guard, and the trigger that ends the deferral is a one-line grep: `work_payload` gaining a content field, `tool_support` becoming true, or model output reaching a publication, deployment, grant or tool.

⭐ **B3'S DEFERRAL TRIGGER IS NOW A SCRIPT RATHER THAN A SENTENCE (`.13.1.2`, REPAIR-0201).** `ACTION-BOUNDARY` pins the four facts the deferral rests on — `claude.rs::EXEC_ARGS` passing `--restricted` and `--tools ''`, `codex.rs::EXEC_ARGS` passing `--sandbox read-only`, `threads::work_payload` dispatching eight keys and no acquired bytes, and every `AdapterCapabilities` declaring `tool_support: false` across 5 files and 4 `Adapter` implementers. ⛔ **The deferral is NOT revisited — it is still correct.** What changed is that the day an action-bearing surface lands, the commit that lands it FAILS and says B3 is due. ⚠️ The fourth fact is pinned here because nothing enforces it at runtime (`.13.1.1`): the ladder that would refuse a tool-declaring adapter has no caller, and the only ceiling permits `tool_support: true`. A weaker guarantee honestly placed beats a stronger one imagined. Falsified five ways against the working tree, each self-reversing.

🔴 **AND REGISTERING IT EXPOSED A DEFECT IN THE PREVIOUS COMMIT — MINE.** `SECRET-SCAN`, installed one commit earlier by REPAIR-0200, fired on REPAIR-0200's own fixture. ⛔ **That leaf recorded `gitleaks detect` rc=0 as its evidence, measured against an UNCOMMITTED working tree, while the command scans HISTORY** — so it verified the state BEFORE its own change, and the value it introduced surfaced at `b6445d1` the moment it was committed. ⭐ **The irony is exact: that leaf's headline lesson was "a rename cannot reach a HISTORICAL finding, because `detect` scans every commit". The same fact cuts the other way — a history scan cannot reach an UNCOMMITTED change — and I took only the half in front of me.** ⚠️ The rename was also ineffective on its own terms: `key-delegation-1` scores entropy **3.578** against a ~3.5 threshold it was never measured against. The value is now `idem-test-000001` — still 16 characters so `baseline == 308` holds, entropy **2.899** so it trips nothing — and the fixture says BOTH properties are load-bearing. Fix: `gitleaks git --staged` as a FIRST arm (**0.03 s**), falsified by staging a high-entropy literal. ⚠️ `detect --no-git` measured and REJECTED — it walks `target/` and did not finish in 120 s. ⭐ The staged arm means the next such value is one you FIX, not one you permanently annotate.

✅ **ALL THREE RED THINGS ARE NOW GREEN, AND BOTH SUPPLY-CHAIN GATES ACTUALLY RUN (`.11.4.7.2.3`, REPAIR-0200).** `gitleaks detect --source . --redact` returns `no leaks found`, rc=0. 🔴 **A MEASUREMENT CORRECTED THE PLAN MID-LEAF:** the leaf preferred renaming the fixture over an allowlist entry, and ⛔ **a rename cannot reach a historical finding** — `detect` scans all 488 commits and attributes a finding to the commit that INTRODUCED the line, so after the rename the scan still returned rc=1 with the identical fingerprint. The exact-fingerprint entry is the only instrument that clears it, which is what `.gitleaksignore` exists for. ⚠️ The rename is kept as forward hygiene and is **exactly 16 characters**, because `delegation_representation.rs:172` asserts `baseline == 308` over an envelope containing that field — any other length would silently have moved a measurement instrument's number.

⭐ **THE GATE-PLACEMENT QUESTION IS SETTLED ON WHAT EACH CHECK IS TRIGGERED BY, NOT ON COST — and the timings turned out to be almost decoration.** A commit CAN introduce a secret, so `SECRET-SCAN` is **change-triggered** and joins the doctrine gate (1.07–1.15 s over three runs, against a 6.65 s enforcer, ~+17%). An advisory appears against code nobody touched, so `cargo deny` is **TIME-triggered** — gating it on commits is both too often and too rarely — and it goes in a new `.githooks/pre-push` (1.15 s, advisory DB cached under the repository-local `CARGO_HOME`). ⛔ Both **SKIP LOUDLY** when their tool is absent rather than failing closed, with a notice naming what was not checked: failing closed would block every contributor without the tool, and a skip is a weaker guarantee than a pass that must never READ like one. ⚠️ So **E4 (CI) is a backstop again rather than the only stop** — which is the concrete half of blocker C1 closed.

✅ **THE RUSTLS ADVISORY IS TAKEN (`.11.4.7.2.2`, REPAIR-0199).** `cargo deny check` returns `advisories ok, bans ok, licenses ok, sources ok`, rc=0. ⭐ **The lock diff is FOUR LINES and is enumerated in the leaf rather than summarised** — a lockfile bump is a supply-chain change, and "just a patch update" is a claim about a file nobody read. Nothing transitive moved; the 37 other dependencies behind latest are deliberately untouched, because the leaf owns one advisory and not a refresh. ⛔ The exposure at its real width: rustls accepted TLS 1.3 handshake messages at the wrong encryption level after a key-changing message in the same record — the transcript stays AUTHENTICATED, so it is not handshake forgery; the effect is that a peer may send in plaintext what should have been encrypted.

🔴 **AND THE REGRESSION RUN IS THE STRONGEST EVIDENCE THIS TREE HAS PRODUCED ALL SESSION.** `cargo test --all --locked --no-fail-fast` reaches **94 test binaries**: **102 suites ok / 1 failed, 815 tests passed / 1 failed**. ⭐ **The single failure in the ENTIRE workspace is `.11.4.7.2.4`'s known browser control** — so the four crates that the earlier fail-fast abort left unknown (`cli`, `core`, `node`, `server`) are in fact green. ⚠️ This does NOT make G1's unit-baseline line re-derive: the suite still does not pass, and the record's claimed 39 + 12 shape no longer exists against 76 test files. What it does is replace an UNKNOWN with a number. ⭐ It also prices `--no-fail-fast`, which `.11.4.7.2.4` asked for as a measurement: 11 binaries vs 94, at **319.3 s** of test execution (one suite is 135.23 s). The cost of knowing was run time, not extra failures.

🔴 **G1–G2'S SIXTEEN CLAIMS ARE RE-DERIVED (`.11.4.7.2`, REPAIR-0198): 3 STAND, 4 NARROW, 9 MUST BE RE-EARNED — AND THREE THINGS ARE RED RIGHT NOW.** ⛔ The verdicts are the smaller half. `cargo deny check` returns rc=1 (**RUSTSEC-2026-0285**, `rustls 0.23.43`, fixed ≥0.23.45 — DEPENDENCY drift, since rustls was not in the lock at the gate commit; positive control: `tokio` was). `gitleaks detect` returns rc=1 on one `generic-api-key` hit that is a FALSE POSITIVE — a test fixture named `idempotency_key` — and the gate is red anyway. And `cargo test --all --locked` returns **rc=101**, aborting at the 11th test binary, so `cli`, `core`, `node` and `server` never execute. Record: `docs/decisions/2026-09-15_g1g2-sixteen-claims-re-derived.md`; the original is byte-unchanged since it was written. Owners `.11.4.7.2.1`–`.4`.

🔴 **AND THE REASON THE FIRST TWO WERE INVISIBLE IS THE UNPUSHED GAP.** `grep -c 'deny\|secret-scan\|gitleaks'` returns **0** for `scripts/check_doctrines.sh`, **0** for `scripts/check_doctrines.project.sh` and **0** for `.githooks/pre-commit`. Both live only in `.github/workflows/supply-chain.yml`, which runs in REMOTE CI. ⚠️ **This sentence originally continued "— and remote CI has never run", and that cause was WRONG**; `.13.3` (REPAIR-0220) measured 41 remote runs with the supply-chain workflow **green** at `c17841c`, and `.13.4` corrects the copy here. ⭐ The conclusion survives on the reason the next sentence already gave: `origin/main` is at 2026-09-12 while the gitleaks finding entered on 2026-09-13, inside the unpushed range, so even a CI run on `origin/main` would not have caught it. Both findings post-date the last push; that, and not a never-run instrument, is why two supply-chain gates were red with nobody able to see it.

⭐ **THE FAILING TEST IS THE REPAIR WORKING, WHICH IS WHY IT IS A LEAF AND NOT A FLAKE (`.11.4.7.2.4`).** `browser_roundtrip` fails on `cleanup_confirmed == true` while the assertion directly above it — `kind == "time_budget_exceeded"` — PASSES. REPAIR-0089 deliberately split those two facts so the product would stop claiming an unobserved termination; it now reports `cleanup_confirmed: false` honestly, and the control fails on the honest answer. ⛔ Reproduced in TWO load states — shared machine `elapsed_ms` 41170, quiet machine single-threaded 40022 — so "flaky" is refused as a measurement, not as a matter of taste. ⚠️ It must NOT be repaired by loosening the assertion to accept either value: REPAIR-0089 already refused that in writing, because it would stop proving the deadline cancels a real in-flight navigation.

⛔ **THE PHASE-1 DEFERRAL COUNT IS SIX, NOT FIVE**, and the record was inconsistent the day it was written rather than drifting: `grep -c "^| [1-6] |"` returns 6, the Outcome line and the closing bullet both say five, and `git log -S` puts the sixth row in the SAME commit as the sentence. 🔴 **Deferral #5's revisit trigger FIRED and nothing revisited it** — the fuzz baseline was deferred until "the first untrusted parser (Phase 4's resource packs)"; Phase 4 closed, `fetcher.rs` / `git.rs` / `reasonbraid-extract` / `-browse` now parse untrusted input, `git ls-files | grep -ic fuzz` returns **0**, and the word appears in exactly ONE decision record — the one that deferred it — and ZERO times in Phase 4's. ⭐ **A deferral with a trigger nobody checks is an omission with extra steps** (`.11.4.7.2.1`).

🔴 **THE ADAPTER-LOAD LADDER HAS NO PRODUCTION CALLER (`.13.1.1`), AND THE DIRECTOR'S QUESTION FOUND IT RATHER THAN A CENSUS.** Asked whether a locally spawned open-weight model (Kimi, Qwen, GLM, DeepSeek) or a remote instance could join, and whether the network cares which agent type connects: **authorization is model-agnostic** — `git grep -nE "(model|provider|harness)" -- crates/reasonbraid-server/src/authority` returns **0** — and the `Adapter` contract is transport-agnostic, so all of them are legitimate implementers. ⛔ But the answer I gave first was WRONG about admission: `verify_ladder` (`PHASE-8.4.4`'s five-rung fail-closed ladder) has **no production caller** — six of its eight `git grep` hits are inside its own `#[cfg(test)]` module, and `pub use` hides it from `dead_code`. `AllowedCapabilities::dev()` is the crate's only ceiling and sets `tool_support: true`. ⚠️ Nothing unverified loads today — the adapters are compiled in — but the control is INERT for the third-party case it exists for. What actually holds the action boundary is declaration-side: every adapter declares `tool_support: false`, and the two CLI adapters pass `--restricted --tools ''` and `--sandbox read-only`.

🔴 **G6–G7'S SEVEN SHIPPED LINES ARE RE-DERIVED (`.11.4.7.1`, REPAIR-0195): THREE MUST BE RE-EARNED, TWO ARE NARROWED, TWO STAND.** ⛔ The gate's conclusion is UNCHANGED — still NOT MET for Internet exposure — and the Phase-7 record is byte-unchanged; the new record ADDS to it (`docs/decisions/2026-09-15_g6g7-shipped-lines-re-derived.md`). ⭐ "Since" is exact: the gate closed 2026-09-08, the earliest corrective repair is 2026-09-09, and comparing every repair's timestamp against the record's returns **zero** that predate it.

- **Must be re-earned:** (2) enrollment / rotation / revocation / tenant-isolation — the line names FOUR things and the corpus falsified all four; (3) non-escalation / confused-deputy — `.9.3.1` is that shape itself; (8) rate-limit / breaker / storm — `.3.3.4.9` found the breaker's two verbs mutating on the connection pool with no transaction, guard or record.
- **Narrowed:** (4) SSRF/redirect — below; (7) backup restore — the exercise runs, `R-59-2`'s fixture defects are open.
- **Stand:** (6) supply chain (narrowed only by the ledger's empty `tested_versions`); (10) runbooks and disclosure.

🔴 **LINE (4) IS THE ONLY ONE OF THE SEVEN STILL AN OPEN DEFECT, and it is a two-site contrast inside one product.** `fetcher.rs` (R0) sets `reqwest::redirect::Policy::none()` — "the manual per-hop policy owns redirects" — and re-parses and re-classifies every hop. `git.rs` (R1) sets `Policy::limited(5)`, auto-follow, with a pre-flight `classify()` on the FIRST destination and `.dns_resolver(ClassifiedDns…)` behind the hops — while its own header claims "every dial (redirect hops included) passes the destination policy". ⛔ **CORRECTED the same day by its author**: this first said the DNS hook was R1's "ONLY destination control" and that the INITIAL url was unclassified. Both overstate it — `classify()` handles an IP literal explicitly (`if let Ok(ip) = host.parse::<IpAddr>()`). ⭐ The defect is narrower and sharper: **the redirect hops alone**, so the exposure is an origin that REDIRECTS to a private address, not a caller naming one. ⛔ **The dependency settles it in its own words**: hyper-util 0.1.20's connector reads *"If the host is already an IP addr (v4 or v6), skip resolving the dns and start connecting right away."* So an IP-literal destination — the original URL's or any of five auto-followed hops' — never reaches the hook. ⚠️ **SOURCE-MEASURED at three sites; runtime reproduction PENDING** and not claimed. Owner `.7.2`, whose goal line says "validate destinations … at every redirect and dial, including numeric IPv4/IPv6" verbatim; record `R-44-45-1`, promoted to frontier row 1b.

⚠️ **Re-earning (2), (3) and (8) is NOT repair work** — those defects are fixed. What is missing is the COVERAGE measurement that would let each line be counted again, which is a different activity and is owned per line. ⛔ Four gate records remain to re-derive: `.11.4.7.2` (G1–G2, 16 claims), `.11.4.7.3` (G3, 7), `.11.4.7.4` (G4+G5, 4).

🔴 **THE FIVE GATE RECORDS ARE CENSUSED: 35 verdict claims and 19 deferrals, every one counted BEFORE the full source read (`SIGNOFF-REPAIR.11.4.7`, split four ways).** G1–G2 carries 16 claims and 6 deferrals, G3 seven and four, G4 one and five, G5 three and four, and G6–G7 eight plus three external gaps and a seven-row unsupported matrix. ⛔ **The records are not dishonest** — each claims the evidence its suites then carried; what was never measured is those suites' COVERAGE, which is `R-59-1`'s test-name defect and `R-63-2`'s test-comment defect one level up, at a release gate. The corpus to map against them is **110 of 198** repair commits that touched product source or a migration.

🔴 **A DEFECT FOUND BEFORE ANY RE-DERIVATION RAN, AND IT IS ARITHMETIC.** `2026-09-07_phase1-gate-record.md` says "**Met** — with **five** named deferrals" in its Outcome line and "the **five** numbered deferrals above" in its `answers:`, and its own table lists **SIX**. `git log -S` over the sixth row returns exactly one commit — the gate package itself — so the count was **wrong when written**, not overtaken later. A deferral is a named limitation with a revisit trigger, so an undercount is one limitation not being carried forward, which is the entire function of the list. ⭐ The book repeated it and is **corrected to six with a note**; the record is a dated decision and is superseded rather than edited (`.11.4.7.2`). ⚠️ `.11.4.5.2`'s "a count written while reading is not a count" earns its fourth and most consequential instance.

🔴 **AND A GATE CANNOT SEE THE CLAIM IT WAS BUILT FOR.** `scripts/check_visibility_policy.sh` exists because the superseded private-repository instruction "had already leaked past two" reviews; all four of its sentence shapes anchor on the full word `repositor(y|ies)`, so **two live instances written with the abbreviation are invisible to it**, in the phase-6 gate record and `PHASE-7.md`. Neither is in the three-entry exceptions file. ⚠️ **Both are correct HISTORY** — dated 2026-09-07 and 2026-09-08, before the 2026-09-09 correction — and that is precisely why nothing ever failed and the gap survived; the risk is a NEW sentence using the abbreviation. New owner `.11.2.4`. ⭐ **The gate then proved PRECISE by refusing the leaf that documented it**, so the defect is a narrow coverage gap rather than a gate that does not fire — and the repair must keep that precision rather than trade it for a looser match.

⛔ **Two measurement traps recorded rather than smoothed over, both mine.** The first repair-corpus pathspec returned **8** because `git show -- 'crates/*/src'` matches the directory path and not the files beneath it; the corrected form is proved in both directions before its number is used. And the first visibility census used `\brepo\b`, which `git grep -E` does not support — blind in exactly the way the gate is, and caught only because a known line existed to test the tool against. **A search returning nothing is a claim about the SEARCH until a positive control says otherwise.**

🔴 **THE BLOCKER REGISTER EXISTS — `SIGNOFF-REPAIR.13`, and it is an obligation rather than a record.** The director instructed, three times in one session and each time stronger: a blocker is handled promptly or at minimum put front and center and discussed explicitly; it is **reminded until he actively interacts with it**; and it is **listed in the mdBook and task-tree owned/tracked**. ⛔ **Eight rows**, each carrying who it is blocked on, its owner, what it does NOT block, and an `Ack?` column that starts at `no`: **A1** the held director decision on auditing the metrics read (`.3.5.2.1`), **B1–B3** the three external G6 preconditions (outside threat-model review, penetration test, prompt-injection suite — `.13.1`), **B4** public-name clearance and **B5** the licence (`.13.2`), **C1** remote CI (`.13.3` — the row as shipped read "remote CI has never run"; `.13.3` measured that FALSE on 2026-09-17 and rewrote it to the 167-commit unpushed gap), **C2** no gate record's shipped count re-derives (`.11.4.7`). The public face is the new [blockers chapter](docs/book/src/blockers.md), registered in the book directly after the qualification page.

⭐ **Three of the eight had no leaf at all before this**, which is the measurement that justifies the activity rather than the instruction alone: B1–B3 lived as prose inside a CLOSED phase's gate record, B5 was tracked by nothing anywhere, and C1 was a cadence note in `COMMIT.md`. ⛔ Two rows DID have owners and are deliberately not re-owned — a register that duplicates a leaf becomes a second copy that drifts, which is `INDEX-FRONTIER`'s failure in another costume. ⚠️ **The re-surfacing rule is the load-bearing part and it is not mechanizable**: every stopping-point reply lists every `Ack? = no` row as a labelled list, and a row clears only when the director engages with THAT row — a general question about the area does not clear it, and neither does answering that question.

🔴 **The defect the register repairs is not that blockers were unowned — it is that being owned made them INVISIBLE.** `.3.5.2.1` had sat correctly held, correctly acceptance-bearing and correctly at frontier row 8 for two days. Every individual rule was followed and the effect was burial on a slower timescale than an unowned finding would have suffered. Ownership and surfacing are two obligations, and §15 only ever covered the first.

🔴 **The book contradicted itself in ONE chapter about a repair two leaves spent commits proving, and `.3.3.4.12.2` (REPAIR-0194) closes it.** `authority.md` said under "Administering a federation direction" that a card import "is now fenced by a revocation from **either** side" and, forty lines later under "Importing a portable agent card", that nothing there "can fence them" and that the ordering "arrives with" `SIGNOFF-REPAIR.3.3.4.12` — a leaf that had landed. ⛔ The error is CONSERVATIVE, so nothing unsafe follows; what follows is that the project's primary review surface disagreed with itself, and no gate can catch it because both sentences are well-formed and only their conjunction is wrong.

- **The superseded services were deleted, not marked.** `git grep -c "federation::propose\|federation::accept\|federation::revoke" -- crates` returns rc=1, no match, and `cargo check -p reasonbraid-server --all-targets --locked` returns rc=0 afterwards — the build quantifies over every test, bench and example target, which a grep cannot. 🔴 **Why nothing had flagged them:** `dead_code` does not apply to a `pub` item in a `pub mod`. The two sibling leaves that deleted their superseded bridges did so because the compiler complained; here it could not, by construction.
- Three present-tense sites corrected, each RECORDING the superseded sentence rather than replacing it silently. ⛔ This file's two earlier mentions and the tree's four are dated records of what `.3.3.4.11`/`.11.3` measured and are deliberately untouched — correct as history.
- Promoted: `docs/knowledge/a-repair-owns-every-sentence-that-states-its-limit.md` — a repair that falsifies a limitation owns every sentence stating it, and the instrument is `git grep` for the limitation's own words rather than for the leaf id.

🔴 **CAN THIS PROJECT STILL STATE ITS OWN RELEASE-GATE POSITION? `SIGNOFF-REPAIR.11.4.7`, frontier row 1b.** Opened after the director asked what blocks Internet exposure, and **WIDENED at his challenge** from one gate record to **five**. `docs/decisions/` holds five gate records — G1–G2, G3, G4, G5, G6–G7 — and four make countable "shipped / Met" claims, every one counted BEFORE the full source read. §16.12 line (2) of the G6/G7 record is "authenticated enrollment, rotation, revocation and **tenant-isolation** tests", and the corrective review has since REPRODUCED cross-tenant defects inside exactly that subject: `.6.1.1` (a caller receiving another tenant's whole thread projection through MCP), `.3.5.3` (an administrator reading another tenant's node inbox in full), `.3.3.4.10.3` (a prune destroying another tenant's rows), `.6.1.2` (answering another tenant's recruitment call) and `.9.2.1.2` (both publish verbs admitting any enrolled principal). ⛔ **The records are NOT dishonest** — each claims the evidence its suites then carried and could not know their coverage; the defect is a gate line counted against a suite whose coverage was never measured. ⚠️ G6/G7's THREE EXTERNAL gaps are unchanged and out of local scope: the externally reviewed threat model, the prompt-injection action-boundary suite, and the penetration test.

🔴 **Tranche 4b (DOC-0021) reconciled eight records and 40 clauses, and its finding is about `SIGNOFF-REPAIR.3.4` — the very leaf whose rarity ranked seven of them.** That leaf is `active`, all FIVE of its children are `done`, and **two of its own goal line's mechanisms have no child**: the line reads "actor/subject participation **and consent**; bind replay hashes **to target** and authority context", and `.3.4.1`–`.3.4.5` carried delegability, the authority-CONTEXT half of the hash, cache freshness, the codecs and ADR-009. ⛔ How it stayed invisible is arithmetic nobody checked: the split's own bullet says "five children along the **four** mechanisms the goal line names plus the two follow-ups" — and the goal line names five. ⭐ **This is a THIRD shape of lost ownership**, distinct from a leaf that never cited a routed record (`.11.9`) and from `attach` (a goal line that does not make a clause visible): here the goal line names the mechanism VERBATIM, so no census reading that text can find the gap. The ledger's closed state set is deliberately NOT extended — the required action is `unowned`'s, open a leaf — and the check the shape earns is stated where its users are: at every split, count the goal line's mechanisms against the children and make the split's own sentence reconcile. New owners `.3.4.6` and `.3.4.7`.

🔴 **Four further live findings, each routed rather than reported (§15).** (1) **A role can auto-initiate exactly ONCE per tenant, for ever** — `create_thread_auto`'s idempotency key is `format!("auto_{}_{}", role, tenant_id)`, a function of the role and the tenant ALONE, against an `idempotency` table keyed `(tenant_id, idempotency_key)`, so the second initiation replays the first thread when the body matches and answers `idempotency_mismatch` when it does not. ⭐ The test `R-76-77-3` names cannot see it by construction: one success, then only POLICY refusals. Annotated at `.5.2`, whose goal line already says "make repeated legitimate auto initiation possible". (2) **The R1 Git acquisition opens its repository with gix's DEFAULT permissions over an untrusted remote** — `gix::init_bare` is `ThreadSafeRepository::init(…, create::Options::default())`, while gix 0.87.1 ships the unused `open::Options::isolated()`, whose own doc is "prohibiting accessing the environment or spreading beyond the git repository location"; so the operator's global git config and git environment variables are honoured while fetching a caller-supplied URL, against §12.5's default refusal of hooks, filters and alternates. ⚠️ Distinct from `.7.2.1`, which bounded where gix WRITES. Attached to `.7.2`. (3) **The quarantine gate does not exist anywhere in the product** — `git grep -c "quarantine_status" -- 'crates/*/src'` returns rc=1, no match, so migration 0028's column is written by its DEFAULT and read by nothing; the §9.8 code `evidence_quarantined` is constructed nowhere either, and the book's errors chapter already lists it among the eleven registered codes this build never emits without anyone noticing the schema is waiting for it. Attached to `.7.4`. (4) **`.3.3.4.12` left the three superseded federation direction services standing** — `git grep -c "federation::propose\|federation::accept\|federation::revoke" -- crates` returns rc=1, no match, yet they remain `pub` on a `pub mod`, which is why the compiler's dead-code analysis cannot see them and why `.3.3.4.8` and `.3.3.4.11.3`, which DELETED their superseded bridges, set no precedent that fired here. New owner `.3.3.4.12.2`.

⚠️ **The book contradicts itself in ONE chapter about a repair two leaves spent commits proving.** `docs/book/src/authority.md` says under "Administering a federation direction" that since `.3.3.4.12.1` an import "is now fenced by a revocation from **either** side", and under "Importing a portable agent card" that the import is "**not** order[ed] … against a concurrent agreement revocation … That ordering arrives with `SIGNOFF-REPAIR.3.3.4.12`" — a landed leaf, in the future tense. ⛔ The error is CONSERVATIVE, so nothing unsafe follows from it; what does follow is that the director's only window into the project disagrees with itself, and no gate sees it because both sentences are individually well-formed. `.3.3.4.12.2` owns the correction. ⛔ The tree's own and this file's earlier mentions are correct HISTORY of what `.3.3.4.11`/`.11.3` measured and must not be "fixed".

⭐ **Two clauses were REFUTED at HEAD and both were RIGHT when written**, told apart by `git show` at the review baseline rather than by a reading. `R-76-77-3` clause 2 said a newer narrow grant shadows an older broad one: at `9c2d2ba` the loader was `ORDER BY valid_from DESC LIMIT 1`, so it did; `select_authority_in_tx` now pages every active candidate and returns the first ALLOWED one. `R-36-39-3` clauses 1–3 said the evaluator accepts any selector for a tenant target: `.3.3.2`'s own baseline line states the clause verbatim and the subset rule closed it. ⚠️ And one was NARROWED, which is the more useful half: `expired_and_revoked_grants_are_denied` genuinely exercises no revocation (`git grep -c "GrantStatus::Revoked"` over that suite returns rc=1, no match) — but the INVARIANT is covered, by `.3.3.2`'s pure controls, so the defect is a test name and a doc comment that overclaim, and a reader who trusts the name stops looking where the coverage actually is.

⭐ **`.9.3.4`'s decision is taken, and measuring it REVERSED the answer this session had pre-announced (DOC-0020).** I had said I would close it as "measured, extension is input to a roadmap version" on the strength of the leaf's own premise — that `GrantAction`/`TargetSelector` are frozen wire vocabularies under §9.8. 🔴 **That premise is false**: §9.8 is the REASON-CODE registry, and `git grep -c "GrantAction\|TargetSelector" ROADMAP.md` returns **0**. ⭐ The project had moreover ALREADY RULED on how to extend it (`2026-09-06_authority-boundary.md`: *extend the checker FIRST — an over-broad grant must be unrepresentable*), which the leaf never consulted. **Decision: `GrantAction` extends with administrative verbs; `TargetSelector` does NOT** — `Threads { threads }` enumerates objects at grant time and the ordinary flow publishes a publication whose id does not yet exist, so the variant would be unusable for the very operation it was added for; the residual (narrowed by VERB, tenant-wide by OBJECT) is accepted and published. 🔴 **And the extension is a MIGRATION**: `permitted_actions` is a JSONB array of wire names, so no stored boundary can contain a name that postdates it and the verb stops working for every existing tenant unless a disposition is chosen — fail-closed, and still live breakage. ⛔ **The leaf's "three sites" was stale and this session staled it**: re-derived, **6**. Decomposed into `.9.3.4.1` and `.9.3.4.2`, each with acceptance and a red-first control; **not implemented here** — 207 `GrantAction::` references across 19 files, three changes at once, and the same decomposition argument REPAIR-0189 used on `.9.2.1`.

🔴 **A finding THIS SESSION published did not survive the director's grading question, and the correction refutes my correction as well as my mechanism; `.9.2.1.1.1` closed under DOC-0018.** REPAIR-0190 published *"this build is I/O-bound"*, *"about 9 % CPU utilisation"* and *"the completed runs say the same thing"*. Graded on `docs/CLAIM_VERIFICATION.md` §4.1's three axes separately: **PROSE fails** (no mechanism was established), **NUMBER fails** (9 % is the lib-only arm; the `--tests` arm beside it is **207 %**, a factor of 22), **NAMED INSTANCE fails** (the arms disagree by an order of magnitude and were published as agreeing). ⛔ **Re-derived as an INTERVAL** — three runs of the same work: **627.0 s / 148.9 s / 140.7 s** wall at **54.2 % / 217.9 % / 219.7 %** CPU, with total CPU essentially CONSTANT (340.0 / 324.5 / 309.0 s, a 31 s spread). The work does not change; the waiting does, by **4.5×**, and run 1 spent ~472 s not computing. 🔴 **Run 1 exceeds ten minutes, so `.9.2.1`'s original ">10 minutes" note was RIGHT and my correction of it was wrong** — I contradicted it from one warm run, which is the stochastic rider `CLAIM_VERIFICATION` warns about. ⛔ **The mechanism is WITHDRAWN, not replaced**: run 1's non-CPU wall time is consistent with `docs/decisions/2026-09-12_checkpoint-cost-model.md`'s adjudicated ~21.9 s-per-executable cost (472/21.9 ≈ 22) but is not evidence for it, and that ruling stands as the leading candidate. 🔴 **The real failure was leg 2** — the standard says your own project's history is the cheapest oracle and that the earlier ruling wins unless you name a difference; that ruling names `syspolicyd`, which I observed at 74 % and never looked up. ⭐ **Leg 3 closed where it belongs:** `scripts/measure_check_phases.py` now reports `user_seconds`/`system_seconds`/`cpu_percent` per phase and gains a `--self-test` (8 controls) whose arms ARE the three measurements that were got wrong. **What survives, with its conditions: warm 141–149 s at ~218 % CPU; cold 627 s at 54 %.**

🔴 **Both publication verbs admitted ANY ENROLLED PRINCIPAL — enrolment in any tenant was the whole predicate for writing a publication into a Git repository and for declaring it effective; `.9.2.1.2` closed under REPAIR-0192, and with it the parent `.9.2.1`.** `api::held_publication_authority` is ONE definition both verbs call: the request names an `owning_authority` and `authority::grant_held_by` decides — `register_target`'s shape and `.9.3.1`'s predicate, never a sixth spelling. It runs BEFORE the path is resolved and before the publication is loaded, so an unauthorized caller reaches neither the filesystem nor the database. ⭐ **Naming a grant is not holding one, and that is the load-bearing leg**: grant ids are derivable (`grt_<principal_id>`), so a check that only asked whether an active grant EXISTS would be no check at all. The control names another principal's real active grant — refused — beside the MATCHED PAIR where only the holder differs and the same request is admitted. ⚠️ **The limit is RECORDED, not papered over** (`.9.3.4`): no `GrantAction` and no `TargetSelector` can NAME a publication, so a held grant is effectively TENANT-WIDE for these verbs; the book says so rather than implying the verb is scoped, and `.9.3.4` is promoted to frontier row 1b because the caveat has now had to be written twice. **Blast radius measured, not assumed:** no client outside `tests/policy.rs` drives either verb. **`bash scripts/run_pg_tests.sh policy command_api profiles authority` → 4 suites, 111 tests, 0 failed, rc=0.** ⭐ `.9.2.1`'s decomposition paid: three red-to-green cycles, three bisectable commits, each child's acceptance on its own evidence.

🔴 **`mark_effective` recorded whatever Git object ids the caller declared, and THREE of this project's own fixtures drove the transition with ids that resolve to nothing; `.9.2.1.3` closed under REPAIR-0191.** Nothing opened a repository, so a publication's Git provenance was whatever the request said it was — and the sharper half is that the bypass was EXERCISED by the suite meant to qualify it: `git grep -n '"git_object_ids": \["abc123"' f3d77c9 -- crates/reasonbraid-server/tests/policy.rs` returns **3 sites**, in a suite that was green at `13 passed; 0 failed`. `publisher::missing_objects` now answers PER ID, in the CORE both publish verbs go through — `git grep -n "publications::mark_effective" f3d77c9 -- crates/reasonbraid-server/src` returns **2 production callers**, so a handler-level repair would have left the sibling open and its own suite green. ⭐ **Containment became a TYPE:** `resolve_repository` returns a `PublicationRepository` with no other constructor, so a verb taking one cannot be reached with an unresolved path. ⛔ An unparseable id and a well-formed absent one are the same answer and both are asserted; a repository that fails to open is a repository failure, never a verdict about the ids. ⚠️ **`repo_path` is now REQUIRED on `POST /v1/policy-publications/{id}/effective`** — a change to a shipped request shape, and the design question behind it is recorded rather than assumed. The three fixtures are RE-SEEDED with ids read back from a real repository, not relaxed. ⚠️ Limit stated: an object is proved to EXIST, not to be this publication's own — binding the ids to its refs needs the repository recorded with the row. **`bash scripts/run_pg_tests.sh policy` → `13 passed; 0 failed`, rc=0; `cargo test --test publisher` → `7 passed; 0 failed`.**

🔴 **The publish verb took its repository location from the REQUEST BODY, and any enrolled principal named any path on the server's filesystem; `.9.2.1.1` closed under REPAIR-0190.** `api.rs::publish_publication` read `repo_path` and handed it to `gix::open`, while the publisher's own header said the bundle went "into the LOCAL bare repository". The deployment now declares one root (`rb-server --publication-repo-root`), the request names a location inside it, and `publisher::resolve_repository` is the single predicate — CANONICALIZING BOTH SIDES, which is what lets one question answer two escapes: in the symlink case every component the caller named **is** inside the root, so a string containment test refuses `..` and admits the symlink. An undeclared root closes the verb with the new `503 publication_repository_unconfigured`; an unusable declared one refuses the BOOT, before the migrations run. ⛔ `.11.12` is untouched — the secret-store ordering and the `--host` gate remain its to decide. **Live control: 6 legs; offline control: 8 arms; `bash scripts/run_pg_tests.sh policy` → `13 passed; 0 failed`, rc=0.** ⚠️ **Falsified by a MATCHED PAIR rather than by neutralizing the shipped predicate** — the harness refused that edit as a security change, and the contrast is stronger because it is durable: the same three locations, against a server whose declared root is the directory above, are now accepted and reach the record. 🔴 **The repair exposed a pre-existing fixture defect in the very leg it was told to preserve**: "a publish into a NON-repository path refuses" ran after the publication was already `effective`, so the STAGE check answered it and `gix::open` was never reached — green for the wrong reason for its whole life. ⚠️ Two limits stated, not implied: containment is decided once per request, so a symlink swapped before `gix::open` would not be seen; and both publish verbs are still authorized by ENROLMENT ALONE, which is `.9.2.1.2`. ⛔ **This paragraph's build-cost sentence was WITHDRAWN by `.9.2.1.1.1` (DOC-0018) — see the paragraph above it.**

🔴 **The derived Knowledge Map was generated from the WORKING TREE and committed against the INDEX; `.11.4.5.4` closed under REPAIR-0188.** ⛔ **This session's own commit `991bbf8` shipped a dangling link** — `git show 991bbf8:KNOWLEDGE_MAP.md | grep -c 'a-control-is-calibrated'` returns **1** while `git ls-tree --name-only 991bbf8 docs/knowledge/` returns **0**, with every gate green. The pre-commit hook regenerates the map and stages it, so the map is committed against the index, while the generator enumerated `ls docs/{tasks,decisions,knowledge}/*.md` — the working tree. ⛔ **THE FIRST CENSUS WAS TOO NARROW AND ITS NUMBER WAS WRONG**: `docs/knowledge/` alone gave 2; all three families the generator actually lists gave **3 of the 139** commits touching the map, the third (`0558ed6`) a `docs/decisions/` record the narrow scan could not see. ⭐ **All three share one shape** — a commit sequenced BETWEEN the work that creates a source file and the commit that adds it; two are changelog rotations, and `.11.4.1.2` REQUIRES a rotation to be its own commit, so the spine's correct sequencing rule is what exposed the wrong source. **Fixed by deriving from the commit** (`git ls-files -- ':(glob)…'`, `git show ":$f"`), which removes the failure mode per `.11.4.5.3`'s ruling and also stops an UNSTAGED `answers:` edit reaching the map. 🔴 **The first cut of the fix would have published 54 broken links**: a git pathspec wildcard MATCHES `/`, so `docs/tasks/*.md` returns **69** paths where `ls` returns **15**, and the link is built from `basename`. Caught by diffing the fresh render against the committed map — ⚠️ the KNOWLEDGE-MAP gate could NOT have caught it, because it compares a render against a file the hook had already overwritten with that same render. ⭐ The corrected generator reproduces the committed map BYTE-FOR-BYTE, so the repair changes the failure mode and not the output. New `--self-test` 7/7, falsified three ways with each variant reddening only its own arms; SELF-TEST now covers 24 instruments at 1.59 s; 18 checks green. ⛔ The three historical commits are NOT rewritten.

🔴 **The table-arity gate modelled the OPPOSITE of the renderer, and its own `--self-test` asserted the same false answer; `.11.2.3` closed under REPAIR-0187.** ⛔ **A whole-corpus scan reporting 0 defects in 323 tracked files was measuring the wrong predicate**: `scripts/check_table_arity.sh` treated a pipe inside an inline code span as part of the cell, where GFM splits a table row into cells BEFORE inline parsing, so only a backslash escape protects a pipe. ⭐ **Asked of mdbook 0.5.2 — the renderer this project publishes its book with — nine shapes, each rendered before anything was asserted**: `` | `x | y` | 2 | `` emits **2 cells** with the backticks LITERAL and the third cell **discarded**; `` | `x \| y` | 2 | `` emits `<code>x | y</code>` and `2`, the repair form. 🔴 **The self-test was the reason the defect survived, not the thing that would have caught it** — written from the same reading as the code, it could not disagree with it, which is `TOOLBOX.md`'s measured blind-spot rule reproducing exactly. ⛔ **The wrong model was wrong in BOTH directions**: it also scored an unpaired backtick run as a defect, which the renderer does not, so a false positive shipped alongside the false negative. ⚠️ **The damage was on this project's own doctrine page**, deliberately retained for a session as the falsification target: the `INDEX-FRONTIER` row published **516 of its 1,027** rationale characters and a bare `—` where `scripts/check_tree_index_frontier.sh` belongs. Repaired and re-rendered: **3 cells**, enforcer named, `<code>| — |</code>` intact. ⭐ **The leaf's own warning that the ratchet would block its own fix did NOT materialise, and the reason is recorded rather than the relief**: the ratchet runs the working-tree parser against BOTH sides of its comparison, so a rule correction moves `now` and `before` together — 0 vs 1, a fall. ⛔ **Two of the leaf's own recorded details were corrected by measurement rather than repeated**: the published cut is at *"8 completed trees write `"*, not at *"fixed this exact"*, and the corpus now holds **1** such row, not 2, because `RECONCILIATION.md:111` was genuinely repaired by REPAIR-0175. All **9** arms are now the renderer's verdict; corpus **0**; 18 checks green. Promoted to `docs/knowledge/a-control-is-calibrated-against-the-renderer.md`.

🔴 **The leaf named the MCP seam and the defect was in the shared CORE, where the HTTP verb was equally open; `.6.1.2` closed under REPAIR-0185.** ⛔ **Measured live before the repair, on BOTH surfaces**: a role enrolled in tenant A recorded a `decline` AND a `recuse` against tenant B's recruitment call — through the seam and over plain HTTP. A new seven-leg control reported **5 breaching**; after, **0**, 5/5. 🔴 **`POST /v1/calls/{call_id}/respond` takes NO tenant at all** and rides the same `respond_to_call_core`, which fetched the call by id alone — so the HTTP verb had nothing to be inconsistent about and was simply unbound. ⛔ **A repair at the seam would have left that verb wide open AND the seam's own suite green** — a sibling caller broken while every visible test passes is what a misplaced check looks like from the inside, and it is the argument for putting the binding in the core, derived from the call (`inspect_call`'s shape). ⛔ **THE FIXTURE WAS WRONG FIRST AND MANUFACTURED THREE GREEN LEGS**: the control posted to `/v1/calls/{id}/responses` and the route is `/respond`, so every HTTP leg read a **404 as a refusal** — including the POSITIVE leg written to stop the repair being a blanket refusal, which was itself passing by over-refusal. Caught by reading the failure TEXT rather than the count: `{"raw":""}` is not a refusal any handler here writes. ⛔ **A leg the acceptance asked for was REMOVED rather than kept green**: "every response kind" includes `join`, but `join` is refused earlier by the absence of an enrolled node, so it would pass without the binding ever being reached. The two kinds that skip the eligibility gate carry the claim. ⭐ **The proposal half was settled by MEASUREMENT and is not a defect**: `lifecycle::register_proposal` has no grant check and no audit row — and neither does the HTTP verb behind `POST /v1/policy-proposals`. The tool is NOT weaker than the surface it claims parity with; the header was describing a control neither has, and now states what each of its three verbs actually brings, separately. ⚠️ **SECOND header this session asserting a check nobody wrote**, and the shape is summary rather than carelessness: both said something true of ONE member and applied it to a list. ⛔ **Published as the THIRD and corrected on challenge**: the third was `lifecycle.rs`'s approval comment, which re-reading shows is ACCURATE about its SQL — the check existed and matched a subject; what was wrong is that the subject was an unverified claim. A different defect, and folding it in made the generalization fit a population it does not describe. ⭐ **Promoted**: when two surfaces share a core, the binding belongs in the core. Pending leaves **55 -> 54**. 18 doctrines green, `mdbook build` rc=0.
🔴 **Citing an authority was the same as holding one on THREE surfaces, not the two the leaf named, and no site in the family had ever consulted `valid_from`; `.9.3.1` closed under REPAIR-0184.** ⛔ **Measured live before the repair**: a principal in tenant A recorded a policy **RETRACTION** under tenant B's grant, registered a deployment target owned by it, recorded a correction under a grant whose `valid_from` was tomorrow, and filed a policy **APPROVAL AS ANOTHER PRINCIPAL**. A new nine-leg control reported **6 breaching**; after the repair, **0**, with 12/12 and the eleven pre-existing policy tests unchanged under a stricter registration path. ⛔ **Reachable rather than theoretical**: the dev enrolment mints `grt_<principal_id>`, so naming another principal's grant needs nothing but an id they have seen. 🔴 **The census took the family WHOLE and that is what found the third site**: `git grep -n "FROM authority_grants" aee579a -- crates/reasonbraid-server/src/` — PINNED to the pre-repair commit, because this repair changed the population — returns **9 sites, of which 5 ask the grant-validity question in four different spellings** — `policy::register` on `status` alone, `policy::resolve`/`corrections`/`deployments` adding expiry, `lifecycle::record_approval` adding expiry AND a subject. ⛔ **`git grep -n "valid_from"` over those four modules returns rc=1 — NOT ONE of the five consulted it**, though the column is `NOT NULL` and has been in the schema since migration 0004, so a grant scheduled to begin tomorrow authorized everything today. ⛔ And the mirror: `expires_at IS NULL OR …` sat in four predicates for a column that is ALSO `NOT NULL` — four sites defending a nullability that does not exist, none defending a start date that does. ⭐ **The odd-one-out rule paid again**: the ONE site that bound a subject is the one that had had the attention, and it was still HALF-bound — it matched the grant to `input.approver`, a string off the wire, while the handler above it resolved the caller and discarded it. ⭐ **The repair is one definition, two shapes**: `authority::grant_is_live` and `authority::grant_held_by` (binding `subject_kind` AND `subject_id`; the existing site matched the id alone). ⭐ **`.9.1`'s measured asymmetry is settled** — registration no longer accepts a grant the resolver in the same module would refuse — while `.9.1`'s SEMANTIC question, whether a policy's owner must be the registrar's own grant, is deliberately left to it. 🔴 **Falsified one arm at a time**, each neutralization proved landed by a predicate census: the subject binding -> legs A and B; `valid_from` -> leg C; approver-is-the-caller -> **leg G alone**. ⚠️ **The falsification found a defect in the CONTROL** — two approval legs shared a proposal and a successful approval advances it, so each reported the other's outcome; invisible while the repair holds. Each leg now has its own. ⚠️ **One leg is deliberately NOT red-first and is labelled so in the test**: leg G was refused by the original code, which bound the grant to the claimed approver; the repair moves that binding to the authenticated caller, and leg G guards the link the move needs in its place. ⚠️ **THE SECOND ACCEPTANCE LEG THIS SESSION TO ASK FOR SOMETHING THE SCHEMA CANNOT EXPRESS**: `GrantAction` has ten variants, nine thread-shaped plus `TenantAdmin`, and `TargetSelector` is `TenantWide | Threads` — `git grep -n "Publication\|DeploymentTarget\|Correction" -- crates/reasonbraid-core/src/authority.rs` returns **rc=1**, so "the grant's action covers the target" has nothing to compare. Opened as `.9.3.4` rather than met in a weaker form quietly. Pending leaves **55 -> 55**: one closed, one opened. ⭐ The caption was written **56** from arithmetic and the tree's own awk re-derivation said 55 — the count is a command's output, never a subtraction. 18 doctrines green, `mdbook build` rc=0.
🔴 **The severest defect this reconciliation produced is REPAIRED, and one part of the finding behind it was corrected by measurement rather than confirmed; `.6.1.1` closed under REPAIR-0182.** `crates/reasonbraid-mcp` carried a PRIVATE 5,366-character re-implementation of the server's three inspection reads while its own header claimed *"the SAME queries + the SAME authorization as the HTTP handlers"* and *"every read runs the reader classification"*. The copy ran neither. ⛔ **Measured live on a real cluster before the repair**: a principal in tenant A received tenant B's ENTIRE thread projection — the subject, the objective, the participants map, the creator's `hpr_…`, the budget and the ceiling id — with `"visibility":"network"` printed beside it. A new fourteen-leg control reported **8 legs breaching**; after the repair, **0**, with 6/6 tests. ⭐ **The repair removes the failure mode rather than patching it**: a new `mcp_read` seam in the server crate (`mcp_read_internal`, beside `mcp_write_internal`) calls three halves now SHARED with the HTTP handlers — `authorize_inspection`, `thread_inspection`, `inbox_inspection` — and the MCP crate's copy is DELETED, proved dead by the compiler rather than by a grep. Adding the three checks beside the copy would have left a second implementation free to omit them again, with the sentence now true and the next omission harder to see. ⚠️ **THE CORRECTION, and it is a PROSE claim of this activity's own**: `LIVE_STATUS.md`, `DEV_NOTES.md` and `RECONCILIATION.md` published *"Every tenant's policy documents and their clauses come back"*, which reads as a per-tenant scoping being bypassed. `git grep -n "tenant" -- migrations/0038_policy_registry.sql crates/reasonbraid-server/src/policy.rs` returns **rc=1** — the word occurs in neither the registry's schema nor its module. There is no tenant dimension to bypass: the registry is SITE-GLOBAL, and the HTTP `api.rs::list_policies` returns the same unfiltered set to any enrolled caller. **What STANDS**: no `WHERE`, no principal check, and a response labelled with the caller's own tenant. **What FALLS**: the framing that made it a cross-tenant leak. ⭐ The oracle was one this leaf did not build — the very surface the module claimed parity with. Acceptance leg 1 was REWRITTEN before it was met rather than met in a weaker form quietly, and the schema question is opened as `.6.1.5` instead of being answered by a filter on one of six sites while the write surface admits any enrolled principal. 🔴 **Falsified one arm at a time**, each neutralization proved landed by a gate-call census and each rc read directly, never through a pipe: removing `authorize_inspection` -> **3 breaches, leg A only**; `authorize_tenant_admin` -> **2, leg B only**; the enrolment check -> **1, leg C only**. No arm carries another's proof. ⛔ **`git diff` was useless as the landing proof** — `mcp_read.rs` is a NEW file, so an unstaged edit to it produces no diff at all, and the first falsification's evidence printed nothing while the harness reported success. ⭐ Three side effects, each deliberate and each a divergence nobody had found: the MCP thread read now applies the `.1.3.1` derived view, the inbox read the `node_inbox_state` view, and `get_policy_bundle` loses the tenant parameter that could scope nothing — the conformance golden asserts that ABSENCE, because a golden that merely stopped requiring the field would pass again the day someone re-added it. Pending leaves **54 -> 54**: one closed, one opened. 18 doctrines green, `mdbook build` rc=0.
🔴 **The director asked whether the findings were OWNED or only ROUTED, and the audit says routed; `.11.13` closed under REPAIR-0181.** This session surfaced six findings and reported each with an owner. Extracting each named leaf's section and checking for an `- Acceptance:` line of its own: **8 of 10 had none, and 6 had no children** — `.6.1`, `.9.3`, `.9.2`, `.11.2`, `.8.1`, `.4.4`, `.10.1`, `.7.3`. ⭐ **The one finding that WAS properly owned got that way by accident of vocabulary**: `.11.12` exists because the clause ledger classified its clauses `unowned`, and that state's rule FORCES a new leaf. Everything classified `owned` or `attach` received a sentence inside a container and a report line saying "owner: `.6.1`". ⛔ **That is a gap the ledger's own six states cannot see.** `owned` means a leaf owns the clause AND its own text makes it visible; `attach` means the text does not, and is gated by `ATTACH-LANDED`. Both measure whether a clause is VISIBLE to a leaf. **Neither asks whether that leaf can be picked up and finished.** A container with a six-mechanism goal line, no acceptance and no children answers no. **FIX, in this commit:** eight containers censused and split into **eleven bounded leaves**, each carrying its own reproduce, owns and acceptance, and each acceptance naming a control to be observed RED before its repair (`docs/CLAIM_VERIFICATION.md` leg 2) — `.6.1.1`–`.6.1.4`, `.9.2.1`, `.9.3.1`–`.9.3.3`, `.11.2.3`, `.11.4.2.2`, `.11.12`. The frontier is re-ranked by severity: the MCP cross-tenant read leak leads, then the authority impersonation, the write seam's unbound call, the table-arity parser and the publish verbs. ⚠️ **The re-run audit first reported two of the eleven as having no acceptance and the AUDIT was wrong** — both opened `- Acceptance, …` with a comma. Fixed in the LEAVES rather than by loosening the matcher, because a matcher that guesses at prose is `.11.6`'s measured failure mode; the same shape as `.11.4.4`'s `- Opened:`/`- Status:` repair. ⚠️ The pending count rises 47 -> 54, and that direction is correct: it rises because routing became ownership. ⛔ **No product code changed** — this commit writes the leaves that will change it. **Promoted:** a leaf that cannot be picked up and finished is not an owner (`TOOLBOX.md`). 18 doctrines green.
🔴 **This activity corrected a number IT published, and `git show` proved the number was wrong when written rather than overtaken; tranche 4a closed under `.11.9.1.3.1` (REPAIR-0180).** `RECONCILIATION.md`'s `R-80-82-1` clause 3 row — written by `.11.9.1.1.1` and attached to `.8.1` — said `tests/policy.rs` accepts an unbound verdict digest `"sha256:00"` **"in four places"**. A second record sent a second reading to the same file, which ran the command: the literal appears **7** times in **7 DISTINCT test functions**. ⛔ **And the corpus did not move**: `git diff --stat b227ce8 HEAD -- crates/reasonbraid-server/tests/policy.rs` is EMPTY and `git show b227ce8:…/policy.rs | grep -c` returns 7. The figure was wrong WHEN WRITTEN, inside the ledger this project built precisely so findings would re-derive. ⭐ The finding is STRONGER than published, not weaker, and both sites are corrected with the superseded figure named. **Promoted:** a count written while reading is not a count — every number in a durable record comes from a command, and `git show` at the original commit distinguishes *stale* from *false* (`TOOLBOX.md`). 🔴 **Any enrolled principal can act as any authority whose grant id they know.** `corrections.rs::authority_holds(pool, grant_id)` takes the grant ID AND NOTHING ELSE — `WHERE grant_id = $1 AND status = 'active' AND (expires_at IS NULL OR expires_at > now())` — never the caller, the action, the selector, the boundary, the publication or `valid_from`; and `record_policy_correction` admits on enrolment alone. A suspension, retraction or waiver is recorded citing an authority the caller does not hold. ⭐ The module's own error type for a missing grant is already called `GhostAuthority`. `deployments.rs` repeats the shape for `owning_authority`. 🔴 **No second review can EVER be scheduled for a (publication, trigger) pair.** `review_id` is `format!("rev_{pub}_{trigger}")` — deterministic — and `migrations/0045_policy_reviews.sql:12` makes it the PRIMARY KEY; the dedupe skips only a still-`due` review, so once it is `done` the INSERT collides and **`if inserted.is_ok()`** discards the error. ⭐ `.9.3`'s goal line already REQUIRED "permit subsequent reviews after completed occurrences"; this leaf did not discover the requirement, it located the cause. ⚠️ The same `is_ok()` makes a total storage failure return a successful empty schedule. 🔴 **`rb-server` migrates the database on the line BEFORE it validates the profile it refuses to boot without** — `SecretStore::resolve`'s own comment says "an undeclared profile refuses the boot — never a silent fallback", and the refusal arrives after the schema has changed — and `--host` carries no predicate at all, so `--host 0.0.0.0` binds every interface while the startup line still prints "(Phase 0 dev profile)". New leaf `.11.12`, whose text insists the two halves be decided separately. ⚠️ **Two fixtures declare digests bound to nothing**: `deployments::assign` checks only `is_sha256_hex` and `crates/reasonbraid-server/tests/policy.rs:2724` passes `sha256:` plus sixty-four `a`s to a 200; `mark_publication_effective` takes `git_object_ids` from the body under enrolment-only auth and never looks for them — `.9.2`'s goal line says "reject fabricated effective Git IDs" verbatim. ⚠️ And a test comment claims a measurement the test does not make: the metrics control is headed "the replay counter's delta matches the replayed command" and asserts only `delta("authorization_denials")`. ⭐ **The FIFTH two-records-one-finding pair**: `R-53-5` clause 1 and `R-75-1` clause 1 are both "one waiver satisfies a trigger named repeated". Tranche 4 sized at **10,519 characters** (4.12x the proved size) and split five ways on the derived boundary with ONE declared deviation — a 423-character singleton folded into its sibling. **26 clauses** (22 `owned`, 2 `attach`, 2 `unowned`); **204 ledger rows clean / 49 self-test controls**; ⭐ `ATTACH-LANDED` fired on this leaf's two unattached clauses — the FOURTH consecutive tranche. 18 doctrines green; no product code, schema, test or script changed.
🔴 **A browse worker that deadlocks above a sixty-fourth of its own output limit, four adapter defects each at two sites, and a deadline read by nobody; tranche 3c closed under `.11.9.1.2.3` (REPAIR-0179) and tranche 3 is COMPLETE at 15 records and 69 clauses.** ⭐ **The RATIO is the finding.** `browse.rs::run_browse` polls `try_wait()` in a sleep loop until the child EXITS and only then reads its piped stdout — true and unactionable on its own. The request it writes sets `max_output_bytes` to **4 MiB**; an OS pipe buffer is at most **64 KiB**. So the worker blocks on its own write for any response above roughly 64 KiB, never exits, and the loop runs to the deadline and returns `TimedOut` — an error naming TIME for a failure about BUFFERING. Two more on the same function, both on paths that only run when something already went wrong: the stdin error's `?` drops a live `Child` (which in Rust neither kills nor waits), and the timeout's `child.kill()` reaches the worker but not the browser beneath it, with `stderr` set to `Stdio::null()` so nothing is left to diagnose either. 🔴 **Four adapter defects, each at TWO sites, because `codex.rs` and `claude.rs` are the same file twice**: `buf[buf.len() - 1024..]` is a BYTE slice of a `String` and panics off a char boundary — on the failure path; `lines.next_line()` reads to a newline unbounded under a `buf.len() < 8192` check made BEFORE the append, so the real bound is 8,191 bytes plus one unbounded line; the `children` map is inserted into and never removed from, growing by one unreaped `Child` per operation for the process's life; and the `turn.completed` arm returns immediately while the EOF arm awaits the drain and calls `child.wait()` — ⭐ **the SUCCESS path leaks what the failure path cleans up**. 🔴 **A deadline is computed, shipped and read by nobody, settled by a SINGLE-HIT GREP rather than a reading**: `deadline` appears exactly once across every adapter source and the supervisor — `contract.rs:58`, the struct field's own declaration. 🔴 **A completed item reaching the retry gate is dead-lettered and auto-quarantined** — `retry.rs:80` is a catch-all `Some(_) => Refuse` and `worker.rs:275` maps every refusal to `report_dead_letter` — ⭐ **and that report never acknowledges its own journal row, measured as a CONTRAST over a population of two**: `Node::emit_event` runs `record_outgoing_event` -> `send_event` -> `acknowledge_event`; `Worker::report_dead_letter` runs `record_outgoing_event` -> `send_event` -> `eprintln!`. Two of two is the whole population. ⚠️ **One clause NARROWED rather than confirmed**: `OutcomeUnknown` does propagate out of `tick()` as an error, and the type's own doc says that is deliberate — "a fact, not a retry recommendation" — so whether a caller's loop re-dispatches in-process is NOT established. ⚠️ **And the ranking's own limit, measured**: the narrowest CANDIDATE `.10.1` is the OWNER of only **1 of these 5** records — `R-36-39-8` lands on `.7.3`, three land on `.4.4` — which is the clearest instance yet that a candidate list is a suggestion. The ranking still did its job: it groups records that must be READ together, not records that will be OWNED together. **22 clauses** (16 `owned`, 6 `attach`, 0 `unowned`); **179 ledger rows clean / 49 self-test controls**; ⭐ `ATTACH-LANDED` fired on this leaf's six unattached clauses before they were written — the THIRD consecutive tranche. **Promoted:** prefer a ratio, a single-hit grep or a two-site contrast to a reading (`TOOLBOX.md`). 18 doctrines green; no product code, schema, test or script changed.
🔴 **The MCP read surface leaks across tenants on ALL THREE of its read tools, and the module's own first paragraph says it does not; tranche 3b closed under `.11.9.1.2.2` (REPAIR-0178).** `crates/reasonbraid-mcp/src/lib.rs:1`–`:13` opens by claiming *"the READ tools over the inspection verbs — the SAME queries + the SAME authorization as the HTTP handlers"* and *"every read runs the reader classification (the per-reader visibility the HTTP surface enforces)"*. Measured against all three: ⛔ **`get_policy_bundle` is the worst and worse than the record said** — it never reads its own `principal` argument, and `server::policy_bundle(pool, _tenant_id)` takes the tenant as an UNDERSCORE-PREFIXED UNUSED PARAMETER over `SELECT policy_id, version, digest, clauses FROM policy_versions ORDER BY policy_id, version`, **no `WHERE` clause at all**, then labels every tenant's policy clauses with the caller's own tenant id in the response envelope. ⛔ **CORRECTED by `.6.1.1` (REPAIR-0182):** `policy_versions` has NO tenant column — `git grep -n "tenant" -- migrations/0038_policy_registry.sql crates/reasonbraid-server/src/policy.rs` returns rc=1 — so the registry is SITE-GLOBAL and HTTP `api.rs::list_policies` returns the same unfiltered set to any enrolled caller. The missing `WHERE`, the unchecked principal and the false tenant label all STAND; the cross-tenant FRAMING does not. The schema question is `.6.1.5`. ⭐ The underscore is the tell: the compiler was told the argument is unused, and the sentence at the top of the file says it is the authorization. ⛔ **`list_inbox`** declares `principal` in its params and its body never reads it. ⛔ **`get_thread`** DOES classify, gets `Network` for a foreign tenant, and returns the FULL projection anyway with `"visibility": "network"` printed beside it; no `ThreadInspect` grant is consulted anywhere. 🔴 **`join_call` is the THIRD instance of the two-caller-identifiers family**, after `.3.5.3` and `.3.5.4`: gated on the caller's own `tenant_id`, then `respond_to_call_core` fetches the call by `call_id` ALONE and never compares the two — and `decline`/`recommend`/`recuse` skip the eligibility gate entirely, so nothing else binds them. 🔴 **`policy.rs` queries `authority_grants` two different ways twenty-seven lines apart** — `:185` admits on `status = 'active'`, `:458`–`:459` requires `status = 'active' AND (expires_at IS NULL OR expires_at > now())` — so registration accepts an expired grant the resolver in the same file would refuse. ⚠️ And its selector default is fail-OPEN (`.unwrap_or("*")`) in the one place `.9.1`'s goal line says fail-CLOSED. ⚠️ **Two measurements NARROWED the record rather than confirming it, and both are recorded as narrowings**: the `body["tenant_id"]` panic is real but not reachable through the MCP tool, which passes a typed payload; and the quota-per-retry behaviour is stated verbatim in the module header, so the documentation is accurate today. ⭐ A CONTRAST rather than a count: `api.rs` calls `snapshots::submit` at four sites and exactly two — the R0 and R5 the record names — discard the `Result`. ⭐ **The ledger's FIRST `none` row lands here, completing the closed set**: all six states now have an instance. **27 clauses** (22 `owned`, 3 `attach`, 1 `none`, 0 `unowned`). ⭐ `ATTACH-LANDED` fired on this leaf's three unattached clauses before they were written — the SECOND consecutive tranche it has caught. 🔎 **And a signal that outgrew this leaf: `MEMORY.md` stands at 7,161 bytes against a 7,168-byte cap**, and three consecutive leaves have had to evict a standing warning to land. Routed to `.11.4.2` with its measurement; raising the cap is the one repair `MEMORY_ARCHITECTURE.md` forbids. **157 ledger rows clean / 49 self-test controls**, 18 doctrines green. No product code, schema, test or script changed.
🔴 **One missing predicate, three sites, two records, and no leaf had connected them; tranche 3a closed under `.11.9.1.2.1` (REPAIR-0177).** A lookup keyed on `operation_id`/`command_id` with the owning NODE absent from the predicate lives in three places: `node_channel.rs::event_id_for_operation` is `SELECT event_id FROM node_events WHERE operation_id = $1 ORDER BY received_at LIMIT 1` and is the reconciliation lookup the HANDSHAKE performs; the receipt insert is `ON CONFLICT (event_id) DO NOTHING` where migration 0003 makes `event_id` the whole primary key and `node_id` a plain column, so a node writing first under a known event id turns the rightful node's receipt into a silent no-op; and `migrations/0021_node_inbox_delivery_state.sql` calls an inbox row `consumed` on `e.operation_id = i.command_id` with no `e.node_id = i.node_id`. ⭐ Two of the three are `R-48-49-3`'s and the third is `R-85-1`'s — the ledger's fourth two-records-one-finding pair and the first whose shared finding is a SCHEMA shape rather than a code path. `SIGNOFF-REPAIR.4.3`'s goal line already demands the proof that would have caught all three: *prove command-id collisions across nodes cannot consume or hide foreign work*. 🔴 **And the cursor high-water mark is derived from a table that prune deletes from**: `enqueue` writes `MAX(cursor)+1` and `current_cursor` reads `COALESCE(MAX(cursor), 0)` over `node_inbox`, so pruning everything rewinds it to 0, the next enqueue re-issues cursor 1, and a node that already acknowledged cursor 1 discards the work as seen — the `journal_lost`-class anomaly the module's own header names. ⚠️ The test that would catch it stops one step short: it leaves survivors 3, 4 and 5, a partial prefix that KEEPS the highest cursor, so `MAX` never has to answer for an empty inbox. ⭐ **A dated answer to a question the record asked with a question mark, and it is good news that expires.** `R-75-1` asks whether the hardcoded `expires_at` of `2026-09-15` lacks future validation. MEASURED on 2026-09-14 on both sides — `corrections::record_correction` never compares the field to `now()`, and `reviews::schedule_reviews` selects waivers with no expiry predicate — so the two `policy.rs` tests do NOT break tomorrow and the clause is `declined` as a present defect with the record's own conditional standing. 🔴 The same measurement found what the record did not state: `repeated_waiver` fires on waivers that lapsed long ago, and on the FIRST waiver, since one row yields one pair. Written into `.9.3`. ⭐ **`ATTACH-LANDED`, registered one commit earlier, fired on this leaf's own three unattached clauses before they were written** — naming `R-70-3`/`.4.3`, `R-75-1`/`.9.3` and `R-85-1`/`.4.1`, rc=1 — and went green once each sentence landed. The gate's first production use caught the exact failure it was built for, in the very next tranche. **20 clauses** (15 `owned`, 3 `attach`, 1 `handled`, 1 `declined`, **0 `unowned`**); tranche 3 sized at 5,041 characters and split 1,624 / 1,655 / 1,762 on the same narrowest-candidate boundary that formed it. **130 ledger rows clean / 49 self-test controls**, 18 doctrines green. No product code, schema, test or script changed.
⭐ **The one ledger state that could not check itself is now a gate, and the "would have fired" claim was tested rather than reasoned; `.11.11` closed under REPAIR-0176 and registered as `ATTACH-LANDED`.** `attach` is the only one of the reconciliation ledger's six states whose Next action is not "none": it needs a SENTENCE written into a leaf the classifier does not own, while every other property of a row — a real record, a state in the closed set, a real owner, no duplicate clause — is visible in the row itself. 🔴 That is why `SIGNOFF-REPAIR.11.9`'s own mechanism was live INSIDE the instrument built to stop it: tranche 1 classified three clauses `attach` and performed none of the three attachments, found only because a human read two leaves end to end. ⭐ **The leaf had PUBLISHED a prediction — "3 of 3 at tranche 1's close, 0 today" — before any instrument could test it, so it was tested at FOUR points by TWO independent routes**: the new refusal copied into a detached worktree at each tranche's closing commit, and a throwaway re-implementation over `git show` written before the instrument was touched. Both report **3 breaching of 3** at `5862837`, then **0 of 7**, **0 of 15** and **0 of 26** at `b227ce8`, `887d050` and `ebe363b` — and the three it names are exactly `R-53-4` clauses 2 and 3 on `.7.1` and `R-63-1` clause 4 on `.11.4`, the set found by hand. ⭐ **That settles the shape question on the measurement, for the second time in this family and with the opposite answer**: zero today, every historical instance caught, population bounded by one tracked file — `REASON-CODE-DOC`'s shape. `.11.9`'s rejected gate would have fired on 114 of 131 the day it was registered. ⛔ **The rule is the RECORD ID in the OWNER'S OWN SECTION, never a phrase** — a paraphrase matcher is `.11.6`'s measured failure mode — and it deliberately does NOT require the words `ATTACHED CLAUSE`, which are a formatting convention rather than a contract. ⚠️ The id carries a right-hand boundary because `R-53-4` is a prefix of `R-53-41`, and a leaf that attached the longer must not read as having attached the shorter. 🔴 FALSIFIED twice and observed RED both times, because a gate never seen red is not known to gate: removing one record id from `.5.2`'s attached block returned rc=1 naming the row, and the same neutralization on `.11.4` made the REGISTERED enforcer exit 1; both restored to rc=0. **49 self-test controls** (42 before — seven new, fired in BOTH directions including the prefix boundary, the absent-section case and the skipped-without-sections contract), **110 ledger rows clean**, and the enforcer green at its unchanged **18** top-level checks — `ATTACH-LANDED` runs inside `PROJECT-SPECIFIC`, taking that slot from 7 sub-checks to 8, and is the registry's 24th named doctrine. No product code, schema or test changed.

🔴 **The gate built to catch silently-dropped table cells disagrees with the renderer that publishes this book — and two tracked rows are losing content right now while it reports zero; tranche 2c closed under `.11.9.1.1.3` (REPAIR-0175).** ⛔ **ASKED THE RENDERER, not the specification**, which is this leaf's promoted method and the reason the answer is trustworthy: mdbook/pulldown-cmark, the renderer this project's own book is built with, was handed a two-column table whose row is `` \| `x \| y` \| 2 \| `` and emitted **2 cells — `` `x `` and `` y` `` — discarding the `2` entirely**. `check_table_arity.sh:39`–`:44` models the opposite (a pipe inside a code span is part of the cell), and its `--self-test` arm at `:73` asserts **0** defects for exactly that shape, so the false green is written into the control. ⚠️ **A self-test written alongside the code shares its blind spots** — `TOOLBOX.md`'s existing measured rule, confirmed a second time on a different instrument. **Census over all 322 tracked markdown files, both rules side by side:** current rule **0** defective rows, GFM rule **2** — and both were then rendered to confirm. `DOCTRINE_ENFORCEMENT.md:39` renders its third cell as a bare `—`, so **the doctrine registry's published page never names the enforcer for `INDEX-FRONTIER`** and its rationale is cut mid-sentence; `RECONCILIATION.md:111` truncated its Evidence cell at "three `.map_err(". ⭐ The second was in this leaf's OWN deliverable, written one commit before the instrument that would have caught it was measured; it is repaired here with its superseded text named. ⛔ `DOCTRINE_ENFORCEMENT.md:39` is deliberately LEFT as the last real-world specimen in 322 files, so `.11.2` can falsify its parser repair against a row it did not write. **The other five measurements, each one changing or sharpening the record that prompted it:** 🔴 `pipefail` does NOT fix the demo's negative control — the four-way truth table is rc=0 in ALL cells, because `!` negates a pipeline whose status is 1 whether the CLI failed or merely matched nothing, so the record's DEFECT is right and its REMEDY is refuted (the fix is to check the producer's status separately). 🔴 `load_harness.sh` prints `PASS: every command committed` for `--commands 0` and `--commands -5` having issued **zero** requests, and issues **104** for a requested 100. 🔴 **7 of 9** staged-list-consuming doctrine checks read the WORKTREE, not the staged blob — `check_task_tree_ownership` among them. 🔴 `check_docpaths.sh` matches `/Users` and `/home` only, so it cannot see `/Volumes`, this checkout's own prefix. 🔴 The handoff census prints `handoff: OK` at rc=0 when both its censuses fail, and its `ps -Ao` arm covers **620 rows across 40 uids** where its own documentation claims one. ⛔ **REFUTED and recorded:** `R-87-1`'s "this active project retains `MAINTAINING.md`" is false and was false at the baseline (`git ls-tree 9c2d2ba`), so the sentinel finding stands and the framing that put this repository in the blast radius does not. **38 clauses** — 21 `owned`, 11 `attach`, 6 `handled`, **0 `unowned`** — closing tranche 2 at 14 records and 84 clauses. ⭐ **No new leaf was opened**, because `.11.2` and `.11.3` already own every surface these four records touch. 🔴 **11 `attach` of 38, against 7 of 72 in the three earlier tranches, and the cause is structural**: these leaves' goal lines are lists of NAMED MECHANISMS, so anything inside the surface but outside the list is invisible to a census driven by them; the earlier leaves state PROPERTIES, which generalise. All 11 written into `.5.2`, `.11.2` and `.11.3` in this commit; **26 of 26** `attach` rows verified present by hand. **110 ledger rows clean / 42 self-test controls** rc=0; 18 doctrines green. ⛔ No product code, schema, test or script changed.

🔴 **An enrolment request PANICKED the handler and the node got a dropped connection instead of an answer; repaired under `.4.1.6` (REPAIR-0174).** REPRODUCED end to end through the supported routes before one line changed: issuance `200` for a non-ASCII host claim, then `panicked at crates/reasonbraid-server/src/ca.rs:142:75` — the `expect("leaf params")` — with `PROBE enrol TRANSPORT ERROR`, `server afterwards: still serving`, `nodes rows: 0`, `token used_at: Some(None)`. ⭐ **The blast radius is MEASURED, not inferred**, and the leaf's opening inference (process survives, connection drops, transaction rolls back) is replaced by the measurement rather than left standing beside it. 🔴 **And the measurement found the consequence the inference had NOT reached: the token is left UNCONSUMED**, and an outstanding unused token refuses a second issuance for the same node id — so that node id cannot be enrolled at all until the token lapses. ⚠️ Bounded **to the token's lifetime (default 1 h, max 1 day)**, and bounded ONLY because `.4.1.1` already supersedes a lapsed token; before that leaf it would have been permanent. ⛔ An availability defect, stated as the bound it is. **FIX, two parts:** `issue_node_leaf` returns `Result<IssuedLeaf, HostClaimRefused>` (⭐ `.4.2.7`'s promoted rule — *a signature is a promise the body must keep* — on a second surface; both call sites map it to a typed `400`), and the ISSUANCE route checks the claim beside the node-id shape and lifetime range, **because that is where a human typed it**, which makes the unenrollable state unreachable rather than merely survivable. ⛔ **The accept set is NOT narrowed, and that is a DECISION**: `check_host_claim` is `CertificateParams::new` and nothing else, so it restates no grammar and cannot drift from what issuance will do; a stricter DNS grammar is DEFERRED with its compatibility question named (`1.2.3.4` and `*.example.com` are plausible operator inputs) — `docs/decisions/2026-09-14_host-claim-checked-at-issuance.md`. ⛔ The other four `expect`s in that function take server-controlled values and are untouched. ⭐ **FALSIFIED each half SEPARATELY, and the two neutralizations hit DISJOINT controls**: removing the issuance check gave `18 passed; 1 failed` (`left: 200, right: 400`); restoring the `expect` under the unchanged signature gave `19 passed; 2 failed`, both new controls panicking at `ca.rs:194` and the redemption one also failing at the test client's own `expect("request")` — the transport error reproduced a second time. ⭐ The issuance control runs its POSITIVE arm FIRST and then enrols the SAME node id with a good claim, which is what distinguishes this repair from a merely later refusal; the redemption control writes its token row directly, after ASSERTING that the route will not mint it. ⚠️ One control uses `.err().expect(...)` rather than `expect_err`: `IssuedLeaf` carries `key_der` and derives no `Debug` (§16.5), and deriving one to make a test compile would be a secret-containment regression bought with convenience. **82 live tests / 5 suites + 103 lib tests** rc=0; strict all-target server lint rc=0. One documented wire narrowing on `POST /v1/nodes/enroll-tokens`; no migration, no schema change.

⭐ **The ledger's FIRST `declined` row, and a probe that inverted the record it was checking; tranche 2b closed under `.11.9.1.1.2` (REPAIR-0173).** **19 clauses** over four records: 1 `handled`, 8 `owned`, 8 `attach`, 1 `unowned`, **1 `declined`** — the state written into the vocabulary before any instance existed, precisely because a deliberate rejection is invisible to every search. ⛔ `R-51-2` clause 4 is declined **as stated**: the mechanism is real (a VISIBLE `Option::None` serialises to `null`, no `skip_serializing_if`), the contradiction is not — the module's only wire-absence claim says "a hidden field is ABSENT, never nulled", which concerns HIDDEN fields and stays true; absent means hidden, `null` means visible-and-unset, and the two stay distinguishable. 🔴 **The `unowned` clause is a reachable panic, and the MEASUREMENT inverted the record's framing.** `issue_node_leaf` does `CertificateParams::new(vec![host_claim]).expect("leaf params")`, and `host_claim` is bound unvalidated from `POST /v1/nodes/enroll-token` into the token row, required only to MATCH at redemption, then handed to that `expect` — and `issue_node_leaf` returns `IssuedLeaf`, not a `Result`, so the panic is the ONLY way it can report a bad claim (`.4.2.7`'s promoted rule on a second surface). The record said "malformed host", so a probe asked rcgen 0.14.10 what malformed means: of **eight** inputs exactly **one** returned `Err` — `héllo`. `""`, 300 characters, `not a dns name!!`, `*.example.com`, `1.2.3.4` and `..` were all ACCEPTED. ⭐ **Two findings pointing opposite ways**: the panic is real and far narrower than "malformed" suggests, and the accept set is far WIDER than a DNS name, so the certificate's host binding is effectively unvalidated — the larger finding, and not what the record was pointing at. ⚠️ **Eight inputs are a SAMPLE, not the boundary**: "only non-ASCII panics" is NOT established, and the new owner `.4.1.6` says so in those words, with the blast radius explicitly NOT claimed (no `[profile]`, no `CatchPanicLayer` — both measured; what that yields at runtime is an inference, and `.4.2.7`'s cost was a whole process). ⭐ **The third two-records-one-finding pair, caught in ADVANCE**: `R-52-2` clause 2 and `R-80-82-1` clause 5 are the same stranded publisher CAS retry, and the split note had told this child to read the tranche-2a row first. 🔴 **All 8 `attach` clauses written into their leaves IN THIS COMMIT**, the rule added one commit ago: `.10.2` (a completion invariant that accepts a FAILED terminal; an unbounded `drain` in the harness a hostile adapter meets first), `.4.1` (the CA signed for 365 days with no renewal and no check at load; a leaf never compared against its issuer's `not_after`), `.5.1` (the visibility DOC says every field defaults to `self_only` while `Default` publishes 11 of 14 to tenant or network; `field_visible`'s doc example reversed beside correct code; `ReaderClass::Full` never receiving `incarnation_id` or `visibility` despite "the full profile"), `.9.2` (the publication commit embeds the wall clock, so its object id changes every second, against its own "idempotent re-write"). ⭐ One `handled` row is answered by the SOURCE rather than the tree: `.4.2.7` DELETED `ca.rs`'s `from_hex` and left the reason in the module. **72 ledger rows clean / 42 self-test controls** rc=0; uncited 106 -> 102; 18 doctrines green. ⛔ No product code, test or script changed; the probe was untracked and removed.

🔴 **A failed database read was delivered to the caller as a verdict about the site's configuration; repaired under `.11.10` (REPAIR-0172).** REPRODUCED before anything changed, with the probe declaring its own premise first: `dev-local` IS declared (migration 0053 seeds it), and with the pool closed `route` answered `UndeclaredRegion { region: "dev-local" }` — *"the region `dev-local` is undeclared — the routing refuses"*. ⭐ **A statement about the site, produced by a failure that touched no site state at all**, which would send an operator to inspect declarations that are perfectly correct. Root cause: three `.map_err(|_| RegionRefusal::…)` converting ANY `sqlx::Error` into a policy verdict. ⭐ **The repair already existed one module away, with its reason written down**: `.3.2.1` drew the same line for `pair` in the site-authority service — *"an unavailable database must never masquerade as an undeclared region"* — and `R-53-2` named `pair` AND `route`; only `pair` was reached. Fix: a `RouteError` carrying `Refused(RegionRefusal)` and `Storage(sqlx::Error)` apart, the three reads propagating with `?`. ⛔ **`RegionRefusal` is byte-identical** — both variants, their fields and their `Display` wording — because `PHASE-8.5.2`'s acceptance and the operator contract rest on those names; ⛔ the `Option<bool>`/`unwrap_or(false)` decode is deliberately untouched too (a different clause). The storage message names neither the region nor "undeclared", so a caller that logs only the `Display` is not still told the declaration is missing. ⚠️ **Reachability MEASURED and stated the honest way round: `route` has NO production caller today** — five test callers, and the seam's own comment names `PHASE-8.5.3`'s store-and-forward as the future one; opened now precisely because an inheriting leaf censuses its own goal line and would not read this (`.11.9`'s mechanism). ⭐ **FALSIFIED against the ACTUAL superseded mechanism**: a type-changing repair cannot be falsified by reverting the file (the control would not compile), so the neutralization kept the new signature and restored ONLY the `map_err` — **`2 passed; 1 failed`**, the single failure this leaf's control, naming `Refused(UndeclaredRegion { region: "dev-local" })`, with the other two suite tests still GREEN (narrow enough to be the defect, not something else). ⭐ The control's positive arm runs FIRST on a live pool, so a repair that merely stopped refusing cannot pass. **3/3 regions tests** rc=0, both owned clusters stopped and removed. ⛔ No migration, no wire change, no HTTP route.

🔴 **The ledger recorded three clauses as "this leaf will drop it" and then let the leaf keep them droppable; fixed, with tranche 2 sized and split, under `.11.9.1.1` / `.11.9.1.1.1` (REPAIR-0171).** ⭐ **The split is DERIVED, not felt**: tranche 2 is **14 records / 7,192 characters / ~57 sentence-clauses** against tranche 1's **7 / 2,554 / ~19** — **2.8x by character** at an acceptance demanding the SOURCE read per clause — so it splits three ways on each record's own narrowest candidate, the same measurement that formed the tranche (2,781 / 1,405 / 3,006 chars). 🔴 **The heaviest finding is about the instrument itself: tranche 1 classified three clauses `attach` — the state whose Next action is *attach it or the split drops it* — and performed NONE of the three attachments.** `.7.1` carried no word about the caller-declared `scheme` column, `.11.4` none about `escalation.rs`'s now-false header. `.11.9`'s defect was live INSIDE the instrument built to stop it; `attach` is the only one of six states whose action is not "none", which is why it was easy to miss. **All 7 attached** (3 from tranche 1, 4 new) into `.7.1`, `.8.1`, `.11.4`, and the vocabulary now says **in the commit that classifies it**. ⛔ **A published population is CORRECTED, not withdrawn: 8 of the 112** (the count when this leaf opened; it now reads **106**, because the seven attachments made three leaves NAME six records, which then left the set — the convergence measured rather than asserted) uncited records ARE named in the tree, in text belonging to no leaf — six are the rows of `#### Historical census dispositions for .2.2`, one sits in `## Leaf .3.3.4.1 closure evidence` and one in `## Commit acceptance — SIGNOFF-REPAIR.11.9.1`; three of them are this tranche's own. 🔴 **CORRECTED 2026-09-14 under the director's grading question**: the first version located these by LINE NUMBER and two of the eight were STALE before the session ended (`:3747`→`:3972`, `:4791`→`:5040`) — a line number into a growing file is `LIVE-DOC-CURRENCY`'s failure in another costume. ⚠️ The reproduction also reports **13** today, not 8, because a `## Commit acceptance` heading is not a leaf heading either and five of this session's own blocks name records: **8** is a true statement about the population at leaf-open, NOT a stable quantity. ⚠️ The instrument's RULE (a leaf's own section) is NOT wrong; the available CONCLUSION was — "114 uncited" is not "114 nobody re-read". **Tranche 2a: 6 records / 27 clauses** — 9 `handled`, 13 `owned`, 4 `attach`, 1 `unowned`, every one confirmed at the SOURCE. 🔴 **Four are live defects**: `POST /v1/resolvers` still admits on tenant administration and upserts a site-global row by `resolver_id` alone (a built-in pack's declared classes are REPLACEABLE); `POST /v1/workflow-profiles` admits on **enrolment alone** and resolution takes `ORDER BY version DESC`, so any enrolled principal shadows a §13.1 built-in site-wide; `reconciler::reconcile` reads `git.effective` in **no branch**, so §15.8's "effective ref moved → freeze" cannot fire (a unit test asserts immutable 7 + effective 8 is `Consistent`); and the decision-family close consults only the CALLER's `unresolved` list while `projection.open_challenges` is maintained and never read. ⛔ **None opened as new work** — `.7.1`, `.8.1` and `.9.2` already name them in their goal lines. ⭐ **`.3.2` is the mechanism caught at the leaf that ranked these six**: it pinned a rigorous closed set of SIX site actions and closed `done`, and BOTH records asking it to census the shared-registry write SCOPE first were uncited by it — a rigorous closed set is still the wrong one if its scope was never measured. New owner `.11.10`: `regions::route` turns any `sqlx::Error` into an undeclared-region verdict, the repair `.3.2.1` already wrote one module away and never applied here; ⚠️ reachability measured — **no production caller today**, five test callers, and `PHASE-8.5.3` is the lane that would inherit it. **53 ledger rows clean / 42 self-test controls** rc=0; 18 doctrines green. ⛔ No gate proposed and no product code touched.

🔴 **Three doctrine ENFORCERS were writing their own scratch off the repository volume, and the gate written for §13 could not see them; repaired and the gate extended under `.11.2.2` (REPAIR-0170).** Seven sites, all shell: `check_task_acceptance.sh`, `check_waiver_routing.sh` (two), `check_self_tests.sh`, `update_scaffold.sh`, and the two probe scripts that `git init` throwaway git repositories in their scratch. (⛔ CORRECTED 2026-09-13: the first wording said the probes CLONE. They do not — they `git init`, `add` and `commit`, which is `R-84-1`'s own phrase "create throwaway git repositories". The script that genuinely clones is `update_scaffold.sh`, `git clone --depth 1` at its line 24. The substance — whole git repositories written off-volume — is unchanged; the verb was a named instance and those are exact.) Locality MEASURED, not asserted: the checkout and `target/doctrine_scratch` are device `16777244`, the ambient directory `16777232`. 🔴 **`STORAGE-LOCALITY` enumerated `git ls-files -z -- "*.rs"`**, so the shell family was outside the scan by CONSTRUCTION rather than by exception — invisible, not waived. 🔴 **And this leaf's own published number was WRONG: 7 sites, not the 5 it opened with.** The opening census ran `git grep … | grep -v '^docs/'`, and that convenience filter excluded the directory holding the files `R-84-1` names (the record names them by ROLE, not by path) — it says "task-acceptance and waiver-routing **probe scripts**" and both spell `WORK=$(mktemp -d)` under `docs/tasks/artifacts/`. ⚠️ `MEMORY.md` carries that exact warning ("scope a census WHOLE", after a glob silently matched nothing) and it was breached anyway; corrected here rather than quietly used, with the superseded 5 left in place. **DECISION: the gate IS extended** to tracked `*.sh`/`*.py` — it fires on **0** today and would have fired on all **7**, the `REASON-CODE-DOC` shape and not `.11.9`'s rejected 114-of-131. ⛔ **The rule is about the ARGUMENT, not the call**: both tools name by exclusive creation, which is the half this doctrine wants, so a pattern flagging the call itself would condemn 15 conforming sites and teach bypass. ⭐ The one Python hit is a deliberate CONTROL — a probe that measures where a child's temporary file lands, which pinning `dir=` would turn into an assertion — and is a reviewed exception. 🔴 **The extended gate flagged ITSELF on the first run**, its diagnostic spelling the command it matched: the `SELF-TEST` founding incident repeating in a file that carries a `--self-test` of its own and still could not see it (⛔ CORRECTED: the first wording said the file's header CITES that incident; it does not — it names only its own `--self-test` flag), fixed by RE-WORDING rather than a path exclusion so it self-polices. FALSIFIED against the exact pre-repair sources: all **7** named, rc=1; restored rc=0. **265 files scan clean, 22 self-test classifications** (8 before), both probes green (10/0 and 5/0). ⛔ NOT claimed: a Makefile recipe, a CI step or a Rust `tempfile` call reaches none of the three patterns.

🔴 **A tenant administrator READ another tenant's node inbox — command ids, thread ids, delivery state and payloads; repaired under `.3.5.3` (REPAIR-0169).** REPRODUCED over the supported HTTP surface before anything was changed: an administrator of tenant A, naming its OWN tenant and tenant B's node id, received **both** of B's rows in full. ⭐ **Why it survived a repair that named its three siblings**: `R-31-32-1` named FOUR verbs, `.3.3.4.10`'s census scoped itself to node administrative MUTATIONS and measured "four of the five mutations carry no tenant predicate", and `.3.3.4.10.3` then bound the three inbox WRITES. The inspection is a READ, so it fell outside a census that was correct about its own scope and silent about the record's. ⛔ Found by `.11.9.1`'s clause reconciliation, not by a test — no leaf had ever cited that record. ⭐ **The defect stated so it generalises: the caller supplies TWO independent identifiers and the handler checked one of them.** The admission proved the caller administers the tenant it NAMED; the select asked only for the node id. **census of all 24 GET routes before repairing, and its 3 hits CLASSIFIED down to 1**: one is this defect, one is `admin_metrics` (already `.3.5.2.1`, deliberately process-global), and one — `inspect_call` — is a FALSE POSITIVE that produced the sharper rule, because it loads the call FIRST and authorises against `call.tenant_id`, deriving the tenant from the target instead of accepting it. ⚠️ **Census limit stated, not hidden: 10 of the 24 handlers hold no SQL of their own** and delegate to a module the scan cannot follow — unmeasured, not clean, and now owned by `.3.5.4`. Fix: `AND tenant_id = $2` in the select, the sibling mutations' shape; ⛔ deriving the tenant from the node was REJECTED here because removing a required parameter is a wire change on a public route, the class `.3.5.2.1` already holds for a director decision; ⛔ no transaction and no guard, because a read has no check-then-act. ⭐ **FALSIFIED in the strongest form available — the control was written BEFORE the fix, so the pre-repair run IS the neutralized build** (`7 passed; 1 failed`), with nothing reverted or reconstructed and therefore none of `.4.2.8`'s "broke a different thing" hazard. The control asserts the absence TWICE (empty rows AND no victim command id anywhere in the text) and carries a positive arm, so a fix that merely emptied the result fails it. **4 suites / 55 tests** rc=0; strict server lint rc=0. ⚠️ **A third finding, in prose rather than code**: the sibling control's own comment claimed it covered "read, move or destroy … all three" and asserted two, and that same sentence had propagated into `cli.md` ("all three verbs" under a block showing four) and `authority.md` ("Three operator verbs"). All three corrected — a description of a check is not its coverage.

🔴 **A leaf's own ranking instruction was REJECTED by its own census, and classifying seven records found TWO live defects; closed under `.11.9.1` (REPAIR-0168).** `.11.9.1` said to start from the SMALL-fan-out records, "there the routing was a genuine assignment". ⭐ **Measured, the two orders are ANTI-correlated**: all five fan-out-1 records point at a CONTAINER leaf (`.11.4` is named by **99 of 131** records, `.3.3` 41, `.4.1` 37), while every record with fan-out 9 or more reaches a leaf named by 10 or fewer. ⭐ **`R-6-27-1` settles it** — its entire body is the artifact's boilerplate, it carries **no finding at all**, and it still received a candidate leaf. Fan-out measures how sure the REVIEWER was; what an uncited routing being a real miss depends on is whether the TARGET needed the record. ⚠️ **Auditor's asymmetry honoured**: `.11.9`'s statement is not called wrong, its question is NAMED as a different one. **Adopted ranking: the narrowest candidate's frequency** (`--rank`), and the 107 remaining records split into six tranches at that distribution's natural gaps. 🔴 **Tranche 1's 26 clauses produced two LIVE findings, each with a new owner.** (1) `inspect_node_inbox` — the FOURTH verb `R-31-32-1` named — admits on the caller's tenant and then selects from `node_inbox_state` by `node_id` alone, so a tenant administrator reads another tenant's command ids, thread ids and payloads; `.3.3.4.10`'s census scoped itself to MUTATIONS and repaired the other three. Owner `.3.5.3`. (2) Three doctrine ENFORCERS write scratch to ambient `TMPDIR`, and `STORAGE-LOCALITY` enumerates with `git ls-files -z -- "*.rs"` so it cannot see them; owner `.11.2.2`, opened with the raw 19 hits CLASSIFIED to 5 breaching (12 conforming Python pass a repository-derived `dir=`). ⭐ **The vocabulary gained a state the acceptance did not have — `attach`**: a clause whose owner's OWN TEXT does not make it visible, which is `.11.9`'s defect caught BEFORE it costs anything. 3 rows carry it. ⚠️ `declined` carries 0 and says so: it is invisible to every search, so it exists before an instance does. 🔴 **The instrument was wrong a FIFTH time and this leaf's own prose exposed it** — the elided-citation rule matched any `:N`, so source line numbers written after a census citation inherited its filename, silently moving 114 to 111; now anchored to its opening backtick, with two controls. 🔴 **The frontier's own pending-count command was also wrong**: it matched `- Status: pending` and missed the **11** leaves carrying only `- Opened: pending`, publishing 36 where the tree holds **46**. **42 controls** rc=0, ledger clean. No product code touched.

🔴 **A REPLACEMENT handed the machine it was replacing its session back, reverted its certificate's host, and never closed its incarnation; all three repaired under `.4.1.5` (REPAIR-0167).** Each REPRODUCED in its OWN run before being fixed: `heartbeat after the replacement: 200`; `nodes.host_id … names: host-a` after moving to host-b; `incarnations: 2 open of 2`. 🔴 **The severe one PARTIALLY UNDID `.4.1.3`**: that leaf's renewal guard asks *"does a usable certificate exist for this node id?"*, and a replacement issues one **for that same id**, so the REPLACED process's predicate becomes true again. ⚠️ Bounded to a replacement inside `LEASE_TTL` of the revocation — the ordinary operational case. 🔴 **The host**: `rotate` reads the SAN's host from `nodes.host_id`, so the first AUTOMATIC rotation (within half a leaf lifetime) silently reverted the certificate to the machine the node no longer runs on; the control drives a real rotation and reads `SAN: [DNSName("host-b")]`. 🔴 **The incarnation**: ⛔ the writer's own comment (*"re-enrollment cannot duplicate — the `nodes` primary key refuses a second enroll"*) is FALSE for a replacement and is the sentence that hid it; corrected in place. ⚠️ Its consequence MEASURED and narrower than implied: both selectors already `ORDER BY valid_from DESC LIMIT 1`, so the defect is the **LEDGER**, not the selection. ⛔ **No property traded**: `.4.1.3.1`'s withheld tail still replays — the control asserts the replacement takes its own lease in the same run, and the ritual passes unchanged. ⭐ The epoch is BUMPED not deleted, because deleting resets it to 1 and breaks the monotonicity `.4.2.3`'s argument rests on. FALSIFIED each part SEPARATELY (`1 passed; 1 failed` ×3, each naming its own clause). **72 tests / 5 live suites** rc=0. 🔎 **Carry this**: a guard asking *"does a usable credential exist for this id?"* is satisfied by ANY such credential, including one issued after the event it defends against.

⚠️ **The wake gate is a DRAIN SWITCH, not a concurrency limiter — stated for the first time, and every state it has now has a control; closed under `.4.2.10` (REPAIR-0166).** census taken from the FILTER rather than the tests: **5** reachable states, of which the existing control covered **2**. ⭐ **The three uncovered ones all mean "deliver" and each reaches that answer by a DIFFERENT SQL route** (missing `concurrency` key → `NULL = 0` is NULL; missing `availability` block → the whole path NULL; a negative number → simply not zero), so one does not stand for the others. ⭐ **The drain state is re-asserted LAST**, so four positive arms cannot all be vacuous — `.4.2.6`'s lesson applied to a filter. 🔴 **The substantive answer is a CLAIM NARROWED, not a defect repaired**: the gate tests for exactly zero and never compares a declared concurrency against an ACTIVE count, so `concurrency: 2` does not cap a node at two; `presence_state` already names zero `Draining`, so this is the design and the book now says so plus what an operator must enforce themselves. ⛔ **My own hypothesis MEASURED AND REFUTED**: the `::bigint` cast at **6** query sites would raise on a non-numeric value, but the field is typed `Option<i64>` and the writer stores the RE-SERIALIZED struct, not the raw body — unreachable. FALSIFIED by removing the gate (`36 passed; 1 failed`, only this control). **37/37** rc=0. ⛔ No production code changed.

🔴 **A node that rotated its certificate and then stayed DOWN past its old one's expiry was permanently locked out; repaired under `.4.2.9` (REPAIR-0165).** CONFIRMED and LIVE: `handshake` rotates automatically when the leaf nears expiry and `install_identity` wrote only memory; census of the identity-file writers finds ONE, `save_workload_identity`, which takes an *enroll response* and runs once at enrollment. ⭐ **The BOUND is the work, and a guess would have been wrong BOTH ways**: a restart usually RECOVERS (stale cert loads, rotation window still open, next handshake rotates again) — but a node down past that certificate's expiry (≤ `ROTATE_REMAINING_SECS` = 300 s on a 600 s leaf) cannot rotate out, because `rotate` REQUIRES a usable certificate. Operator-issued token needed: **`.4.1.1`'s lockout class through a different door**. **DECISION: persistence belongs to the CALLER** — the channel does no file I/O and does not start; `Node::open` (which has the journal path) writes the exact two files the loader reads, and the control asserts the FILENAMES, because "a sink fired" would pass with a sink writing anywhere. ⭐ **PERSIST FIRST, then swap memory**: a crash between them leaves the NEW identity on disk, which the next start corrects; the reverse would reintroduce this very defect. A sink failure is reported, NOT fatal. FALSIFIED by removing the sink call (`29 passed; 3 failed`, exactly the three new controls, the node-level one reporting `cert.der … NotFound`). **76 node tests + 46 live channel/replacement/work tests** rc=0.

🔴 **The node carried a fencing token from one generation with the epoch of another; repaired under `.4.2.8` (REPAIR-0164).** REPRODUCED with a driven interleaving over self-describing generations: **`242 torn of 40,000`** reads, first `("fnc_444", 455)`. ⭐ **Why it looked safe: the WRITE was atomic** — `handshake` took both locks in one scope, so two writers could never interleave; but all **4** fenced read paths took two SEPARATE acquisitions, and a reader straddles a complete write. ⚠️ **Bound stated BEFORE the work and unchanged: an AVAILABILITY defect, not a fencing bypass** — the server refuses the mismatched pair, so the cost is a spurious `401` and a reconnect; repaired because a request that cannot possibly succeed should not be constructible. ⭐ **0.6 % of reads under maximal contention** explains why it never surfaced: it presents as an unexplained `401` followed by a successful reconnect, indistinguishable from a network blip. Fix: ONE field `lease: Arc<Mutex<Option<Lease>>>`, one acquisition — the mixed state is now **UNREPRESENTABLE**. ⛔ A "take both locks" helper and accepting the window were both rejected with reasons. 🔴 **The FIRST falsification was WRONG and is recorded**: it split the WRITE (which the old code never did) and failed ten times louder (`4193 torn`); redone against the true superseded mechanism, the split READ, `242 torn`. A neutralization that exaggerates the defect is not evidence for the repair. **72 node tests + 53 live channel tests** rc=0; no wire field, route or documented behaviour changed.

🔴 **`.4.2`'s split read all 15 routed records and STILL dropped six clauses; parent closed under `.4.2` (REPAIR-0163).** ⭐ **The citation claim is MEASURED for the first time: 15 routings, 15 cited, 0 uncited** — against `.4.1`'s **36 uncited**, the leaf whose dropped clause started the reconciliation question. 🔴 **But citation is not accounting**: a clause-by-clause pass over the record BODIES found **SIX clauses the split did not carry**, in the very leaf that had avoided the citation form of the defect — the strongest possible evidence for `.11.9`'s reframing that the unit is *a clause with an owner*, not *a record with a citation*. **All six now owned**: ⚠️ **two CONFIRMED at the source** — the node reads its fencing token and lease epoch under **TWO separate mutexes** (`.4.2.8`; an AVAILABILITY defect, not a fencing bypass — the bound stated BEFORE the work), and `install_identity` writes only in memory so a rotation may not survive a restart (`.4.2.9`); ⛔ **four recorded as the REVIEWER's claims** under the reproduce-first prohibition — the wake gate's positive-concurrency coverage (`.4.2.10`) and three replacement clauses grouped as ONE question about what a replacement ENDS (`.4.1.5`, flagged against `.4.1.3.1`'s deliberately preserved tail). Every remaining clause of all 15 records is dispositioned. ⛔ **The pending count ROSE 36 → 40, deliberately** — a reconciliation that finds work makes the tree larger, and hiding that would be the defect. Documentation only; no code touched.

⛔ **The reason-code "drift" is mostly REFUTED by the code's own doc; the real gap was that the book had NO error page — closed under `.11.7` (REPAIR-0162).** ⛔ **The leaf's own numbers were wrong: 18 emitted and 9 unregistered, not 19 and 10** — `unknown` is a CLIENT-side sentinel built when a body will not parse and never travels the wire; a search's hits published as a defect count, in a leaf about a published set nobody re-derived. ⛔ **The 11 unemitted registry codes are DELIBERATE and already documented** ("the *complete* §9.8 list, not a Phase-0 subset") — **all 11 KEPT**. ⛔ The 9 unregistered emitted codes are preserved verbatim by `ReasonCode::Unknown`, the designed forward-compatibility path. 🔴 **The real gap: nothing published what the product emits.** DELIVERED `docs/book/src/errors.md` — all 18 with status, registry status and meaning, derived from the sources; the 11 with their reason; four client rules (incl. `commit_outcome_unconfirmed` means UNKNOWN, not failed). ⭐ **A gate IS registered here — `REASON-CODE-DOC` — one commit after rejecting one of the SAME RULE SHAPE**: it fires on **0** breaches today and would have fired on all 9, so it catches the NEXT drift; `.11.9`'s would have fired on 114 of 131 and was rejected on exactly this test. ⛔ Registry NOT extended and no emitter changed (its contract mirrors §9.8, which is in a roadmap FROZEN at v0.4.1); the §9.8 question is routed to `.11.7.1` WITH the measurement, because the freeze says a version bump must cite this kind of evidence. FALSIFIED (an undocumented code -> named, rc=1). **19 checks / 14 self-test controls** rc=0. No product code changed.

🔴 **An AGENT ROLE granted `tenant_admin` can issue node enrollment tokens and revoke nodes — the documentation said "an authorized human" and the code never checked; DECIDED under `.4.1.4` (REPAIR-0161).** MEASURED by driving the live route: `200`, with the ledger recording `authority_grants.subject_kind = 'role'`. **DECISION: narrow the CLAIM, not the code**, on four independent lines — §16.2 names no issuer restriction, §16.4 specifies authorization over typed actions/resources, §16.3 says "a human, service, or agent may delegate"; and across **117** `resolve_principal` sites authorization NEVER depends on principal KIND (every branch selects which identity TABLE to read). ⭐ **The decisive evidence was an ORACLE NOBODY BUILT for it**: adding the kind check fails **3** controls, **2 of them pre-existing**, which already drive an agent role here and require GRANT-based adjudication — and one fails because a kind check refuses `401` BEFORE the authorization that writes the denial record, destroying the audit evidence. ⚠️ **Scope widened by the leaf's own census**: 6 sites, not 2, including `POST /v1/nodes/revoke` with the same sentence and the same gate. ⚠️ **The governance consequence is now STATED in the book**: granting `tenant_admin` to an agent lets it extend the node population; a deployment that does not want that must WITHHOLD THE GRANT — narrowing the route would be a grant-model change, not an endpoint check. FALSIFIED (`15 passed; 3 failed`, naming the change). **5 suites / 91 tests** rc=0. ⛔ No product behaviour changed. Recorded as `docs/decisions/2026-09-13_issuance-is-a-grant-not-a-kind-of-principal.md`.

⛔ **The record-reconciliation gate is REJECTED on its own census, and the leaf's own number was wrong; closed under `.11.9` (REPAIR-0160).** ⛔ **131 review records, not the 53 the leaf claimed** — 53 is the artifact DIRECTORY's file count, and `MEMORY.md` has carried 131 correctly since `.1`, so two live documents disagreed unnoticed in the leaf whose subject is exactly that. **Census: 131 records / 614 routings / 301 to a SPLIT leaf / 16 cited / 285 uncited; at RECORD granularity 17 of 131 cited, 114 by none.** 🔴 **The gate would fire on 114 of 131 on day one** — a backlog wearing a gate's clothes. ⭐ **The census changed the QUESTION**: fan-out is a MEDIAN of 4–5 candidate leaves per record, so a reviewer naming five was writing a SUGGESTION LIST, not five assignments; "a leaf must account for every record routed to it" is unsound at the population level. 🔴 **Which reframes the original defect**: not a missing citation but one record holding THREE findings of which ONE found an owner — the unit is a clause with an owner, and "accounted for" includes DELIBERATELY DECLINED, which no search sees. 🔴 **The instrument was WRONG FOUR TIMES**, each caught by a different control (70 of 131 — two id shapes; parent sections swallowing children; 3 of 19 citations — heading line vs line RANGE; and the elided `` `:N` `` form, caught only against the REAL tree). One control was itself wrong first. Discharged: the "no human restriction" clause now owns **`.4.1.4`** — `resolve_principal` returns `Human` OR `Role` and nothing asks which, while two doc sites say "an authorized human". The 114-record backlog is ROUTED to `.11.9.1` with its size. No product code touched.

⛔ **The `MEMORY.md` eviction worry is REFUTED by its own census; instrument delivered under `.11.4.2.1` (REPAIR-0159).** **26 standing warnings, ZERO existing only in `MEMORY.md`** — all 26 are recorded in a durable layer, so the implied rule ("every warning must first exist in a durable layer") would fire never. 🔴 **The instrument was WRONG TWICE first, and the sequence is the evidence**: 31 (split at every marker — over-count), 21 (split at a sentence terminator, which here sits INSIDE `**…**` — under-count), 26 (correct). ⚠️ **Its own `--self-test` passed throughout** because the fixtures shared the bug's idiom; reading the OUTPUT against the file is what caught it. ⛔ **Then the tool's verdict was itself a false claim** — it printed `ORPHAN (only in MEMORY.md): 13` and hand-classification found **13 of 13 recorded**; a search population published as a defect count, renamed `UNCITED` with the classification printed beneath it. 🔴 **The real finding is narrower: the POINTER is missing, not the record** — 13 of 26 name no leaf, so an eviction costs findability, not fact. ⛔ **NOT mechanized**: "a warning must name its leaf" would flag 3 legitimate ones (the derived frontier note, the `docs/knowledge/` pointer, the `project_env.py` rule) — `.11.4.5.3`'s shape a second time. ⭐ Ships instead: `scripts/census_memory_warnings.py`, tracked, 16-control `--self-test`, in `TOOLBOX.md`, run BEFORE evicting so the choice is derived. **Seven recorded cap crossings, four in this session — the cap was NOT raised.** No product code touched.

⛔ **The secret store's published scope was wider than the code in two of three sentences; narrowed under `.4.2.5` (REPAIR-0158).** ⛔ **The census REFUTED the leaf's own opening sentence** — exactly 2 `SELECT`s of key material exist in the workspace, one IS the store and the other a test fixture, so node keys are not read in production at all and the ONE key read IS routed. 🔴 **But it found the real defect: the seam is READ-ONLY** — the CA bootstrap `INSERT`s into `server_ca` DIRECTLY and then reads back through the store with an `expect`, an invariant only while both ends are one database. ⭐ So the boldest published sentence is the false one: *"the external store arrives as a configuration change, not a code migration"* is backwards — the first external profile ABORTS the server on first boot. *"every secret read goes through the declared profile"* is also false (the enrollment token is read straight from its table). census of published copies: the book carries **none** — measured, so there is no book drift. **DECISION: narrow, not widen** — a write path for a backend that does not exist is unfalsifiable behind a one-profile registry (`.4.2.6`'s lesson). ⭐ **The narrowing is a TRIPWIRE, not a comment**: a test pins the profile count with the reason in its assertion message, so a second profile cannot be added without confronting the unrouted write. ⛔ No runtime change; the decision record gains a Correction that PRESERVES the original wording. FALSIFIED by declaring a second profile (`3 passed; 1 failed`). **103 lib tests** rc=0.

🔴 **A malformed server response ABORTED the node process where the signature promised an error; repaired under `.4.2.7` (REPAIR-0157).** REPRODUCED with a driven decode: `end byte index 2 is not a char boundary; it is inside 'é' (bytes 1..3 of string)` — from `fn from_hex(&str) -> Result<…>`. ⭐ **The condition is NOT "non-ASCII"**: it needs an EVEN byte length (clearing the odd-length guard) whose chunk boundary falls INSIDE a character, so the control asserts both structural facts about its input before decoding. ⭐ **The call site already handled the error the decoder never produced** (`map_err(ChannelError::Malformed)?`, one line away) — a `Result` the body did not honour, not missing error handling. 🔴 **census, and the grep found what the reasoning did not: 7 hand-rolled hex decoders, 6 panicking, and the 1 SAFE one is the only one on the untrusted wire** — attention went exactly where someone was worried. ⛔ **The server's `pub` copy is DELETED, not repaired**: no caller by `git grep`, confirmed independently by making it private (`error: function from_hex is never used`); a repaired dead path is one no control reaches. ⛔ **4 TEST-local copies measured and deliberately unchanged**, reason recorded so a later census does not re-raise them. ⚠️ Trust boundary stated, not inflated: needs a malicious/faulty control plane, not a network attacker — but the cost is the whole node process. FALSIFIED by reverting only the indexing (`24 passed; 2 failed`, the third control green). **Node 70 + rotation/enrollment 55 tests** rc=0. Promoted → `docs/knowledge/a-signature-is-a-promise-the-body-must-keep.md`.

🔴 **A fenced session's `ack` marked the NEW session's delivery acknowledged — and an acknowledged row is one the retention prune DELETES; repaired under `.4.2.4` (REPAIR-0156).** `ack` ran as three separate pool statements, so the admission decision and the write took different snapshots; `events` closed the identical window in `.2.2` and `ack` was never given the same treatment. REPRODUCED with the lease row held: the ack `COMPLETED — it never asked about the lease it was writing under`, the session was fenced, and it still marked **3 rows** (`left: 3, right: 0`). 🔴 **The severity is the PRUNE, measured not asserted:** 9 sites touch `acknowledged_at`, classified to exactly ONE mutating consumer — the retention `DELETE … acknowledged_at <= cutoff` — so a stale ack makes work the live session still holds eligible for deletion. Fix: ONE transaction calling `verify_fencing_in_tx`, the shape `events` already has. ⛔ **`poll` DECIDED, not assumed — NO**: all three of its statements are reads, a fenced poll leaves no trace, and wrapping it would lock the channel's most frequent call for no invariant. ⛔ **No tenant guard on `ack`**: that would cut the tail `.4.1.3.1` preserved. ⭐ The in-transaction cursor read ships with its limit NAMED — no control discriminates it; its justification is not taking a second pool connection per acknowledgement. FALSIFIED with the transaction kept and only the re-verification removed; arm 2 proved red SEPARATELY under the same neutralization. **8 suites / 117 tests** rc=0.

🔴 **A heartbeat whose check had passed REVIVED a lease that lapsed before its write landed; repaired under `.4.2.3` (REPAIR-0155).** The renewal's `UPDATE` named the node, the epoch and the certificate — never `lease_expires_at` — so the expiry was the one condition still owned by a pre-check. REPRODUCED against the live route with the write stalled at the lease row (`blocked_on` proves it WAITING before the lapse commits, so the ordering is measured, not hoped for): renewal **ALLOWED**, fresh expiry, **1 live lease** on a dead session — and presence, delivery and every fenced write are functions of that clock. Fix, two parts: the expiry joins the epoch (`.2.2`) and the credential (`.4.1.3`) INSIDE the write, and the classifier names the third refusal. ⭐ **The WORDING is a repair part** — the same request answered `lease_expired` sequentially and `fencing token refused` when its write was overtaken, so the wire answer was a function of the scheduler. **The clock was DERIVED:** the DATABASE clock, because one statement must not mix two and because `node_presence.online` reads this column that way — so a renewal can never succeed for a node the API reports `offline`. ⚠️ **Residual NAMED, not bundled:** `lease_expires_at` is process-WRITTEN and database-READ, so the 60 s TTL is nominal and carries the skew; pre-existing, not widened here, owned by `.4.2.3.1` and stated in the book's honest limits. ⛔ The pre-existing expiry control does NOT discriminate this repair (it only drives the sequential order) — labelled a regression control. FALSIFIED twice, each part alone (`35 passed; 1 failed` both times, only this control). **7 suites / 79 tests** rc=0.

🔴 **A captured handshake FENCED THE LEGITIMATE NODE OUT OF ITS OWN SESSION, and a captured rotation returned a second PRIVATE KEY; repaired under `.4.2.2` (REPAIR-0154).** REPRODUCED byte for byte with the node crate's own helpers: replayed handshake `200` + the real node's next heartbeat `401`; replayed rotation `200` + two distinct `key_der` values from one captured request, 3 live certificates. **DECISION: a per-request nonce consumed once, NOT a server challenge** — no extra round trip, **no clock** (a skew-window design would put a clock on the auth path of a repo that just fixed three cross-clock defects), and it is enrollment's existing shape. ⛔ Consuming the CERTIFICATE was rejected: a lost-response retry is indistinguishable from a replay and the rule would break §17.4 recovery — a control asserts this refuses replays, NOT reconnects. ⚠️ The nonce is consumed only AFTER the signature verifies, so forged traffic cannot burn one. 🔴 **A DEADLOCK I introduced, found by the suite HANGING**: an FK to `nodes` made the rotation wait on the row it already holds — and would have queued every handshake behind any in-flight revocation. FK dropped, consume moved in-transaction. ⚠️ **A hanging suite is a third failure mode beside red and green**; diagnosed from the process table, not the code. ⚠️ Wire narrowed: `nonce` required (`422` if absent). FALSIFIED. **Broad run: 44 suites / 374 tests, 0 failed** rc=0.

🔴 **The bad-proof control covered the WRONG rung, and a disabled signature check would have gone unnoticed; closed under `.4.2.6` (REPAIR-0153).** The existing negative presents `cert_der: "00"`, which fails the chain check and stops — coverage of "an unparseable certificate is refused", not of "a bad proof is refused". ⚠️ Every rung answers the same `401` BY DESIGN (no existence leak), so the six new negatives prove their rung by CONSTRUCTION, each satisfying every rung but its own; ⭐ a POSITIVE control differing in exactly one factor (the signing key) is what proves the signature negative reached the signature rung. 🔴 **FALSIFIED decisively:** with the signature check neutralised a forger receives a live lease and fencing token, and **only the new control fails** — the pre-existing one stays green; the chain rung behaves identically. ⚠️ Scope correction recorded: the captured-proof fixture is `.4.2.2`'s, because the canonical bytes are not exposed and re-deriving them would duplicate an unchecked wire contract. ⛔ **No production code changed** — control coverage only. **34/34** rc=0.

🔴 **A rotation in flight outlived a revocation and left the withdrawn node a LIVE certificate; repaired under `.4.2.1` (REPAIR-0152).** REPRODUCED against both live routes: with the rotation stalled after its proof check, a complete revocation answered `200`, the rotation resumed ALLOWED, and the node finished with **1 live certificate** — which restores the lease renewal and the work delivery that `.4.1.3`/`.4.1.3.1` gate on exactly that certificate. ⭐ **The first reproduction was an ARTEFACT** (`500 canceling statement due to lock timeout`) and chasing it found the mechanism: **the `nodes` foreign key does not prevent the race, it makes it deterministic in the WRONG direction** — the insert is forced to land after the revocation commits. Fix: rotation is ONE transaction taking the node row FIRST, then re-reading the certificate; ⭐ no new lock or guard — the same row the revocation already takes, so the two serialize. ⛔ Folding the liveness into the INSERT (the `.4.1.3` shape) is REJECTED: READ COMMITTED snapshots at statement start, so the window between two statements needs a lock held across both. 🔴 **The first control did NOT discriminate the repair** — removing the lock left it green — so a third arm asserts the lock directly in `pg_stat_activity`. FALSIFIED separately for each part. **8 suites / 117 tests** rc=0.

🔴 **`.4.2` censused and split into seven children (REPAIR-0151); NOTHING is repaired yet.** ⭐ The census read the **15 routed source-census records** as well as the goal line — `.11.9`'s lesson applied the day it was written — and names the clauses belonging to other leaves rather than dropping them. 🔴 **A rotation can outrun a revocation and leave a LIVE certificate behind** (`census-2.md:56`, `.4.2.1`): `rotate` is three unguarded statements, the revocation updates unrevoked certs under the exclusive guard, and an insert landing after it is unseen — which **undoes `.4.1.3` and `.4.1.3.1`**, both of which gate on exactly that certificate. ⛔ NOT the claim `.4.1` refuted (that was whether a revoked node can rotate; this is whether an in-flight rotation can commit across a revocation). 🔴 **The rotate proof is entirely STATIC** (`{channel_version, node_id, cert_der}`), so a captured request replays verbatim and **each replay returns a fresh PRIVATE KEY**; the handshake proof has no nonce/timestamp/challenge (`.4.2.2`). ⚠️ Bounded: both die with the certificate; the wire is plain HTTP so capture is cheap. **Four more confirmed at source**: `renew_lease` can revive a lapsed lease (`.4.2.3`), `ack` writes on a pre-check (`.4.2.4`), the `secret_store` claim is wider than the code (`.4.2.5`), the node's `from_hex` panics on non-ASCII (`.4.2.7`). **The bad-proof control refuses before the signature** (`.4.2.6`) and is a PREREQUISITE for `.4.2.2`. ⛔ No runtime change; all seven carry the reproduce-before-writing-up prohibition.

✅ **A node enrollment token no longer outlives the authority that issued it; decided and repaired under `.4.1.2` (REPAIR-0150).** Redemption validated only the token's own fields, so a token issued by a since-revoked administrator still enrolled the node. **censused before arguing:** the token table records NO issuer, but `authorization_records` already stores each decision's `grant_id`/`boundary_id` and the issuance writes it in the SAME transaction — the link is one column away and EXACT. There is also **no route that cancels a token**, so the only prior recovery was waiting out the TTL. ⭐ **Redemption-time re-checking is REJECTED on this codebase's own history**: `enroll` takes no tenant guard, so reading a grant's status there is the class `.3.3.4.5` repaired. Voiding at REVOCATION needs no new guard — that transaction holds the exclusive guard and the token row is already redemption's serialization point, so the row lock orders them. ⛔ Tenant-wide voiding rejected (revoking node X would void node Y's token). 🔴 **The one-live-token index had to key on `voided_at`** or the repair would re-enter `.4.1.1`'s lockout — neutralising it returns that defect's message verbatim. 🔴 **The checked fixture plan caught the new FK's cost, and the census behind the fix was WRONG TWICE** (missed parents-without-child; then scoped to two directories while a list lives in `crates/reasonbraid-mcp/src/lib.rs`) — **a census scoped to the directories you expect is not a census**. FALSIFIED in three discriminating parts. **Broad run: 44 suites / 371 tests, 0 failed** rc=0.

🔴 **An unauthorized caller could PANIC the token-issuance route, and an authorized one could mint a century-long bearer credential; repaired under `.4.1.2.1` (REPAIR-0149).** Found by measuring the PREMISE of `.4.1.2`, whose argument rests on "the token expires" — the expiry was an unvalidated `i64` off the wire. **REPRODUCED, and it is three failures, not one:** `i64::MAX` from an UNAUTHORIZED caller panicked `TimeDelta::seconds` and dropped the connection, **in front of the authority guard** (`resolve_principal` only parses the header); `1e15` from an admin panicked at `at + ttl` **inside** the guard; a century returned `200` with `expires_at: 2126-09-14`. ⭐ **The source reading predicted the wrong one** — the two middling values answered a correct `403`, so only driving the route said which value, which caller, which site. Fix: a range check (`1 … 86 400`) beside the node-id shape check, before any arithmetic, typed `400`. ⚠️ **A deliberate wire narrowing**, stated in the book; `ttl_seconds <= 0` refused too, because a token born expired is `.4.1.1`'s lockout row. ⚠️ Three fixtures were **completed, not deleted** — `.4.1.1`'s reason (lapse through the product's expiry, never a write to the row under measurement) is preserved exactly. 🔎 **`census-1.md`'s `R-31-32-5` already named this and `.4.1`'s split carried 1 of its 3 clauses** — owned as `.11.9`. FALSIFIED in two DISCRIMINATING arms (dropped connection vs a 2126 expiry). **6 suites / 74 tests** rc=0.

✅ **A revoked node is handed NO new work, and the work is WITHHELD rather than dropped; decided and repaired under `.4.1.3.1` (REPAIR-0148).** `.4.1.3` bounded the tail to the remaining lease but left the server dispatching into it. **census of the delivery ladder BEFORE the rung was chosen:** 2 writers into `node_inbox`, 7 readers, exactly **1 that delivers rows to a node** — and that one has **two** callers, `poll` AND the handshake, so a filter written into `poll` alone would have been correct today and silently incomplete. ⛔ **The enqueue rung is REJECTED on that census:** refusing at dispatch destroys work for a node about to be REPLACED, and `node_replacement`'s ritual is the control that proves the withheld tail must survive. The delivery read now asks the question the RENEWAL asks, asked the same way, and it is a **filter, not a refusal** — `poll` still admits and answers the true cursor, `ack`/`events` untouched, so `.4.1.3`'s seam holds. 🔴 **A claim recorded in three live documents was STALE and a control caught it:** `node_presence.suspended` is not *ever revoked* — **migration 0017 supersedes 0012** — so the rejection stands on a narrower fact, that 0017 never asks whether the surviving certificate is IN DATE. Promoted to `docs/knowledge/a-schema-object-is-its-latest-migration.md`. ⚠️ A fixture was **completed, not deleted** (a credential-less poller is unreachable through the live routes). FALSIFIED in three separate neutralizations (2 / 2 / 1 failures — both halves load-bearing, the controls discriminate). **7 suites / 92 tests** rc=0.

🔴 **Revoking a node did NOTHING to a node that was running, and kept doing nothing; repaired under `.4.1.3` (REPAIR-0147).** MEASURED on every channel surface after a real revocation, not read: `heartbeat ALLOWED · poll ALLOWED · ack ALLOWED · events ALLOWED`, only `rotate`/`handshake` refused — the two operations that re-present a certificate. The fencing check reads no ledger fact by design, so a node heartbeating inside the 60 s lease TTL renewed for ever and never reached the check that would have refused it. ⭐ **`.1.3.1`'s decision was RIGHT and unfinished**: "suspension gates re-entry" has a bounded reading, and what shipped was the unbounded one. Lease RENEWAL now requires a certificate neither revoked nor expired; poll/ack/events keep their pure fencing check, which IS the intended tail. ⛔ **`node_presence.suspended` is the WRONG predicate** — ⚠️ **corrected 2026-09-13 by `.4.1.3.1`:** the rejection stands but *not* for the reason recorded here; **0017 supersedes 0012**, so a replaced node reads NOT suspended and was never at risk. The column is disqualified because it never asks whether the surviving certificate is IN DATE. ⚠️ **Two limits:** the tail is up to a full lease, and during it the server still hands the node NEWLY enqueued work (measured, deliberately not repaired here — ✅ **closed by `.4.1.3.1`/REPAIR-0148**); and nothing at the transport re-checks a certificate — the dev wire is plain HTTP, already named by `PHASE-7`. Registered as **R-REVOKE**. FALSIFIED (`30 passed; 1 failed`, exactly the one control). **5 suites / 53 tests** rc=0.

🔴 **A token that expired unused locked its node out for good; repaired under `.4.1.1` (REPAIR-0146).** Migration 0018's partial index keys `(node_id) WHERE used_at IS NULL`, and an EXPIRED token still satisfies that — so a token issued and never consumed held the index permanently and every later issuance for that node id answered `409`, while the lapsed token enrolled nothing. ⭐ **The 409's own message named the recovery and the recovery was the defect**: *"consume or expire it before issuing another"*. REPRODUCED against the live routes first, body captured. ⛔ **The obvious repair is UNAVAILABLE** — a partial index predicate must be IMMUTABLE and `now()` is not — so migration 0059 adds `superseded_at`, the index keys on it, and issuance stamps a lapsed row inside its own transaction at the admission's own database time. ⛔ **Not DELETE** (an audit row would name a token id resolving to nothing), ⛔ **not REUSE in place** (an audit row would resolve to a DIFFERENT token's facts — and it needs no migration, which is what makes it tempting), ⛔ **never `used_at`** (it was never redeemed). ⚠️ **Bound:** scoped to the admitted tenant, so another tenant's lapsed token still blocks — `.3.5`'s global-index territory, neither widened nor repaired here. FALSIFIED by neutralizing only the supersede predicate (`13 passed; 1 failed`, identical body) and restoring (`15 passed; 0 failed`); 0018's invariant falsified in the same test. **3 suites / 46 tests** rc=0.

⚠️ **`.4.1` censused five mechanisms before splitting, and REFUTED one of them (REPAIR-0146).** The rotation/revocation serialization worry is **not** a defect: `rotate` opens no transaction and takes no tenant guard, but `verify_rotate_proof` selects `(revoked_at IS NOT NULL OR expires_at <= now())` and answers `proof_refused` — **a revoked node cannot rotate**. Recorded as refuted so it is not re-raised. Two mechanisms stay OPEN and are named as unmeasured rather than assumed: `.4.1.2` (redemption never re-checks the issuer's authority — a bearer-semantics DECISION, not a presumed defect) and `.4.1.3` (`revoke_node` carries no lease write, so what a revoked node's live fencing token still authorises is READ, not measured — ⛔ measure what `.1.5.2`'s revocation epoch already fences first).

⚠️ **"Measure the population before proposing the rule" is 4 of 8, not a law (`.11.6`, REPAIR-0145).** The general population is NOT mechanically countable (`grep -c superseded` -> 49, nearly all about superseded designs), so the census ran where "a rule was proposed" is a mechanical fact: **11 checks across 9 leaves** shipped by this repository, dated per check. **4 of the 8 assessed had their rule revised by the census that preceded it**; twice the obvious rule was rejected outright. ⭐ **One was measured and shipped UNCHANGED** and is counted as such. ⛔ The leaf's running "ten instances" counts something DIFFERENT (severity reversals in repair leaves shipping no gate) and is not conflated; a leaf whose census was corrected but whose rule was not is counted as NOT revised. Decision: a `TOOLBOX.md` method statement, **no gate** — the shape is a judgement over prose, and the one detectable shape is already gated by `GAP-CLAIM-CENSUS`. ⚠️ One author, one repository, denominator 8: the statement is scoped to "the gates this project has shipped", not asserted generally.

⛔ **SUPERSEDED BY `.11.8.1` (REPAIR-0255) — the three counts in this paragraph are wrong in both directions.** The census key DELETED path parameters, so a collection and its item collapsed onto one key (the item counted *described* by the collection's contract line) and a mid-path parameter produced a stem no book ever writes (documented routes counted *absent*). Re-measured at REPAIR-0144's own commit `0fbb85f` with only the key replaced: **103 routes — 30 described, 3 mentioned, 70 absent**. Over today's larger corpus the repaired instrument reports **104 routes — 49 described, 3 mentioned, 52 absent**. ⭐ What this leaf CONCLUDED survives the correction: a measured backlog, not a gate. The original sentence follows, unedited. ⚠️ **The book does not name 80 of the server's 103 product routes (`.11.8`, REPAIR-0144).** Refined census, now a tracked instrument (`scripts/census_route_documentation.py`, `--self-test`, discovered by the `SELF-TEST` gate): **103 product routes — 22 described (named beside an HTTP method), 1 mentioned, 80 absent**, the naive `111/24/87` corrected by excluding 8 `#[cfg(test)]` fixture routes and by splitting "named" into contract-line vs passing mention. 🔴 **The instrument UNDERCOUNTED by 42 % first** — a per-line scan missed `.route(` calls whose path is on the next line (53 of api.rs's 92) — caught only by cross-checking an independently obtained count. ⛔ **No gate proposed:** "every route must appear in the book" would flag the console's static assets and accept a bare mention, wrong in both directions before it is written; the answer is a measured backlog the instrument re-derives. ⛔ **80 is not 80 defects** and "described" is a proxy — a contract line is not proof the description is correct. ⚠️ **This leaf documents NO route**; it measures.

🔴 **A node id enrolled by one tenant answered `500` to another, and was one arm away from answering with a CERTIFICATE; closed under `.3.5.1` (REPAIR-0143).** `nodes.node_id` is a GLOBAL primary key, but the redemption's existence check was `EXISTS(… node_id = $1 AND tenant_id = $2)` — tenant-scoped — so a node owned by someone else reported false, the insert ran, and the key aborted the transaction. ⚠️ Reachable today: the index blocks a second UNUSED token, not a second token, so once the first tenant enrols the second tenant's issuance succeeds. The query now reads the node's OWNER and refuses a foreign one, audited, reusing the same-tenant wording so nothing names the owner. 🔴 **The naive fix is a TAKEOVER, demonstrated:** the replacement branch swaps the node key and issues a certificate with NO tenant check once certs are revoked — with the refusal arm removed, tenant B's redemption returned `200` with `cert_der` for tenant A's node. FALSIFIED in both directions. ⛔ **The index is UNCHANGED — no migration**; this repairs the refusal, not the uniqueness, and ⛔ the leaf's own proposed per-tenant repair is REFUSED by the measurement. **3 suites / 50 tests** rc=0.

✅ **The enforcer now runs every instrument's own `--self-test` (`.11.4.3.1.7.2`, REPAIR-0142).** Nothing ran them before, which is how one sat broken from the commit that added it. Measured before proposing: **17 self-tests / 1.01 s** against a 3.15 s enforcer; registered, the gate is **4.27 s over 18 checks**. The check DISCOVERS its population, so a new instrument is covered the day it lands. ⛔ **It catches nothing today** — all 17 pass; its value is preventing a control from silently stopping. ⚠️ **11 of 28 check/census scripts have NO self-test** and this does not reach them; it also does not assert the 17 are GOOD, only that they still pass. 🔴 **The gate recursed while being built** — registering it put the flag's literal into the enforcer's description, discovery found the enforcer, and the enforcer ran every check again; killed at 400 s. Now guarded by an exported variable that makes any nested invocation exit immediately, with the enforcer also excluded by path. FALSIFIED end-to-end against the REAL defect: `make gate` prints `❌ SELF-TEST` and refuses the commit.

✅ **A revoked boundary no longer leaves the metrics surface open (`.3.5.2`, REPAIR-0141).** `GET /v1/admin/metrics` gated on the GRANT's status and never joined the boundary; revocation updates only `enrollment_boundaries` and does not cascade, so a withdrawn administrator kept the surface. REPRODUCED against the live route first — boundary `revoked`, grant still `active`, route still `200` — then repaired by joining the boundary and requiring it live. FALSIFIED at `left: 200, right: 403`, only that control. ⛔ Deliberately NOT the inspection carve-out: that exists to inspect AUTHORITY state during a revocation, and counters are not authority state. ⛔ **Two things confirmed UNREPAIRED:** the process-global width is unchanged (deliberate; 7 aggregate counters, no identities) and the read is still **unaudited** — both need the route to name a tenant, a breaking signature change, owned by `.3.5.2.1`. **37 passed / 0 failed.**

🔴 **Revoking a boundary does NOT remove access to the metrics surface, and nothing is repaired yet (`.3.5`, REPAIR-0140).** `GET /v1/admin/metrics` gates on a hand-rolled query over the GRANT's status and never joins the boundary; `revocation.rs` revokes a boundary with `UPDATE enrollment_boundaries` alone, leaving its grants `active`. The read also writes **no authorization record**, unlike the nine repaired tenant-admin inspection routes. ⚠️ Bounded: the payload is **7 aggregate counters, no tenant dimension, no identities** — an aggregate-volume side channel, NOT a data leak. ⚠️ The admission census classifies it "identity only", which means it reaches no RECOGNISED gate, not that it is unauthenticated. 🔴 **The one-unused-token index is still GLOBAL** (migration 0018, no tenant column), so the cross-tenant `409` oracle and hold-out reproduce unchanged. ⭐ The leaf's third surface is already closed by `.3.3.4.10` (five node mutations, guarded and tenant-bound); the residual 42 mutating identity-only routes stay owned and unrepaired. Split into `.3.5.2` (metrics) and `.3.5.1` (token index + the redemption-lineage question `.4.1` shares). ⛔ **NOTHING is repaired in this commit.**

🔴 **A node clock behind the server rotated its certificate five minutes AFTER it expired; reproduced and closed under `.3.4.3.1.3` (REPAIR-0139).** A leaf issued at server-time `T` expires at `T + 600` with rotation due from `T + 300`; with the node 600 s BEHIND, the check did not fire when due, nor with one second of validity left, nor at expiry — it first fired at `T + 900`, leaving the channel dead for five minutes. ⚠️ The OPPOSITE direction to the dispatch outage (a node AHEAD), which is why the warning now sits in the code at the point of use. The decision is extracted as a pure `rotation_due` and evaluated in the server's terms through `.3.4.3.1.2`'s offset; the channel keeps its own copy because the check runs inside `handshake`, before the journal is reachable. ⭐ FALSIFIED on the WIRING: dropping the offset leaves the three pure-function controls GREEN and fails only the wiring control — the decision was right, what fed it was not. ⚠️ One test-only dev-dependency (`time 0.3`, already resolved in the workspace; `cargo deny check bans` ok). ⛔ **No field failure is asserted** — reproduced in a driven control, not a deployment; the cert's `not_after` is the server's PROCESS clock and the offset is measured against its DATABASE clock, sound only to the 1 s tolerance `.3.4.3.1.1` declared. **69 tests** rc=0.

✅ **The dispatch outage is CLOSED; the node now tells the time in the server's terms (`.3.4.3.1.2`, REPAIR-0138).** A node whose clock ran >60 s AHEAD of the server refused EVERY dispatch — it did no work at all. Both responses now carry `server_time` (from `clock_timestamp()`, folded into the existing epoch query so a poll gains no round trip); the node stores `offset = server_time − midpoint(sent, received)` and evaluates `decided_at` through it, and REPORTS a disagreement ≥5 s. ⭐ **This SUPERSEDES `.3.4.3`'s clamp, measured before changing anything:** `min(decided_at, received_at)` mixes a LOCAL instant into a server-clock comparison, so a corrected node 600 s behind computed `expires 02:56:58` against `server now 03:05:58` and found everything stale — bounding a cross-clock comparison and removing it do not compose. Four controls asserting the clamp are superseded, each with a comment naming its replacement; the property is now asserted in BOTH directions. 🔴 **A THIRD cross-clock comparison was found here and not by the census:** the retry gate filters `updated_at >= decided_at` (local vs server), so a node behind filters every post-decision attempt out and **the retry bound never trips** — fail-open, now repaired with its own control. ⚠️ NOT clock security: the offset is the server's own statement. ⚠️ Coordinated wire change (`deny_unknown_fields`) — server and node ship together. **10 targets / 115 tests + 4 suites / 57 tests**, rc=0; falsified in both halves. ⛔ `.3.4.3.1.3` (the certificate's opposite direction) remains OPEN and unreproduced.

🔴 **A self-test searched for a string it contained, and had been failing since its own first commit; closed under `.11.4.3.1.7.1` (REPAIR-0137).** `census_pg_test_clusters.py`'s citation guard asserted that `run-selftest-no-such-cluster-name` appears in **zero** tracked files — while that literal was written in its own tracked source. The file's add-commit and the string's introducing commit are the SAME (`cb2f197`), so the arm passed exactly once, before the file was tracked. ⚠️ It survived because **NOTHING RUNS ANY `--self-test` here** — the 17 registered doctrine checks included (`grep -n "self-test" scripts/check_doctrines.sh` -> 0); that half is OPEN at `.11.4.3.1.7.2`. ⛔ The guard's other direction never broke, so cited clusters stayed protected — what was missing is proof the guard is not simply returning non-zero for everything, i.e. HALF-verified, which must not authorise a deletion. The probe is now generated per run (absent by construction, not by luck); FALSIFIED by restoring the literal. ⭐ With the control passing, the **§8 artifact review** it gates ran for the first time since `cb2f197`: 34 clusters / 1,797,794,711 bytes → **30 retired, 1,591,802,083 bytes freed**, residue verified at 4 / 205,992,628 with `retirable: 0`; the four survivors (three cited evidence, one under the one-hour floor) were confirmed present afterwards.

🔴 **The stored certificate expiry was never read from the certificate; closed under `.3.4.3.1.1` (REPAIR-0136).** Both issuance sites computed `Utc::now() + LEAF_TTL_SECS` independently of the `not_after` signed into the certificate — two derivations of one quantity that never agreed, because the certificate's instant is truncated to whole seconds and the stored one is not (falsified at `…12.973747Z` against `…12Z`), and the enrollment path samples its `now` before the transaction's database work. ⛔ **No certificate was ever accepted or refused wrongly** — nothing enforces the stored value; it was a record disagreeing with its artifact. `issue_node_leaf` now returns the signed instant and a public `ca::leaf_not_after` extracts it from a DER. The leaf's other option (unify on one clock) is UNAVAILABLE: a verifier enforces the signed `not_after`, which cannot move without changing what the CA signs — so the server's two clocks are **declared** coherent within 1 s and that assumption is now MEASURED by a control (⚠️ in CI, on one host; a remote database with a drifting clock is NOT covered). Recorded for `.3.4.3.1.2`: of the three instants the server sends, **only `decided_at` is ever read** by the node. **3 suites / 49 tests + lib 102 + mtls 1**, rc=0.

🔴 **Three clocks, two of them compared against each other in opposite directions, and NOTHING is repaired yet (`.3.4.3.1`, REPAIR-0135).** The census this leaf owed refuted the leaf's own framing. **Three** server instants reach the node (`decided_at`, `cert_expires_at`, `lease_expires_at`); **two** are compared against the node's own clock and they fail OPPOSITE ways — a node AHEAD refuses EVERY dispatch, a node BEHIND by >300 s makes `cert_expires_soon()` fire only after the workload certificate has expired, breaking the channel; the third is received and never read. ⛔ **And the SERVER has two clocks:** `decided_at` is PostgreSQL `clock_timestamp()` (22 sites) while the cert/lease instants are the server process (`Utc::now()`, 8 sites), with nothing reconciling them — so an offset correction built today would silently assume they agree. Split into `.3.4.3.1.1` (the server's own clock split, prerequisite), `.3.4.3.1.2` (the node evaluating server instants in the server's terms — closes the outage), `.3.4.3.1.3` (the certificate direction, ⛔ **arithmetic on a source reading, NOT reproduced**). ⛔ **The dispatch outage remains OPEN and unrepaired.** Documentation-only commit; no behaviour or qualification change.

🔴 **ADR-009 is titled "chain-in-envelope" and carries no chain; measured under `.3.4.5` (REPAIR-0134).** `AuthorityContext` holds one `on_behalf_of` string, so a depth-2 delegation has nowhere to go; a control pins it so the title cannot keep implying a capability the type lacks. ⚠️ Third independent measurement of the same absence, after `.3.4.1` (the grant-chain flags `delegable`/`max_delegation_depth` have no producer) and `.3.4.1.1` — **delegation is one hop deep everywhere, and the vocabulary around it is written for something larger.** The comparative wire-size measurement the withdrawn claim never had now exists as a re-runnable instrument asserting its own table: against a 308 B undelegated baseline, chain-in-envelope is 505/684/861 B at depths 1–3 against a token's 1,066/1,824/2,582 B (ES256), i.e. **177 B against a constant 758 B per hop, ~4.3×**. ⛔ No size ADVANTAGE is offered as the reason for the choice, which rests on the subtraction and the existing revocation lifecycle. ⚠️ Depth 1 encodes the shipped envelope; depths 2–3 are prototype-vs-prototype, and size is not the axis that decides the question — tokens buy offline verification and an unreachable delegator, which no byte count settles. Core suite **74 tests** rc=0.

🔴 **An untrusted HTTP body could say `join` while carrying a decline, and was recorded as a join; closed under `.3.4.4` (REPAIR-0133).** `POST /v1/calls/{call_id}/respond` and the MCP `join_call` seam decode `RecruitmentResponse` from client input. Measured against the shipped decoder: `{"kind":"join","reason":"I decline"}` **accepted as join**, `["join"]` accepted, same pair for `observe`; the members-carrying responses were already strict. Cause is the `.3.3.3.2.2.1` Serde branch — an internally tagged UNIT variant discards the rest of the map and accepts the sequence form, and the `deny_unknown_fields` this type declared switches off neither. ⭐ The census ranked by REACHABILITY, not resemblance: **28 tagged enums, 8 with the shape, exactly 1 decoded from untrusted input** — and it is not `Decision::Allowed`, the type the leaf was opened around, which has no untrusted producer. Repaired for `RecruitmentResponse`, `Decision` and `CachedDecisionKind`; ⚠️ `Decision` is deliberately TIGHTENED beyond its own declaration because the leniency read a denial and its evidence back as an allowance. ⛔ The four adapter types with the same shape are NOT repaired — none is deserialized. ⛔ No evidence any real client exercised it; this is what the endpoint ACCEPTS, not an incident. Core lib 53, `authorization_evaluation` 6, server lib 101, pg suites **3 / 96 tests** rc=0; FALSIFIED twice, each reversion isolating one crate.

⭐ **A latent authorization state is now unwritable, and removing it found two fixtures that had been producing it (`.3.4.1.1`, REPAIR-0132).** `CommandAuthz` held the delegated subject and the delegation scope as two independent `Option` fields; `selection.rs` read the scope under `if let Some(scope)`, so a subject-without-scope would have delegated with the subject's FULL grant selector — the §16.3 widening invariant skipped rather than failed. They are one `Delegation { subject, scope }` now. ⛔ **No defect is repaired and no reachable behaviour changes:** the only production producer always set both. ⚠️ But the "unreachable" claim held for production only — `tests/authority.rs` constructed the state twice, so the widening gate was being SKIPPED inside a test that reads as exercising delegation; both surfaced as compile errors. Giving those fixtures their scope makes the invariant RUN where it did not, which is a strengthening and is stated rather than absorbed. The unconstructibility is proved by the compiler (`error[E0063]: missing field `scope``), not by a test. **4 suites / 66 tests, zero failures**, every existing delegation control unchanged.

🔴 **A node's cached-decision freshness window compared two different clocks;
closed under `.3.4.3` (REPAIR-0131).** `decided_at` is the SERVER's database
clock, while `is_fresh` is evaluated against the NODE's process clock, so a
cached allow held by a node running behind the server stood for **`skew + 60 s`**
of observed time rather than 60 s. ⚠️ That window is load-bearing: the revocation
epoch is bumped only by explicit revocation, so a grant reaching its own
`expires_at` is bounded at the node by the freshness TTL **alone**. The window now
runs from `min(decided_at, received_at)` — the node's own receipt — which binds
only when the node received a decision before its clock says the server made it,
is byte-for-byte the shipped behaviour otherwise, and can never refuse work the
plain rule allowed. ⛔ The opposite direction is UNCHANGED and NOT repaired: a
node clock running *ahead* refuses every dispatch past 60 s of skew, fail-closed
and journaled, owned by `.3.4.3.1` with the question of whether the wire should
carry a duration instead of an instant. Skew is now bounded, **not detected** —
nothing reports that a node's clock disagrees with the server's. **10 targets /
116 tests, zero failures**; FALSIFIED three times, each reversion isolating one
part (no clamp → both skew controls fail with the dispatch allowed; no receipt
refresh → only the replay control fails; no guard → only the redelivery control
fails).

🔴 **A revoked delegation was answered with a success, and invisibly; reproduced
and closed under `.3.4.2` (REPAIR-0130).** `request_hash` covered the operation,
the actor and `envelope.body`, while `authority_context` is a SIBLING of `body`.
Because the idempotency claim is made BEFORE authorization, a second request with
the same key and body but a **revoked** subject answered **`200 replayed=true`,
`ok: true`** and wrote **zero authorization records**. ⛔ No new effect is applied
by a replay, so this was not an escalation of what was written — it was an
unauthorized request told it had succeeded, with no audit trace. The hash now
binds the authority context and such a request is `409 idempotency_mismatch`.
⚠️ A request with NO authority context hashes byte-identically to before, so every
historical undelegated key keeps replaying; a historical delegated key now
conflicts rather than replaying, which is the safe direction and is documented as
the wire change it is. **36 passed / 0 failed**; the affected set passes **6
suites / 62 tests**. FALSIFIED **35 passed / 1 failed** against the exact
pre-`.3.4.2` sources.

`SIGNOFF-REPAIR.3.4` is censused and split into five children (REPAIR-0128),
with two defects measured from source and not yet repaired. ⛔ **CORRECTED at `.3.4.1` (REPAIR-0129):** the census recorded `evaluate()`
never reading `grant.delegable` as a defect. The measurement is right and the
inference was wrong — `delegable` and `max_delegation_depth` govern grant CHAINS,
which have no producer (no grant's parent is ever another grant), and §16.3
conditions delegation on invariants rather than on a flag. A delegated request
must pass four gates, all enforced: the actor's own authority **for the same
target**, the subject's authority, the widening invariant, and the actor's own
thread participation — measured at `403 … is not a participant of this thread`,
naming the ACTOR. 🔴 **The replay hash does not bind the authority context**:
`request_hash` covers the operation, the actor and `envelope.body`, while
`authority_context` is a sibling of `body`, so two requests differing only in
`on_behalf_of` share an idempotency key and the second replays the first's result
without its own authority being evaluated (`.3.4.2`). Bounded depth is
deliberately unenforced in the dev profile and says so in a comment; the
delegate-without-scope pair is unreachable today and is recorded so it is not
rediscovered as a defect. ⭐ The cached decision's two halves fail in opposite
directions — a future EPOCH reads as stale, a future `decided_at` does not
(`.3.4.3`). No production source changed; nothing here is repaired yet.

**`SIGNOFF-REPAIR.3.3.4` is closed at `.13` (REPAIR-0127), and the closure states
its own limits.** Two tracked instruments re-derive the census rather than
re-reading it. The direct named-call surface fell **42 → 27** while the corpus
grew **101 → 118 files**; a new route census classifies all **118 registered
routes** by the gate each actually reaches. ✅ **18 routes run on a guarded
transaction**, all of them mutating: admission, mutation and final-effect evidence
in one commit under a declared guard, at database time sampled after the wait,
each mode derived and each repair falsified against its own superseded source.
⛔ **46 mutating routes do not**, and every one has a named repair owner —
policy 15, evaluations/routing 9, evidence 5, recruitment 3, resolvers/resources
3, deployment 3, regions 3, adapters 2, workflow 1, directory matching 1, plus
`PUT /v1/profiles/{role_id}` which is identity-gated by design. **Being owned is
not being repaired; this closure certifies none of the 46.** No obsolete
bypassing executor remains (84 authority functions, 10 unreferenced, all 10
`#[test]`). ⚠️ Both instruments were wrong before they were right, once in the
direction of reporting live code as dead; the corrections live in the scripts.
Broad step: **19 live suites / 262 tests / zero failures**, plus 99 library unit
tests. The instruments are lexical and bounded — no AST, no dynamic dispatch, no
runtime SQL — and classify which gate is REACHED, not whether it is the right one.

The card import's revocation race is now **closed on both sides** under
`.3.3.4.12.1` (REPAIR-0126). `.11.3` predeclared the limit, `.12` closed the
importing half by putting the direction verbs on their own tenant's exclusive
guard, and the import now declares BOTH tenants in one predeclared sorted set —
the importing tenant exclusive because it issues a grant, the origin tenant
shared because it reads the origin's agreement row. The set goes to the runner,
which sorts it; two acquisitions could not prevent inversion and the guard API
offers no later upgrade. An origin id that does not parse declares no second key,
because the set is built before the transaction and a refusal there would move a
card check ahead of the admission. **6 passed / 0 failed**; the affected set
passes **5 suites / 89 tests**. FALSIFIED **5 passed / 1 failed** at `the import
waits on the ORIGIN tenant's guard` — and the superseded import does not block at
all there, having never declared the key. ⛔ Still `.5.3`'s: what an effective
agreement guarantees a consumer ACROSS a change, since a re-proposal with
different terms still ends an agreement without a revoke.

The three federation direction verbs are now ONE guarded transaction each under
`.3.3.4.12` (REPAIR-0125): admission, mutation, the acceptance's cross-domain
receipt and the effect record share a commit under the LOCAL tenant's
**exclusive** guard. All three were the `.9` shape — an already-committed
shared-guard admission and then a pool mutation — with no guard, record or
receipt. ⭐ The mode is derived from the classify-then-write-over-a-possibly-absent-row
reason, not the epoch one: none advances the revocation epoch. 🔴 Proposing to a
tenant that does not exist RAISED a foreign-key violation and answered **`500`**;
it is a recorded `404 not_found` now. That defect was found by running the
enumerate-the-constraints check promoted one commit earlier, on its first use.
The wire is otherwise unchanged, with three previously indistinguishable idle
states now separated in the record (an unchanged re-proposal and an
already-accepted direction as `no_op`, a never-proposed one as `refused`).
**4 passed / 0 failed**; the affected set passes **4 suites / 72 tests**.
FALSIFIED **1 passed / 3 failed** against the exact pre-`.12` sources, the
ordering control failing at `the revocation is waiting`. ⚠️ **Half the card
import's revocation race is now closed and the leaf says which half**: the import
holds the importing tenant's key, so an importing-side revocation fences it and
an ORIGIN-side one still does not. `.3.3.4.12.1` owns the both-tenant guard set.

**`SIGNOFF-REPAIR.3.3.4.11` is closed** (REPAIR-0118/0119/0120/0122/0123/0124,
plus `.3.3.4.7.4` as REPAIR-0121). The profile/card family's three mutating
routes each run as one transaction with their evidence: the writer serializes at
the role's version anchor, the attestation holds its read AND write under that
anchor inside a shared-guard transaction, and the card import runs the whole
ladder — admission, rungs, grant, identity, quota, enrollment, receipt and the
PROFILE — inside one exclusive-guard transaction, with three unguarded bridges
deleted. ⭐ The family's last leaf was the book chapter the surface had never had,
and it produced two defects and three corrections: a repeated import answered
`500` (repaired as `.11.5`), an unknown field returns `422` rather than `400`,
and **an expired capability claim still satisfies a requirement** because the
eligibility check never reads `expires_at` (owned by `.5.1`). `docs/book/src/profiles.md`
documents all seven handlers, and
`docs/knowledge/writing-the-documentation-is-a-verification-pass.md` records the
method. ⛔ Still open and named rather than implied: card replay and provenance
(`.5.3`), capability expiry and ranking (`.5.1`), and the import's ordering
against a concurrent agreement revocation (`.3.3.4.12`, the next frontier row,
because the three federation direction verbs take no guard at all).

🔴 **A repeated card import answered `500` and recorded nothing; reproduced and
closed under `.3.3.4.11.5` (REPAIR-0123).** `agent_roles` carries
`UNIQUE (tenant_id, name)` and `enrollments` carries `UNIQUE (tenant_id, kind,
name)`, while the import writes the card's `display_label` into both, so the
`agent_roles` insert RAISED. ⚠️ Not only a re-import: any card whose label matches
an identity already in the importing tenant collides. ⭐ Making the import atomic
at `.11.3` is what turned this from a survivable mess into an unrecordable one —
a raised constraint aborts the transaction that now also carries the admission
and the effect record, which is exactly what
`docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md` said at
`.10.1` and `.11.3` did not apply to its own inserts. Both inserts now use
`ON CONFLICT … DO NOTHING RETURNING`; the answer is `400 invalid_command` naming
the label, with an effect record carrying the same sentence. ⛔ It says the label
is taken and deliberately does NOT decide what a repeated import ought to do —
card replay is `.5.3`'s. **4 passed / 0 failed**; the affected set passes **5
suites / 72 tests**. FALSIFIED **3 passed / 1 failed** against the exact
pre-`.11.5` sources. ⚠️ Found by running a documentation claim instead of
publishing it.

🔴 **An orphaned identity on the card-import path was reproduced and closed under
`.3.3.4.11.3` (REPAIR-0122).** The route committed the grant, the `agent_roles`
row, the quota row, the enrollment and the cross-domain receipt as one
transaction and then wrote the profile on the connection pool. Measured with
every new profile version row made to fail: the import answered **`500`** and
left the role behind (`left: 1, right: 0`) with its grant, quota, enrollment and
receipt durable. It also read its admission, its boundary and its federation
agreement outside the transaction that used them. The whole ladder now runs in
ONE transaction under the importing tenant's **exclusive** guard — derived,
because the import issues a grant and issuance asserts that scope. ⚠️ An existing
`cards.rs` assertion caught an unintended change to the grant-refusal message
during development; the response and the effect record now share ONE renderer, so
two descriptions of one refusal are the same string by construction. Three
superseded bridges are deleted: the unordered grant creator, the pool-taking
boundary loader and the pool-taking profile writer. **3 passed / 0 failed**; the
affected set passes **9 suites / 156 tests**. FALSIFIED **1 passed / 2 failed**
against the exact pre-`.11.3` sources. ⛔ Moving the agreement read inside buys
snapshot consistency and NOTHING more — `federation::{propose,accept,revoke}` take
no guard, so nothing here fences a concurrent revocation; `.3.3.4.12` owns it.
`.3.3.4.11`'s three transaction children have landed; `.11.4` still owes the
profile/card book chapter.

The administrative refusal vocabulary is corrected from three codes to four
under `.3.3.4.7.4` (REPAIR-0121), and the correction has the same shape as the
mistake it repairs. `.7.3` classified every `ControlApiError::unauthorized` in
the fourteen handlers as the admission's own denial — right **thirteen times out
of fourteen**. The fourteenth is the card import's allowlist rung, which refuses
an ADMITTED tenant administrator because the importing tenant holds no effective
federation agreement with the card's origin: a precondition about the two
tenants, not about the caller's grant. Re-measured across all fourteen handlers
with a re-runnable census: `handlers not found: none`, exactly one such site.
`AdministrativeRefusal::Unauthorized` is added so that refusal can be recorded
without the record and the response disagreeing — the one thing the type exists
to prevent. ⛔ A denied admission still writes no effect record at all, so a
`refused` outcome carrying this code always means the request was allowed and the
operation was not. No migration; migration 0058 pins only the outcome `kind`.
**68 core tests** and **25 live effect tests** pass; FALSIFIED **9 passed / 1
failed** by leaving the code on the fail-closed list. ⚠️ This is the seventh
instance of the pattern `.11.6` censuses and the second inside `.7`.

🔴 **A silent lost update on the audited attestation path was reproduced and
closed under `.3.3.4.11.2` (REPAIR-0120).** `attest_capability` read the current
profile on the pool and wrote it back through a different transaction. Measured
on the unchanged route: two administrators attesting two DIFFERENT capabilities
of one role both received **`200`**, and the published profile carried one
upgrade and not the other — an audited attestation dropped with no error to
either caller. ⭐ Serializing the writers, which `.11.1` had already done, would
not have closed it: the losing writer's version NUMBER was correct while its
CONTENT was stale, so the READ had to move under the same anchor lock as the
write. The verb now runs one shared-guard transaction from the admission through
the anchor lock, the read, the upgrade, the new version and the effect record;
the mode is shared because an attestation writes no authority and advances no
revocation epoch. The wire is unchanged — the single `404` still answers both
idle states and the record distinguishes them — plus the
`x-reasonbraid-authorization` receipt on every admitted answer. **38 passed /
0 failed**; the affected set passes **4 suites / 65 tests**. FALSIFIED **36
passed / 2 failed** against the exact pre-`.11.2` sources. ⚠️ The mode does not
change in this repair, so no lock-holding fixture discriminates it; the
discriminating properties are the lost update and atomicity, and the third new
control is labelled a regression control.

🔴 **A concurrency defect on the profile write surface was reproduced and closed
under `.3.3.4.11.1` (REPAIR-0119).** The writer computed its version as a
read-then-write against migration 0019's `UNIQUE (role_id, version)`. Measured on
the unchanged route with four concurrent writers for one role, run twice: **five
of the eight answered `500`**, logging `duplicate key value violates unique
constraint "profile_versions_role_id_version_key"`. A role rewriting its own
profile from two places at once lost writes to an internal error. ⭐ The
serialization now happens at the role's own anchor row, created INSIDE the
acquisition — `SELECT … FOR UPDATE` over a row that does not exist yet locks
nothing, so a lock-only fix would have protected every write except a role's
first. The `UNIQUE` constraint is kept as the backstop that proves the lock
works. ⭐ The route takes NO tenant authority guard, derived rather than skipped:
it is gated on identity, evaluates no grant and produces no authorization record,
so there is no authority decision to order it against. `written_at` is now the
transaction's own database time; status codes, response fields and the content
addressing are unchanged. **35 passed / 0 failed**; the affected set passes
**4 suites / 62 tests**. FALSIFIED **34 passed / 1 failed** against the exact
pre-`.11.1` sources. ⚠️ The discriminating control is probabilistic and labelled
as such — no lock-holding fixture can discriminate this repair, because both
shapes contend on the same anchor row at different points. ⚠️ Newly tracked, not
merely noted: the `/v1/profiles` surface has **no book chapter**, owned as
`.3.3.4.11.4`.

The profile/card surface `.3.3.4.11` (REPAIR-0118) is censused and split into
three children before any implementation, and the census changed two of the
parent's own assumptions. Seven handlers on six route entries, of which exactly
**three mutate**; `.7.1`'s closed vocabulary assigns this parent two operations
and deliberately no `ProfileWrite`, because `put_profile` is a self-declaration
gated by identity with no authorization record at all. ⭐ Unlike `.10`, **all
three mutations are already tenant-bound**, so there is no tenant-binding repair
in this family. What is missing is the transaction and the evidence, and three
defects are measured from source: `profiles::write_profile` computes its version
as a read-then-write against migration 0019's `UNIQUE (role_id, version)`, so two
concurrent writes for one role collide and one caller receives **500** instead of
a serialized second version; `attest_capability` reads the profile on the pool
and writes through a different transaction, so two administrators attesting
different capabilities of the same role **silently lose one of the two
attestations**; and `import_profile_card` commits the grant, role, quota,
enrollment and receipt and writes the profile only afterwards, so a failure there
answers 500 leaving an **orphan role with no profile**, having also read its
admission, its active boundary and its effective agreement outside the
transaction that uses them. ⚠️ One limit is recorded against the import child in
advance: `federation::revoke` takes no guard, so no guard set here can fence a
concurrent agreement revocation — that ordering arrives with `.3.3.4.12`. Guard
modes are not decided at the split; each child derives its own. Task-tree and
index only; no production source changed and no repair is claimed.

🔴 **A cross-tenant destructive defect was reproduced and closed under
`.3.3.4.10.3` (REPAIR-0117).** `node_inbox` has carried a `tenant_id` column
since migration 0003 and none of the three operator verbs used it. Measured on
the superseded routes, with an administrator of tenant A acting on a node whose
inbox rows belong to tenant B: quarantine answered **200** and quarantined the
foreign row, replay answered **200** and re-sequenced it, and prune answered
**200** with `{"deleted":2,"before":2,"after":0}` — **destroying both of the
other tenant's rows**. It was reachable by any ordinary tenant administrator
through the supported HTTP surface. ⚠️ Three of this project's own fixtures
depended on the defect, seeding a node into one tenant and administering it from
another, which is why nothing caught it; they are CORRECTED, with every original
feature assertion intact. All three verbs now select bound to the admitted tenant
inside one shared-guard transaction, the prune's before/after counts are bound
too, and each records its outcome. **7 passed / 0 failed**; the affected set
passes **6 suites / 72 tests**. FALSIFIED **3 passed / 4 failed** against the
exact pre-`.10.3` handlers. `.3.3.4.10` is now closed: all five node
administrative mutations the parent's census found are one guarded transaction
with a tenant-bound target and a final effect record.

Node certificate revocation `.3.3.4.10.2` (REPAIR-0116) is now ONE guarded
transaction, taking the **exclusive** guard — the opposite answer from `.10.1`
one commit earlier, and derived rather than alternated: it advances the tenant's
revocation epoch, so it is a revocation in the sense the guard contract means and
must fence admissions. Its superseded shape ran the tenant-bound existence probe
as a SEPARATE pool query and then mutated on `node_id` alone, so the check and the
act read two different snapshots; both are now one transaction, the node row is
selected `FOR UPDATE` bound to the admitted tenant, and the UPDATE carries its own
tenant predicate. The submitted reason is PERSISTED with the effect rather than
checked for blankness and discarded, and gains the 1 024-byte control-character-free
bounds; `revoked_at` is the transaction's own database time; every answer carries
the receipt. The single `409` still answers both idle states and the record
distinguishes a repeat (`no_op`) from a node that never had a certificate
(`refused`). **30 passed / 0 failed**; the affected set passes **5 suites / 74
tests**. FALSIFIED **25 passed / 5 failed** against the exact pre-`.10.2` handler,
where the request COMPLETED while a shared holder held the tenant's guard. ⚠️ The
discriminating fixture is a SHARED holder here and was ATOMICITY for `.10.1` —
same parent, same sequence, two different correct fixtures, because one repair
changed the lock mode and the other did not.

Node administration `.3.3.4.10` is censused and split (REPAIR-0114), and its
first child `.10.1` (REPAIR-0115) puts enrollment-token issuance onto the guarded
effect transaction. The census measured something worse than the parent's note
assumed: **four of the five node administrative mutations carry no tenant
predicate at all** — the three inbox verbs and the certificate revocation select
rows by `node_id` alone, and revocation's tenant-bound existence probe is a
separate pool query outside the mutation's transaction. That finding is annotated
at `.3.5`, which owns real target ownership; `.10` owns putting the verification
inside the mutating transaction. `.10.1` takes the **shared** guard, not the
exclusive one `.9` took, and the reasons are measured: the refusal is decided
atomically by one insert that does nothing on conflict, same-node contention is
already settled by the partial unique index, and issuance touches nothing the
reservation path reads. `expires_at` now runs from the transaction's own database
time; every admitted answer carries the receipt, and the pre-admission validation
refusal deliberately does not. **11 passed / 0 failed**; the affected set passes
**6 suites / 84 tests**. FALSIFIED **7 passed / 4 failed** against the exact
pre-`.10.1` handler, where the token COMMITS although its evidence could not be
written (`left: 200, right: 500`). ⚠️ A shared-guard repair cannot be falsified by
a lock-holding fixture in EITHER mode, because the superseded admission took the
shared guard too — the discriminator is atomicity, and the ordering property is
labelled a regression control rather than presented as proof. ⚠️ Reproduced and
routed to `.3.5`, not introduced here: the one-unused-token index is global rather
than per tenant, so one tenant's outstanding token both reveals itself to, and
blocks, another tenant's administrator for the same node identity.

Spend-breaker arm and reset are now ONE guarded transaction under `.3.3.4.9`
(REPAIR-0113) — the SECOND family onto the effect transaction, and previously the
weakest administrative path in the server: each verb admitted the caller under a
shared guard in its own transaction and then mutated **on the connection pool**,
outside any transaction, under no guard, recording nothing about what it did.
Both now run one exclusive-guard transaction holding database time sampled after
the wait, the admission, tenant-bound `FOR UPDATE` selection, the mutation and the
effect record. The exclusive mode is measured rather than copied: classifying an
arm against a row that MAY NOT EXIST cannot be done under a row lock, and the
reservation path holds the shared guard, so exclusive is what orders an arm
against every in-flight reservation in the tenant. Status codes, messages and
success bodies are byte-unchanged; the one addition is the
`x-reasonbraid-authorization` receipt on every answer, without which the effect
record is unreachable. The `409` that has always collapsed "no breaker armed" and
"armed but untripped" still does, and the record distinguishes them as
`refused`/`invalid_transition` against `no_op`. Neither verb takes a caller
reason, so neither invents one, and neither advances the revocation epoch —
asserted on the success path too, which is where a copied-from-`.8` mistake would
show. **25 passed / 0 failed** (16 from `.8` plus 9 new); the affected set passes
**5 suites / 94 tests**. FALSIFIED against the exact pre-`.9` handlers restored
from `HEAD`: **17 passed / 8 failed**, with an arm APPLYING while another
operation held the tenant's authority guard, and
`authority_that_ends_while_a_breaker_request_waits_refuses_it` reporting
`left: 200, right: 403`. ⚠️ That falsification also corrected the controls: an
EXCLUSIVE-holder fixture could not discriminate the repair, because the
superseded admission took a SHARED guard and waited behind it too — promoted to
`docs/knowledge/proving-a-race-is-closed.md`, and it applies to `.10`–`.12`.

Grant and boundary revocation are now ONE guarded transaction under `.3.3.4.8`
(REPAIR-0112), and it is the first route to produce a final effect record. The
route used to run two guarded transactions — a shared-guard admission, then an
exclusive-guard service — so a revocation could apply on authority that had
already stopped holding. One `transact` now holds database time sampled after the
guard wait, the admission, tenant-bound `FOR UPDATE` selection, the status change,
the epoch bump and the effect record. The two superseded services are DELETED,
not deprecated. Every answer including refusals carries
`x-reasonbraid-authorization`; a 403 or 400 records its admission only, while a
404 records `refused`/`not_found` in the caller's own tenant, preserving `.3.1`'s
one indistinguishable answer for missing and foreign. Two documented wire
changes: the reason gains the 1 024-byte control-character-free contract, and
`revoked_at` is the transaction's own database time. **16 passed / 0 failed** on
three independent runs; FALSIFIED **11 passed / 5 failed** — the pre-`.8` shape
makes `authority_that_ends_while_a_revocation_waits_refuses_it` report
`left: 200, right: 403`, the revocation applying after the caller's own
administration ended while it queued. `.9`–`.12` adopt the effect record next;
until each does, its operations have no effect row, which reads as an absence and
never as a success. `.9` is now done; `.10`–`.12` remain.

`.3.3.4.7.3` (REPAIR-0111) corrects a defect in `.7.1`'s own representation,
found by its first consumer. The stored refusal code was typed against the §9.8
`KnownReasonCode` registry so the record and the response could not disagree; the
registry has no `not_found`, which is exactly what a revocation's 404 returns. A
census rejected the obvious one-value patch: the registry publishes **20** codes,
the product emits **19** distinct ones, **10** of those are absent from the
registry and **11** registry codes are never emitted. A second census — parsing
all 14 administrative handler bodies — fixed the replacement at **three** domain
refusals of an admitted operation, `invalid_command`, `invalid_transition` and
`not_found`, whose wire names are literally the strings the response carries.
No migration: the `outcome` column's `CHECK` pins only the `kind` discriminant.
Core: **68 passed / 0 failed**, strict lint rc=0. The registry drift itself is
routed to `.11.7` with its two-way census attached.

The final administrative effect record now has durable storage under `.3.3.4.7.2`
(REPAIR-0110). Migration 0058 adds `administrative_effects`, keyed by the
admission's own `authz_…` id, with the `kind` discriminant of both JSON columns
constrained to the declared vocabularies and a COMPOSITE foreign key
`(record_id, tenant_id)` so an effect cannot cite another tenant's admission.
The writer runs on the caller's already-guarded connection and takes no guard of
its own, so evidence and mutation share one commit and an evidence failure rolls
the protected write back; the reader filters by tenant before decoding. The
layering is deliberate: the database constrains the discriminant and the core
codec enforces everything beneath it, asserted by five rows that pass the column
`CHECK` and fail the codec. `bash scripts/run_pg_tests.sh administrative_effects`
returns rc=0 with **8 passed / 0 failed**, cluster stopped and removed. FALSIFIED
**5 passed / 3 failed** against three individually attributable injections — a
dropped composite key admits a cross-tenant effect, a widened `CHECK` admits an
invented `grant_revoke_all`, and a writer swallowing its error stops reporting a
duplicate — while five controls passed at that same baseline. ⚠️ The FIRST
falsification attempt returned 0 passed / 8 failed on invalid injected SQL: a
negative build where controls the injection cannot touch also fail is a broken
fixture, not a falsification. The 26 fixture plans that delete
`authorization_records` were censused and updated before the constraint
shipped; the sweep also hit one NON-plan array (`migration_upgrade.rs`'s
pre-upgrade snapshot at version 55), which the adjacent-suite gate caught
with `relation "administrative_effects" does not exist` and which is
reverted — the re-audit is structural, parsing every `delete_tables(…)`
argument list rather than matching a line's shape.
No production route wrote an effect record at that leaf; `.3.3.4.8` is the first
producer and `.3.3.4.9` the second.

The final administrative effect representation is defined under `.3.3.4.7.1`
(REPAIR-0109), in `reasonbraid-core` only: no schema, no server change, and no
route writes an effect record yet. An admission record says a caller was allowed
to ASK; it says nothing about what the local mutation did, and `.7.2` then `.8`
close that. The closed operation set was measured rather than chosen — all **27**
guarded-admission call sites in `api.rs` classified as 9 reads and 18 mutations,
of which **14** belong to this family's `.8`–`.12` children and 4 are routed to
the top-level `SIGNOFF-REPAIR.5.2`/`.7.1`. The 14 partition 2+2+5+2+3 across those four children, reproducing
scopes they declared independently. Operation and target are one closed type, so
an operation carrying a target that cannot belong to it is unrepresentable; every
target is caller-supplied, so it exists at refusal as well as at success. The
outcome is `applied`/`no_op`/`refused`, where only `applied` asserts a protected
change and a refusal names a §9.8 `KnownReasonCode`. `cargo test -p
reasonbraid-core` returns **68 passed / 0 failed** including 10 new controls; the
object-only control is FALSIFIED against a permissive decoder (`["breaker_arm"]
decoded as an operation`, 9 passed / 1 failed) and the file restored. A control
also refused the leaf's own first draft: `Option<T>` does not make a serde field
required, so a dropped `submitted_reason` would have read back as "no reason
submitted".

Standalone read admissions are ordered against authority changes under
`.3.3.4.6` (REPAIR-0108). The eight frozen-tenant reads COMMIT a record naming
the selected parent's status and the grant's selector, and that evidence was
being selected with no ordering against the writer that changes it; so was the
ordinary standalone admission behind thread inspection, node-token issuance,
automatic thread creation, recruitment calls and every `tenant_admin` gate.
Baseline `command_ordering`: 4 passed / 3 failed — the admission committed under
a held exclusive guard (`admission rows 0 -> 1`), authority that ended during a
wait still returned 200, and the thread-inspection admission committed too. The
repair takes the SHARED guard before any authority row is read and samples
`clock_timestamp()` after that wait, in `authorize_tenant_admin_inspection` and
in the new `authorize_guarded`; `authorize` is unchanged as the explicit-time
standalone API. The nested-transaction hazard was censused first: all 28
guarded-admission call sites, 0 with an open transaction before them. The
approved frozen-boundary carve-out keeps its own control, so a boundary revoked
while a read waits still admits its eligible administrator. `command_ordering`
goes 4 passed / 3 failed → 7 passed / 0 failed and the affected set passes rc=0
with 5 suites / 71 tests / zero failures. Response queries still run after the
admission commits, so a receipt proves admission, not a shared snapshot or
delivery.


The overdue `CLAUDE.md` §8 artifact review ran under `.11.4.3.1.7`
(REPAIR-0107), with the judgement in a tracked instrument rather than in a
habit. `scripts/census_pg_test_clusters.py` censused **13 retained PostgreSQL
clusters / 676,882,885 bytes**, all `state: stopped`, with 0 live servers and 0
tracked citations, and retired **11 / 573,416,291 bytes** after re-checking
every condition immediately before each removal; the residue census verified the
2 survivors, both held back by the age floor because they are this session's own
baselines. Its `--self-test` fires six refusals plus the citation guard's two
directions. The measurement corrected the leaf's own starting guess:
`target/debug/deps` is 134,478,136 KiB against `incremental`'s 58,989,732 KiB,
so the cache Cargo does NOT collect is the larger one and `.11.4.3.1.6` retired
only the other. Nothing under `target/debug` was deleted — that directory is
what the linker resolves by hash — and `.11.4.3.1.8` owns the decision, with
"measured and not worth acting on" available as a legitimate outcome.


The ordering chapter's expiry-during-wait row is now a control rather than an
argument under `.3.3.4.4.1` (REPAIR-0106). `.3.3.4.4` published five contract
rows and shipped two controls; the census behind the finding is
`grep -n 'async fn a_\|async fn the_' crates/reasonbraid-server/tests/command_ordering.rs`
returning 2 test functions, and `grep -rn 'UPDATE authority_grants'
crates/*/tests/*.rs` returning 21 sites across four suites of which none runs
while a request is blocked. The new control holds the exclusive guard, ends the
caller's grant underneath the blocked command and asserts 403 with an unchanged
event count; the suite goes 2 → 3 controls at rc=0. It is falsified rather than
merely green: with `acquire_in_tx` removed, `command_ordering` returns 1 passed
/ 2 failed, and `api.rs` was then restored with an empty `git diff HEAD`. The
acceptance as opened was corrected by building it — the control separates the
WAIT, not a database clock from a process clock, because both are sampled after
the wait on this path. No production source changed.


Node results are ordered against authority changes under `.3.3.4.5`
(REPAIR-0105), and the defect was the same shape as `.3.3.4.4` one path
further on: `node_channel::events` opened a plain transaction, locked the
node's LEASE row, wrote the receipt and folded the result while authorizing on
the PROCESS clock, with no tenant guard anywhere. Baseline under a held
exclusive guard: the result ran to completion and wrote both rows
(`receipts 0 -> 1, contributions 0 -> 1`), authority that ended while a result
waited was still used to fold, and an exclusive guard could be acquired freely
while the handler sat blocked on the lease row — the inversion, observed in
`pg_stat_activity` rather than argued. A SECOND defect was found by these
controls and had not been suspected: `events` discarded the application's error
and committed regardless, so a SQL failure's COMMIT ran as a ROLLBACK while the
handler answered `200 {"accepted":true}` for a receipt that was never written
(measured with an injected, reverted `event_log` trigger: `node_events` rows 0).
The repair resolves the effect's tenant with a lock-free read, takes the shared
guard BEFORE the lease, binds every effect's SQL to that tenant, samples
`clock_timestamp()` after the guard and idempotency waits, and probes the
transaction with `SELECT 1` before COMMIT. An ordinary channel receipt takes no
guard, because it has no tenant-bound effect to order. `node_result_ordering`
goes **2 passed / 4 failed → 6 passed / 0 failed**; the affected family passes
at rc=0 with **7 suites, 46 tests, zero failures**
(`node_result_ordering node_work node_channel node_replacement node_inbox
quarantine command_ordering`), cluster stopped and removed, and `budget` keeps
its 7 settlement controls green. Node credential/lease proof, partial-result
handling and budget settlement guarantees keep their own owners `.4.1`–`.4.5`.


Thread commands are ordered against authority changes under `.3.3.4.4`
(REPAIR-0104). The defect was not a narrow race window: `run_thread_command`
opened a plain transaction, claimed its idempotency key and authorized on the
PROCESS clock with no tenant guard anywhere, so a command and a revocation had
no defined order at all. The reproduction is therefore deterministic rather than
a race — an exclusive guard is what a revocation holds, so the control holds one
and watches the command run underneath it: `event rows 0 -> 1` at baseline,
while the sibling control (shared guard, unrelated tenant) passed. A FALSE
reproduction was caught first and is recorded: `event rows 0 -> 0`, a request
rejected at deserialization that "completed" without reaching any lock and
looked exactly like the defect. Asserting the effect count beside the timing is
what separated them. The repair factors the guard runner's own per-key
acquisition into `acquire_in_tx`, so both entrypoints take the identical lock
rather than a reimplementation; the mode is Shared, so commands stay concurrent
with each other while a revocation's Exclusive mode fences them. Decision time
is now `clock_timestamp()` sampled after the guard and idempotency waits, so a
grant that expires while a command is queued is evaluated as expired.
`authorize_in_tx` keeps its explicit timestamp parameter, so the standalone
API's documented evaluation-time compatibility is untouched. The FULL owned PostgreSQL collection passes at **rc=0** — 41 commands, 42 suites, **295 tests, zero failures** — with `pg-tests: stopped and removed target/pg-tests/run-n7nqvz4b`. That breadth is the right gate here rather than a focused set: every HTTP and MCP command routes through the transaction this leaf changed, and the new `command_ordering` suite is registered in the runner so it travels with the collection and with CI instead of being run by hand.
Replay-hash and consent semantics remain `.3.4`; automatic-initiation preflight
remains `.5.2`.


Authority writer coverage is re-derived under `.3.3.4.3.4` (REPAIR-0103), and
the first finding was about the evidence rather than the code: `.3.3.4.1`'s
census existed as a table plus a corpus hash with **no tracked producer**, so it
could not be repeated and a rebuilt predicate would have looked comparable while
differing by a name. `scripts/census_authority_paths.py` is now that instrument,
with a `<sha>` mode, and its correctness check is exact reproduction of the
recorded baseline at `1ba6184` — 101 files, 1,749,975 bytes, SHA-256 `340c4af6…`
and 42 locations, all four matching. Only because they match is the new number
evidence. Re-run at the current commit: the corpus has grown to 111 files /
1,971,693 bytes while direct named-call locations fell 42 → 39, and a per-file
diff separates two events a total would have merged — three calls MOVED into the
new `authority/issuance.rs` (net zero, a module split), and three
`create_grant_in_tx` sites genuinely went away, that name now having zero
references including its declaration. Obsolete bridges: none remain — 86
functions declared across nine authority modules, 10 unreferenced, all 10
`#[test]` functions reached by the harness. The two unguarded families are
re-verified rather than assumed: `revoke_node` still calls
`bump_revocation_epoch` on a raw transaction and `import_profile_card` still has
no guard, both matching their recorded owners `.3.3.4.10` and `.3.3.4.11`, so
the ownership table needs no correction. The compatibility set passes live at
rc=0 — 109 tests across seven suites including `migration_upgrade`, cluster
stopped and removed. Limits are unchanged and not widened by re-running the
instrument: bounded, lexical, blind to dynamic dispatch and arbitrary SQL, and
not a security-boundary proof.


The CLI's configured endpoint is checked on every verb under
`.3.3.4.3.3.3.3.2.3.3` (REPAIR-0102), and the reproduction upgraded the finding
from reasoned to demonstrated: pointed at
`http://operator:secret@127.0.0.1:<port>`, an ordinary verb sent
`authorization: Basic b3BlcmF0b3I6c2VjcmV0` — the transport converts URL
userinfo into credentials on the wire rather than ignoring it.
`ApiClient::for_base` now canonicalises through the same `canonical_server` the
bootstrap path has always used, and `ApiClient::new` is crate-private, so the
only construction reachable from outside the crate is the checked one; all 22
`lib.rs` call sites are migrated. The control asserts on what the ORIGIN
received, because the claim is that nothing was sent, and that the refusal does
not echo the credential. A live base is normalised while a stored bootstrap
identity must already be canonical — configuration input and a durable binding
are deliberately different. The whole CLI crate passes `--all-targets` at **rc=0** — 12 library, 4 bootstrap-state, 5 end-to-end, 5 transport, 4 state-publication and 12 state-writers tests — with the run status captured BEFORE any pipe, after an earlier run in this session reported exit 0 through a `grep` while a test had failed. Severity is unchanged by the
measurement: the input is the operator's own configuration, so this is hardening
rather than a third-party escalation.


The CLI's transport is bounded under `.3.3.4.3.3.3.3.2.3.2` (REPAIR-0101). The
baseline is the defect stated precisely: against an origin that completes the
TCP handshake and never answers, `run_enroll` **did not return within 90
seconds**. That matters more here than in an ordinary client because the CLI
persists a bootstrap request key BEFORE dispatch and holds the state lock across
the response — both deliberate — so unbounded, one silent peer held the store for
the life of the process. The bounds are a 10 s connect ceiling, a 60 s
whole-request ceiling covering the body, an 8 MiB reply ceiling read in chunks,
and no redirect following. The 60 s figure is derived rather than chosen: the
server's whole-operation budget is 15 s and a caller can wait behind another
caller's operation first, so the worst legitimate case is about 30 s and the
bound is twice it. The reply ceiling matches the store's own 8 MiB limit,
because a reply the store could never hold cannot become a published outcome.
Redirects are refused because the base is operator-configured and a redirect
would carry the development principal header and a bootstrap key to a host
nobody named. What survives a refusal is what recovery needs: the ORIGINAL key,
a released exclusion, and a reused key on the next attempt. Four controls pass
plus the whole CLI crate at `--all-targets`, with two reverted injections and
three repeats plus one run under deliberate saturation. Repeat 1 FAILED — `the refusal took 75.032698709s` against a 60-second bound, with a clippy running alongside, because the assertion was `< 75s`. That failure IS the evidence for the margin: wall clock contains the runtime scheduling as well as the deadline, so a margin tight enough to separate 60 from 75 measures the host, not the client, and fails wherever saturation is normal. The margin was widened to 100 s, the declared 60 s value pinned deterministically against the number the book documents, and the reply control timing assertion replaced with a semantic one. Repeats 2 and 3 then passed, and the corrected controls passed again with all 12 cores deliberately saturated (load average 1.86 → 14.15, 134.46 s, 4 of 4) — heavier load than produced the original failure. A first attempt at that loaded run was discarded rather than counted: its clippy was cached, checked one crate and loaded nothing. This bounds ONE process's transport; process death mid-request,
server restart and filesystem failure remain `.3.3.4.3.3.3.3.3`, and a timeout
still says nothing about whether the server committed. One census routed out
rather than folded in: `canonical_server` guards only the bootstrap path while
`ApiClient::new` has 22 unvalidated call sites, so 1 of 23 construction paths
checks the configured endpoint — owned by `.3.3.4.3.3.3.3.2.3.3`.


The enforcer now runs **17 registered checks**. `INDEX-FRONTIER` (REPAIR-0100)
extends to the project's own index the rule `BOOK-FRONTIER` already applied to
the book: `docs/TASK_TREE.md`'s Frontier column may not name a leaf the owning
tree's row 1 does not. Measured rather than estimated — over the last 60
commits, 39 breach and 21 genuinely agree, and the drift begins at `1ebfebe`,
the commit that CLOSED the leaf the row then went on naming for 39 commits. The
rule is deliberately narrow because the census rejected the obvious one: 8
completed trees write a dash row and point at the NEXT tree's first leaf, and
`PHASE-8`'s row 1 is a cross-tree prerequisite its cell summarises accurately,
so blanket equality would flag legitimate rows. The generator that was this
leaf's provisional preference was rejected on the same census — it would destroy
accurate curated prose in 13 of 14 rows. Only an ACTIVE tree whose row 1 names
its own leaf is checked, which is also the only row that moves. The gate caught
its own author on its first live run: closing the leaf moved row 1 and the index
still pointed at the leaf being closed.

The enforcer previously reached 16 checks with `LOCKSTEP-CLAIM`. `LOCKSTEP-CLAIM` (REPAIR-0099)
closes the half `TASK-ACCEPTANCE` structurally cannot: that gate proves a leaf's
box is ticked and cites something re-runnable, never that the cited edit landed.
A ticked bold LOCKSTEP box may no longer name a core live document the commit
does not stage. It is keyed on the author's own claim rather than a blanket
requirement, because the census showed the blanket rule was wrong — over 25
commits, 10 closed a leaf and CHANGELOG.md was staged 10/10 but MEMORY.md only
6/10, so "closing a leaf must stage MEMORY" would have asserted something the
project does not do, flagged four innocent commits, and still missed the defect.
Re-measured with the finished gate over 30 commits: 11 add a claim and were
genuinely exercised, 10 pass, exactly one (6bf0c40) breaches, zero false
positives; the other 19 pass vacuously because they add no claim, and that
distinction is recorded so 30 green is not read as 30 tests. The honest escape
is `lockstep: <doc> unchanged (<why>)` in the claim's own section, and the
refusal says explicitly that deleting the document's name is not a discharge.
`scripts/check_lockstep_claim.sh --against <sha>` re-runs the predicate over any
past commit, which is what made the leaf's acceptance executable rather than
argued.


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
| Corrective review | In Progress | `.1`, `.2.1`, `.2.2`, `.3.1` complete; `.3.2.1` site service passes ten live controls and strict lint. `.3.2.2` operator CLI passes 3 live controls plus adjacent service/ownership checks and strict lint; `.3.2.3` HTTP enforcement passes 8 focused controls; the selected 58-test security run and final 12-test registry run pass. `.3.3.1` core subject serialization passes 49 unit + 3 subject controls and 40 live compatibility tests; `.3.3.2` bound evaluation passes 51 core unit + 3 subject tests, 6 evaluator controls, 32 live authority/command API tests and strict lint; all results consumed and the owned cluster removed. `.3.3.3.1` actual-parent command selection passes 37 live tests, six evaluator controls and strict lint. `.3.3.3.2.1` frozen-read eligibility passes 40 live tests, ten pure evaluator controls and strict lint; all results consumed and the owned cluster removed. `.3.3.3.2.2.1` provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all confirmation/shutdown results consumed, owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint; all results and shutdown consumed. Exact scoped receipt readback passes 18 live authority tests and 30 HTTP tests plus strict lint, including final denied-record readback; all results/shutdown consumed. Actual-parent selection and frozen-read audit/readback `.3.3.3` are complete; The `.3.3.4.1` source census/contract is complete (42 named-call locations plus transitive/mutation coverage). `.3.3.4.2` qualifies the guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass, including cancelled-BEGIN replacement and the formerly stale authority executable. Seven acceptance-checker controls also pass, correcting nested-evidence owner selection while preserving real-tree box refusals. All results/shutdown consumed; owned clusters/probes removed. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. Bounded HTTP waits and restart qualification follow; ordinary no-key behavior remains intentionally distinct. The final administrative effect REPRESENTATION `.3.3.4.7.1` passes 10 representation controls within 68 core tests, strict core lint and the falsified object-only control; its census derives the closed fourteen-operation set from 27 guarded-admission call sites. Its STORAGE `.3.3.4.7.2` then passes 8 live controls (falsified 5/3 against three attributable injections): migration 0058 keyed on the admission with a composite tenant-binding foreign key, a writer on the caller's already-guarded transaction, and a tenant-filtered reader. `.3.3.4.8` is the first producer of an effect record; `.3.3.4.9` is the second, putting spend-breaker arm/reset onto one exclusive-guard transaction — 25 live controls, the affected set 5 suites / 94 tests, falsified 17/8 against the exact pre-`.9` handlers. |

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

Inherited state-lock release is now repaired under .11.4.3.1.2.11.2: the private
guard explicitly unlocks before File close, including post-acquisition failures.
A permanent five-path child control fails on unchanged production and passes after
repair; all 32 selected CLI tests, final twelve-writer rerun, six raw-fork actual-API
scenarios and strict CLI lint/format pass. Two test probes release explicitly too.
Original assertions and 339 other non-Markdown sources remain unchanged. Broader
inherited-descriptor abrupt-owner-death qualification stays owned by the existing
restart leaf. Full checkpoint/public push/remote CI remain pending, with all
qualification categories unchanged. Evidence:
docs/tasks/artifacts/signoff_review/state-writer-lock-release.md.

The ec8df08 checkpoint passes eight gates and the lock controls, then stops at two
PDF tests; PostgreSQL/demo remain unstarted. An unchanged-binary wrong-ZIP result
and three exact-helper native-clock collisions establish unsafe fixture ownership.
Exclusive unit/stdio inputs are repaired under .11.4.3.1.2.12; all twelve selected
tests, strict lint/format and three independent locality cases pass. Original PDF paths
were not captured. The analogous production R2 path now has immediate next repair
owner .7.3.3 before the full checkpoint; Git scratch and remaining fixture names
have separate concrete owners. Qualification categories remain unchanged. Evidence:
docs/tasks/artifacts/signoff_review/extraction-fixture-ownership.md.


Production-boundary diagnosis .7.3.3.1 now reproduces eight wrong-owner extraction
responses through the exact input acquisition span and unchanged worker; two
own-input controls pass. No HTTP/database reproduction or production fix is
claimed. Direct-worker completion .7.3.3.2 and exclusive input/digest integration
.7.3.3.3 precede the full checkpoint; larger transport/retention limits stay .7.3.4.
Qualification categories remain unchanged. Evidence:
docs/tasks/artifacts/signoff_review/extraction-input-boundary.md.


Direct-worker completion .7.3.3.2 is repaired and qualified under REPAIR-0060.
The permanent controls first reproduced the defect on the unchanged spawner:
`7 passed; 1 failed`, with an early request failure leaving pid 39486 in the
process table after the spawner returned. Every exit path now passes through one
bounded stop and reap, and every return reports never-started, consumed or
explicitly unconfirmed completion; a failed stop request is recorded rather than
read as a termination. Sixteen process/evidence controls, six spawner controls
(four synthetic injections), 81 server library tests, twelve adjacent extractor
tests, strict server lint and workspace format pass; api.rs is byte-identical.
Exclusive same-volume inputs and digest-bound cleanup remain .7.3.3.3; pipes,
descendants and aggregate retained storage remain .7.3.4. No HTTP, database,
full-CI or push claim follows. Qualification categories remain unchanged.
Evidence: docs/tasks/artifacts/signoff_review/extraction-worker-completion.md.


The 2026-09-11 book read found the superseded private-visibility instruction
still live in the book introduction and the governance charter after two hand-run
corrections, and the book roadmap page naming a frontier seven committed leaves
stale. REPAIR-0061 corrects both statements against the unchanged director
ruling, removes the duplicated frontier in favour of the per-leaf maintained
qualification page, and registers two self-tested checks — VISIBILITY-POLICY and
BOOK-FRONTIER — each falsified by reintroducing the exact defect it exists for.
Documentation and enforcement only: no visibility, remote, production or
qualification-category change. Decision:
docs/decisions/2026-09-11_mechanized-document-invariants.md.

Exclusive extraction input ownership .7.3.3.3.1 is complete under REPAIR-0063.
One private 0600 file per request is created under a runtime-discovered
repository root with checked owned parents; occupied candidates are skipped
whole and removal requires both a finished reader and an unchanged (device,
inode, one link) identity, retaining anything else with its relative path named.
Eleven controls pass — 32 simultaneous creators hold 32 distinct documents, and
the real worker describes an owned input by its own digest. 90 server library
tests, 16 completion controls, 12 adjacent extractor tests, strict server lint
and format pass. One control was itself racy on ambient worker selection; a
widening probe and its own retained input identify it, and the serialized
controls pass 40 repeated runs. api.rs is unchanged and still carries the
superseded span until .7.3.3.3.2. Qualification categories are unchanged.
Evidence: docs/tasks/artifacts/signoff_review/extraction-owned-input.md.

The R2 wiring .7.3.3.3.2 is complete under REPAIR-0064, closing the production
input boundary .7.3.3. The handler's extraction leg is one bound call: the
acquired bytes become an owned private input, and a response whose parent digest
is not that input's digest is refused as `extraction_source_mismatch` before any
snapshot, derivation or receipt write. Seven integration controls pass, including
the mismatch refusal naming both digests and eight concurrent callers each
receiving their own document. 90 server library tests, 16 completion controls,
12 adjacent extractor tests, strict server lint and format pass, and the live
adjacent profiles suite passes 31 tests with its cluster stopped and removed.
One gap is explicit with its census: no live test drives a SUCCESSFUL R2
acquisition through to a snapshot and derivation, because the live R2 test
refuses at the loopback destination gate. Concrete owner: .7.3.3.4.
Qualification categories are unchanged; no full-CI or push claim follows.

That join is closed under .7.3.3.4.1 (REPAIR-0095). `ApiState::with_acquisition`
lets a deployment supply the R0 fetcher its acquisition legs use, while `new`,
`with_gate`, `api_router` and `api_router_gated` all keep building the shipped
https-only public-destination policy. A live control serves one Atom document
from a local origin and resolves ONE reference through three deployments that
differ only in that fetcher, so each production gate is measured on its own: the
shipped state refuses `scheme_not_allowed`, the shipped destination policy
refuses `destination_refused` naming `loopback`, and a loopback-admitting policy
acquires. The evidence is then read back and asserted against the served bytes —
the snapshot's `raw_digest` and `byte_length`, the resolver id and locator, and
the derivation rows' content and digests — with the refused reference asserted to
hold zero snapshots. The profiles suite passes 32 of 32 live and its cluster was
stopped and removed; 97 server library tests, 16 completion controls, seven
owned-input controls, strict all-target server lint and format pass. Three
injections, each reverted and re-run green, prove the control goes red: one extra
served byte (which left the chunk digests identical, so the raw-byte leg is the
one that caught it), a discarded supplied fetcher, and a derivation whose content
is not the worker's chunk.

That control also MEASURED a finding rather than inferring it: the R2 pack
advertises five media types its own acquisition leg refuses. The R0 sniff accepts
a declared content type only when it is `text/html`, `application/xhtml+xml` or
`text/*`, so the identical feed succeeds served as `text/xml` and is refused
`media_type_refused` served as `application/atom+xml`. A caller is therefore
ranked onto a resolver that cannot acquire its document and receives an
acquisition refusal instead of the explicit `resource_unresolvable_now` the §12.2
contract reserves for "no eligible resolver". The per-format census and the
repair decision are owned by .7.3.3.5. Neither the registry row nor the sniff
changes before that census exists.

The census is complete under .7.3.3.5.1 (REPAIR-0097), with NO production
behaviour change, and it changed the finding's shape. Two mechanical controls in
the fetcher's own module measure the predicate: a DECLARED content type is
accepted only from `text/html`, `application/xhtml+xml` and any `text/*` — three
arms, enumerated in both directions — and all five advertised types are refused.
UNTYPED, the verdict is a property of the bytes rather than of the format: an
all-printable body is accepted whatever format it belongs to, and the same body
with one non-text byte is refused; ZIP and tar cannot reach that branch by
construction. So "the five advertised formats are unacquirable" is true but the
wrong SHAPE — the leg's rule is not about formats, and no subset of the
advertisement satisfies it. Narrowing the registry row is rejected on that
measured ground; the accepted repair is that the acquisition leg admits the
RANKED resolver's own advertised media types, with the destination policy,
scheme list, byte ceiling, ratio brake, redirect policy and time ceiling all
explicitly outside the change. The controls pass on unchanged production, as a
census must, and were falsified by adding `application/pdf` to the accept set.
The census measures the predicate, not real files of each format; a given real
PDF's untyped verdict depends on that PDF and is not claimed. Decision:
docs/decisions/2026-09-12_r2-acquisition-accept-set.md.

The repair ships under .7.3.3.5.2 (REPAIR-0098). `sniff_kind` takes the ranked
pack's advertised types and returns the additive `SniffedKind::DeclaredType` for
a declared type in that set; `Fetcher::fetch_admitting` supplies it while
`fetch`, `fetch_head` and `fetch_authenticated` pass an empty slice, so R0's
shipped accept set is byte-for-byte unchanged; and
`resolvers::advertised_media_types` reads the row's own media_types per
resolution, returning an EMPTY set for a missing or malformed row so a bad
advertisement cannot widen a gate. Bound census: `git grep -n "fetch_admitting"
-- 'crates/**/*.rs'` returns exactly one call site, the R2 arm; the R0 arm, the
R5 authenticated arm and the R3 preflight pass no admitted set, and no
destination policy, scheme list, SSRF control or ceiling changed. The profiles
suite passes 33 of 33 live with its cluster removed, including the unchanged R0
and R1 resolver controls; 99 library tests, 23 extraction controls, strict
all-target server lint and format pass, and a whole-workspace `cargo check
--all-targets` returns 0. Three injections, each reverted and re-run green: an
admitted set that is not the ranked row's fails the acquisition, an empty set at
the call site fails it identically, and adding ONE unadvertised type makes the
unadvertised document acquire and fails the negative control — the decision's
bound, proved live. One measured fact is deliberately flipped: .7.3.3.4.1's
recorded `media_type_refused` for `application/atom+xml` was the defect, and it
stays recorded as the evidence the repair rests on.

The sibling join .7.3.3.4.2 is closed under REPAIR-0096, with no production
change: the gap was coverage. The seven mismatch controls .7.3.3.3.2 added all
call extract_acquired_bytes directly and never reach a database, so they prove
the refusal is RAISED and nothing proved the handler HONOURS it. A dishonest
worker injected through the existing R2_WORKER_BIN override now returns a
well-formed reply describing bytes the request never supplied; the handler
refuses it `extraction_source_mismatch` naming both digests, and the control
asserts the ABSENCE three ways — zero snapshots for the exact reference, zero
derivations joined to it through parent_snapshot_id, and the whole store's
snapshot and derivation counts unchanged across the request. The profiles suite
passes 33 of 33 live with its cluster stopped and removed; strict all-target
server lint and format pass. Two injections, each reverted and re-run green:
removing the digest binding makes the handler accept the foreign document, and
**writing a snapshot before reporting the same refusal leaves the
extraction_source_mismatch assertion PASSING while the count fails with left: 1**
— the evidence that this control measures the absence rather than the error
kind. Production source is byte-identical to REPAIR-0095 afterwards. The parent
.7.3.3.4 is complete. Evidence:
docs/tasks/artifacts/signoff_review/r2-acquisition-join.md.

The source-165cb3a full checkpoint STOPPED at its fourth command
(`04-pg-demo rc=101`, 3,137s): 36 of 40 suites started, 35 passed with 259 tests,
and `site_authority` returned `9 passed; 1 failed`. Four suites, the
demonstration and gates five to eight never ran. REPAIR-0065 root-causes it to
`migration_upgrade` recreating `public` with `DROP SCHEMA … CASCADE; CREATE
SCHEMA public`, which drops the PUBLIC `USAGE` grant a fresh database ships, so
every later non-owner role could not resolve a qualified name. The fixture now
restores the owner and the grant through one helper, and the site privilege
probe resolves the audit table by catalogue OID so an unprivileged caller is
refused `OperatorRequired` (403) rather than `Error::Sql` (500). This was a
misclassification, not an escalation: the forensic copy shows the outsider never
held operator membership. The reproduced sequence and the affected family of six
suites now pass; the new control is falsified against the unchanged query.
The full checkpoint has NOT passed and no push claim follows.
`02-check` took 3,922s with only 940s accounted; that gap is owned by
.11.4.3.1.2.15. Qualification categories are unchanged. Evidence:
docs/tasks/artifacts/signoff_review/site-operator-schema-usage.md.

The re-run checkpoint on source-5c8609e stops earlier, at `02-check` (rc=2,
2,608s): `pg_guard` panics creating a fixture directory named from the process
id and a clock reading that this host does not advance between concurrent
callers. REPAIR-0067 replaces the clock with a monotonic discriminator and skips
an occupied candidate; 0 failures in 60 parallel runs against 1 in 15 before.
The family census it triggered found two production instances with concrete
owners: evidence identifiers minted from the same shape, measured at about one
distinct value per twelve calls (.7.4.1), and four ambient Git scratch paths
built from the process id alone (.7.2.1). The full checkpoint has NOT passed.
Qualification categories are unchanged.

Evidence identity .7.4.1 is repaired under REPAIR-0068. Snapshot, derivation and
claim-assessment identifiers were minted from a clock and a process id, measured
at about one distinct value per twelve calls; all three now use one `evidence_id`
built on a v7 UUID, with controls measuring 400 concurrent and 1,000 rapid
sequential identifiers all distinct. Because the three columns are primary keys a
collision was always a refused insert, so no stored row can hold another's
identity and there is nothing to reconcile. The storage-failure misclassification
the diagnosis exposed — every fault reported as `ReferenceMissing` and mapped to
HTTP 400, with the R2 pipeline still reporting a successful acquisition — is
routed to .7.4.2. Qualification categories are unchanged.

**The full pre-push checkpoint PASSES at source 7233122** — the first complete
run in this project's recorded history. All eight commands return 0: build,
format/strict lint/workspace tests with the pinned browser, Python controls, the
full owned PostgreSQL collection with an explicit demonstration, thirteen
doctrines, pinned cargo-deny, pinned Gitleaks and the book. 40 of 40 registered
suites ran with 291 tests passed and 0 failed; the demonstration reports ALL
acceptance checks passed; both scanner receipts record scope `gate` with exit 0.
The pass was falsified before publication: no DATABASE_URL skips, 16 real browser
controls rather than an absent-browser early return, and the only ignored tests
are the env-gated live-provider dispatches. This satisfies the condition the
recorded policy places on the already-authorized push. ⚠️ **Two clauses of this
paragraph were stale and are corrected rather than deleted (`.13.4`,
DOC-0042):** it said *"remote CI has never run and must be consumed after it"*
— remote CI HAS run, 41 times, green at `origin/main` (`.13.3`, REPAIR-0220) —
and it listed *"the license decision"* as open, which B5 cleared on 2026-09-15.
No external gate closes: G6/G7 and name clearance remain open, historical phase
closures remain under corrective review, and .7.4.2, .7.2.1, .7.3.3.4,
.11.4.3.1.2.15 and .11.5 remain open. Evidence:
docs/tasks/artifacts/signoff_review/checkpoint-7233122.md.

Evidence storage faults are now reported honestly under REPAIR-0071. Ten map_err
arms across the snapshot, derivation and claim modules collapsed every storage
failure into a caller error that the handlers rendered as HTTP 400; each enum now
carries a Storage variant preserving its SQLx source, the handlers use the
existing internal_with_log idiom, and the R2 pipeline records an
`evidence_unstored` acquisition error instead of discarding a failed snapshot
while reporting a successful acquisition. The control injects a storage fault,
restores the database before asserting, and requires 500 rather than 400 while a
genuinely absent reference stays 400; against the unrepaired source it fails
printing the defect verbatim. Live profiles/evaluation/cards, 92 library tests,
strict lint and format pass. The R2 leg is qualified at the handler boundary
only, because a live successful acquisition has no coverage (.7.3.3.4).

Remote CI ran for the first time on a626768: doctrines and supply-chain pass,
and the rust workflow FAILS at `codex_adapter_passes_the_conformance_suite`
("the lose trigger refused instead of dispatching"). That suite passes locally,
so it is environment-dependent and is exactly what remote execution exists to
find. It is the next repair; no remote-CI green claim is made.

The first remote CI failure is being diagnosed under .11.4.3.1.2.19. The
certification harness discarded the adapter's refusal reason at three arms, so
the CI log carried no cause at all; those arms now append it. The conformance
stubs also stopped naming their directory from the clock, on measured evidence
of their own. The remote cause remains UNPROVED and no remote-CI green claim is
made: the instrument was pushed so the next run names the cause rather than
having it guessed.

The first remote CI failure is root-caused and repaired under .11.4.3.1.2.19.
The instrument pushed as REPAIR-0072 made the next run name its cause: Linux
ETXTBSY, `execve` refusing a stub still open for writing while a parallel thread
forked to spawn. macOS does not enforce it, so every local run had passed.
REPAIR-0072's stub-naming change was NOT the cause and the evidence says so —
the failing path already carried that naming and the failure moved between
adapters. REPAIR-0073 writes one stub per adapter kind per process behind a
OnceLock, removing the race rather than retrying around it. All adapter targets
pass locally, which is explicitly not evidence about Linux; remote confirmation
is still required and no CI-green claim is made.

The conformance repair held on the runner and pg-tests SUCCEEDED remotely for
the first time — the full PostgreSQL collection with its demonstration, on a
Linux runner. The `check` job now fails in reasonbraid-browse: six tests report
browser_launch_failed, "browser exited before publishing a loopback endpoint",
with zero gated navigations observed. The pinned executable's version was
verified by the launcher and --no-sandbox/--headless are already passed, so
neither is the cause. REPAIR-0074 retains the worker's own stderr on failure so
the next run names it. The cause is UNPROVED and no repair of it is claimed.

The browse failure is repaired and remote-confirmed: relative TMPDIR keeps
Chrome's singleton socket inside the 108-byte limit and all browse controls pass
with render_succeeded true. The remaining known `check` failure was four
git::tests::* reporting AuthorMissing, whose cause is proved: gix resolves a
commit signature from git configuration and the fixtures borrowed the
developer's. REPAIR-0080 gives them their own identity. It is the first
remote-only failure reproduced LOCALLY — suppressing ambient git configuration
recreates the runner's condition — and that probe exposed a fourth commit site
the first pass missed. Six git tests pass both with ambient configuration
suppressed and with it present. Remote confirmation of the whole `check` job is
still required and no CI-green claim is made.

REPAIR-0080 is confirmed on the runner: the server library now reports 92
passed, 0 failed there, with the four AuthorMissing failures gone. The check job
still fails, on a different and newly exposed defect — a check-then-act in the
pg_guard fixture parent creation, which loses a race between parallel threads on
Linux and has never lost it on macOS. REPAIR-0082 owns the R1 acquisition
workspace: it is now exclusively created under .project-data/git on the
repository volume, proved by identity before removal, and released when the last
holder drops, which also closes a leak that ran on every SUCCESSFUL acquisition.
97 server lib tests, strict lint and the rendered book pass locally.

REPAIR-0083 repairs the check-then-act in the pg_guard fixture parent, the sole
instance of that shape in the workspace. It was never Linux-only: a probe lost
346 of 640 creations locally, and the suite reproduces the exact CI failure here
once the warm control directory is removed. Six consecutive cold-tree runs pass.
No remote-green claim is made until the runner confirms it.

REPAIR-0085 closes the §13 storage-locality family: 26 breaches repaired across
16 files and a STORAGE-LOCALITY gate registered so the policy is now checked
rather than merely stated. Remote CI is still red, on a DIFFERENT defect each
time rather than the same one recurring: the four AuthorMissing git tests are
confirmed fixed on the runner, the pg_guard race is fixed, and the current
failure is an intermittent ETXTBSY in the conformance stubs which PASSED in the
preceding run. No remote-green claim is made.

REMOTE CI IS GREEN. Run 34652116508 for c17841c passes rust (book, check,
pg-tests), doctrines and supply-chain, with 669 tests passed and 0 failed suites
in check. Verified by re-derivation from the API, falsified for hidden skips,
and durable at origin/main. Push cadence returns to ~300 commits per COMMIT.md.
That local-only navigation-deadline defect is now REPAIRED under .11.4.3.1.2.27
(REPAIR-0089), and it was a product defect rather than a flaky test. The browse
worker returned one error kind for two independent facts, so an unconfirmed
cleanup overwrote the render's own kind — eleven distinct kinds, not just the
budget — and left the caller with an operator's fact. A refusal now carries the
render's kind plus explicit cleanup_confirmed/cleanup_error fields; only a
SUCCESSFUL render under unconfirmed cleanup is still named for the cleanup. The
wire change is additive and the server-side spawner is unaffected. 25 browse
controls pass with the pinned browser; both strict lints and format pass; the
controls were falsified against the superseded expression. The underlying
escaped-writer condition remains real and unrepaired — it is now reported
honestly instead of overwriting a result. That defect is now FIXED and GATED under
.11.4.4 (REPAIR-0092): five leaf sections asserted two different statuses, which
is why .7.4.1 sat on the frontier seven commits after closing, and the census
that found them also found 29 headings at levels 7-11 -- which Markdown does not
treat as headings at all, so the tree's deepest leaves were invisible as
structure. Both are repaired without deleting a word and both are now enforced:
TASK-STATUS and HEADING-DEPTH, each fence-aware, each with a two-sided
self-test, each falsified against the unrepaired tree. The enforcer runs 15
checks.

The checkpoint's unexplained hour is now ACCOUNTED under .11.4.3.1.2.15
(REPAIR-0090), which unblocks .11.5's verification lanes. Of 02-check's 3,922
seconds, cargo reported 813 and the harnesses 127; the other 2,982 are macOS
first-execution validation of each newly written executable on the repository
volume at about 21.9 seconds apiece, fixed rather than size-proportional, cached
per file identity, against about 0.15 seconds on the boot volume. It is not
inherent to the gate: the same four commands on the Linux runner, from a cold
checkout, take 444 seconds with 18.7 (4.2%) unaccounted. The leaf's own leading
candidate, nine rustdoc doctest-harness builds, is at most 8% and is refuted.
Planning number: one more integration-test file costs about 22 seconds of every
future checkpoint. scripts/measure_check_phases.py makes this re-derivable by one
command. Two levers -- a macOS security setting and the repository's volume --
are now DECIDED and CLOSED under .11.4.3.1.2.28 (REPAIR-0093). The macOS
Developer Tools setting is rejected: it exempts the shell from validating
exactly the untrusted-content workers this project builds, and it cannot be
committed, so no other machine or runner would inherit it. The volume move is
rejected on measured capacity: the repository is 3.77 MiB but its build tree is
185 GB, against 249 GB free on the boot volume. The lever actually pulled was on
neither list -- the REMOTE run is now the authoritative pre-push gate, verified
from the workflow files as a superset in which two of the eight commands are
stricter remotely. Nothing is removed, weakened or skipped; only four cheap
local gates run before a push, and the full checkpoint becomes a deliberate
diagnostic. The named trade: gate latency is now bounded by the ~300-commit push
cadence, which remains the director's standing instruction and is unchanged.

Two defects introduced by this session's own work are now task-tree owned under
.11.4.5. The doctrine registry could contain shell expansion -- and the driver
EXECUTED it, on every commit and in CI -- which is fixed and falsified under
.11.4.5.1 (REPAIR-0094) by a self-guard that runs before the registry is
assigned. The second, a LOCKSTEP box claiming a document its commit did not
touch, is NOT fixed: its census is done and decisive (1 breach in 6 commits,
zero false positives, and the obvious blanket rule measured and rejected), and
it sits at frontier row 2 with its acceptance recorded.
