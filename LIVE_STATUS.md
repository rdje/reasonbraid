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
| Phase 7 — Internet qualification | Mostly Done | Hardening machinery exists; G6/G7 Internet exposure remains NOT MET. Local repairs and external threat-model, injection and penetration-test evidence remain required. |
| Phase 8 — federation and interoperability | In Progress | Through regional routing historically recorded; `.5.3` store-and-forward, `.5.4` exit export/import and `.6` G8 remain. Shared authority and protocol gaps are prerequisite repairs. |
| Phase 9 — stable release | Not Started | G9 requires sustained operational evidence and the outstanding release decisions. |
| Corrective review | In Progress | `.1`, `.2.1`, `.2.2`, `.3.1` complete; `.3.2.1` site service passes ten live controls and strict lint. `.3.2.2` operator CLI passes 3 live controls plus adjacent service/ownership checks and strict lint; `.3.2.3` HTTP enforcement passes 8 focused controls; the selected 58-test security run and final 12-test registry run pass. `.3.3.1` core subject serialization passes 49 unit + 3 subject controls and 40 live compatibility tests; `.3.3.2` bound evaluation passes 51 core unit + 3 subject tests, 6 evaluator controls, 32 live authority/command API tests and strict lint; all results consumed and the owned cluster removed. `.3.3.3.1` actual-parent command selection passes 37 live tests, six evaluator controls and strict lint. `.3.3.3.2.1` frozen-read eligibility passes 40 live tests, ten pure evaluator controls and strict lint; all results consumed and the owned cluster removed. `.3.3.3.2.2.1` provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all confirmation/shutdown results consumed, owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint; all results and shutdown consumed. Exact scoped receipt readback passes 18 live authority tests and 30 HTTP tests plus strict lint, including final denied-record readback; all results/shutdown consumed. Actual-parent selection and frozen-read audit/readback `.3.3.3` are complete; The `.3.3.4.1` source census/contract is complete (42 named-call locations plus transitive/mutation coverage). `.3.3.4.2` qualifies the guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass, including cancelled-BEGIN replacement and the formerly stale authority executable. Seven acceptance-checker controls also pass, correcting nested-evidence owner selection while preserving real-tree box refusals. All results/shutdown consumed; owned clusters/probes removed. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. Bounded HTTP waits and restart qualification follow; ordinary no-key behavior remains intentionally distinct. The final administrative effect REPRESENTATION `.3.3.4.7.1` passes 10 representation controls within 68 core tests, strict core lint and the falsified object-only control; its census derives the closed fourteen-operation set from 27 guarded-admission call sites. Its STORAGE `.3.3.4.7.2` then passes 8 live controls (falsified 5/3 against three attributable injections): migration 0058 keyed on the admission with a composite tenant-binding foreign key, a writer on the caller's already-guarded transaction, and a tenant-filtered reader. `.3.3.4.8` is the first producer of an effect record; `.3.3.4.9` is the second, putting spend-breaker arm/reset onto one exclusive-guard transaction — 25 live controls, the affected set 5 suites / 94 tests, falsified 17/8 against the exact pre-`.9` handlers.. Fixture ownership `.11.2.1.1` censuses all 61 `create_dir_all` sites in tracked Rust with 0 unresolved and repairs the 3 that adopted: exclusive per-call creation on the repository volume, falsified against an occupied path, with 89 selected controls across five suites (63 of them `profiles` on a disposable cluster), strict lint for both crates and the 25-check gate green. |

## 2026-09-21 — Every shipped surface is judged, and the coarse signal was wrong in both directions (`SIGNOFF-REPAIR.11.4.6.2`)

`REASONBRAID-REPAIR-0379`. Every surface the product ships now carries a verdict in a guarded table.

- ⭐ **43 members — 30 route families, 10 binaries, and 3 product routes in no `/v1/` family at all** (`/`, `/app.js`, `/style.css`; the pinned population was short, which is `.11.4.6.1`'s binaries finding one lane over). **23 covered · 3 internal · 17 gap**, every gap naming an owning leaf because a finding nobody owns is a complaint (§15).
- 🔴 **The coarse mention test calls 21 of 30 families covered; per ROUTE only 14 are.** Seven it credits carry undocumented routes — `calls` worst at 3 of 4, where `.11.8.1` had already re-derived by hand that the chapter's line is a contract for a DIFFERENT route. The pattern: the reads are written down and the operator's verbs are not.
- ⭐ **And it under-reports too**: `/` scores a bare mention while `web-ui.md` is a whole chapter about it. A signal wrong in both directions has to be adjudicated, not tightened.
- ⚠️ **The weak binary signal hid a real gap**: `rb-release-manifest`'s entire book presence is one table cell, and ADR-027's `pubkey` verb exists so a third party can verify a release without the signing key.
- ✅ `SURFACE-JUDGEMENT` registered at 0.16 s / 0.5%, falsified four ways; `.11.4.6.3`–`.11.4.6.8` hold the seventeen gaps.

## 2026-09-21 — The census demanded one space, and an aligned contract line read as a passing mention (`SIGNOFF-REPAIR.11.8.2`)

`REASONBRAID-REPAIR-0378`. Checking the evidence before keying a judgement to it is what found this.

- 🔴 **The route census demanded ONE space between a method and its path.** `profiles.md:29` writes `GET` + four spaces + `/v1/profiles/{role_id}/card` in an aligned block — a contract line by the instrument's own definition — and it was classified `mentioned`, the class for a path with no method beside it. ⛔ The direction UNDER-reports documentation, which inflates a published gap; it survived `.11.8.1`'s five-sided falsification because every arm there wrote a single space.
- ✅ Corrected: **104 routes — 62 described, 2 mentioned, 40 absent** today; **50 / 2 / 52** at `.11.8.1`'s own commit against its published **49 / 3 / 52**.
- ⭐ The historical harness reproduces `49 / 3 / 52` with the separator unchanged BEFORE the swap, so the delta is attributable to the separator alone.
- ⛔ The `absent` column does not move at either commit: no false gap was ever manufactured, and `.11.8`'s measured-backlog conclusion stands.

## 2026-09-21 — The findings, graded: one did not hold, and it is the batch's own headline (`SIGNOFF-REPAIR.11.2.1.3.2.1.2`)

`REASONBRAID-REPAIR-0377`. The director's *ensure your findings still hold*, over every figure this batch published.

- 🔴 **The census was at 88.4% of the bytes.** A route never tried on it — filesystem → code — found **13 directories / 303,276 KiB** it did not claim: a **shell** producer (`target/demo`, 106,000 KiB) in a third corpus, and eleven **ORPHANS** whose producers are deleted, the largest being `target/claude-stubs` at **270 directories** `git grep` cannot attribute to anything. ⛔ No corpus can reach an orphan; only walking the filesystem can.
- ✅ Fixed: three independent routes now agree to the KiB at **2,619,172**. 🔴 The shell corpus's first version counted the 64 GiB build cache as fixtures, which forced the cargo boundary to be derived rather than assumed.
- ⭐ **One came back stronger**: *the guarded families add nothing* re-derived by decoding each fixture's uuid-v7 creation time from its own name — the newest survivor in every guarded family predates its guard.
- 🔴 Smaller misses: *seven of the nine are `policy-*`* is **six**; `conformance-stubs` moved 34 → 37 entries, exactly as its decision record predicts.

## 2026-09-21 — The book-coverage population, pinned — and a sibling had already closed the finding (`SIGNOFF-REPAIR.11.4.6.1`)

`REASONBRAID-DOC-0111`. Measuring `.11.4.6`'s gap first refuted its own headline.

- 🔴 **The MCP chapter exists and a SIBLING put it there** — `docs/book/src/mcp.md`, 7,539 bytes, since `REPAIR-0294` — so the leaf's second acceptance clause was already met. 🔴 Both its figures moved, published unpinned: **30** route families, **22** `SUMMARY.md` entries (18 content).
- ⭐ **The population reframes the backlog: 9 of 30 families are mentioned by NO chapter and seven of the nine are `policy-*`** ⚠️ **CORRECTED: it is SIX of the nine, not seven** — `policy-decisions`, `policy-drift`, `policy-outcomes`, `policy-projections`, `policy-proposals`, `policy-reviews`; the other three are `audit`, `deployments` and `evaluations`. The nine themselves re-derive exactly.  — the gap is one subject, the policy lifecycle, not fourteen scattered surfaces.
- ⚠️ The binary half is a weak signal and says so: *0 of 10 unmentioned* rests on matching `rb`, two letters. ⛔ No chapter written here; the per-member judgement is `.11.4.6.2`.

## 2026-09-21 — Consumed cleanup is delivered, and three parents close on verified acceptance (`SIGNOFF-REPAIR.11.2.1`)

`REASONBRAID-DOC-0110`. Seven leaves (`REPAIR-0370`–`0376`) gave every accumulating fixture family in tracked Rust a producer-owned cleanup rule.

- ⭐ **Every acceptance clause answered by a COMMAND, not from memory** — both states produced in three suites, the family **2,375 → 2,375** with PEAK 2,376, `git diff --diff-filter=D` over the batch returning **nothing**, `census_retained_fixtures.py` untouched and `browser-lifetime-controls` **21 → 21**.
- 🔴 The batch's largest finding was about its own instrument: the census had been reporting **15.9% of the bytes**, reading only Rust while the largest families are created by Python. The population is **2,264,752 KiB / 4,334 fixtures in 35 of 37 families**. ⚠️ **CORRECTED by `.11.2.1.3.2.1.2`, which the director's *ensure your findings still hold* is what found: that figure is 88.4% of the real one.** Re-derived by a route never tried before — FILESYSTEM → code — 13 directories under `target/` were claimed by no derived family, holding **303,276 KiB across 440 entries**: one LIVE producer in a third corpus (`target/demo`, 106,000 KiB, written by `scripts/demo_two_host.sh`, a SHELL script) and eleven ORPHANS whose producers no longer exist, the largest being `target/claude-stubs` at **270 directories** that `git grep` cannot attribute to anything. At `50a9712` the census reads `*.rs`, `*.py` AND `*.sh` and reconciles against the filesystem: **41 families derived, 37 present, 2,421,904 KiB / 4,450 fixtures, plus 11 orphans at 197,268 KiB / 328 entries = 2,619,172 KiB**, agreed to the KiB by `du -sk` and by an independent walk. The superseded figure stands per `TOOLBOX.md`.
- ⛔ One named exception, owned in a decision record: `conformance-stubs` keeps no cleanup, because its `static OnceLock` closes an `ETXTBSY` race the remote runner already caught and the retrofit would reintroduce it.
- ⏳ Frontier moves to `.11.4.6` — the six MCP tools are a shipped user-visible surface with no book chapter.

## 2026-09-21 — The long tail, and the one producer a `Drop` guard structurally cannot reach (`SIGNOFF-REPAIR.11.2.1.3.2.6`)

`REASONBRAID-REPAIR-0376`. `release-tool-controls` and `r2-join-controls` guarded; `conformance-stubs` declined.

- ⭐ **The release-tool controls had hand-rolled this rule already** — three `remove_dir_all(&dir).ok();` lines, cleanup on success skipped by a panic, which IS the guard's semantics; and `.ok()` swallowed a failed cleanup silently. Deleted; the guard owns it.
- 🔴 **`conformance-stubs` is DECLINED for a STRUCTURAL reason**: its fixture lives in `static STUBS: OnceLock<Stubs>` and **a `static` is never dropped**, so a guard there would be decoration. The stubs are written once per test BINARY, so *this test passed* is not a question about them. Recorded beside the `OnceLock`.
- ⭐ Measured by PEAK, not a flat count: `release-tool-controls` **6 → 6, PEAK 9**; `r2-join-controls` **52 → 52, PEAK 54**. 63 live controls pass with the cluster removed; 55 passed / 0 failed elsewhere.

## 2026-09-21 — The two largest families left, and the live run found two defects review had not (`SIGNOFF-REPAIR.11.2.1.3.2.5`)

`REASONBRAID-REPAIR-0375`. `cached-decision-live` 411 fixtures and `node-replacement` 150 — 67,924 KiB from four call sites.

- 🔴 **Nine of the ten `journal_path` helpers returned the db file and one returned the DIRECTORY**, so the mechanical conversion produced `…/node.db/node.db` and the drill died on `SqliteError code 14`. Nine agreeing is not ten agreeing.
- 🔴 **And that test DELETES ITS OWN FIXTURE** (*the machine burned down*), so the guard found nothing to remove and panicked a passing control. `NotFound` on a guard's own path is now SUCCESS, with a control that produces the case. ⛔ Both were invisible to review and to strict lint; the live cluster run is what caught them.
- ⭐ Measured not by a flat count: PEAK **412** and **151** during the live runs — 151 not 152, because that test deletes the first fixture before creating the second. 2 + 12 + 41 live controls pass, every cluster removed; 167 passed / 0 failed across core and node.

## 2026-09-21 — Every `journal-tests` call site is guarded, and the prediction from two commits ago came true (`SIGNOFF-REPAIR.11.2.1.3.2.3`)

`REASONBRAID-REPAIR-0374`. The server's 12 sites and 1,308 fixtures — 55% of the family.

- ⭐ **All 62 call sites bind the guard; zero `join("target/journal-tests")` literals remain.** 41 passed / 0 failed against a live cluster, removed on each of three runs, `target/pg-tests` 216 before and after.
- ⭐ The count alone could not tell *created and removed* from *never created*, so full-rate sampling was used: **PEAK 2,376** over 4,907 samples — one fixture at a time, because 36 of 41 controls serialize behind one mutex. 🔴 The first pass was a false null at 1 s intervals and the sampler was validated against a cluster-free suite (PEAK = baseline + exactly its 10 tests) rather than explained away.
- ⭐ `journal-tests` is now derivable ONLY through `FAMILY_GUARD` — so this is the commit that would have silently dropped a 2,375-fixture family from the population had the census not learned the guard shape two commits earlier.

## 2026-09-21 — I replaced the hand-written LIST and left the SCOPE typed, and the scope was 84% of the bytes (`SIGNOFF-REPAIR.11.2.1.3.2.1.1`)

`REASONBRAID-REPAIR-0373`. The fixture census was reporting **15.9%** of the bytes.

- 🔴 The real population is **2,264,752 KiB / 4,334 fixtures in 35 of 37 families**, not 359,820 — **6.29x**. `target/pg-tests` ALONE (1,309,464 KiB, 216 entries) is **3.64x** everything the census believed in, because the largest fixture families here are created by PYTHON and the census read only Rust. ⚠️ **CORRECTED by `.11.2.1.3.2.1.2`, which the director's *ensure your findings still hold* is what found: that figure is 88.4% of the real one.** Re-derived by a route never tried before — FILESYSTEM → code — 13 directories under `target/` were claimed by no derived family, holding **303,276 KiB across 440 entries**: one LIVE producer in a third corpus (`target/demo`, 106,000 KiB, written by `scripts/demo_two_host.sh`, a SHELL script) and eleven ORPHANS whose producers no longer exist, the largest being `target/claude-stubs` at **270 directories** that `git grep` cannot attribute to anything. At `50a9712` the census reads `*.rs`, `*.py` AND `*.sh` and reconciles against the filesystem: **41 families derived, 37 present, 2,421,904 KiB / 4,450 fixtures, plus 11 orphans at 197,268 KiB / 328 entries = 2,619,172 KiB**, agreed to the KiB by `du -sk` and by an independent walk. The superseded figure stands per `TOOLBOX.md`.
- ⛔ The same defect as `.11.2.1.3.2.1`, one level up and shipped in the commit that fixed it: replacing a hand-written list with a derivation moves the typed decision to the CORPUS, where re-running the instrument can never find it.
- ✅ Fixed by deriving from Python's one producer idiom, `local_directory(root, "target/<family>")`. 🔴 The self-reference trap then fired mid-repair — 37 became 39 as the census read its own self-test text — and is closed by treating a file that IMPORTS the patterns as an instrument, asserted against the real tree.

## 2026-09-21 — The five families the control run caught growing, and the drop order that decides whether the guard is safe (`SIGNOFF-REPAIR.11.2.1.3.2.4`)

`REASONBRAID-REPAIR-0372`. The five families the previous leaf's own runs caught growing are now guarded.

- ⭐ 72,368 KiB / 966 fixtures from only **6 call sites**, converted at **19** call sites — a different shape, because each helper's product feeds a longer-lived object, so `dummy_node` returns `(Fixture, Node)` and `stub_binary` returns `(Fixture, PathBuf)`.
- 🔴 **The tuple order is load-bearing**: bindings drop in reverse declaration order, so the guard goes first to be dropped last. The reverse would delete the directory under an open SQLite file — and would very likely still have passed on POSIX, so the rule is now an executable control returning `["second", "first"]`.
- ⭐ **Δ0 KiB and Δ0 fixtures on all six families** across two full runs of both crates, where the identical runs added +8,820 KiB / +57 one commit earlier. Retention produced again: `dead-letter-tests` **64 → 65** keeping `node.db`, `-shm` and `-wal`; restored to 64.
- ✅ 166 passed / 0 failed; clippy `-D warnings`, fmt and gate clean. ⏳ `.11.2.1.3.2.3` owns the server's remaining 1,308.

## 2026-09-21 — A passing test now removes its own fixture, and a failing one still keeps it (`SIGNOFF-REPAIR.11.2.1.3.2.2`)

`REASONBRAID-REPAIR-0371`. The cleanup guard ships and 50 of the 62 `journal-tests` call sites bind it.

- ⭐ `reasonbraid_core::fixture::Fixture` behind a `test-support` feature: exclusive 0700 creation, same-device and non-symlink assertions, and a `Drop` that retains on `std::thread::panicking()` and otherwise re-checks `(dev, ino)` before removing.
- ⭐ **Both halves PRODUCED in the real suite**: one assertion in `kp1` neutralized → 1 failed / 9 passed, `kp1` **16 → 17** with its `node.db`, `kp2`–`kp9` and `e2e` each still **16**; restored, re-run 10/0.
- ⭐ **Measured with a control**: six suite runs took `journal-tests` **2,375 → 2,375, +0 KiB** while five unconverted families grew **+8,820 KiB / +57** from the same runs.
- 🔴 The refactor nearly blinded the census — 62 producing sites → 12 in one commit; the shared pattern set learned `Fixture::create` in the same commit and it is 62 again. ⏳ `.11.2.1.3.2.3` owns the server's remaining **1,308**.

## 2026-09-21 — The population was thirteen families and it is twenty-six (`SIGNOFF-REPAIR.11.2.1.3.2.1`)

`REASONBRAID-REPAIR-0370`. Bounding `.11.2.1.3.2`'s scope refuted both of its typed numbers.

- 🔴 **351,000 KiB across 3,979 fixtures in 24 present families of 26 derived** — the published *221,496 KiB across thirteen* is **63.1%** of it, with **129,504 KiB** in six families never counted, every one a TWO-STEP join a hand list cannot see. ⚠️ **CORRECTED by `.11.2.1.3.2.1.1`: that figure is the RUST-derived population only, and the census's scope was typed rather than derived.** The corpus now includes the PYTHON producer (`project_env.local_directory(root, "target/<family>")`), and at `028217c` the population is **2,264,752 KiB across 4,334 fixtures in 35 of 37 derived families** — so 351,000 was **15.9%** of the bytes, and `target/pg-tests` ALONE (1,309,464 KiB) is 3.64x it. The superseded figure stands per `TOOLBOX.md` as a statement about tracked Rust. Re-derive rather than read: `python3 -B scripts/census_fixture_population.py`. ⚠️ **CORRECTED by `.11.2.1.3.2.1.2`, which the director's *ensure your findings still hold* is what found: that figure is 88.4% of the real one.** Re-derived by a route never tried before — FILESYSTEM → code — 13 directories under `target/` were claimed by no derived family, holding **303,276 KiB across 440 entries**: one LIVE producer in a third corpus (`target/demo`, 106,000 KiB, written by `scripts/demo_two_host.sh`, a SHELL script) and eleven ORPHANS whose producers no longer exist, the largest being `target/claude-stubs` at **270 directories** that `git grep` cannot attribute to anything. At `50a9712` the census reads `*.rs`, `*.py` AND `*.sh` and reconciles against the filesystem: **41 families derived, 37 present, 2,421,904 KiB / 4,450 fixtures, plus 11 orphans at 197,268 KiB / 328 entries = 2,619,172 KiB**, agreed to the KiB by `du -sk` and by an independent walk. The superseded figure stands per `TOOLBOX.md`.
- ⛔ **`.11.2.1.3.1.1` re-derived that figure as *unchanged* one commit ago** — true of those thirteen directories, silent about the population. ⚠️ The *98.5%* was a DENOMINATOR error: 98.5% of the short subtotal, **62.2%** of the population.
- 🔴 *Roughly 40 call sites in `journal.rs`* is **17**, and **62** across the family; the whole retrofit horizon is **222** across 26 families.
- ✅ `scripts/census_fixture_population.py` ships tracked and self-tested, importing the family patterns rather than copying them, every figure PINNED to a commit. ⛔ No product code, no fixture removed.

## 2026-09-21 — Eleven findings held, two did not, and both had one root cause (`SIGNOFF-REPAIR.11.2.1.3.1.1`)

`REASONBRAID-DOC-0109`. The director's *do your findings still hold*, each re-derived by a structurally different route.

- ⭐ **Eleven of thirteen hold exactly**: 61 call sites (raw grep gives 65 lines; the gap is four comments explaining the rule), the three breaches verbatim, 14 of 15, 496 packages, 59 expressions, 1/2/1 → 0/0/0 on two binaries still on disk, both clean commits, 221,496 KiB, and `historical-residue.json` absent from every tree of every reachable commit.
- ⭐ **One came back STRONGER**: `CARGO_TARGET_TMPDIR` re-tested with a bare crate in its own workspace, no runner, no project environment — still `None`. It is the toolchain, not this project.
- 🔴 **Two did not hold, one root cause** — a hand-written census over a hardcoded family list, **13 families short of 26**: it is **13 individuals not six**, and its **57 was published unpinned** and is 85 at HEAD. ⭐ The conclusion is unchanged and stronger: **all 13 absent**.
- ✅ Durability paid: `scripts/census_fixture_citations.py` ships tracked and self-tested, families derived from the producer.
- ⛔ No decision changed, no product code, no cap raised.

## 2026-09-21 — Every individually-cited fixture is already gone, and three records say otherwise (`SIGNOFF-REPAIR.11.2.1.3.1`)

`REASONBRAID-DOC-0108`. Measure the citations before removing anything.

- ⭐ **57 tracked references into the thirteen fixture families; six name an INDIVIDUAL entry — and all six are ABSENT**, three of their four families empty. `target/` is gitignored, so a citation into it dangles by construction. ✅ Cleanup is therefore SAFE: nothing live is cited. ⚠️ **CORRECTED by `.11.2.1.3.1.1`'s verification: the population was 13 families short, so it is **13** individuals, not six — and the conclusion is UNCHANGED and stronger, because **all 13 are ABSENT**. The superseded six stand per `TOOLBOX.md`. The *57* was a per-family line sum over that short list and was published UNPINNED; at `69374f6` it is 57, at HEAD 97 lines / 119 occurrences over 26 families. Re-derive rather than read: `python3 -B scripts/census_fixture_citations.py`.**
- 🔴 **Three records assert present-tense retention of bytes that are gone**, plus `.11.2`'s own *raw … remain intact* where only the tracked half does. 🔴 **One cites a durable record that was NEVER COMMITTED** — `historical-residue.json`, *nineteen files with sizes/hashes*, recorded nowhere.
- ⭐ One record got it right, and its form is the rule: name the artifact AND say where durability lives.
- ⛔ No fixture removed, no evidence deleted, no cap raised.

## 2026-09-21 — The rule becomes a gate, and its own calibration found three false negatives in it (`SIGNOFF-REPAIR.11.2.1.2.3`)

`REASONBRAID-REPAIR-0369`. The enforcer now runs **26** checks.

- ✅ **`RUNTIME-ROOT` ships** — registered, mirrored, in the scaffold's NEUTRAL list, 14 self-test classifications, whole-tree scan under a second. Calibrated over 42 commits across all 743: fires on **40**, on 24 rising to 46 sites, and **0** today.
- ⛔ Not a ban on the macro: **13 uses name tracked source** that travels with its crate and are correct; a pattern on the macro would condemn all thirteen and teach bypass.
- 🔴 **The gate's own `--calibrate` falsified the gate three times**, disagreeing with the census that opened the leaf — a pure-traversal join names the root, a destination can be PUSHED from a for-loop array literal, and the root can be RE-BOUND to an alias. The third read as SOURCE: a false negative. After all three the two instruments agree exactly at **46 / 13 / 0**.
- 🔴 **`.11.2.1` does NOT close**: four of five clauses met, but `explicit consumed cleanup` was never touched — **221,496 KiB across thirteen fixture families, 2,375 entries in one**. Owned by `.11.2.1.3`.
- ⛔ No product code; no cap raised.

## 2026-09-21 — Zero scratch bases left, and a clock in a shipped binary (`SIGNOFF-REPAIR.11.2.1.2.2.2`)

`REASONBRAID-REPAIR-0368`. The last seven; `.11.2.1.2.2` closed on its own executed acceptance.

- ✅ **ZERO: 13 storage-base expressions in tracked Rust, every one SOURCE, no scratch bucket, `0 unresolved`**, and no `CARGO_TARGET_TMPDIR` reader anywhere. Every base reaching generated data derives the root at RUNTIME.
- ✅ **`rb-bench`'s clock REPAIRED, not routed** — a second-granularity timestamp was the only uniqueness source for a shipped binary's report directory; it now proves the name by exclusive creation. Dependencies priced at 496 before and after.
- 🔴 **The browser suite failed 4 controls; attribution was TESTED** — the pre-repair source passed 18/18, the repaired source then passed 18/18 four times. Same source, both outcomes, 1 in 5: the change is excluded, and the occurrence is routed to `.11.2` with its condition. 🔴 That routing first named the wrong leaf and reading it refuted the claim.
- ⛔ No cap raised. One production file: a developer benchmark binary, harness green.

## 2026-09-21 — The server's eleven, and a correction to the number two commits ago (`SIGNOFF-REPAIR.11.2.1.2.2.1`)

`REASONBRAID-REPAIR-0367`.

- ✅ **`reasonbraid-server` contributes ZERO storage bases** — 3 ambient readers and 5 compile-time scratch bases anchored on the runtime root; every device, `symlink_metadata` and 0700 assertion unchanged, because only the anchor was wrong. **89 tests, 0 failed.**
- 🔴 **Correction to this batch's own previous commit: the population is 46 / 13, not 47 / 12.** A Rust tail expression has no `;`, so a fixed-window census let `rb-bench`'s tracked-corpus read borrow the `target` literal four lines below it. Re-derived against `21b44c2` in a throwaway worktree.
- 🔴 **A second parser defect is the `SELF-TEST` family**: the census flagged the new helper's own doc comment, which quotes the macro to explain why not to use it.
- ⛔ Both surfaced through the instrument's own `0 unresolved` requirement, not review. No product code; no cap raised.

## 2026-09-21 — The branch every reader took to be the exception was the only one ever taken (`SIGNOFF-REPAIR.11.2.1.2.1`)

`REASONBRAID-REPAIR-0366`. The parent's remaining clause, audited.

- 🔴 **`CARGO_TARGET_TMPDIR` is UNSET in this project's runs** — measured three ways, two runner-free — so the fallback written as the exception at 17 sites was the only path ever taken, and a source comment said the opposite.
- 🔴 **That fallback resolves at COMPILE time**, baking one checkout's absolute path into the artifact; confirmed with `strings` on a binary whose `file!()` is relative, so the absolute strings are storage bases.
- 🔴 **Population corrected from 17 to 47**: **59 expressions, 0 unresolved** — 47 scratch, **12 tracked SOURCE where the compile-time form is CORRECT**, which is what makes a rule statable rather than a ban.
- ✅ Three predicates became one (`reasonbraid_core::repository_root`, server delegating); all 28 node expressions moved; the rebuilt binary measures **0, 0 and 0** where the old one measured 1, 2 and 1. Core 59 passed, node 32 + every integration target, server `--lib` green; strict lint rc=0 across three crates; gate 25 green.
- ⛔ Not claimed: that the variable is unset on every machine. No cap raised.

## 2026-09-21 — A name proposed ownership that only creation can prove (`SIGNOFF-REPAIR.11.2.1.1`)

`REASONBRAID-REPAIR-0365`. The fixture census the parent ordered, and the three sites it found.

- 🔴 **The parent's fifteen-file source list was stale by 14 of 15** — a sibling, `REPAIR-0085`, had already removed the fractional-second naming and installed exclusive creation. Clock-named tokens **0 of 15**; exclusive creation present in **14 of 15**.
- ⭐ **61 `create_dir_all` sites, all classified, 0 unresolved**, breach set **3** — and only one of the three is in the parent's list. 🔴 The census's own first version reported 6; three were its parser's false positives, repaired before publishing.
- ⭐ **A process id separates concurrent binaries and nothing else** — not within a binary (every test shares it), not across runs (it is reused). Adoption reproduced against an occupied path with the earlier run's sentinel intact; refusal and name collision reproduced too.
- ✅ Repaired at all three; `profiles` **63 passed / 0 failed** on a disposable cluster, four other suites green, strict lint rc=0 for both crates, `make gate` 25 checks green. `496` packages before and after.
- ⛔ No collision claimed, no product code, no cap raised.

## 2026-09-21 — The warning about the key collapse named the wrong site (`SIGNOFF-REPAIR.11.31.1.1.1`)

`REASONBRAID-DOC-0107`. Third verification round, asking the remaining leg — durability.

- 🔴 **The finding is inside the previous correction**: the comment warning that two sites share one adjudication key named `snapshots::submit`, which does not collapse (its two sites are in different functions). The real one is `profiles::write_profile_in_tx`/`agent_profiles.updated_at`, the two halves of an upsert.
- ⭐ **Found because a count disagreed with its own enumeration by one** — 5 caller-supplied escapers reported, 4 enumerable by key.
- ✅ **Nothing else moved**: the guard checks sites not keys, so `--check` is rc=0 at 43 sites over 42 keys with every verdict count unchanged, and the escaper narrative is correct.
- ⛔ Third round running, the verdicts and the conclusion are untouched; each error has been in a sentence about the evidence, never in a measurement.

## 2026-09-21 — A sample was published as a traversal (`SIGNOFF-REPAIR.11.31.1.1`)

`REASONBRAID-REPAIR-0364`. The director's *ensure your findings reflect reality*.

- 🔴 **`.11.31.1` claimed all 24 caller-supplied sites were followed by hand; eight were.** The claim is false — `api.rs:7113` passes `Utc::now()` into the budget reservation path — and two of that leaf's own acceptance criteria were unmet when I marked it done.
- ✅ **Now met**: the adjudication is guarded DATA — 43 sites over 42 keys, each with the facts it was judged against — and `--check` refuses unadjudicated, phantom and **moved** sites. Falsified three ways, restored byte-identically.
- ⭐ **The conclusion survives on better ground**: provenance matters only where a value escapes; 19 of 24 are contained. The 5 that escape were each verified — four to the database clock, one a **demonstrated false positive of my own escape triage** (a cross-branch match in `create_reservation_in_tx`).
- ✅ Three claims checked against the running system: the hook is active so every commit was genuinely gated; all three new instruments are discovered by SELF-TEST; `.11.30`'s repair is real at the site.
- ⛔ No verdict about the 43 changed, and no product code.

## 2026-09-21 — Seven findings held, three had drifted, one described its author (`SIGNOFF-REPAIR.11.33.1`)

`REASONBRAID-REPAIR-0363`. The director's *ensure the findings still hold*, re-derived by different routes.

- ✅ **Seven held exactly** — the 432/89 population, the **14 covering terminals** (same paths, independent implementation), **4,092** and **6,129** references by grep, **no ceiling ever raised** by textual diff, and all **7 `rust_clock` sites contained** by reading the return structs. One came back stronger than published.
- 🔴 **One framing defect**: `BACKTICK-SUBSTITUTION` is unconditional, so the founding instance would have made it refuse **626 consecutive commits**, not the 15 published — and 15 was set beside three rejected gates' FALSE-POSITIVE rates, which is not like-for-like. The comparable number is **zero**. Corrected in all five places.
- 🔴 **Three figures had drifted**, two moved by my own commits in the same batch, none carrying a revision. Each now does, and `--calibrate` self-anchors so the class cannot recur silently.
- ⭐ Bare numerals in registry rationale cells: **131 before this batch, 140 after my row, 131 now.**
- ⛔ No verdict, threshold, registry value or gate behaviour changed — the gate was right, its calibration sentence was not.

## 2026-09-21 — None of the forty-three is the defect (`SIGNOFF-REPAIR.11.31.1`)

`REASONBRAID-DOC-0106`. The adjudication, and a correction to the axis the population was counted on.

- 🔴 **A Rust variable is not a second clock.** The hazard needs two; a value read back with `SELECT clock_timestamp()` is already truncated, so PROVENANCE — not the language the variable lives in — is the discriminator.
- ⭐ **The empty cell is the answer**: crossing origin with escape, **`(rust_clock, reaches_return)` is 0**. No site both invents a clock value and hands it out. All 24 caller-supplied sites were followed by hand; every caller passes `database_now_in_tx` or `tx.database_now()`.
- ⛔ **The gate is declined** on the soundness of the escape leg, which is a triage; the narrower rule needing no escape analysis fires on 7 of 43 correct sites. Trigger recorded.
- ⭐ **The convention was the real gap**: 30 of 43 are safe because somebody used `database_now_in_tx` and nothing said why. Now in the book beside the microsecond note.
- ⛔ No product code, registry row, gate or threshold changed; no cap raised.

## 2026-09-21 — The precision hazard has a population (`SIGNOFF-REPAIR.11.31`)

`REASONBRAID-DOC-0105`. The census the leaf ordered before any gate.

- ⭐ **69 timestamp write sites, every one classified, none left over**: 16 database-clock, 3 read back, **43 bound from Rust and never read back**, 7 in the node's SQLite journal, **0 unresolved**.
- ⭐ **Exact, not heuristic**: sqlx binds positionally, so the census resolves column → `$N` → the Nth `.bind(…)`. And the column TYPE comes from the 82 migrations rather than from its name, removing 4 TEXT columns a name pattern had called timestamps.
- 🔴 Three parser defects found by the corpus and repaired in the parser, each of which had been presenting itself as a finding about the code.
- ⛔ **A gate on the candidate class is refuted by the census itself — 43 of 43.** Mechanizability turns on the escape question, owned by `.11.31.1`. ⚠️ No site is claimed defective; none has been adjudicated.
- ⛔ No product code, registry row or gate changed; no cap raised.

The entries before those above were rotated into reachable Git history at the
**second rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 4ab7cf9572d838790353eb79bf5528c28ae8e5f1:LIVE_STATUS.md
```

That snapshot is 52594 bytes and 384 lines, and contains 34 dated
entries; its Git blob is `09f7ca6d5b7023c52e7c40c9694ff54f602d4928` and its SHA-256 is
`a4683d540ddc5615a210a8859866c2aeb56a818e2011ef3f387c3db958fcd323`. It carries the first rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **17 record(s) rotated out, 18 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
