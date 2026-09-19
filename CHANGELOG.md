# CHANGELOG.md

## 2026-09-19 — The governance library takes the operator's authority, and a refusal that aborts its own audit is not one (`SIGNOFF-REPAIR.6.1.5.4`)

The last open child of `.6.1.5`, and the third instance of one template: the evidence retention sweep (`0063`), the workflow-profile registry (`0071`), and now the policy library (`0074`). Each is a store with no tenant column whose write was admitted on enrolment alone.

- 🔴 **REPRODUCED AT RUNTIME BEFORE THE REPAIR WAS WRITTEN, in four legs.** Mallory — enrolled, holding no site authority — registered `red-org-baseline 1.0.0` and got **200**. Alice's own registration at that coordinate was refused **400** `already exists`. Alice's `GET /v1/policies` then returned, under that governance id, Mallory's clause *"a publication needs no authority"* with Mallory's grant as the owning authority. And Mallory appended a `2.0.0` that withdrew the authority clause — **200**.
- ⚠️ **Stated at its real width.** Unlike the workflow registry, `policy::resolve` names an EXPLICIT `(policy_id, version)`, so a foreign registration does not silently re-shape another tenant's deliberation. What it does is take a coordinate the rightful author then cannot use, and put text under a governance id every enrolled principal reads through the list, the resolve verb, the impact map and the MCP bundle.
- ✅ **`policy_register` is a site capability**, following `migrations/0063`/`0071` exactly: `0074` widens both CHECK constraints, `Action::PolicyRegister` joins the enum (and the `clap::ValueEnum` CLI with it), `site_authority/policies.rs` wraps the registration in `authorized(...)` so the write, its authorization and its audit commit together, `policy::register` takes a CONNECTION rather than the pool, and `authority::grant_is_live` became executor-generic so the owning-authority check reads inside that same transaction.
- ⛔ **The READS are ASSERTED unchanged, not merely left alone.** `GET /v1/policies`, the resolve verb, the impact map and the MCP bundle all still answer any enrolled principal, and the control has both tenants exercise them after the refusal — so a later repair that bound one fails a control rather than silently reversing DOC-0071.
- 🔎 **THE CONTROL CAUGHT A DEFECT THE REPAIR ITSELF INTRODUCED.** `policy::register` detected a duplicate by letting the INSERT violate the primary key; inside a transaction PostgreSQL ABORTS on a failed statement, so the audit write then failed with *current transaction is aborted* and the caller got **500** instead of an audited refusal. `.7.1.2.1` could not have hit this — `workflows::register` computes `MAX(version)+1` and violates nothing. `ON CONFLICT (policy_id, version) DO NOTHING` plus `rows_affected() == 0` keeps the constraint as the arbiter and the transaction alive.
- 🔎 **AND A THIRD, in the same function, that only mattered once the refusal became an AUDIT RECORD.** The shipped code turned every failed INSERT into `Duplicate` and every failed grant lookup into `GhostAuthority` — harmless as a 400 nobody stored, not harmless when a database outage would have written *"that policy version is already registered"* into an operator's trail. `register` now returns the two-level result `authorized`'s own doc comment prescribes.
- ⚠️ **Two wire changes, named rather than slipped in.** The body gained a required `reason`, like every other site act. And two refusals moved **400 → 403** with an audit id — a ghost owning authority and a taken coordinate — because each is a question about the DATABASE, and answering either before the gate hands a principal with no site authority an existence oracle over the site's grants and over a registry it may not write. The five rules that ask only about the submitted document are unchanged and still answer 400 to anyone.
- ⚠️ **The published write-census figures move `26/17 → 25/16`** — one movement, one cause: `POST /v1/policies` left the precise walk and left `identity only` in the same change, exactly as `POST /v1/workflow-profiles` did. `policy_versions` is still a site-global table and is meant to be; what left the population is the route's unguarded admission.
- 🔎 **FIXTURE-WRITE-REACH caught the consequence nobody would have looked for:** three suites began writing `site_audit` the moment registration became a site act and none purged it. All three plans now do.
- ✅ **VERIFIED:** `policy` 21/0 (19 before this leaf); `migration_upgrade` 5, `site_authority` 11, `site_registry_http` 8, `mcp_write` 5, `site_operator_cli` 3, `profiles` 62, `mcp` 6 — all ok, 0 failed. Falsified in situ with NO test edit: reverting `api.rs::register_policy` alone to the shipped gate returns Mallory **200** with `"version":"2.0.0"` and the withdrawn-authority clause; `cmp -s` byte-identical after restore, 21/21. Clippy, fmt, `make gate` (21/21), `mdbook build` and the link check all rc=0.

## 2026-09-19 — Seven published claims re-derived; six hold, one was wrong and it was carried rather than measured (`SIGNOFF-REPAIR.6.1.5.3.1`)

A director instruction to verify this session's findings against `docs/CLAIM_VERIFICATION.md` — re-derive · falsify · durability, each a different question. Every claim was re-derived by a route that does not share a parent with the one that produced it.

- 🔴 **ONE NUMBER IS WRONG AND IT IS MINE.** `.6.1.5.2` and `.6.1.5.3` published *"reads **40** site-global tables → 29"*. Run in a worktree at `4c5c8e6`, the commit immediately before `migrations/0073`, the census answers **38**. The movement is **38 → 29**.
- ⭐ **The corrected figure is the coherent one, and that is how the error announces itself:** 38 − 29 = **9**, exactly the nine lifecycle tables that gained a `tenant_id`. The published 40 implied eleven had left. The same bullet published *"9 tables left and 0 joined"* — the arithmetic never closed, and nobody subtracted.
- 🔎 **Provenance, traced not guessed:** `40` was `.7.1.2`'s figure and true when written; `migrations/0072` bound both routing journals and took it to 38; no leaf restated it; I read it out of `MEMORY.md` and carried it forward. `CLAIM_VERIFICATION` §3 Leg 1 names this exactly — *a number appearing in N places has N chances to be stale*.
- ⭐ **THE REPAIR IS DURABILITY, NOT ARITHMETIC, and the sibling instrument proves it.** `census_shared_registry_writes.py` has named the five live documents that restate its count since `.7.1.1.1`, and refuses with that list — which is **why the write figure `39/30 → 26/17` was right in the very same commit**. `census_registry_read_reach.py` had no such list. It does now, and its refusal prints its own count, the baseline's, the five documents, and the instruction: *subtract, and if the arithmetic does not close, the starting figure is the stale one.*
- ✅ **CLAIM 1 HOLDS AND WAS CONSERVATIVE.** *"Twenty suites could not start"* rested on ONE measured suite and an inference from the gate's plan list — a per-item claim carried by a container's answer. Measured per item, with the 22 plans' routing lines reverted and each target run alone: **all 20 server suites `0 passed`**, every failure a `MissingDependency`, plus `cli_end_to_end` 0/5 and `reasonbraid-mcp` losing 2 of its 6. **21 targets could not start.**
- ✅ **The other five hold**, each by an independent route: the 15 write sites by an SQL-verb parse rather than the original `grep -c`; the 27 purge plans from the commit's own diff (28 hits, one classified out as `migration_upgrade.rs`'s coverage loop, not a plan); the MCP claim by a wider pattern plus the call graph (`policy_bundle` → `policy::list` → `FROM policy_versions`); the write census in a worktree at `4c5c8e6`; and the eleven gated verbs by joining handler-level tenant binding to the route table, which returned exactly eleven and independently confirmed ten read verbs.
- ⚠️ **One figure could not be re-derived and is NAMED rather than assumed either way.** `.7.1.2.2`'s *"29 tables → 27"* matches no quantity either baseline pins; two hypotheses were tested and refuted. That is not a claim it is wrong — it is Leg 3 answered `no`. Owned by `.7.1.2.2.3`, together with `residue 16 → 14` in the same sentence.
- ✅ **VERIFIED:** `census_registry_read_reach.py --self-test` ok (0 failures), `--check` rc=0; falsified by dropping one baseline row, which prints the counts, all five restating documents and the subtract instruction, restored byte-identical; the corpus re-census returns **0** remaining occurrences of the stale figure; the per-suite measurement restored the tree byte-identical and `quota` and `administrative_effects` return 1/0 and 25/0; `make gate` green.

## 2026-09-19 — The lifecycle reads are bound to their own tenant, and two predicates had no control until falsification said so (`SIGNOFF-REPAIR.6.1.5.3`)

The third half of `.6.1.5`: `.6.1.5.2` gave every lifecycle row an owner, `.6.1.5.2.1` gated every write, and this binds the reads. Until now every tenant's governance trail was listed to any enrolled caller.

- ✅ **Ten list reads bound plus one probe** — proposals, decisions, approvals, projections, publications, deployments, drift, corrections, outcomes, reviews, and `publications::stage`'s projection probe.
- 🔴 **`publications::stage` PROBED A PROJECTION FOR EXISTENCE ONLY**, so a publication could be staged against **another tenant's projection** — its compiled bytes and declared unrepresentables, which are a function of that tenant's own resolution request. ⛔ **The predicate was written and every arm still passed.** Falsification found it: every other arm reads a list, and no control had ever staged across the boundary. The new arm stages Alice's publication against Mallory's projection and requires 400; neutralized, it returns 200 with `"projection_id":"rdb-m-proj"` inside Alice's own publication.
- 🔴 **AN EXISTENCE ORACLE OVER ANOTHER TENANT'S ROLLOUT STATE, found while ordering the predicates.** `record_drift` probed `deployment_assignments` before checking the publication's owner, so a foreign caller got `assignment (…) does not exist` for an undeployed pair and `publication … does not exist` for a deployed one. Two answers that differ enumerate whose publications are deployed where. The ownership check now answers first.
- **Scope extension, deliberate and named: `deployment_assignments`.** Not one of the nine — but DOC-0029 deferred it to `.6.1.5` by name, DOC-0071 ruled it tenant-owned by its PUBLICATION, and no child named it. Leaving a decided table with an unbound read and no owner is how a data model gets decided by accident.
- ✅ **The MCP half is satisfied by MEASUREMENT, not assumption:** `git grep -c` over the nine tables across `crates/reasonbraid-mcp` and `mcp_read.rs` returns **rc=1, no match**. That surface reads exactly one policy table — `policy_versions`, the shared library — so `.6.1.1`'s parity is preserved rather than re-opened.
- ⛔ **The library stays open and the control now ASSERTS it.** `GET /v1/policies` answers both tenants, because a policy only its author can read is not governance. A future repair that bound it fails here instead of silently reversing DOC-0071. `.6.1.5.4` owns its write, the half that is actually wrong.
- **Reads left deliberately unbound, each with its reason:** the library's four; `publications::load` and `projections::load` (every caller passes `owned_by` first); `deployments::load_assignment` (its only caller's join already refused); `record_drift`'s assignment probe (ownership now answers before it); and `stage`'s decision and approval probes (matched to a `proposal_id` already proved the caller's).
- ⭐ **The control runs BOTH WAYS ROUND over ten tables from one chain-building closure**, so a repair that returned nothing to anybody passes the absence assertion and fails the presence one.
- ✅ **VERIFIED:** `policy` **19/0** (was 18); `migration_upgrade` 5, `mcp_write` 5, `mcp` 6, `regions` 3, `allowlist` 2, `routing` 4, `evaluation` 3, `administrative_effects` 25, `command_api` 39, `site_authority` 11, `profiles` 62 — 0 failed. Clippy rc=0; fmt rc=0; gate green; book + links rc=0; the read census re-pinned, two rows moving from unbound to bound with none joined or left.
- **FALSIFIED TWELVE TIMES, ONE PREDICATE PER RUN, RESTORED BYTE-IDENTICAL AFTER EACH.**

## 2026-09-19 — Eleven lifecycle verbs require the caller to own the record, and a refusal after the git write is not one (`SIGNOFF-REPAIR.6.1.5.2.1`)

`.6.1.5.2` gave every lifecycle row an owner and deliberately gated nothing, which made the gap visible: a foreign caller's drift observation appeared in the owner's own trail, labelled correctly and authorized by nothing.

- 🔴 **THE POPULATION IS ELEVEN, NOT THE FIVE THE LEAF WAS OPENED WITH.** `.6.1.5.2` touched five write sites and noticed the gap at those five. The question is which verbs act on a record named off the wire without checking it is the caller's, and the answer is every mutating verb over a publication or a proposal: stage, effective, failed, publish, drift, correction, outcome, schedule, review-done, deploy, receipt.
- ⛔ **The three `held_publication_authority` verbs are the sharpest, because they look guarded and are not.** `.9.2.1.2` made them require a grant the CALLER HOLDS — a real check answering a different question: *may this principal act on publications at all*, never *is this publication theirs*. Mallory holds her own live grant and passed every gate but the one that mattered, which is `.6.1.5.1`'s finding on `record_approval`, a second time.
- ✅ **The anchor is the caller's TENANT at ten of the eleven**, `.7.1.2.1`'s site-capability template being rejected because it would require an operator for a tenant to correct its own publication. A foreign record answers exactly as an absent one, or the refusal enumerates every other tenant's publication ids.
- ✅ **`schedule_reviews` is the eleventh and SCOPES rather than refuses**, because it names no id at all — one POST used to materialise review rows for every tenant's publications. Its three source reads move to this leaf with it: the write gate and the read binding are one change there, so `.6.1.5.3`'s population drops by that verb.
- 🔴 **ARM 4 PASSED FOR AN UNRELATED REASON, AND FALSIFICATION IS THE ONLY THING THAT SAID SO.** With the publish verb's ownership check removed, the arm still went 400 — because the path ends at `mark_effective`, whose own check refused Mallory **after** `publisher::publish` had already written `refs/rb/publications/…` into the git repository. ⛔ A 400 after a completed side effect is not a refusal, and no status-code assertion can tell the two apart. The arm now asserts the ref; re-falsified, it fails with that ref's path.
- 🔎 **The ordering inside the publish verb is load-bearing in both directions.** `.9.2.1.2` requires the authority to answer before a path is resolved; the containment control requires a path escape to be reported as a path escape. Ownership therefore sits after containment and before the publication is read — the first point at which the verb learns anything about the record.
- 🔎 **Three fixtures seeded publications and proposals by raw SQL with no tenant and are repaired rather than worked around.** An ownerless publication is now actionable by nobody, so they refused Alice as loudly as they refused Bob and would have passed for the wrong reason.
- ⚠️ **Operator-visible, and in the book:** an unattributable publication is FROZEN — it can be marked effective, failed, published, deployed or corrected by nobody. Deliberate, and an upgraded deployment should check for ownerless rows before relying on them.
- ⭐ **A consequence worth naming: the parent-vs-caller anchor is no longer black-box observable.** Once every verb requires the caller to own the parent, caller and owner always coincide. `.6.1.5.2`'s control was rewritten accordingly; the lineage is still measured by the backfill coverage in `tests/migration_upgrade.rs`. The anchor stays PARENT anyway — a gate can be wrong, and a row derived from its parent cannot be mislabelled by a caller a faulty gate admitted.
- ✅ **VERIFIED:** `policy` **18/0** (was 17); `migration_upgrade` 5, `mcp_write` 5, `regions` 3, `allowlist` 2, `routing` 4, `evaluation` 3, `classification` 1, `profiles` 62, `quota` 1, `administrative_effects` 25, `command_api` 39, `site_authority` 11, `site_registry_http` 8, `mcp` 6, `cli_end_to_end` 5 — 0 failed. Clippy rc=0; fmt rc=0; gate green; book + links rc=0. The read census follows: `record_receipt` moves from an unbound read to a bound one.
- **FALSIFIED ELEVEN TIMES, ONE GATE SITE PER RUN, RESTORED BYTE-IDENTICAL AFTER EACH** — because the suite stops at the first failing assertion, and a single neutralization of "the fix" would have claimed ten arms on the strength of one.

## 2026-09-19 — A suite purges what it asserts over and what it writes, and only the second is mechanizable (`SIGNOFF-REPAIR.7.1.2.2.2`)

`.6.1.5.2`'s regression run was the first batch in which `evaluation` and `routing` could both start since `migrations/0072`, and it found an order-dependent failure.

- 🔴 **`routing evaluation` failed `evaluation` `2 passed; 1 failed`; `evaluation routing` passed both.** `the_shadow_trials_record_the_seeded_assignment_and_the_cohorts` read five trials where it asserts three, the extra two being `rr-trial` and `rec-trial` — `routing.rs`'s own shadow-recommendation fixtures, which have cited a trial as their `evidence_ref` since REPAIR-0274.
- 🔎 **Both plans were wrong, for two different reasons.** `tests/evaluation.rs` asserts the CONTENT of `evaluation_trials` and never purges it. `tests/routing.rs` writes that table and never purges it either.
- ✅ **The rule adopted is BOTH halves, because they do different jobs:** purge-what-you-assert makes an assertion correct; purge-what-you-write stops a suite polluting anyone else. Only the second is derivable from the code.
- 🔴 **WHICH HALF ACTUALLY FIXED IT WAS MEASURED, AND IT WAS NOT THE ONE THE LEAF WAS OPENED ON.** Removing `evaluation.rs`'s purge brings the `2/1` failure straight back. Removing `routing.rs`'s purge leaves **every suite green**. So the write-side sweep is hygiene, not the repair — and running the falsification both ways round is what produced that finding rather than confirming a guess.
- ✅ **New instrument:** `scripts/census_fixture_write_reach.py`, wired into the doctrine gate. It takes the route-to-table reach from `census_shared_registry_writes.py` rather than re-deriving it, so the two cannot disagree, then matches each suite's route literals against its declared plan. At this commit: **29 plans, 0 refused, 2 exempt**. Five plans swept — `classification`, `evaluation`, `profiles`, `quota`, `routing`.
- ⛔ **Its sharpest bound, stated rather than implied: it catches this defect only because the ASSERTING suite also posts to that route.** A suite asserting over a table it never writes stays invisible. No such case exists in the tree today; when one appears it needs a different instrument, not a wider regex.
- ⚠️ **Two over-approximations, each pinned by a self-test arm** so narrowing either is a visible decision: it matches path literals rather than HTTP verbs, and a path no route serves charges nothing.
- ⛔ **Exemptions carry their reason, and *"a migration INSERTs into it"* was rejected as the rule** — `usage_quotas` is migration-seeded and purged safely by eight plans. The rule is whether the PRODUCT depends on rows no test creates: `workflow_profiles` (the eight built-ins `quick_advice` resolves through) and `resolver_capabilities` (R0/R1/R2) qualify. A self-test arm refuses an exemption without a reason.
- ⭐ **The two fixture gates compose:** adding the missing tables made `.7.1.2.2.1`'s foreign-key gate demand `evidence_snapshots` and `derivations` in two of the plans, each instrument answering its own question.
- ✅ **VERIFIED:** `routing evaluation` now 4/0 then 3/0; the reverse stays green; `classification` 1, `profiles` 62, `quota` 1 — 0 failed. `--self-test` 11 controls; `check_fixture_plan_children` rc=0 and 15/15; fmt rc=0; `make gate` green. Falsified both ways round, restored byte-identical after each.

## 2026-09-19 — The lifecycle stores the tenant it already derived, and five of the nine rows take their parent's (`SIGNOFF-REPAIR.6.1.5.2`)

`.6.1.5` decided the nine policy lifecycle tables are tenant-owned by omission: the write already derives the caller's tenant, checks a thread or a verdict event against it, and then stores nothing. `migrations/0073` stores it.

- 🔴 **THE LEAF'S OWN TWO FRAMINGS WERE MEASURED WRONG BEFORE A LINE WAS WRITTEN, and both are corrected in it.** *"The 15 write sites"* is one `policy_versions` INSERT (the LIBRARY — `.6.1.5.4`'s, and it takes no column at all), **nine** lifecycle INSERTs, and **five** UPDATEs that set a status on a row already carrying its tenant. The sites that fill a tenant are nine.
- ⛔ **And *"derived from the authenticated caller"* is right for four and WRONG for five.** A drift row about Alice's publication carrying Mallory's tenant would be invisible to Alice while she is the only party it concerns — `.6.1.5`'s own trap, re-entered from the write side. `policy_publications` takes its **proposal's** tenant; `policy_drift`, `policy_corrections`, `policy_outcomes` and `policy_reviews` take their **publication's**.
- 🔎 **`policy_projections` is the odd one, and DOC-0071's stated reason does not reach it.** That record justified all nine with *"each starts, directly or transitively, in a TENANT'S THREAD"*. `ProjectionRequest` is `{projection_id, target, resolution, lock}` and `ResolutionRequest` is `{policies, target, exception_grants}` — no thread, no proposal, no tenant anywhere on the path. The verdict stands for a different reason: `bytes` and `unrepresentable` are a function of the caller's own resolution request, so the row discloses which policies its author compiled for which target. Ownership by **authorship** — and, exactly as with `routing_recommendations`, an authorship never recorded cannot be recovered.
- ✅ **THE BACKFILL COVERAGE, MEASURED against a seeded pre-`0073` database rather than asserted:** `policy_proposals` **1/3**, `policy_decisions` 1/2, `policy_approvals` 1/2, **`policy_projections` 0/2**, `policy_publications` 1/2, `policy_drift` 1/2, `policy_corrections` 1/2, `policy_outcomes` 1/2, `policy_reviews` 1/2. The three refusals are deliberate: an orphaned chain whose thread is in no aggregate, a thread id present under **two** tenants, and every projection.
- ⛔ **The ambiguity clause is load-bearing and was proved so.** `aggregate_state` is keyed `(tenant_id, aggregate_id)`, so an aggregate id is unique per tenant and not globally. Removing `HAVING count(*) = 1` takes `policy_proposals` from 1/3 to **2/3** — a coin toss wearing a join's clothes.
- ⚠️ **This LABELS rows; it does not GATE writes, and the control asserts that it does not.** Mallory still stages Alice's approved proposal and still records drift, corrections and outcomes against Alice's publication. The rows now say *Alice* instead of nothing, which is what makes the gap visible. Owned by `.6.1.5.2.1`.
- ⭐ **`SIGNOFF-REPAIR.7.1.2.2.1`'s gate paid for itself immediately:** the nine new foreign keys refused **27** purge plans the moment the migration was staged, by name, instead of surfacing one dead suite at a time twenty commits later.
- ✅ **The censuses follow the repair, each movement with one cause:** writes **39/30 → 26/17** (thirteen routes left in a single step), reads **38 site-global tables → 29**. 13 rows left and 0 joined; 9 tables left and 0 joined.
- 🔎 **That movement is also a COVERAGE LOSS, named rather than discovered later.** The read census enumerates tables *with no tenant dimension*, so storing the tenant removed all nine from the instrument that had been watching their 28 still-unbound readers. The population is now pinned by name in `.6.1.5.3`.
- ✅ **VERIFIED:** `policy` **17/0** (was 16), `migration_upgrade` **5/0** (was 4), `mcp_write` 5/0; twenty-seven further suites whose purge plans this migration touched run with **0 failed**. Clippy rc=0 across three crates; fmt rc=0; `make gate` green; book + links rc=0.
- **FALSIFIED SEVEN TIMES, IN SITU, RESTORED BYTE-IDENTICAL.** The decisive one: `publications::stage` taking the CALLER's tenant instead of its proposal's — the obvious implementation, and the wrong one — fails with `left: Some(mallory), right: Some(alice)`. ⚠️ The scheduled-review arm had to be run ISOLATED to be claimed, since the drift assertion stops the suite first.
- 🔎 **A defect found by the regression run and OWNED:** `routing evaluation` in that order fails `evaluation` 2/1, the reverse passes both — `tests/evaluation.rs` asserts the content of `evaluation_trials` and never purges it. Independent of this leaf; owned by `.7.1.2.2.2`.

## 2026-09-19 — The fixture-plan gate could not see an ALTER-added column key, and twenty suites were dead (`SIGNOFF-REPAIR.7.1.2.2.1`)

Found while writing a migration of the same shape as `0072`'s and asking which purge plans it would break. The answer was: twenty-two were already broken, and the doctrine gate written for exactly this failure was green.

- 🔴 **MEASURED ON A CLEAN TREE AT `8f32631`, BEFORE ANY REPAIR:** `RB_DEMO=0 bash scripts/run_pg_tests.sh administrative_effects` → **`0 passed; 25 failed`**, every one `purge checked fixture plan: MissingDependency { parent: "tenants", child: "public.routing_recommendations", constraint: "routing_recommendations_tenant_id_fkey" }`. The suite could not start.
- ⛔ **And `python3 -B scripts/check_fixture_plan_children.py` returned rc=0**, with `--json` reporting `{"inline_edges": 45, "alter_added": [], "refused": []}` — `inline_edges` unmoved since the gate was written, `alter_added` empty although `migrations/0072` adds two foreign keys.
- 🔎 **ONE SYNTAX IS THE WHOLE ROOT CAUSE.** `fk_graph()` reads `REFERENCES` only inside a `CREATE TABLE` body; `alter_added_keys()` matches only `ALTER TABLE … ADD CONSTRAINT … FOREIGN KEY`, which it refuses rather than models. `0072` writes `ALTER TABLE routing_resolutions ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id)` — a column-level reference on an ALTER, invisible to both. The instrument that exists to refuse an edge it cannot read did not know there was one.
- ⭐ **The gate's own header predicted this failure and its prediction was too narrow:** *"every foreign key this corpus declares is inline in a `CREATE TABLE`"*. True when written, falsified by `0072` — a model calibrated on the corpus that existed, with no arm that fails when the corpus grows a shape.
- ⚠️ **`.7.1.2.2` swept five plans and its record says five.** Those are exactly the five suites that leaf ran. Not a reporting failure: the instrument meant to enumerate the rest was green.
- ✅ **The repair:** `fk_graph()` now walks `ALTER TABLE <child> … ADD COLUMN <col> … REFERENCES <parent>` as well, and publishes `inline_edges` and `added_column_edges` **apart**, so a regression to zero added-column edges is visible rather than silent. `ADD CONSTRAINT … FOREIGN KEY` deliberately stays a refusal: no migration here uses it, and modelling a shape with no instance is a change no observation demands.
- 🔴 **THE POPULATION IS 22, NOT THE 21 THE LEAF WAS OPENED WITH.** The opening figure was a hand census over the server tests and the MCP crate; the repaired gate walks every tracked `.rs` and found `crates/reasonbraid-cli/tests/cli_end_to_end.rs` in a crate that census never looked at. ⭐ A sweep keyed to where its author expects the problem is the exact failure this leaf repairs, and it happened once more inside the leaf.
- ⚠️ **`added_column_edges` is 3, not 2.** The third is `migrations/0060`'s `node_enrollment_tokens.issued_under` → `authorization_records`, unmodelled for twelve migrations and harmless only because every plan reaching that parent already ordered its child ahead of it.
- ✅ **VERIFIED:** the suite measured at `0/25` is **25 passed / 0 failed**. Every other swept suite run: `cards` 6, `classification` 1, `command_api` 39, `command_ordering` 7, `escalation` 4, `evaluation` 3, `federation` 4, `identity_store` 4, `invitations` 6, `mcp_listen` 1, `node_channel` 40, `node_enrollment` 21, `node_inbox` 8, `node_replacement` 2, `node_result_ordering` 6, `node_work` 8, `profiles` 62, `quarantine` 1, `quota` 1, `mcp` 6, `cli_end_to_end` 5 — **0 failed anywhere**. `--self-test` 15 controls (was 8); gate rc=0; fmt rc=0; `make gate` green; 73 script unit tests ok.
- **FALSIFIED IN SITU, RESTORED BYTE-IDENTICAL:** removing only the new ALTER walk — the pre-repair parser exactly — fails the self-test at `added_column_edges == 1` and reports `refused = 0` against the swept tree, reproducing the blindness. ⚠️ The arm carrying the real `0072` shape had to be run **in isolation** to be claimed: the self-test stops at an earlier assertion and that arm never executed under the neutralization.

## 2026-09-19 — The two lifecycle gates nothing could observe are now predicated (`SIGNOFF-REPAIR.6.1.5.1.1`)

`.6.1.5.1` wrote a tenant check the way its siblings write theirs, watched it admit a foreign tenant, and opened this leaf on the two verbs that still do it that way.

- 🔴 **BOTH GATES WERE OPEN, OBSERVED SEPARATELY.** ARM 1: Mallory registered a policy proposal against **Alice's thread** — 200. ARM 2: Mallory recorded a **governance DECISION on Alice's proposal**, citing a verdict event from Alice's thread, with herself as the sole electorate participant — 200, stored.
- ⛔ **And worse than open: UNOBSERVABLE.** `lifecycle::register_proposal` and `record_decision` gated on `rls::with_tenant_claim` alone. `rls.rs`'s own module doc records why that binds nothing here — *"The dev profile's superuser connection bypasses RLS regardless; the claim-setting is harmless there and binds the moment the app role lands"* — and `migrations/0046`'s `FORCE ROW LEVEL SECURITY` does not reach a superuser either. Every control in this repository runs as superuser, so no control could ever have watched these admit or refuse anything, and none did.
- ⭐ **THE CENSUS ANSWERS THE SYSTEMIC QUESTION, which is why it was the acceptance clause.** `git grep -n "with_tenant_claim" -- crates` returns five call sites: `api.rs::thread_inspection`, the thread event replay and the thread listing each pair the claim with an explicit `WHERE tenant_id = $1` and were never at risk; the two lifecycle verbs were claim-only. ⛔ **Two outliers against a convention the rest of the code keeps — not a systemic RLS problem, and it must not be written up as one.**
- ✅ **The repair is that same convention:** `AND tenant_id = $2` on the `aggregate_state` probe, `AND tenant_id = $3` on the `event_log` verdict probe, with the claim kept as the second belt where the app role is in force.
- ⚠️ **ARM 2 had to be ISOLATED to be claimed at all.** The first neutralization replaced the `aggregate_state` predicate text, which is IDENTICAL in `register_proposal` and `record_approval`, so three sites opened at once and the suite failed at ARM 1 before ever reaching ARM 2. A second run neutralized only the `event_log` predicate. ⛔ Without it, ARM 2 would have been reported as observed when it never ran.
- ⚠️ **Bound honestly: this is NOT a claim that RLS is broken.** Under the app role the policies bind exactly as `2026-09-08_rls-tenant-claim.md` describes. What was wrong is relying on them ALONE where every control runs as superuser.
- ✅ **VERIFIED:** `policy` **16 passed / 0 failed**, rc=0; `mcp_write` 5 ok. Clippy rc=0; fmt rc=0; gate green; book + links rc=0. Falsified twice, each neutralization degrading the predicate to `($n IS NOT NULL)` so the claim alone remained — the pre-repair behaviour, not an arbitrary break — and restored byte-identical both times. The positive arms run after each negative one: Alice still proposes on her own thread and still decides on her own verdict.

## 2026-09-19 — An approval is bound to its proposal's tenant, by a predicate the running profile cannot bypass (`SIGNOFF-REPAIR.6.1.5.1`)

`.6.1.5` named `record_approval` the weakest link in the policy lifecycle: the one write verb taking no tenant where both its siblings take one. This binds it, and finding out how cost a full implementation cycle.

- 🔴 **REPRODUCED: Mallory's approval of Alice's proposal returned 200 and was STORED** — `{"approval_id":"apt-app-foreign","approver":"hpr_…","grant_id":"grt_hpr_…","proposal_id":"apt-prop",…}`. ⭐ Mallory is enrolled in her own tenant and holds her own live grant, so every other check on the path passes for her: the grant's liveness, the approver being the authenticated caller (`.9.3.1`), the decision's parentage. The control measures the tenant binding and nothing else.
- **The anchor is the PROPOSAL'S THREAD, decided rather than taken by symmetry.** An approval is an act upon a proposal; the decision is only its evidence, and `record_decision` has already bound that decision to the same thread. Anchoring on the decision would check the weaker of the two links.
- 🔴 **THE OBVIOUS IMPLEMENTATION WAS WRITTEN, RUN, AND OBSERVED TO BE A NO-OP.** The first version used `rls::with_tenant_claim`, exactly as the two siblings do. Mallory still received 200. `rls.rs`'s own module doc says why — *"The dev profile's superuser connection bypasses RLS regardless; the claim-setting is harmless there and binds the moment the app role lands"* — and `migrations/0046`'s `FORCE ROW LEVEL SECURITY` does not reach a superuser either.
- ✅ **So the gate is an explicit predicate** on `aggregate_state`, whose `PRIMARY KEY (tenant_id, aggregate_id)` makes it exact and indexed, and which holds in EVERY profile. The RLS policy stays as a second belt wherever the app role is in force.
- ⭐ **Which makes `register_proposal`'s and `record_decision`'s enforcement a live gap, not a style difference** — both doc comments claim *"checked under the CALLER's tenant claim"*, and no control has ever observed either gate working or failing, because every control runs as superuser. Owned by `.6.1.5.1.1`; `.6.1.5`'s decision record is qualified rather than left to be misread. ⚠️ It does not weaken that decision, which argued from INTENT: an intent is not undone by its mechanism failing to bind.
- 🔎 **A second test's fixture was exposed and repaired rather than worked around.** `citing_an_authority_requires_holding_it` inserts proposals by raw SQL against a fabricated `thread_id` existing in no aggregate, so the new precondition refused Alice's own approval. That is `a-self-test-cannot-be-tidier-than-the-real-input` exactly; it now creates the thread it claims.
- ✅ **VERIFIED:** `policy` **15 passed / 0 failed**, rc=0; clippy rc=0; fmt rc=0; gate green; book + links rc=0. Falsified against the FINAL implementation and restored byte-identical; the control also asserts the STAGE did not advance, and the positive arm proves the owning tenant still approves and still reaches `approved`.

## 2026-09-19 — The policy library is shared by design; the lifecycle is its tenants', by omission (`SIGNOFF-REPAIR.6.1.5`)

`.6.1.5` had asked, since `.6.1.1` opened it, whether the policy registry is site-global by design or by omission. DOC-0066 re-scoped it from one table to ten. The answer is both, for different tables, and the code proves which is which.

- ⭐ **THE QUESTION HAD ONE NAME AND TWO SUBJECTS.** `policy_versions` is a governance **LIBRARY**: keyed `(policy_id, version)`, its ownership model is `owning_authority` — a GRANT, not a tenant — and a policy only one tenant can read is not governance. **Site-wide by design.** The nine **LIFECYCLE** tables start, directly or transitively, in a tenant's THREAD. **Tenant-owned by omission.**
- 🔎 **THE MEASUREMENT THAT SETTLES IT: the lifecycle already KNOWS the tenant and simply does not keep it.** `lifecycle::register_proposal` takes a `tenant_id` and runs its thread check under `crate::rls::with_tenant_claim`, its own doc comment stating *"a proposal may only name a thread of the caller's tenant — the RLS layer enforces the read"* (`.1.3.1`); `record_decision` does the same for its verdict event. ⛔ Then the `INSERT` stores no tenant, so every downstream read is site-wide. **A system that did not intend proposals to be tenant work would not have written an RLS claim to enforce it.**
- ⭐ **AND THAT IS WHY THE TRAP THIS LEAF WAS OPENED AROUND DOES NOT APPLY.** `.6.1.5` forbade a tenant-scoped read over an unscoped write, because it would hide rows from their own author. The lifecycle's write is NOT unscoped: storing the tenant RECORDS a binding the write already performs. ⚠️ The trap does still apply to `policy_versions`, which is why that table is not tenant-scoped.
- 🔴 **THE WEAKEST LINK, FOUND WHILE MEASURING:** `record_approval` takes `&principal` and no tenant, where both its siblings take one. So an approval is the one lifecycle write with no tenant enforcement of any kind — and `publications::stage` reads approvals to decide whether a publication may be staged, which makes a disclosure-shaped defect a control-surface consequence. `.6.1.5.1` owns it, first in the chain.
- **The population is 47 SQL sites — 32 reads, 15 writes — across ten tables.** The six the leaf was opened over were `policy_versions` alone. Both counts are re-derived by command in the record rather than listed by hand.
- ✅ **DOC-0029's deferral is DISCHARGED.** `policy_publications` joins the lifecycle (its tenant is its proposal's). `deployment_assignments` is tenant-owned **by its PUBLICATION, not by its target** — a target is site-wide by design, and the tenant comes from the side that has one.
- ⚠️ **Routed, not pre-empted:** whether `owning_authority` must be a grant the REGISTRAR holds stays `.9.1`'s, exactly as `policy.rs`'s own comment routes it.
- ✅ **Falsifiable, and the refutation is named:** the claim turns on one observable — `git grep -n "with_tenant_claim" -- crates/reasonbraid-server/src/lifecycle.rs`. Remove those two claims and the OPPOSITE decision becomes correct, which is what makes this evidence rather than preference. The same command found the counter-example the decision now owns.
- Split into `.6.1.5.1`–`.4`, each carrying its own acceptance (`.11.13`). No code changed: this leaf decides and splits.

## 2026-09-19 — The routing journal is read by its own tenant (`SIGNOFF-REPAIR.7.1.2.2`)

`.7.1.2` classified both routing journals as shared, disclosure-only rows whose READ must name the tenant — DOC-0029's remedy, fitting exactly. This binds them.

- 🔴 **THE LEAK CARRIED THE OTHER TENANT'S PRINCIPAL, not merely its traffic.** Against the unbound read, Alice's list holds Bob's row in full — `{"arm":"policy_proposal","caller":"hpr_01a0b96c-…","case_class":"governed","rule_id":"rule_governed","surface":"resolve_verb"}` — and the shadow half hands Bob Alice's recommendation with the evaluation trial it rests on.
- ⭐ **The two tenants resolve DIFFERENT classes on purpose.** A control asserting only a row count would pass against a repair that returned the wrong tenant's single row; asserting content makes that impossible.
- ✅ **`migrations/0072`** adds `tenant_id REFERENCES tenants` to both journals, the five write sites derive it from the authenticated caller, and both reads carry `WHERE tenant_id = $1`. ⛔ The thread-create sites deliberately do NOT use `body.tenant_id`, though it is in scope: that write happens BEFORE the command authorization, so a caller-supplied tenant would let anyone inject rows into another tenant's audit trail.
- ⭐ **THE STORED ROWS GET TWO DIFFERENT ANSWERS, because the two tables recorded different things.** `routing_resolutions.caller` holds a principal primary key, and each principal carries exactly one tenant — so its history is DERIVED back by join. `routing_recommendations` never recorded an actor at all, so nothing can attribute its history: those rows stay NULL and are read by NOBODY. ⛔ Inventing an owner for an audit row that asserts who did something would be worse than losing its visibility.
- 🔎 **The migration broke five test purge plans, and that is the harness working.** `routing_recommendations` gained its first FK to `tenants`, so `delete_tables` refused with `MissingDependency { parent: "tenants", child: "public.routing_recommendations" }` across five suites. A plan that did not name a child would leave rows behind between tests.
- ⚠️ **Stated at its real width:** a DISCLOSURE path, not a write path. `routing::resolve` reads neither journal, so no row here ever bound anybody's outcome.
- ✅ **The census follows the repair: 41/32 → 39/30, 29 tables → 27, residue 16 → 14.** Both journals left the site-global population and two routes stopped being site-global writers at all. Each movement is a repair, never a recount, and the arm deriving the figures now records the whole chain.
- ✅ **VERIFIED:** `routing` 4/0; `policy` 14, `regions` 3, `allowlist` 2, `mcp_write` 5, `migration_upgrade` 4 — 0 failed. Clippy rc=0; fmt rc=0; both censuses `--check` rc=0 and self-tests green; gate green; book + links rc=0. **Falsified in situ with only the two read predicates removed** — the writes still recorded the tenant, so the arm measures the disclosure and nothing else — `routing.rs` restored byte-identical, 4/4 after.

## 2026-09-19 — The operator's verb takes the operator's authority (`SIGNOFF-REPAIR.7.1.2.1`)

`.7.1.2` named the workflow registry a shared CONTROL surface from four source sites and owed a runtime reproduction. This is that reproduction, and the repair.

- 🔴 **THE TAKEOVER WAS OBSERVED, AND IT IS WORSE READ THAN WRITTEN: Alice's thread, in Alice's own tenant, executed Mallory's steps.** With the shipped enrolment gate in place, `a_foreign_tenant_cannot_rewrite_the_default_workflow` fails at the RESOLUTION — `left: Array [String("solicit"), String("decide")]` against `right: Array [String("solicit"), String("synthesize"), String("decide")]`, the thread reporting `"workflow_profile":"quick_advice","workflow_steps":["solicit","decide"]`. Mallory holds no authority in Alice's tenant and never touched her thread.
- ⭐ **THE CONTROL ASSERTS THE RESOLUTION, NOT THE INSERT, and that choice is most of its value.** "Tenant B's INSERT succeeded" proves nothing: `workflow_profiles` is keyed `(profile_id, version)` and gaining a version is the registry working as designed. What makes it a defect is that `resolve` takes the HIGHEST version site-wide, so the only honest observation is a thread in ANOTHER tenant running the foreign steps.
- 🔎 **A THIRD TEST FAILED IN THE RED RUN AND IT IS EVIDENCE, NOT NOISE.** `the_synthesis_record_is_derived_content` failed with `the synthesis requires the current step to be 'synthesize' (it is 'decide')` — `profiles.rs`'s `pool()` deliberately does not purge `workflow_profiles`, so the probe's `quick_advice` v2 was still the highest version three tests later. ⛔ **One registration silently changed an unrelated deliberation's behaviour**, which is the site-wide blast radius in terms that cannot be argued with. It is `ok` under the repair, because no row is written.
- ✅ **`workflow_register` is a site capability**, following `migrations/0063`'s precedent exactly: `0071` widens both CHECK constraints, `Action::WorkflowRegister` joins the enum (and the `clap::ValueEnum` CLI with it), `site_authority/workflows.rs` wraps the registration in `authorized(...)` so the write, its authorization and its audit commit together, and `crate::workflows::register` takes a CONNECTION rather than the pool — which is what makes that one transaction possible.
- ⭐ **THE SHAPE WAS DECIDED BY THE CODE'S OWN WORDS.** `workflows::register`'s doc comment already read *"Register a custom profile (the operator's verb)"*. The intent was recorded and never enforced — `a-claim-of-sameness-is-worth-its-call-graph`'s shape applied to AUTHORITY rather than behaviour. ⛔ Built-ins-only immutability was REJECTED as incomplete (a tenant's custom profile was hijackable on the same path); tenant-scoping was REJECTED because it pre-empts `.6.1.5` for a second registry.
- ⚠️ **Two limits, named rather than slipped in.** The body gained a required `reason`, a wire change, because a registration that changes what the whole site deliberates is exactly what an operator must be able to explain afterwards. And appending a version to an existing profile is still possible FOR A CAPABILITY HOLDER: the registry is versioned by design, and forbidding it would remove the feature rather than repair the authority. ⛔ `GET /v1/workflow-profiles` stays on enrolment — a tenant must see the profiles it may name, and the rows carry no tenant data.
- ✅ **VERIFIED:** `run_pg_tests.sh profiles` **62 passed / 0 failed**, rc=0; `migration_upgrade` 4, `site_authority` 11, `site_registry_http` 8, `routing` 2, `site_operator_cli` 3 — all ok. Strict clippy rc=0; fmt rc=0; `make gate` green; book + links rc=0. **Falsified in situ, all three files restored byte-identical**, and the POSITIVE arm has three legs: a holder still registers (200 + audit header), a thread naming the profile still RUNS it, and the same grant is refused at `GET /v1/admin/adapters` — one verb widened, not the boundary.
- 🔎 **AND THE REPAIR MADE AN INSTRUMENT LIE, WHICH IS HOW A SECOND DEFECT SURFACED.** `make gate` refused: `census_shared_registry_writes --check` reported `POST /v1/workflow-profiles` still `identity only` after it had become a site act. The cause is in `census_admission_paths.py`'s `GATES` vocabulary, which has no term for site authority — so **all TEN routes reaching a site-authority response were classified `identity only`**, the weakest label in the table, because each reaches `resolve_principal` and none of the other needles. Eight are mutating: `/v1/admin/adapters` and its revoke, `/v1/admin/regions` and its two pair verbs, `/v1/snapshots/expire-due`, the named tombstone, and this leaf's route.
- ✅ **`site authority` joins the vocabulary, FIRST**, because a site grant is the strongest admission it knows and a route reaching one is not described by any weaker label it also matches. ⛔ Seven of the eight predate this leaf; the eighth is the one it repaired.
- ⚠️ **The published figures move 42 → 41 and 33 → 32, and the movement has ONE cause rather than a recount**: `POST /v1/workflow-profiles` left the precise walk (its handler now delegates across a module boundary) and left `identity only` in the same change. The other seven were already `module-reach`, so they were never in the precise count — only mislabelled in the pinned set. ⭐ **`.7.1.2`'s residue follows for the same reason: 30 tables → 29, 17 → 16**, because `workflow_profiles` is repaired and no longer written on enrolment alone. A defect count going down because the defect was fixed is the baseline working, not drifting.

## 2026-09-19 — Ten of the seventeen are control surfaces, and four were never ownerless (`SIGNOFF-REPAIR.7.1.2`)

`.7.1.1` published thirty site-global tables and judged none. DOC-0029 had already decided thirteen. This is the adjudication of the other seventeen, and it corrects the earlier record's reach rather than its reasoning.

- ⭐ **FOUR WERE NEVER OWNERLESS, and this is the correction DOC-0029 most needed.** That record argues from CONTENT-ADDRESSING — several tenants share one row by construction, so no scalar can hold an owner — and the argument is correct. ⛔ It never reached a table whose key is a FOREIGN KEY into a tenant-dimensioned one, where a single join recovers the tenant: `agent_profiles` and `profile_versions` via `agent_roles`, `quota_events` via `usage_quotas`, `recruitment_responses` via `recruitment_calls`. **9 of the 40 site-global tables carry a derivation**, so *site-global by data model* must never again be read as *ownerless*.
- ⚠️ **`quota_events` is why a predicate scan is evidence and never a verdict.** `quota::check_in_tx` reads `WHERE quota_id = $1` — no tenant — while the statement one line above resolves that id from `usage_quotas WHERE tenant_id = $1`. The binding is upstream, in a different statement.
- ⭐ **A FOURTH SHAPE nobody had written down: a tenant-owned row read site-wide BY DESIGN, bound at FIELD level.** The directory returns every tenant's profile — a directory restricted to one tenant would not answer the question the product exists to answer — and clamps the request's scope against a derived `ReaderClass`, then filters every profile through `profiles::filter_profile`. §16.8 is SATISFIED there, by field-level disclosure rather than row-level scoping. It was true in code and stated nowhere; it is in the book now.
- 🔎 **The evaluation family is SEVEN, not the six DOC-0029 names.** `migrations/0034_evaluation_trials.sql` creates `evaluation_trial_results` as its second `CREATE TABLE`, so a count taken per FILE sees six. Verdict unchanged, and `.8.2`'s attached clause extends to it.
- 🔴 **TEN ARE SHARED CONTROL SURFACES, and DOC-0029's remedy reaches none of them** — a control surface's problem is the WRITE, and binding its read would hide rows from their own author while leaving the defect in place. The nine `policy_*` are ONE chain: `publications::stage` refuses a `ForeignRecord` whose parent is the wrong PROPOSAL and never asks whether the proposal is the caller's, and `reviews::schedule_reviews` reads drift, outcomes and corrections **with no predicate at all** and schedules a review on another tenant's publication.
- 🔴 **`workflow_profiles` is a LIVE cross-tenant control defect.** `workflows::resolve` takes the highest version site-wide with no `built_in` filter; `workflows::register` appends `MAX(version)+1` for ANY id including the eight built-ins; `POST /v1/workflow-profiles` admits on **enrolment alone**; and thread creation resolves `DEFAULT_PROFILE_ID = "quick_advice"` through it. ⛔ **Any enrolled principal can change the steps every other tenant's next bare thread executes.** ⚠️ The step vocabulary is closed, so it is a control-plane override, not a capability escape. ⚠️ SOURCE-measured at four sites; the runtime RED is owed first and is `.7.1.2.1`'s.
- ⭐ **The instance was known by name and its significance was not.** `.7.1`'s clause 2 cited `POST /v1/workflow-profiles` as proof `.3.2`'s closed set was designed without a census, and read it as an admission gap. Nobody had read `workflows::resolve` next to `workflows::register`.
- ✅ **`.6.1.5` answered by RE-SCOPING it** — from `policy_versions` and its six SQL sites to a registry of **ten** tables. DOC-0029 deferred two of them there by name; the deferral was right and too narrow. ⛔ `.3.2`'s six site actions UPHELD as actions, SUPERSEDED as THE set: they cover none of the thirty.
- ✅ **VERIFIED:** `scripts/census_registry_read_reach.py --self-test` ok (0 failures); `--check` rc=0, `40 site-global tables, unchanged`; the write census unchanged at 42/33. ⛔ The self-test caught a real pathspec bug on its first run: `git ls-files 'crates/*/src/**/*.rs'` returns **29 of 120** files, dropping every top-level `src/*.rs`.

## 2026-09-19 — The residue holds under a second key, and the population is pinned as a set (`SIGNOFF-REPAIR.7.1.1.1`)

Asked whether I trust my own findings, two of `.7.1.1`'s numbers failed `docs/CLAIM_VERIFICATION.md` §4.1 — *the answer is yes, immediately, with no keyboard, and a re-audit triggered by the question is itself the evidence the claim was published before it was earned.* This is that cost paid rather than noted.

- ✅ **The residue 17 STANDS, re-derived by a key that could have disagreed.** It had rested on a backtick-exact match, so a table DOC-0029 named unbackticked would have been counted as unnamed — `docs/knowledge/a-census-is-as-wide-as-its-key.md`, applied to three other instruments this session and not to its own. A loose `\b<table>\b` finds the same **13**: **0 tables move**. ⭐ And the record settles it in its own words — *"twelve of twelve policy, evaluation, deployment and evidence tables"* — so DOC-0029 was never wrong about its bound.
- 🔴 **The durability gap was real.** `42` and `33` were carried in five live documents with nothing to refuse a change, while the instrument's core was rewritten three times in one sitting and every intermediate count moved (`unseen` 27→26→0, `undetermined` 4→9→12→7→0, `mixed` 6→11→30→37).
- ✅ **`.doctrine/shared_registry_baseline.tsv` pins the SET** — 68 rows of `route → admission → tables → arm` — and the published pair is **derived** from those rows by the gate's own summary rather than stored beside them.
- ⭐ **Why a set and not a count, demonstrated rather than argued:** degrading the detector to a count comparison leaves **39 of 40** arms passing, and the one that fails is *a CHANGED row is refused, though the count is unmoved*. One route in and one out holds the number while the population moves underneath it.
- ✅ All three drift shapes observed RED in situ and restored byte-identical: a route appears, a route vanishes, a row changes. ⛔ A negative arm the obvious implementation would have failed: an **absent** baseline reports no drift rather than 68 vanished routes.
- ⛔ **The refusal names the five live documents that restate the count**, turning `.13.4`'s standing corpus rule from a habit into a mechanism. `unseen-write` also joined the refused class, which it was defined as and was never checked for; it is 0 today.
- ✅ **VERIFIED:** `--check` rc=0 reporting `42 site-global writers (33 on identity alone), matching the recorded baseline`; `--self-test` **40/40** in 1.82 s (33 arms before, same cost — the whole-tree derivations are memoized); `make gate` green; `scripts/tests` **73 tests, OK**.

## 2026-09-19 — The question was already adjudicated for thirteen of the thirty, and the census did not check (`SIGNOFF-REPAIR.7.1.2`, re-scope)

`SIGNOFF-REPAIR.7.1.1` published 42 site-global writers and routed the 33 admitted by enrolment alone to `.7.1.2` for adjudication. Opening `.7.1.2` found most of that adjudication already done.

- 🔴 **`SIGNOFF-REPAIR.11.14` is `done`** — *the site-global data model is a design position nobody has taken* — and `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md` (DOC-0029) already decides it: the evidence chain is content-addressed **by design**, **no table is decided tenant-owned**, and what §16.8 requires is that the DISCLOSURE names the tenant rather than that every table carry a `tenant_id`. ⛔ `docs/CLAIM_VERIFICATION.md` leg 2 — the cheapest oracle is the project's own history, and an earlier ruling wins unless the difference is named. `.7.1.1` did not check, and the omission is the same shape as `.7.3.6.4` not searching the tree for `egress`.
- ⭐ **The difference is real and it is a number: `.11.14` decided TWELVE tables; a census run from the WRITE side finds THIRTY, and 17 are not named in the decision record.** Nine of the ten `policy_*` (only `policy_publications` is named), both `routing_*`, plus `workflow_profiles`, `agent_profiles`, `profile_versions`, `recruitment_responses`, `quota_events` and `evaluation_trial_results`.
- ⛔ **The cause is the shape this session keeps finding.** `.11.14`'s census was *"of the twelve policy, evaluation, deployment and evidence tables"* — a population scoped by FAMILY NAME and taken as given. Derived from the producers instead — every mutating route, every table it writes, every migration — it is 30. **A family-scoped census answers a question about the family, not about the surface.**
- ⭐ **And the residue is not more of the same class.** DOC-0029's remedy is *the read is tenant-bound*, which fits shared EVIDENCE. It does not fit a shared CONTROL surface — a table another tenant's resolution, routing or policy decision reads — because a control surface's problem is the WRITE. `policy_*` and `routing_*` are the sharp end by that test, and neither is an accident: `.6.1.5` is open on exactly whether the policy registry is site-global by design or by omission, and `.9.1`'s goal line already says *tenant-scope all material records*.
- ✅ `.7.1.2` is re-scoped from 30 tables to 17 before any classification, with the reconciliation question stated per table and `.11.14`'s own unmet acceptance clause — that `.6.1.5` be answered by the same record or deferred to it by name — carried forward. `.7.1.1`'s record, `LIVE_STATUS.md` and `MEMORY.md` carry the correction.

## 2026-09-19 — Forty-two routes write site-global state, thirty-three on enrolment alone (`SIGNOFF-REPAIR.7.1.1`)

`SIGNOFF-REPAIR.7.1`'s attached clause 2 — two reviewers, one finding — names this census as the PREREQUISITE to designing the resolver bind, because `SIGNOFF-REPAIR.3.2` designed one without it and closed `done` while two shared registries sat outside its six site actions.

- 🔴 **The population is 42, not 2.** `python3 -B scripts/census_shared_registry_writes.py`: **72 mutating routes**, of which **42 write a table carrying no tenant dimension**. By admission: **`identity only` 33** · `guarded transaction` 3 · `not-censused` 3 · `pool tenant-admin` 2 · `pool authorize` 1.
- 🔴 **So 33 routes write site-global state on enrolment alone**, across **30 distinct tables** — `policy_*` 10, `evaluation_*` 7, `deployment_*` 2, `routing_*` 2. ⛔ `.3.2`'s closed set of six site actions covers **none** of them: the gap is an order of magnitude wider than the two instances known by name.
- ⚠️ **A population is not a defect count, and this leaf does not call them defects.** The read census's own 24 site-global routes were a false-positive class, and a prior ruling already holds the derivation graph shared on purpose. Adjudicating the 33 is `.7.1.2`; publishing the population is this leaf.
- 🔴 **Three instrument defects found by following numbers that looked odd — two of them in shipped code.** (1) `tenant_dimensioned_tables()` could not see a schema-qualified `CREATE TABLE`, so the entire site-authority family — **4 of 80 tables** — was unknown to it, and the GET-route census has been reporting `site_audit=?` ever since. ⭐ Found only because this leaf **imports** that function rather than copying it. (2) `UPDATE` appears in SQL in three non-write positions, and the scan called each a table: `DO UPDATE SET` → `set`, `FOR UPDATE OF n` → `of`, `FOR UPDATE` at a literal's end → `the`, `insert`, `invite`. (3) A match could span two joined string literals.
- ⭐ **The tell for (2) was that `the` and `invite` are English while `set` and `of` are SQL.** One explanation had to cover both, and only the grammar did — the first two repairs chased prose-in-comments and were wrong.
- ⭐ **Two arms, answering different questions, because one was blind to 26 of 72.** The call walk under-reports (no trait dispatch, no methods, no submodules) and said `POST /v1/admin/regions` writes nothing. The second asks the corpus rather than the call graph and can only be too WIDE. Every row names the arm that answered it; the 42 headline counts only the precise one.
- ⛔ **`not-censused` is not `none`.** Applying `census_admission_paths.py` outside its one-file corpus reported two fencing-token node routes as admitted by nothing. **Applying an instrument outside its own corpus produces a confident wrong answer, not a missing one.**
- ⚠️ **Not registered as a pre-commit gate, priced rather than asserted:** `--check` costs 1.86 s against a 13.9 s enforcer and guards a class that is 0 today. The self-test already runs every commit at 1.86 s, down from 5.70 s once the three whole-tree derivations were memoized.
- ✅ **VERIFIED:** `--check` rc=0, `--self-test` **33/33**; falsified twice in situ and restored byte-identical (reverting the grammar fix → 30/33, reverting the schema-qualifier repair → 32/33); the sibling census's own self-test unchanged; `unittest discover -s scripts/tests` **73 tests, OK**; `make gate` green.

## 2026-09-19 — The terms are defined, and the R3 pack stops claiming a container it does not have (`SIGNOFF-REPAIR.7.3.6.5`)

The last child of `.7.3.6`, and **G4's open strand closes with it**. All 36 advertised policy lines now carry a defined verdict: **`enforced` 6 · `unverified` 9 · `vacuous` 21 — `undefined` 0, `misdescribed` 0.**

- 🔴 **THE R3 PACK ADVERTISED THE TOP OF THE ISOLATION LADDER AND RUNS AN ORDINARY CHILD PROCESS.** `sandbox_level: "vm_container"` against a worker in an owned process group — rung one. Reproduced: a caller requiring `vm_container` was served `{"resolvers":["r3-browser-worker"],"unresolvable_now":false}`. ⛔ That is the silent downgrade ADR-018's exit clause exists to prevent, performed by the one pack that executes untrusted JavaScript. The level now reads `process`.
- ⭐ **Nothing had ever REQUIRED `vm_container`, which is why the claim survived.** `git grep` returns the ladder constant, one SDK vocabulary test and the advertisement — no caller, no control. **A rung nobody stands on holds any weight you like.** Correcting it broke no test.
- ⚠️ **`security_evidence`'s `container_required: true` stays and is now consistent** rather than contradictory: the level says what the CODE provides, that key what the DEPLOYMENT must add. The defect was one field claiming the other's content.
- ⭐ **Thirteen of the fourteen unsettled lines were settled by DEFINING a term, not by correcting anything.** `archive_policy`'s axis is DEPTH — `ROADMAP.md` §16 says *archive-depth* — so `deny` means the pack does not expand an archive nested inside the container it acquired. 🔴 That dissolves what the first census called its *sharpest instance of a contradiction*: R2 expands the zip or tar it was **given** and refuses one **inside** it, by name. **An undefined term and a contradicted one look identical until the term is pinned.**
- ⭐ `egress_class: "listed"` means the §12.4 destination **classes** are the list — migration `0025`'s own comment had said so since the pack shipped, and a definition in a migration comment is not one a caller can read. `javascript_policy: "allow-bounded"` is defined by enumerating its four actual bounds.
- ⚠️ **One of the first census's own verdicts was wrong and is corrected.** RX's `egress_class: "any"` was graded `misdescribed` because the pack performs no egress; that judged the wrong granularity. The class is the maximum of the **acquisition the pack delivers**, and an enrolled agent's reach is unbounded, so `any` is the honest ceiling — re-graded `vacuous`.
- 🔎 **A book drift found and repaired in passing:** `blockers.md` said all five gate records were re-derived and C2 closed, while `qualification-review.md` — two pages of the same book — still said *"The first is done"*. REPAIR-0265 swept one page and not the other.
- ⛔ **NOT claimed: that G4 is now Met.** What closes is the last un-discharged strand of its re-derivation; the record's *G4 must be re-earned* verdict is unchanged.
- ✅ **VERIFIED:** `run_pg_tests.sh profiles` → **60 passed, 0 failed** (RED before: 59/1); census `--check` rc=0 and `--self-test` 26/26; clippy `-D warnings` rc=0; fmt rc=0; book rebuilt and links green.

## 2026-09-19 — The egress claim is a ceiling, the sandbox claim is a floor (`SIGNOFF-REPAIR.7.3.6.4`)

`resolvers::resolve` filtered both ADR-018 isolation classes with the same test, `declared >= required` — one predicate for two ladders that run in opposite safety directions.

- 🔴 **The sandbox ladder goes up towards more ISOLATION, so a floor is right. The egress ladder goes up towards more REACH, so the same test admitted a pack that reaches further than the caller permitted.** ⭐ The comment above it quoted ADR-018's *the claim is the MAXIMUM* and drew the opposite conclusion in the same sentence.
- 🔴 **Measured over the whole 4 × 6 matrix: the egress filter refused exactly 1 combination of 24, and it refused the wrong one.** Asking for `any` — the widest class — was the only way to narrow the field, and it narrowed it to `rx-agent-mediated`, the single pack declaring no bound. **No value of `required_egress` meant *do not give me a pack that can dial anywhere*,** which is the one thing ADR-018 says the class is for.
- ⭐ **Reproduced on two resolvers differing in ONE field** — `rsv-egress-listed` and `rsv-egress-any`, identical but for `egress_class`. Asking for at most `listed` returned both: `left: [rsv-egress-listed, rsv-egress-any]`, `right: [rsv-egress-listed]`.
- ✅ **The CODE moved; ADR-018 stands unchanged.** `egress` is now `declared <= required`, a ceiling; `sandbox` stays a floor. ⚠️ A decision rather than an obvious inversion — the shipped default `loopback` only makes sense under the capability reading, so the code was internally consistent. It moved because a capability floor over a maximum claim is incoherent: it asks a promise-not-to-exceed to behave like a promise-to-reach.
- ✅ **The default becomes `any` and the behaviour is unchanged** — under the old floor test `loopback` admitted every pack, and under the ceiling test `any` does too. The word now says what it does.
- ✅ **An off-ladder required class is refused by name** (`invalid_command`) instead of yielding a silent empty result that a typo and a genuine absence shared. ⛔ Validated AFTER the tenant binding, so a foreign or absent resource id keeps giving one answer.
- ⚠️ **One existing test changed and the reason is written beside the line:** the RX resolution passed `required_egress: "listed"` and expected the pack declaring `any` — the defect seen from the suite's own side. It now passes `"any"`. 🔎 `.7.3.6.5` may find RX's `any` to be a misdescription, in which case it changes again for a different reason.
- ⚠️ **Recorded, not repaired:** `required_sandbox` defaults to `process`, which excludes the four packs declaring `none` including R0, so the documented default resolves nothing for the flagship pack. Fail-closed; a default-policy question, not a comparison one.
- ✅ **VERIFIED:** `run_pg_tests.sh profiles` → **59 passed, 0 failed** (RED before: 58 passed, 1 failed); clippy `-D warnings` rc=0; fmt rc=0; book rebuilt. Decision: `docs/decisions/2026-09-19_the-egress-claim-is-a-ceiling-the-sandbox-claim-is-a-floor.md`.

## 2026-09-19 — The pair was gated at two cadences, and the looser half is now a pure function (`SIGNOFF-REPAIR.7.3.6.3`)

The R3 pack advertises two deny-policies and enforces them. Nothing derives one from the other, so both sides are gated — and measuring first found that the two gates were built by two earlier leaves that did not know about each other, at very different strengths.

- 🔴 **THE ASYMMETRY IS THE FINDING.** A moved ADVERTISEMENT is refused by `scripts/census_advertised_policies.py`, a doctrine gate in the pre-commit hook: **every commit**. A moved BEHAVIOUR was caught only by `browser_roundtrip`, which needs a real Chrome and therefore runs in CI **at push** — and the push cadence is about 300 commits, with the tree **237 ahead** as this closes. One half of one claim was guarded hundreds of commits more loosely than the other, and nothing said so, because both halves were green.
- ✅ **The repair: the decision becomes a pure function.** `refusing_policy(is_document, url, requested)` is the whole of the R3 request policy, lifted out of the async interception closure. It touches no network, no browser and no filesystem, so `cargo test -p reasonbraid-browse --bins` covers it — **8 → 12 tests in 0.08 s**, wherever cargo runs. ⛔ The end-to-end control is not replaced: a pure function cannot show the decision is actually wired to Chrome's `Fetch` domain.
- ⭐ **Both directions observed RED, each from its own side.** Flipping `subresource_policy` `deny` → `allow` in the producer with the worker untouched: `the verdict 'enforced' was earned for 'deny', the producer now advertises 'allow'`, rc=1. Neutralizing the worker two ways: the **pre-`.7.3.5` defect** (refuse nothing) → `9 passed; 3 failed`; the **almost-fix** (refuse everything, including the navigation) → `8 passed; 4 failed`. Both restored byte-identical.
- ⭐ **The suite separates a policy from a blackout, not merely detects change:** `a_policy_that_refused_everything_would_fail_this` fires on the almost-fix and **not** on the original defect.
- ⚠️ **Two alternatives rejected, and the first is the one that looks best on paper.** Plumbing the advertised word into the worker — one source instead of two — would make a value decide whether the pack isolates anything, and `resolver_capabilities` has no tenant column (`.11.9.1.1.1`), so that value is writable by any tenant administrator. ⛔ **A one-source design that puts the source where an attacker can reach it is worse than two sources with a gate between them.** Sharing the label strings via the adapter crate was rejected too: the labels are not the claim.
- ⭐ One arm documents an edge instead of leaving it to be found: the asked-for comparison is exact, so a trailing slash makes a URL a different URL and the policy refuses rather than guesses — the fail-closed direction, written down.
- ✅ **VERIFIED:** `cargo test -p reasonbraid-browse --bins --locked` **12 passed, 0 failed**; `scripts/ci_browser.py -- cargo test -p reasonbraid-browse --test browser_roundtrip --locked` **18 passed, 0 failed** in 31.20 s with `a_subresource_the_pack_advertises_as_denied_is_not_dialed ... ok`; `census_advertised_policies.py --check` rc=0; clippy `-D warnings` rc=0; fmt rc=0.

## 2026-09-19 — A replace is complete, and the HTTP verb does not perform one (`SIGNOFF-REPAIR.7.3.6.2`)

`resolvers::register` INSERTed **18** columns and its `ON CONFLICT DO UPDATE` wrote **6**, silently keeping eleven — `media_types`, all four advertised policies, the abilities, the authentication classes, the locator patterns, both format lists and the latency range. A corrected advertisement never reached an existing row.

- 🔴 **REPRODUCED AT RUNTIME, both halves, each as its own control so neither RED hid behind the other's panic.** The startup sync failed to restore drift written into the R3 row (`left: "allow"`, `right: "deny"`), and the HTTP verb answered a narrowing re-registration with its own body as the evidence: **`{"registered":true,"resolver_id":"rsv-replace-probe"}`** — `left: 200`, `right: 409`. `test result: FAILED. 56 passed; 2 failed`.
- ⚠️ **The re-read the acceptance demanded changed the repair.** `SIGNOFF-REPAIR.11.9.1.1.1` had already measured that `resolver_capabilities` has **no tenant column** and a single-column key, so any tenant administrator can address any row — the built-in `r0-https-fetcher`'s included. That says the upsert's reach is too LARGE, and it is `.7.1`'s to bind.
- ⛔ **So "write all 18 columns" was rejected as the whole answer**: it would have handed an unbound principal eleven more columns on a site-global row. A repair that satisfies a doc comment by enlarging the surface another leaf has to bind is not a repair.
- ✅ **Split by CALLER, because the two callers have two different trusts.** `sync_gated_entries` at boot replaces **completely** — the advertisement in the binary is the truth and a drifted row loses to it, `registered_at` excepted. `POST /v1/resolvers` **refuses** an already-registered id with `invalid_transition` (409), naming the row.
- ⭐ **It NARROWS `.7.1` and does not discharge it.** The replace half of that finding closes; creating a new site-global resolver on a tenant-admin grant is untouched, and that is the larger half. `docs/book/src/qualification-review.md`'s row is narrowed, not removed.
- ⚠️ **What is lost, stated rather than discovered:** an operator can no longer change a registered advertise through the API. That capability was never usable for its documented purpose — it could not narrow `media_types` and could not correct a policy line — while the part that did work was the part that should not have.
- ✅ **VERIFIED:** `bash scripts/run_pg_tests.sh profiles` → **58 passed, 0 failed** in 13.11 s, every one of the 56 pre-existing tests unchanged; `cargo clippy -p reasonbraid-server --all-targets --locked -- -D warnings` rc=0; `cargo fmt --all --check` rc=0.
- 🔎 **An environment fact found on the way.** The first RED run reported a third failure: two R2 controls panicked with *the extraction worker is absent at `target/debug/reasonbraid-extract`*. They are right to refuse rather than pass vacuously, but the debug-tree retirement (DOC-0060) left them unreportable and its record says only that the next build is cold. **Run `cargo build --workspace --bins --locked` before the pg suites after a debug retirement.**
- ⚠️ **A citation correction carried in the same commit.** REPAIR-0266 attributed *a deployment that narrows a pack's advertisement narrows what that pack may acquire, in the same act* to `docs/decisions/2026-09-12_r2-acquisition-accept-set.md`. That record does not contain the sentence; it is `advertised_media_types`' own doc comment at `crates/reasonbraid-server/src/resolvers.rs:211`, which cites the decision. The finding is unchanged; a NAMED INSTANCE is graded exact, so all five sites are corrected. Decision: `docs/decisions/2026-09-19_a-replace-is-complete-and-the-verb-does-not-perform-one.md`.

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated thirty times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

Retrieve the ledger immediately before the THIRTIETH rotation (2026-09-19)
from the repository root:

```bash
git show 9dc85449a94a383294f759dc1feceb6ce6827700:CHANGELOG.md
```

That snapshot is 93,167 bytes and contains 21 dated entries; its Git blob is
`345b63aab956be88916bef49957138ba38ccfa76`, and its SHA-256 is
`e15fd658db1c012a951498104d7a23d3e9efa18c7eab576f30c0680ad3e86bc7`. The newest
entry it holds that this digest no longer carries is
`2026-09-19 — Three of the thirty-six advertised policy lines are enforced (`SIGNOFF-REPAIR.7.3.6.1`)`.
It carries the TWENTY-NINTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTY-NINTH rotation (2026-09-19)
from the repository root:

```bash
git show e51cb0d2a12ef06471610ab903f7742c3b2a2c74:CHANGELOG.md
```

That snapshot is 95,235 bytes and contains 26 dated entries; its Git blob is
`1d079a79b96458435b1e74238dbfd8626e253f8f`, and its SHA-256 is
`5f68e5a0b82d3ee775c3e7363f2bcb7b8a3a6ed4a0f8e88d4f8ca2d8f07917da`. The newest
entry it holds that this digest no longer carries is
`2026-09-19 — The lease was written by one clock and read by another, so the published 60 s TTL was nominal (`SIGNOFF-REPAIR.4.2.3.1`)`.
It carries the TWENTY-EIGHTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTY-EIGHTH rotation (2026-09-19)
from the repository root:

```bash
git show 61a28a943305135275e38e0b0c6c5f5f46e019c2:CHANGELOG.md
```

That snapshot is 92,534 bytes and contains 26 dated entries; its Git blob is
`83cfd5d589acd1a4bcf499fb59d64f5a7e66bddd`, and its SHA-256 is
`3177e40ce2b183814022c285ee2e7e63b82398c7fa4cabe41f6c8317c091b294`. The newest
entry it holds that this digest no longer carries is
`2026-09-18 — A citation is withdrawn by its tenant; a shared row is tombstoned by the site (`SIGNOFF-REPAIR.7.4.4`)`.
It carries the TWENTY-SEVENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTY-SEVENTH rotation (2026-09-19)
from the repository root:

```bash
git show d3fa246d4f52862c86a9658a3b30373f37b76e67:CHANGELOG.md
```

That snapshot is 95,092 bytes and contains 28 dated entries; its Git blob is
`b37902eb4a661e6aa68b0fd2d5982e6947927c40`, and its SHA-256 is
`1ece34015ed066060edf099c7a2d1f22d5550b1e6001e89e8367869bcdd3e8e6`. The newest
entry it holds that this digest no longer carries is
`2026-09-18 — A second refusal vocabulary, documented nowhere (`SIGNOFF-REPAIR.7.2.10`)`.
It carries the TWENTY-SIXTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTY-SIXTH rotation (2026-09-19)
from the repository root:

```bash
git show fa588db2b53ce12b972f019cc5880a57542d33f8:CHANGELOG.md
```

That snapshot is 93,398 bytes and contains 29 dated entries; its Git blob is
`5d8a67dde01d27e5cfe50674eb0cfc13eab16e53`, and its SHA-256 is
`bdfd2d0c1abc80c4a8f6a7aab462efd5ec71e87faa4bb94841766c292aaf7bdf`. The newest
entry it holds that this digest no longer carries is
`2026-09-18 — A replay over policed history measures deterrence, not cost (`SIGNOFF-REPAIR.11.18.2`)`.
It carries the TWENTY-FIFTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTY-FIFTH rotation (2026-09-19)
from the repository root:

```bash
git show ac5d4e114ebce131448ac1acd0daf7881e97ec69:CHANGELOG.md
```

That snapshot is 93,009 bytes and contains 29 dated entries; its Git blob is
`80211dce88fb82210bfb5ae45203fcc00a4c81d5`, and its SHA-256 is
`0d94554ed5e4e1709457a15efc22dfd9c544f4b2070e46e0afc69d5642d870ec`. The newest
entry it holds that this digest no longer carries is
`2026-09-18 — A slash is not a resolution (`SIGNOFF-REPAIR.11.17.2`)`.
It carries the TWENTY-FOURTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTY-FOURTH rotation (2026-09-18)
from the repository root:

```bash
git show 2c1c5a301b3780717d5853de5bbe5a5b00a8f581:CHANGELOG.md
```

That snapshot is 93,923 bytes and contains 29 dated entries; its Git blob is
`6552b9c0d6c5d04621f404a81c0831904e28d639`, and its SHA-256 is
`7b1ee96ee7af5ba24bd48b71554729662049af12cf1623a27af74dba60bcf7d7`. The newest
entry it holds that this digest no longer carries is
`2026-09-18 — Twelve verification-log rows had lost their first two cells (`SIGNOFF-REPAIR.11.19.1`)`.
It carries the TWENTY-THIRD rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTY-THIRD rotation (2026-09-18)
from the repository root:

```bash
git show 8f67cbcd37fa8ea598600a0b45e4400d71e7afb7:CHANGELOG.md
```

That snapshot is 95,703 bytes and contains 29 dated entries; its Git blob is
`656f77ac7fc771d8210ae4ede3c54c9aec283027`, and its SHA-256 is
`61f068622b606d4a1eb26fb1c9c2787c72f1e8410eddb302da43589fc1405461`. The newest
entry it holds that this digest no longer carries is
`2026-09-17 — The section scanner did not know what Markdown is (`SIGNOFF-REPAIR.11.18`)`.
It carries the TWENTY-SECOND rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTY-SECOND rotation (2026-09-18)
from the repository root:

```bash
git show 78b52e798cff585a6510922ae5cbecce82e0f3c8:CHANGELOG.md
```

That snapshot is 94,680 bytes and contains 25 dated entries; its Git blob is
`7196f600aafecd635b41a80989085eea84dd5eee`, and its SHA-256 is
`5f55f5b50efd364b76b2a8266b15eb7c27e1e47f7646257bff34dfb4b7d08527`. The newest
entry it holds that this digest no longer carries is
`2026-09-17 — The plan checker had no commit-time trigger (`SIGNOFF-REPAIR.11.14.1.2`)`.
It carries the TWENTY-FIRST rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTY-FIRST rotation (2026-09-17)
from the repository root:

```bash
git show f3022429193e25c47fb256b33d650ecefc66dd1b:CHANGELOG.md
```

That snapshot is 93,514 bytes and contains 31 dated entries; its Git blob is
`b765b420c475f55267e697172c6047556a7cbb93`, and its SHA-256 is
`c71a603db91de4957ac57286676115de73f464293438db8166e58ed39b693e66`. The newest
entry it holds that this digest no longer carries is
`2026-09-17 — The standalone assessment route is bound to the citing tenant (`SIGNOFF-REPAIR.11.14.3.8`)`.
It carries the TWENTIETH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTIETH rotation (2026-09-17)
from the repository root:

```bash
git show 1102b602272e10ed92cc2fb3da9b8d2730143c20:CHANGELOG.md
```

That snapshot is 91,931 bytes and contains 37 dated entries; its Git blob is
`2e598750319b67e22da91be9172e37fbdd1d29e0`, and its SHA-256 is
`b28dc01fe5f4cbaf67bc766f7ff9696e630025eb6c75980b52d590b4c8eff7c8`. The newest
entry it holds that this digest no longer carries is
`2026-09-16 — A credential stops at the origin the caller named (`SIGNOFF-REPAIR.7.2.6`)`.
It carries the NINETEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the NINETEENTH rotation (2026-09-16)
from the repository root:

```bash
git show a581ab5eac5f5f4ad691aee8d7c2ddc6563ce136:CHANGELOG.md
```

That snapshot is 95,031 bytes and contains 27 dated entries; its Git blob is
`ddf72154df4cd968ab799e48355f69fc43c331e4`, and its SHA-256 is
`5b9fe69ecaa8b29f8896f92725039d83f1c6f339f79d02dbaae12dca22fab128`. The newest
entry it holds that this digest no longer carries is
`2026-09-15 — Census the five gate records, and find a wrong number inside a release gate (`SIGNOFF-REPAIR.11.4.7`)`.
It carries the EIGHTEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the EIGHTEENTH rotation (2026-09-15)
from the repository root:

```bash
git show b6445d1818e08f7fbcd7a4ca05525a3add1dad80:CHANGELOG.md
```

That snapshot is 94,399 bytes and contains 28 dated entries; its Git blob is
`a17817dcd2e6f79efe176b4593eae45195cda305`, and its SHA-256 is
`fc0818d18865d8e5fb7174b541d50578f3cb5e96594581a108d66433cf6a25ca`. The newest
entry it holds that this digest no longer carries is
`2026-09-14 — Correcting a number this activity published (`SIGNOFF-REPAIR.11.9.1.3.1`, tranche 4a)`.
It carries the SEVENTEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the SEVENTEENTH rotation (2026-09-15)
from the repository root:

```bash
git show 143d7a3c9167b5df39faa0473d8e1ba082c6f8ee:CHANGELOG.md
```

That snapshot is 93,783 bytes and contains 29 dated entries; its Git blob is
`1f82dd9504e85ed96c6a027d3d21cb5c8bbad0f7`, and its SHA-256 is
`804dadbb42ec9d1031c946455d76a754f26b5ceecf58cb4c4ee20766ab88a4df`. The newest
entry it holds that this digest no longer carries is
`2026-09-14 — Ask the renderer, not the specification (`SIGNOFF-REPAIR.11.9.1.1.3`, tranche 2 complete)`.
It carries the SIXTEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the SIXTEENTH rotation (2026-09-14)
from the repository root:

```bash
git show dd4151b9d5652a5cb2095e333c34282b20d1d613:CHANGELOG.md
```

That snapshot is 95,076 bytes and contains 35 dated entries; its Git blob is
`5f4dfee6e8bd8efafb9f8fc91f50f11143db83e5`, and its SHA-256 is
`6dfcb84b4f7a5c5e462ddf288772083308a0fbde0715a43c02837ea7f803292d`. The newest
entry it holds that this digest no longer carries is
`2026-09-14 — The ledger's own next action was never taken (`SIGNOFF-REPAIR.11.9.1.1`, `.11.9.1.1.1`)`.
It carries the FIFTEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the FIFTEENTH rotation (2026-09-13)
from the repository root:

```bash
git show 2c1bbe80ee904026c171b3f82ace3b6fe8edab54:CHANGELOG.md
```

That snapshot is 92,548 bytes and contains 30 dated entries; its Git blob is
`a515c482b0e6e750208d36e538d0ff8777e3cc4a`, and its SHA-256 is
`b2b9b74cfa7ee1c87d2585aeb30f81108964978d441a759c29a1e2fcdaf1e301`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — Every rung of the proof ladder gets its own negative (`SIGNOFF-REPAIR.4.2.6`)`.
It carries the FOURTEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the FOURTEENTH rotation (2026-09-13)
from the repository root:

```bash
git show 2912136f49316ba310753cf9672188c1d6815901:CHANGELOG.md
```

That snapshot is 95,013 bytes and contains 31 dated entries; its Git blob is
`0d6b714ce9a211a48b98aaaef1bf03c441a2c59f`, and its SHA-256 is
`055464b527c1169cb47944106eec5b61d196fcbad8ddd3be5aa9c281e235d64c`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — The self-test searched for a string it contained (`SIGNOFF-REPAIR.11.4.3.1.7.1`)`.
It carries the THIRTEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the THIRTEENTH rotation (2026-09-13)
from the repository root:

```bash
git show 87a0abf90d0fa24e49ca85b5e121722fbacb7344:CHANGELOG.md
```

That snapshot is 95,400 bytes and contains 31 dated entries; its Git blob is
`2b9be00dffbbe7f5837f20710769c81d19ef02ae`, and its SHA-256 is
`ffabf15976215219ee32b88979a88853c237e7e9fead69f525fe51c99271609b`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — There are three clocks, not two (`SIGNOFF-REPAIR.3.4.3.1`)`.
It carries the TWELFTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWELFTH rotation (2026-09-13) from
the repository root:

```bash
git show a972d89550df2aab9c554c474a3301f76d04c201:CHANGELOG.md
```

That snapshot is 95,460 bytes and contains 31 dated entries; its Git blob is
`6cb62466f857386535e63605f3c268442c66bad1`, and its SHA-256 is
`7beecd2a5acd1ecaca3df1e4353511c8239b983b95f3610630458855eaff29a2`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — A `join` carrying a decline was accepted as a join (`SIGNOFF-REPAIR.3.4.4`)`.
It carries the ELEVENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the ELEVENTH rotation (2026-09-13) from
the repository root:

```bash
git show f425ae0a5bff201bedde5d228c9e4659c7408740:CHANGELOG.md
```

That snapshot is 95,641 bytes and contains 31 dated entries; its Git blob is
`18388aa961ae05ef8dd1c104f35f1268cd239b71`, and its SHA-256 is
`2a3b7fb9241e341193b514947b9a5f9356659eebb6c919a4854ae6ae4f50bf55`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — The cache's freshness window compared two different clocks (`SIGNOFF-REPAIR.3.4.3`)`.
It carries the TENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TENTH rotation (2026-09-13) from the
repository root:

```bash
git show 09b4f39d2d489877ec0738b980459a60865c08f9:CHANGELOG.md
```

That snapshot is 95,623 bytes and contains 31 dated entries; its Git blob is
`3b7216e39d03f40944868b6862dcfeefded11831`, and its SHA-256 is
`fba84ccdec894c49a7193bf473af067297deb36ab115bb6a729d6b3b601b904f`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — ⛔ Correction: the `delegable` flag governs chains that do not exist (`SIGNOFF-REPAIR.3.4.1`)`.
It carries the NINTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the NINTH rotation (2026-09-13) from the
repository root:

```bash
git show be8cdb707f09d5d1746d5e991c5ff969b9a096d9:CHANGELOG.md
```

That snapshot is 92,812 bytes and contains 29 dated entries; its Git blob is
`a217f1b62579987706bc1ec5e417f5386332c398`, and its SHA-256 is
`9fe01e8e07ecafe729e0728f2cd2d8411d193555f8e73ae678508e7e96b337fc`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — The guard census, reconciled with instruments (`SIGNOFF-REPAIR.3.3.4.13`)`.
It carries the EIGHTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the EIGHTH rotation (2026-09-13) from the
repository root:

```bash
git show c8e591a2dbc1dc5c1f85d428453f59b747da3249:CHANGELOG.md
```

That snapshot is 93,824 bytes and contains 27 dated entries; its Git blob is
`e42def45b1ea97ee33f50ee81b220d2b51ee5a2f`, and its SHA-256 is
`6dc11f303f1b0e329a78e301869559d87d493a3e6921b392028be7bc534f939b`. The newest
entry it holds that this digest no longer carries is
`2026-09-12 — The profile/card surface, censused and split`.
It carries the SEVENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the SEVENTH rotation (2026-09-13) from the
repository root:

```bash
git show b3802b16601fb4a2da67a8721d7659ae1e36aed9:CHANGELOG.md
```

That snapshot is 94,920 bytes and contains 27 dated entries; its Git blob is
`9591c6945fc43581906629a07b83a99afcad3bf4`, and its SHA-256 is
`fb90861ac348208d4f01557b3dac668ca7a1f9b35ff340ea8eaf67eba9216f56`. The newest
entry it holds that this digest no longer carries is
`2026-09-12 — Revocation becomes one transaction, from the admission to the evidence`.
It carries the SIXTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the SIXTH rotation (2026-09-13) from the
repository root:

```bash
git show 711885c43aad15bf423f55581dee8b2e8f85ab40:CHANGELOG.md
```

That snapshot is 95,390 bytes and contains 28 dated entries; its Git blob is
`80fbb202f0780555d646345863d978f459ffbe15`, and its SHA-256 is
`5eb37168688d4b9e04c57894f54cc2762a87992f50e2a3fc56178eb5eea02d48`. The newest
entry it holds that this digest no longer carries is
`2026-09-12 — Census and retire the runner's retained clusters, with a tracked instrument`. It carries the FIFTH
rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the FIFTH rotation (2026-09-12) from the
repository root:

```bash
git show 55d5f9edf4cacc2e321126e9a248f2d74a248fd1:CHANGELOG.md
```

That snapshot is 93,820 bytes and contains 34 dated entries; its Git blob is
`5670f2ab78d8ec43a889a0851821cff9635a4f70`, and its SHA-256 is
`6c29b639895357fa90bec167405e93268a07f51ca7ae3a7738537f2edb18a6ff`. The newest
entry it holds that this digest no longer carries is `2026-09-12 — Drive an R2
acquisition to its persisted evidence`. It carries the FOURTH rotation's notice
in turn, which names the ledger before it.

Retrieve the ledger immediately before the FOURTH rotation (2026-09-12) from the
repository root:

```bash
git show 1faac4e126325c61842fd17f56feeb32b6bd6f5f:CHANGELOG.md
```

That snapshot is 93,689 bytes and contains 45 dated entries; its Git blob is
`c32524d1c1a7b042cc6a8647136b8729d66d4ac6`. The newest entry it holds that this digest no longer
carries is `2026-09-11 — Own the Git acquisition workspace`. It carries the THIRD
rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the THIRD rotation (2026-09-12) from the
repository root:

```bash
git show 0cda20cfa12c62269d14e5ea78412d1548deab68:CHANGELOG.md
```

That snapshot is 94,066 bytes and contains 68 dated entries — the 38 retained
above plus the 30 rotated out of it, the newest of which is
`2026-09-11 — Bind R2 responses to owned input bytes`. Its Git blob is
`75a374a51aa21a1d7d226dc63f6349bd0b9b477f`.

That snapshot in turn carries the SECOND rotation's notice, which names the
ledger before it:

```bash
git show f75106915ff1f1171b332451257388905b05d815:CHANGELOG.md
```

That snapshot is 93,956 bytes and contains 91 dated entries; its Git blob is
`b9aacfc467e1729cae1a5e76fe1d0adee6398c1d`. It carries the FIRST rotation's
notice in turn:

```bash
git show 25ed7d184203e2d8701800558b785b30c75bb4d0:CHANGELOG.md
```

That earliest snapshot contains 130 dated entries. Its Git blob is
`0bc51d581f9158ebafcef94cfb6717464722c6cb`; exact byte/line counts,
SHA-256 identities and the first transition's evidence are in
`docs/decisions/2026-09-09_changelog-rotation.md`. Use
`git log --follow -- CHANGELOG.md` for earlier versions. Keep the reachable Git
history when cloning or handing off; a shallow checkout may need the named commit
before retrieval. A missing object is a retrieval failure, never evidence that
history was empty.

Historical success statements describe the recorded revisions and assertions.
Current qualification is in `LIVE_STATUS.md` and the mdBook's qualification review;
open repairs remain tracked in `docs/tasks/SIGNOFF-REPAIR.md`.
