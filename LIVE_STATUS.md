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
| Corrective review | In Progress | `.1`, `.2.1`, `.2.2`, `.3.1` complete; `.3.2.1` site service passes ten live controls and strict lint. `.3.2.2` operator CLI passes 3 live controls plus adjacent service/ownership checks and strict lint; `.3.2.3` HTTP enforcement passes 8 focused controls; the selected 58-test security run and final 12-test registry run pass. `.3.3.1` core subject serialization passes 49 unit + 3 subject controls and 40 live compatibility tests; `.3.3.2` bound evaluation passes 51 core unit + 3 subject tests, 6 evaluator controls, 32 live authority/command API tests and strict lint; all results consumed and the owned cluster removed. `.3.3.3.1` actual-parent command selection passes 37 live tests, six evaluator controls and strict lint. `.3.3.3.2.1` frozen-read eligibility passes 40 live tests, ten pure evaluator controls and strict lint; all results consumed and the owned cluster removed. `.3.3.3.2.2.1` provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all confirmation/shutdown results consumed, owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint; all results and shutdown consumed. Exact scoped receipt readback passes 18 live authority tests and 30 HTTP tests plus strict lint, including final denied-record readback; all results/shutdown consumed. Actual-parent selection and frozen-read audit/readback `.3.3.3` are complete; The `.3.3.4.1` source census/contract is complete (42 named-call locations plus transitive/mutation coverage). `.3.3.4.2` qualifies the guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass, including cancelled-BEGIN replacement and the formerly stale authority executable. Seven acceptance-checker controls also pass, correcting nested-evidence owner selection while preserving real-tree box refusals. All results/shutdown consumed; owned clusters/probes removed. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. Bounded HTTP waits and restart qualification follow; ordinary no-key behavior remains intentionally distinct. The final administrative effect REPRESENTATION `.3.3.4.7.1` passes 10 representation controls within 68 core tests, strict core lint and the falsified object-only control; its census derives the closed fourteen-operation set from 27 guarded-admission call sites. Its STORAGE `.3.3.4.7.2` then passes 8 live controls (falsified 5/3 against three attributable injections): migration 0058 keyed on the admission with a composite tenant-binding foreign key, a writer on the caller's already-guarded transaction, and a tenant-filtered reader. `.3.3.4.8` is the first producer of an effect record; `.3.3.4.9` is the second, putting spend-breaker arm/reset onto one exclusive-guard transaction — 25 live controls, the affected set 5 suites / 94 tests, falsified 17/8 against the exact pre-`.9` handlers. |

## 2026-09-21 — The containment inventory is exhausted (`SIGNOFF-REPAIR.11.4.2`)

`REASONBRAID-DOC-0104`. All four named strands discharged; every one of its 31 descendants `done`.

- ⭐ **The acceptance was EXECUTED, and its second clause is now a measurement over the whole history**: **no declared ceiling has ever increased** (10 registry versions), and the two README caps only ever went DOWN across 6 guard versions — 300→60 lines, 16,384→2,400 bytes. Zero raises.
- ✅ Retrieval intact (`rotate_changelog.py --check-all` rc=0). 🔴 `.11.4.2.6` had to be closed first: every child `done`, the parent `pending` with no `- Status:` line, three commits after a commit message called it closed.
- ✅ Frontier moves to `.11.33` — install the backtick census as a gate, or assert the property at runtime.
- ⛔ No row in the table above changed; no cap raised.

## 2026-09-21 — A parent whose children all closed had never run its own acceptance (`SIGNOFF-REPAIR.11.4.2.6`)

`REASONBRAID-DOC-0103`.

- 🔴 A commit message three commits ago called this leaf fully closed; the leaf said `pending` with **no `- Status:` line at all**. Every child was `done` — a parent is not closed by that, because its acceptance is a claim about the RESULT.
- ✅ **Acceptance met and executed**: `--probe-bounds` → all five core live documents REFUSED, each restored byte-identically, each refusal naming a numeric cap. At the leaf's opening, `LIVE_STATUS.md` and `DEV_NOTES.md` both returned rc=0.
- ⛔ No code, no registry row, no threshold changed.

## 2026-09-21 — Collection bounds: rotating the largest tree is refuted by its own references (`SIGNOFF-REPAIR.11.4.2.9`)

`REASONBRAID-DOC-0102`. The last unexecuted words of `.11.4.2`'s scope.

- 🔴 **`docs/tasks/` is 5,642,809 bytes over 72 files; `SIGNOFF-REPAIR.md` alone is 3,964,829 — 70.3%**, and 2.4× every other governed collection combined.
- ⛔ **Rotation refused on measurement**: that file carries **4,092 internal leaf references** (67% of 6,129 across eleven trees), so a closed leaf is an address, not a retired record — the exact opposite of `.11.4.2.6.1`'s ledger measurement, *no consumer cites an individual record*.
- ⭐ **The declared control was already right**: *trees close when exhausted* — the bound is the tree's lifecycle, not its bytes. ✅ Cost measured at **2.04 s across ten tree-reading checks, 4.3% of the enforcer**.
- ⛔ No ceiling, no rotation, no split; no registry row, threshold or file changed.

## 2026-09-21 — The lifecycle field is a taxonomy, not a contract (`SIGNOFF-REPAIR.11.4.2.8`)

`REASONBRAID-REPAIR-0361`. The last named strand of `.11.4.2`; the pressure half shipped at `.11.4.2.6.7`.

- 🔴 **34 rows declare a lifecycle class and nothing evaluated the field** — the only code touching it printed it, and the guard checked non-emptiness alone.
- ⭐ **MEASURED: a class does not predict a mechanism.** Only the three classes with two rows or fewer share an assertion kind; `reader_navigation` (15), `author_overflow` (9) and `hot_live` (5) share none. The registry now states that the field is a reader's taxonomy and the FIFTH field is what binds growth.
- 🔴 **One misclassification found, and it was this author's, three commits old**: `docs/evidence/` is a collection bounded by an index, not a rotating ledger. Corrected — `append_only_history` now shares all three of ceiling/doctrine/growth, and consistent classes went 2 → 3.
- ✅ **One rule ships**: a class must be a word the registry's own header defines, the vocabulary DERIVED from that header. Falsified twice by injection — a plausible typo, and removing a word from the header alone — each restored byte-identically. ⚠️ `external_service` has no instance and is named as untested rather than deleted.
- ⛔ No row in the table above changed; no cap raised; no class changed to flatter a measurement.

## 2026-09-21 — A doctrine document is deliberately unbounded (`SIGNOFF-REPAIR.11.4.2.7.3.2`)

`REASONBRAID-DOC-0101`. The class question, answered on measurement rather than with an invented number.

- 🔴 **Four of four spine doctrine documents accept 200,000 bytes with a green enforcer**, all monotone and at their all-time high, while three sibling ledgers each carry a derived `ceiling=`.
- ⭐ **DECIDED: no ceiling — one cannot be DERIVED.** A ledger's threshold comes from a pinned window of retained history; a doctrine document retires nothing, so there is no quantity to derive from and rotation would delete rationale rather than retire records.
- ⭐ **The growth is structural**: 6,700 → 56,424 bytes (×8.4) against 13 → 24 registered doctrines (×1.8); **92.8% of the file is the one-row-per-doctrine registry section**. Producer: `census_mirror_numbers.py --growth`.
- ⛔ A ratchet on the per-doctrine ratio is declined (one trajectory). ✅ The reopening trigger is a second information role, not a byte count.
- ⛔ No row in the table above changed; no cap, threshold or ceiling raised, invented or moved.

## 2026-09-21 — A comment describing the entry format was read as an entry (`SIGNOFF-REPAIR.11.4.2.7.3.2.1`)

`REASONBRAID-REPAIR-0360`. Found while checking a number's producer before publishing it.

- 🔴 **The shared doctrine-registry parser returned 25 against the enforcer's own printed 24**, the extra entry lifted out of the enforcer's format comment. 🔴 **And `ROUTE-CONTROL` believed it**: a row declaring `doctrine=ID` would have been accepted as registered and enforced.
- ✅ Fixed at the enforcer's own semantics — a comment line is not an entry — **25 → 24**, `doctrine=ID` refused by name, every legitimate doctrine unchanged. Falsified in situ, red by name, restored byte-identically.
- ⚠️ No row ever declared it, so this was a latent acceptance rather than a false verdict granted. 31 evaluated assertions across 34 rows, unchanged.
- ⛔ No row in the table above changed; no cap raised.

## 2026-09-21 — The closure gets a second anchor, and thirteen ungoverned destinations (`SIGNOFF-REPAIR.11.4.2.7.3.1`)

`REASONBRAID-REPAIR-0359`. The repair the census measured and deliberately did not ship.

- ✅ **Every tracked Markdown document must now end at a governed row.** Rows the closure can never name **3 → 0**; undeclared documents **90 → 0**; tokens 21 → 437. `README.md` is excluded by derivation — it is the guard's own subject — and is reported as `excluded_subject`, not as a gap.
- ✅ **13 rows; `ROUTE-CONTROL` evaluates 31 declared assertions across 34 rows** (was 23 across 21), all holding, every index claim checked against the real index first. Six rows are adjudicated NARRATIVE, which is a classification rather than an omission.
- 🔴 **Correction to the previous entry**: `DOCTRINE_ENFORCEMENT.md` is unbounded, and so are `TOOLBOX.md`, `COMMIT.md` and `MEMORY_ARCHITECTURE.md` — **four of four spine documents**, two of which have had rows for months. The defect was the missing ROW; the missing BOUND is a class question, opened as `.11.4.2.7.3.2`.
- ✅ **Falsified against the trees that hid the instances**: today's anchor at `386aa64^` refuses 88 destinations including `DEV_NOTES.md`; at `8aadac3`, 89 including `MEMORY_ARCHITECTURE.md`. Guard cost 0.06 s → 0.23 s (+0.36 % of the enforcer).
- ⛔ No row in the table above changed; no cap raised; `README.md` untouched.

## 2026-09-21 — The closure is anchored at one link graph, and the rule it declined costs fourteen rows (`SIGNOFF-REPAIR.11.4.2.7.3`)

`REASONBRAID-DOC-0100`. The census `.11.4.2.7.2` opened on its second measured instance.

- ⭐ **At `d8df245`: 432 tracked Markdown documents, 21 rows, 21 closure tokens; the closure would ever propose 308.** 340 have a row it names, **3 have a row it can never name**, **89 have no row at all** — and those 89 cost **14** registry entries, not 89, because the registry governs by prefix. Re-derive: `python3 -B scripts/census_routing_closure.py`.
- 🔴 **Two new instances of the class**: `knowledge-map/` is a third hand-placed row (the hint names the generated index, never its sources), and **`DOCTRINE_ENFORCEMENT.md` has no row and is bounded by nothing** — +200,000 bytes, enforcer green. Owner `.11.4.2.7.3.1`.
- ⛔ **"No row" is not "ungoverned" and a row is not a bound** — `README.md` and `MEMORY.md` have no row and both refuse; `AGENTS.md` has one and does not. Only a declared `ceiling=` refuses.
- ⛔ **Anchor B refuted by measurement**: `CLAUDE.md` never names `DEV_NOTES.md`, so the bootstrap-list anchor would have missed the first instance. Falsified `--as-of` both instance commits; `--verify-closure` holds both arms with the registry byte-identical.
- ⛔ No row in the table above changed; no cap, threshold or ceiling moved; `README.md` untouched.

## 2026-09-21 — The rule moves to the document that governs it (`SIGNOFF-REPAIR.11.4.2.7.2`)

`REASONBRAID-DOC-0099`. A director correction, and two defects behind it.

- ✅ **One normative home**: `MEMORY_ARCHITECTURE.md` §6 now carries the resume pointer's one-question rule and *it shall not grow*; the decision record is reduced to provenance. 🔴 The previous commit had created a second authority one commit after declining one on that exact ground.
- ✅ **The governing document is now governed** — it had no registry row at all; it declares `doctrine=MEMORY-ARCH` and is evaluated every commit. 23 assertions across 21 rows.
- 🔴 **That row made `ROUTE-CONTROL` crash on its first real use** — a correct refusal followed by a traceback. Repaired and falsified; an unclassified row now refuses cleanly at rc=1.
- ⚠️ **Second instance of a closure blind spot**, opened as `.11.4.2.7.3`: a document the landing page does not link never enters the routing closure.
- ⛔ No row in the table above changed; no cap raised; the landing page untouched.

## 2026-09-21 — An overwrite-only pointer cannot accumulate (`SIGNOFF-REPAIR.11.4.2.7.1`)

`REASONBRAID-REPAIR-0358`. The previous commit's diagnosis was wrong; this corrects it and the repair.

- 🔴 **`MEMORY.md` was 81% standing warnings** (5,184 of 6,412 bytes), with the resume pointer at 19%, sitting on its cap exactly twice. An overwrite-only file has no way to accumulate — so the growing part was never the pointer.
- ⛔ **The remedy is eviction, which the memory architecture already prescribes** — not the health target proposed one commit earlier, which would have institutionalised the bloat.
- ✅ **6,412 → 425 bytes, 26 warnings → 0, headroom 756 → 6,743, zero facts lost** — 17 instrument-anchored, all 9 unresolved hand-classified as durable first. No cap raised.
- ⚠️ **The `README.md` half of the previous finding is withdrawn**: 12 versions in the project's life and never once refused.
- 🔴 **The eviction broke a self-test arm that required the defect to be present**, repaired in the same commit and falsified by breaking the segmenter in situ.
- ⚠️ **`README.md` was never touched** — byte-identical to `HEAD`, absent from all five of this session's commits, last changed 2026-09-15. The standing instruction it prompted is `docs/decisions/2026-09-21_the-landing-page-is-quasi-static-and-the-pointer-answers-one-question.md`.
- ⛔ No row in the table above changed.

## 2026-09-21 — The containment contract is adopted and met; the donor's document is declined (`SIGNOFF-REPAIR.11.4.2.7`)

`REASONBRAID-DOC-0098`. The donor-package review, and the last named strand of `.11.4.2` but the lifecycle controls.

- ✅ **The contract is adopted and met**; the donor's root document is declined on the donor's own stop condition — two sources claiming canonical authority for the same information. `docs/decisions/2026-09-21_the-containment-contract-is-adopted-the-donor-document-is-not.md` names what enforces each clause.
- ✅ **The guide's behavioural completion test was evaluated clause by clause and holds.** Donor revision state measured, unchanged since 2026-09-05.
- 🔴 **One deliverable unmet, with an owner**: a health target as well as an enforcement ceiling. Only the three ledgers have both; `README.md` and `MEMORY.md` refuse at the cap with no earlier signal, and that cap has been crossed seven recorded times. `SIGNOFF-REPAIR.11.4.2.7.1`.
- ⛔ No row in the table above changed; nothing of the donor's was copied and no cap was raised.

## 2026-09-21 — The rotation notice now describes the object it names (`SIGNOFF-REPAIR.11.4.2.6.8`)

`REASONBRAID-DOC-0097`. Found while running the rotation the gate prescribed one commit earlier.

- 🔴 **At least 31 of 48 rotation notices** called their predecessor *every byte this file held immediately before the rotation*, which a rotation on a moved tree makes untrue. Re-derive: `python3 -B scripts/rotate_changelog.py --audit-notices`.
- ⛔ **Nothing was ever at risk of being lost** — the tool refuses to retire a record the predecessor lacks. The defect was a published sentence, and the two are kept apart.
- ✅ The sentence now names the object and says why it is the right one; both live ledger footers corrected alongside the renderer. Falsified in situ, 3 controls red by name.
- ⛔ No row in the table above changed; no threshold, ledger or retired record moved.

## 2026-09-21 — A declared control is now evaluated, and the row that was false for months is refused by name (`SIGNOFF-REPAIR.11.4.2.6.7`)

`REASONBRAID-REPAIR-0357`. `.11.4.2.6` is fully closed — all four strands.

- ✅ **`ROUTE-CONTROL` evaluates every declared pressure control each commit.** An optional fifth registry field of `kind=operand` terms sits beside the human sentence; 14 rows declare 22 assertions across seven kinds, 6 declare nothing. No sentence was deleted to fit a field.
- ✅ **Falsified with no edit at all**: `--check --as-of 9221467` refuses `LIVE_STATUS.md` by name for the exact claim that was false for months, while two sibling ledgers' identical claim holds at that same commit.
- ✅ **All three census findings repaired**: the task-tree index's unregistered doctrine name, the unanchored identity claim, and the ADR index rule that nothing enforced.
- ⛔ The existing arity and closure legs are proved unchanged; no cap, threshold or ceiling moved. Enforcer cost +1.5%.

## 2026-09-21 — The growth measurement is affordable, and its identity check could not see the semantic at risk (`SIGNOFF-REPAIR.11.4.2.6.7.2`)

`REASONBRAID-REPAIR-0356`. A prerequisite for `.11.4.2.6.7`, which must falsify against a growth claim.

- ✅ **23.24 s → 0.11 s** on the three ledgers, against a 28.74 s enforcer: one `git cat-file --batch-check` per path, inside the shared function rather than beside it.
- ✅ **Output-identical over 24 paths and 4,579 versions, every field compared, 0 disagreements** — HEAD's own script run against the SAME tree.
- 🔴 **The real-tree comparison could not reach the semantic being preserved** (a skip for a deleted path; no routed path has ever been deleted), so it was measured on three paths that do exercise it.
- ⛔ No row in the table above changed; no caller, threshold, registry row or gate moved.
- 🔎 **Found while running this commit's prescribed `DEV_NOTES.md` rotation, and owned**: the rotation notice calls the object it names *every byte this file held immediately before the rotation*, and it is 2,948 bytes smaller than that. The losslessness guarantee is intact; the sentence is not. `SIGNOFF-REPAIR.11.4.2.6.8`.

## 2026-09-21 — Fourteen of the twenty declared controls are machine-expressible (`SIGNOFF-REPAIR.11.4.2.6.7.1`)

`REASONBRAID-DOC-0096`. The census that `.11.4.2.6.7` must have before it proposes a scheme.

- ⭐ **20 rows → 14 expressible, 6 narrative; 19 claims across six kinds**, of which **18 were evaluated in the same run — 17 hold, 1 refuted**. Re-derive, never read from here: `python3 -B scripts/census_route_controls.py`.
- 🔴 **`docs/TASK_TREE.md` declares `TABLE-ARITY` and the enforcer carries `TABLE-ARITY-RATCHET`** — a second instance of the declared-control class, and a NAME defect rather than an absent control: the ratchet does run over staged `*.md`.
- ⚠️ **`KICKOFF.md`'s identity claim names no revision**, and the file changed three days after `PHASE-0` closed.
- ⛔ **The naive extraction alternative was measured at 5 wrong operands in 18 (27.8%)**, with 6 claims invisible to it — so the operand must be DECLARED, not parsed.
- ⛔ No row in the table above changed: this commit measures and proposes nothing.

## 2026-09-21 — The status file is split: the snapshot it is named for, and the log sealed into git (`SIGNOFF-REPAIR.11.4.2.6.6`)

`REASONBRAID-REPAIR-0355`. The last ungoverned core live document is bounded.

- 🔎 **The reader question this file exists to answer is answered by its rows.** Before this commit it held 619,304 bytes in one section, with the 14-row table at line 2,667 and **92.6% of the file above it**. Its own header already said historical records live in the task trees and git; they now do.
- ⛔ **The history is SEALED wholesale, not rotated.** `.11.4.2.6.6.1` measured that its 320 emoji-led candidate lines carry no derivable record boundary — preceding context classifies the 55 id-less candidates exactly as it classifies the 265 id-carrying ones — so rotating at boundaries it does not have was the one step where a record could be silently merged or lost. Sealing preserves every byte exactly and retrievably, which is the property rotation provides.
- ✅ **The boundary is created only from here on.** Entries below carry dated `## ` headings, so `scripts/rotate_changelog.py` serves this file as a third ledger with the boundary enforced from its first record rather than inferred from 320 old ones.
- ✅ **Ceiling 55,000 bytes**, derived at `.11.4.2.6.6.2` from the pinned project window (`LIVE_WINDOW 20.279 × p90 2,726 = 55,280`, rounded down), and `enforced=True` now that the file is under it.
- ⛔ **No book change, and that is checked rather than assumed**: both book references call this file *the snapshot* and point at it whole — `roadmap.md` says it "carries the same snapshot at the repository root", `qualification-review.md` says "Current progress is in `LIVE_STATUS.md`". Splitting makes both statements more true, not less.

The entries before those above were rotated into reachable Git history at the
**first rotation** (`SIGNOFF-REPAIR.11.4.2.6.6`, which owns this ledger's rotation).

⛔ **THIS TRANSITION WAS A SEAL, NOT A ROTATION, and the notice says so rather than
reusing sentences that would be false here.** A rotation retires whole records
until a derived runway target is met. This file had **no derivable record
boundary**: `SIGNOFF-REPAIR.11.4.2.6.6.1` measured 320 emoji-led candidate lines,
of which 265 carried a leaf or work-unit id and 55 did not, and showed that
preceding context classifies the 55 exactly as it classifies the 265 — so no
mechanical rule separates a headline from a continuation. Retiring "whole records"
was therefore not available, and the entire correction log was sealed in one
transition instead.

The exact predecessor — every byte this file held at the commit named below — is:

```bash
git show 9221467fa9a1f05a439ec9243f4a41a34d4af5c7:LIVE_STATUS.md
```

That snapshot is **620,448 bytes and 3,184 lines**; its Git blob is
`503127c660d42a0a4ef856156531168ea58ec3d7` and its SHA-256 is
`ce9468cd0e82cf54a90464df89b14a6fe7c1dfe969072657a1d5a7db8e46869d`. It carries no
earlier rotation notice: this is the first transition of this ledger, so the chain
starts here and every later notice will name this one.

⛔ **Lossless by construction, and verified rather than asserted.** Nothing was
rewritten or summarised: the predecessor object holds the complete prior file, and
its three figures above were re-derived from that object with `git cat-file -s`,
`git show | wc -l`, `git rev-parse` and `shasum -a 256` — not copied from the tool
that performed the transition.

⭐ **From here the boundary exists.** Entries above carry dated `## ` headings, so
`scripts/rotate_changelog.py` serves this file as a third ledger with its record
boundary enforced from the first record rather than inferred from 320 old ones.
Its ceiling is **55,000 bytes**, derived at `SIGNOFF-REPAIR.11.4.2.6.6.2` from the
pinned project window. ⛔ Rotate; never raise the threshold.
