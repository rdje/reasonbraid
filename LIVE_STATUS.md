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

## 2026-09-22 — Tranche 1's seven verdicts, and a spike needed a fourth value the vocabulary did not have (`SIGNOFF-REPAIR.11.4.7.2.1.4.1`)

`REASONBRAID-DOC-0120`. The first of three tranches over the 21 deferrals no record had ever graded.

- ✅ **THREE `discharged`**: the MCP and A2A interop spikes shipped as crates with their own chapters; the second real adapter is `claude.rs` beside `codex.rs`; the directory registry is `agent_roles` + `agent_profiles` behind `/v1/directory/presence` and `/v1/directory/match`, so the dev stand-in *node-id = role-id* is gone.
- ✅ **ONE `not yet triggered`**: the authenticated streaming channel. No gRPC or HTTP/2 dependency exists and reservations carry no signature, but its trigger is non-loopback exposure and `--host` still defaults to `127.0.0.1`.
- ⭐ **TWO `fired and superseded` — A FOURTH VALUE THE INHERITED VOCABULARY DID NOT HAVE.** The WebSocket/SSE/NATS transport spike and the OPA/Cedar policy-engine spike both had their triggers fire, and neither is wanted: multi-host delivery arrived over HTTP/1 polling, and the project answered *which policy engine* by writing one. ⛔ The cause is structural — every row the parent graded was a FEATURE, and a feature is discharged by an implementation while **a SPIKE is discharged by a DECISION**, which can be *we shipped without needing to ask*. ⚠️ The value requires the successor to be pointed at, so it is not a place to put anything awkward.
- 🔴 **ONE `fired and open`, and a census keyed on the obvious word would have missed it.** `git.rs` exists and is used — by `resolvers.rs` and `snapshots.rs`, for evidence acquisition — so *is there git code* returns yes. The deferral is about publication, and `publications.rs` reaches `git::` **zero** times: the manifest digest is computed server-side and stored in PostgreSQL while no immutable content is written anywhere, and no object-store dependency exists at all. ROADMAP §15's *signed manifest plus immutable Git/object content* is half-shipped. Owned by the new `.9.3.5`.
- **14 rows remain**, all in the Phase-1 subtraction record, and tranche 2 uses all four values.

## 2026-09-22 — The stray scan is derived from what initdb writes, and the prefix list it replaces would have missed one of the three (`SIGNOFF-REPAIR.11.4.3.1.9.1`)

`REASONBRAID-REPAIR-0384`. A cluster outside the scanned directory was not kept — it was invisible.

- 🔴 **THE DEFECT.** `census_pg_test_clusters.py` iterated `target/pg-tests` alone, so three retained clusters at the top of `target/` were never judged and never mentioned. The previous leaf printed `194 cluster(s) remain` and was telling the truth about one directory while reading as a statement about `target/`.
- ✅ **THE PRODUCER CENSUS CAME FIRST AND CHANGED THE DESIGN.** `git log -S '<prefix>' -- .` per prefix: `run-` has a live producer, `pg-ephemeral.` had one that was retired when the Python runner replaced the shell one, and 🔴 **`pg-iter.` has never had a tracked producer at all** — every hit is a documentation commit. ⛔ So a prefix list derived from the producers would contain `run-` alone and would still miss one of the three, and a list derived from today's disk would be a second copy of the residue.
- ⭐ **THE POPULATION RULE IS THE ARTEFACT'S OWN: a cluster is a directory holding `data/PG_VERSION`**, which `initdb` writes — an oracle the instrument did not build. Against the real tree it returns the three strays and nothing else, with 206 markers correctly pruned inside the modelled directory.
- ⛔ **DEPTH-BOUNDED ON PURPOSE**, because an unbounded walk would descend `target/debug` — 69 GB — to find nothing; the bound is pinned by two arms in both directions rather than left to be discovered.
- ✅ **SIX NEW SELF-TEST ARMS, TWO OF THEM NEGATIVE** — a `pg-ephemeral.NOTACLUSTER` with no marker is refused, without which the scan degenerates into *any directory*. Falsified with two mutants, each caught by the arm that names it, source restored byte-identical by SHA-256.
- ✅ **THE STRAYS GET THE GUARD'S VERDICT AND IT IS KEEP**: `receipt ABSENT` for all three, so nothing proves the postmaster stopped and the guard will not remove bytes it cannot prove are unowned. No removal, so the residue census is vacuous and is not claimed.
- 🔎 **AND A SECOND REASON APPEARED MID-LEAF, WORTH MORE THAN THE FIRST.** The strays' tracked-citation count went **0 → 2**, and both citations are this lane's own records. **Writing a finding about an artefact makes that artefact cited evidence, and the citation guard then protects it.** ⛔ The precedent — re-word rather than weaken, as `POSITIONAL-REF` and `STORAGE-LOCALITY` both did — does NOT transfer, because here the artefact's name IS the finding's content. Promoted with the limit declared instead: `docs/knowledge/documenting-residue-makes-it-evidence.md`.

## 2026-09-22 — The artifact sweep run, and the class worth sweeping was the one priced by bytes (`SIGNOFF-REPAIR.11.4.3.1.9`)

`REASONBRAID-REPAIR-0383`. Director directive 8, executed — and the re-measurement inverted which class deserved it.

- 🔴 **EVERY FIGURE THE LEAF PUBLISHED HAD MOVED, ONE OF THEM BY SIX TIMES**, which is why its acceptance forbids using its own stale list. `target/debug` 11.9 GB → **69 GB** (74% → 91% of the tree); the whole tree 16 GB → **72 GiB**; top-level logs 257/101/156 → **401 total, 125 cited, 276 uncited**.
- ⛔ **THE LOG SWEEP IS DECLINED AGAIN, ON A SHARE THAT FELL.** The uncited logs grew 2.6 MB → 4.0 MB while the tree grew 4.5×, so their share dropped to **0.0053%**. Deleting them is safe and pointless, and safe was never the question.
- ✅ **THE CLUSTER CLASS IS THE OPPOSITE.** **23 of 217** retained PostgreSQL clusters were judged retirable and held **1,144.9 MiB — 87% of `target/pg-tests`** — because a few `policy`-suite clusters are ~52 MB each against a ~45 KB tail. ⭐ Priced by COUNT, 23-of-217 reads marginal; priced by BYTES it is the only class here worth touching.
- ✅ **RETIRED THROUGH THE PROJECT'S OWN INSTRUMENT, never a hand-written removal**, because its eight refusals ARE the leaf's acceptance mechanised — unreadable receipt, state not `stopped`, live postmaster, symlink, wrong device, under the age floor, cited by a tracked file, already reduced to evidence — and it re-checks each cluster immediately before removing it. `target/pg-tests` **1,360,488 → 186,332 KiB**, **44,744 → 7,114** files, 217 → 194 clusters.
- ✅ **THE GUARD FALSIFIED BY A SECOND ROUTE.** All 23 removed names are absent, and `git grep -l` for each over the whole tracked tree returns **0 of 23** cited. The guard and the deletion are the same program; a grep afterwards is not.
- 🔴 **AND THE THREE STRAYS ARE DELIBERATELY NOT REMOVED.** The acceptance requires them judged by that guard, whose population is `target/pg-tests/run-*` — so `pg-iter.*` and `pg-ephemeral.*` are structurally outside its reach and a hand removal would be the improvisation the leaf was written to avoid. Measured (39 MB each, 0 citations, no postgres alive) and routed to `.11.4.3.1.9.1`, which widens the instrument from its producers rather than from a listing.
- ⚠️ **Surfaced, not acted on:** `target/debug` is **69 GB, 91% of the tree**, and retiring it costs a cold rebuild measured at 68m 16s. That trade is `.11.4.3.1.8`'s and the director's.

## 2026-09-22 — Four deferrals adjudicated, three of six split under measurement, and the trigger check declined at 71% (`SIGNOFF-REPAIR.11.4.7.2.1`)

`REASONBRAID-DOC-0119`. A deferral with a revisit trigger nobody checks is an omission with extra steps — so the six were graded, one command per verdict.

- ✅ **#1 capability advertisement — `discharged`.** The profile advertises `Vec<CapabilityClaim>` behind a `VisibilityClass`, the matcher consumes `Vec<CapabilityRequirement>`, and `/v1/directory/presence` and `/v1/directory/match` serve them. ⭐ The claim even separates `Benchmarked` from `OwnerAttested`, which the deferral never asked for.
- 🔴 **#2 expected-artifact + manual decision-rule create fields — `fired and open`.** Phase 5's workflow-profile registry shipped, the fields did not. `git grep -n "expected_artifact\|decision_rule" -- crates migrations` returns exactly one hit and reading it settles it: an A2A `SemanticLosses` flag recording that a remote decision rule is ALWAYS LOST. ⛔ `CreateBody` is `deny_unknown_fields`, so either field is a typed refusal today, and the registry's `steps` column is a step sequence with no threshold, quorum or veto — the stand-in did not quietly become the thing.
- ⚠️ **#3 LLM synthesis — SPLITS, and only half landed.** The synthesizer arrived with more than was deferred (ADR-030: synthesizer identity, input event range, and a coverage report naming which objection the synthesis excluded and why). The moderator did not: **0** occurrences in server and core, and every one in the workspace is the deliberation BENCHMARK. ⛔ So ROADMAP §13.5's six permitted acts and three prohibitions are **vacuous rather than enforced**.
- ⚠️ **#6 TLS/mTLS + supervision + containers + PG automation + config files — `not yet triggered`, and five items under one row.** `--host` still defaults to `127.0.0.1` and G6/G7 is NOT MET. ⭐ One of the five is built and uncalled — `mtls::` has callers only under `tests/` — which is `SIGNOFF-REPAIR.14`'s subject with the identical measurement, so it is cited rather than re-opened.
- ⛔ **THE MECHANICAL TRIGGER CHECK IS DECLINED, ON THE CENSUS.** 38 deferral rows across 6 records; **27** name a phase; **all 27** name a phase whose tree is already `done`. A check keyed on *the named phase has closed* fires on **27 of 38 — 71%** on registration day, which is `.11.9`'s rejected shape (87%), `.11.15`'s (93%) and `.11.16`'s (71%), against the 9.5% that argued `POSITIONAL-REF` in.
- ⭐ **AND THE CENSUS REDIRECTED THE WORK RATHER THAN ONLY BLOCKING IT.** *27 of 27 fired* is a population whose verdict is unknown, not 27 defects — #4 fired and was discharged. What is owed is a one-time adjudication pass, and **21** phase-triggered rows in the sibling records have never been graded at all. Children `.1`–`.4` own the fuzz baseline, the create fields, §13.5's moderator and the 21.
- 🔴 **A NEAR-MISS KEPT BECAUSE ONLY ONE CHECK WOULD HAVE CAUGHT IT.** The fenced census was published with its `split` escape already consumed, so the block read correctly and was a syntax error. Reading it, re-running the original script and re-deriving by another route all pass while the published text is dead. **Run a command from the record, not from the shell history it came out of.**

## 2026-09-22 — The generator this leaf owned deciding was measured and declined, and the check that replaced it goes red on the exact state that opened it (`SIGNOFF-REPAIR.11.2.9`)

`REASONBRAID-DOC-0118`. A pending leaf re-read against later rulings before being executed.

- ⛔ **THE ACCEPTANCE IS RETIRED RATHER THAN MET.** This leaf owned deciding whether `MEMORY.md`'s mechanical fields become DERIVED. `SIGNOFF-REPAIR.11.4.2.4` (REPAIR-0348) answered that over all **645** commits touching the pointer and **declined the generator** — the field with catastrophic drift had already been repaired by not carrying a value, **482 of 645** `latest_commit` versions were already derive-on-read, the residual risk lands on `next_action` and **222** curated-prose bullets, and in **5 of 10** frontier disagreements the TREE was the stale copy, so a generator sourcing row 1 would have written the wrong value half the time.
- ⭐ **`docs/CLAIM_VERIFICATION.md` leg 2 decides it, and this leaf had already applied that rule to itself once** — it withdrew its own `commit-msg` predicate for the same reason. Building the generator now would be re-proposing a measured, declined option, so no difference being nameable, the ruling wins.
- ✅ **WHAT THE LEAF WANTED IS SHIPPED AS A CHECK, NOT A WRITER.** `POINTER-CURRENCY` writes nothing, so it cannot touch the judgement line or the prose, and it names WHICH copy moved because the census measured the disagreement as symmetric.
- ✅ **FALSIFIED AGAINST THE FOUNDING STATE, over history the gate did not police** (`.11.18.2`'s discriminator). `python3 -B scripts/check_pointer_currency.py --against f41997c` returns **rc=1**, naming `REASONBRAID-REPAIR-0318` and the commit that owns it, against a `HEAD` of `REPAIR-0322`; the same instrument returns rc=0 on the present tree.
- 🔴 **THIS LEAF'S OWN CENSUS EXPIRED THE HEALTHIEST WAY — obsoleted by the work it routed.** `git grep -l 'MEMORY\.md' -- scripts/ .githooks/` was **7** at `aca9e14` and is **12** today, and one of the five arrivals is `check_pointer_currency.py`: the reader the leaf said did not exist.
- ⚠️ **The one residue is findability, and the architecture already answers it.** *Layer A could not say what the last four commits did* sends a reader to the wrong layer — §6 says the pointer answers one question, and `CHANGELOG.md` is layer D's readable face.
- ⛔ **This slice ships NO mechanism**, which is the correct output when the work has been done elsewhere.

## 2026-09-22 — The census gate could not see this project's own census family (`SIGNOFF-REPAIR.11.2.8`)

`REASONBRAID-REPAIR-0382`. A gate that refused a claim its own evidence already answered.

- 🔴 `GAP-CLAIM-CENSUS`'s `CENSUS_RE` listed `scripts/check_` and **not** `scripts/census_`, so every tracked instrument built to enumerate discharged nothing. It refused `REPAIR-0266` for a claim already answered by `python3 -B scripts/census_advertised_policies.py --readers`.
- ⭐ The damage is the **shape**, not the size: the cheapest route to green is a raw `git grep … | wc -l` pasted beside the instrument — a less durable census than the one already cited. The gate degraded a good claim rather than letting a bad one through.
- ⭐ Priced on both legs by a new `bash scripts/check_gap_claims.sh --calibrate [N]`, so the figures are asked and not carried. Corpus `blocked=0 · without=0 · only-instrument=0`; the full history leg finds one blocked row whose verdict the widening does not move.
- ⚠️ The history zero is **deterrence**, not cost (`.11.18.2`), so the founding refusal is reproduced verbatim as a two-sided `--self-test` arm — 30/30 — with its RED kept rather than observed once.
- ⛔ `.11.2.5` is **upheld**, not superseded: it rejected widening as the remedy for a too-loose test, which is the opposite direction.
- 🔎 The leaf's own opening number was stale — *15 tracked instruments* was exact at `a29bfb0` and is 33 today. Anchored, with the producer cited beside it.

## 2026-09-21 — This batch's own findings, graded: eight held exactly, two moved, one is unverifiable (`SIGNOFF-REPAIR.11.26.2`)

`REASONBRAID-REPAIR-0381`. Each figure re-derived by a **different route**, never a second pass.

- ✅ **Eight held exactly**: the population and verdicts (raw shell vs the Python census), the census movement **62/2/40 → 101/1/2** (today's instrument replayed over past trees via `git show` — all four intermediate figures exact), **21 coarse vs 14 per-route**, the seven partly-described families by name, every schema claim, both named instances.
- 🔴 **One wrong**: *24 timing records … nothing stalled*. The log is append-mode — 24 is the file's total across three runs, this run wrote **8**, and eight of the 24 **are** the 2026-09-20 stall. Corrected at four sites.
- 🔴 **One unverifiable**: `.11.26`'s 240-invocation distribution — its artefact is untracked and replaced. The **carried** figure failed again, exactly as the standing note warned. ✅ The capture itself re-derives to the millisecond, so the leaf's conclusion stands.
- 🔴 **One moved**: the gate price 0.16 s → **0.27–0.35 s**; ~1% of the enforcer, decision unchanged.
- ⛔ No book figure moved; the adjudication is unchanged at **40 covered · 3 internal · 0 gap**.

## 2026-09-21 — The stalled child gets sampled while it is stuck, because every lever has failed (`SIGNOFF-REPAIR.11.26.1`)

`REASONBRAID-REPAIR-0380`. The instrument `.11.26`'s own narrowing asks for.

- ⛔ **Three levers were produced deliberately and none reproduced the stall**; two explanations were withdrawn under measurement. A fourth lever would be a fourth guess.
- 🔴 The evidence **cannot be collected after the fact** — `run_command` kills the group in its `finally`, so the child is gone before the timeout surfaces. The snapshot fires from a watchdog at **12 s against the 15 s deadline**.
- ⭐ Each artefact answers something already found useless: the process **tree** (the stalled thing is the grandchild), a paging **rate** (an identical swap total was read during both a failing and a passing run), and a `sample` of the deepest descendant — **with the sampler's own elapsed time**, which turns the instrument's limitation into its discriminator.
- ✅ The firing path is proved **end to end on a real stalled child**, and red under a mutant. Suite: **73 tests, 22.876 s, rc=0**, zero snapshots — correct, nothing stalled.
- ⛔ The defect is neither reproduced nor explained; this is what the next occurrence needs.

## 2026-09-21 — Zero gaps: every shipped surface is documented or deliberately internal (`SIGNOFF-REPAIR.11.4.6.8`, closing `.11.4.6`)

`REASONBRAID-DOC-0117`. The last gap, and the leaf closes on a measured zero.

- ✅ **`rb-release-manifest`** — the one binary with an audience outside the deployment, and its whole book presence was a single table cell. `deployment.md` now carries all its verbs, centred on the **third-party verify** path: before `pubkey`, the only party who could check a release was the party who signed it.
- ⛔ Three claims the code refuses are stated rather than implied: `generate` is not reproducible, `re-sign` is not `generate` under another key, and ADR-027's protected identities and reproducible builders are **deferrals**.
- ⭐ **`.11.4.6` CLOSED: 43 members — 40 covered · 3 internal · 0 gap.** It opened on *fourteen undocumented surfaces* derived by subtraction; the real population was 17 gaps, and the largest cause was **not absence** — fourteen routes were already explained and never addressed.
- ⭐ Census **62 / 2 / 40 → 101 / 1 / 2**: every `/v1/` product route documented.
- ⭐ Its own sentence *nothing gates what it omits* is no longer true — `SURFACE-JUDGEMENT` refuses an unjudged surface. Frontier moves to `.11.2.7`.

## 2026-09-21 — The deliberation verbs, and the note promoted one commit earlier paid for itself (`SIGNOFF-REPAIR.11.4.6.7`)

`REASONBRAID-DOC-0116`. Four routes, two different causes.

- ⭐ **The concept search ran first and changed the work**: `profiles.md` already carries the eight response kinds, so `respond` needed a **contract line, not a section**. ⛔ This leaf's own acceptance clause — *with the eight-response vocabulary* — would have produced a second table of the same eight, only one maintained.
- ✅ The other three were genuine absence: `recruitment.md` gives the initiator's side (the `ThreadInvite` gate, the typed 429 fan-out caps, the panel snapshot at close) and automatic initiation — enrolled **roles** only, the **explicit `thread:create:auto` grant**, and a four-check wake checklist evaluated server-side **before the thread exists**; replies do not inherit it.
- ⭐ **Every `/v1/` product route is now documented**: census **101 / 1 / 2**, the remainder being the console's own page and assets.
- Adjudication **39 covered · 3 internal · 1 gap**.

## 2026-09-21 — Scoped to write sixteen routes up, and fourteen were already explained (`SIGNOFF-REPAIR.11.4.6.6`)

`REASONBRAID-DOC-0115`. An addressing failure, not an explanation one.

- 🔴 **Fourteen of the sixteen already had a section** — revocation, the spend breaker, enrolment tokens, certificate revocation, the inbox verbs, the federation directions — and **none wrote its path beside a method**; one table listed eight admin routes as bare suffixes.
- ⭐ **The repair was seven contract blocks, not sixteen sections**: the census moved **81 described → 95** on notation alone. Only `routing` needed prose, and got it beside the routing journals.
- ⛔ The census was not over-reporting — those routes were genuinely unfindable by reader or tool. The error would have been in its reader, and sixteen fresh sections would have left **two divergent descriptions of each verb**, only one maintained.
- ✅ Promotion taken: `a-coverage-gap-names-what-is-missing-not-what-is-absent`.
- Census **81 / 2 / 21 → 97 / 1 / 6**; adjudication **37 covered · 3 internal · 3 gap**.

## 2026-09-21 — The governance receipts, and the two families went to different chapters (`SIGNOFF-REPAIR.11.4.6.5`)

`REASONBRAID-DOC-0114`. The two surfaces that answer *what actually happened*.

- ✅ Two sections in two **existing** chapters rather than a new one. Census **78 / 2 / 24 → 81 / 2 / 21**.
- 🔴 **The receipt read's admission is not the one a reader would assume**: it is `tenant_admin`-gated and takes `tenant_id` explicitly, where every lifecycle route admits on enrolment. Generalising from the neighbouring chapter would have understated a governance read.
- ⭐ `/v1/deployments` went to `policy-lifecycle.md` because the desired/observed pair it stores is exactly what drift one section below compares; the stage diagram gains the `deployment → receipt` hop.
- ⚠️ A receipt is the target's **claim**, not the deployment's verification — the server re-reads nothing.
- 🔎 Routed to `.11.4.6.6`: `federation-agreements` is already described with **elided paths**, so that gap is notation rather than absence.
- Adjudication **33 covered · 3 internal · 7 gap**.

## 2026-09-21 — The evaluation harness gains its chapter, and the chapter's job was refusing to vouch for it (`SIGNOFF-REPAIR.11.4.6.4`)

`REASONBRAID-DOC-0113`. The only family where no route carried a contract line.

- ✅ **`evaluation-harness.md`** — all seven routes beside their methods, the record shapes with worked examples, and the seeded assignment. Census **71 / 2 / 31 → 78 / 2 / 24**, exactly the seven.
- 🔴 **A gate that measured NOTHING reports `pass`** — `{}` compares no case and returns `passed: true`, while the write side of the same file refuses an empty baseline. The chapter says do not treat a green gate as evidence until it is repaired (`.8.2` clause 3).
- 🔴 **The harness is deployment-wide**: every route admits any enrolled principal and **no `evaluation_*` table carries a `tenant_id`**, so one caller reads and writes every other's records (`.8.2` clause 1).
- ⛔ No new leaf — all four defects are `.8.2` clauses 1–4 verbatim; a fifth would duplicate an owned backlog. They now sit in the surface the director reviews.
- Adjudication **31 covered · 3 internal · 9 gap**.

## 2026-09-21 — The policy lifecycle gains its chapter, and the new gate caught a defect in itself (`SIGNOFF-REPAIR.11.4.6.3`)

`REASONBRAID-DOC-0112`. The first chapter the surface adjudication demanded.

- ✅ **`policy-lifecycle.md`** — the stage ordering, all **nine routes beside their methods**, worked examples, and the three closed vocabularies with their refusals. The route census moves **62 / 2 / 40 → 71 / 2 / 31**, exactly the nine.
- ⭐ **The authorization was derived**: all nine admit on **enrolment alone**, every read filtered to the caller's tenant, and **the grant requirement begins at publication** — so this trail records what a tenant decided, not the authority to enforce it.
- ⭐ An apparent defect was checked and is NOT one: `policy_impact` passes no tenant because `policy_versions` HAS no tenant column — the register is deployment-wide, while `migrations/0073` bound the nine lifecycle tables.
- 🔴 **The gate built one commit earlier caught a defect in itself**: it read the book by filesystem glob while its imported classifier reads `git ls-files`, so seven rows went green against an UNTRACKED chapter. Found only because the two instruments disagreed.
- Adjudication now **30 covered · 3 internal · 10 gap**, held by five leaves.

## 2026-09-21 — Every shipped surface is judged, and the coarse signal was wrong in both directions (`SIGNOFF-REPAIR.11.4.6.2`)

`REASONBRAID-REPAIR-0379`. Every surface the product ships now carries a verdict in a guarded table.

- ⭐ **43 members — 30 route families, 10 binaries, and 3 product routes in no `/v1/` family at all** (`/`, `/app.js`, `/style.css`; the pinned population was short, which is `.11.4.6.1`'s binaries finding one lane over). **23 covered · 3 internal · 17 gap**, every gap naming an owning leaf because a finding nobody owns is a complaint (§15).
- 🔴 **The coarse mention test calls 21 of 30 families covered; per ROUTE only 14 are.** Seven it credits carry undocumented routes — `calls` worst at 3 of 4, where `.11.8.1` had already re-derived by hand that the chapter's line is a contract for a DIFFERENT route. The pattern: the reads are written down and the operator's verbs are not.
- ⭐ **And it under-reports too**: `/` scores a bare mention while `web-ui.md` is a whole chapter about it. A signal wrong in both directions has to be adjudicated, not tightened.
- ⚠️ **The weak binary signal hid a real gap**: `rb-release-manifest`'s entire book presence is one table cell, and ADR-027's `pubkey` verb exists so a third party can verify a release without the signing key.
- ✅ `SURFACE-JUDGEMENT` registered at ~0.3 s / ~1%, falsified four ways; `.11.4.6.3`–`.11.4.6.8` hold the seventeen gaps.

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

The entries before those above were rotated into reachable Git history at the
**third rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 937993fe5e94fd13807477a9c511459e4359c3c6:LIVE_STATUS.md
```

That snapshot is 52104 bytes and 337 lines, and contains 31 dated
entries; its Git blob is `3f53c0646fa5f34a51c86475d91d3a0f92521281` and its SHA-256 is
`00c8ae435b3a7e8df12ddaf5984cecdd5f868b4a5be69caf431d5c453ffd9a3b`. It carries the second rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **13 record(s) rotated out, 19 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
