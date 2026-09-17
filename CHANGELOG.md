# CHANGELOG.md

## 2026-09-17 — A credential binding is tenant-bound (`SIGNOFF-REPAIR.11.14.3.10`)

🔴 **A credential SELECTOR lived on a content-addressed row, so a second tenant drove an authenticated acquisition with the first tenant's credential.**

- **The defect.** `resource_references` is keyed `UNIQUE (original_locator, expected_digest)` — its identity is the CONTENT. `credential_binding_ref` is not content: it is the caller's means of ACCESS, and the R5 arm hands it straight to `broker.resolve`. A second tenant registering the same pair replayed the first tenant's row, inherited a binding it never named, and its resolve attached the OWNER's credential. ⛔ ROADMAP §16.3 invariant 5: *target credentials are selected only after authorization for the concrete target and action.*
- **REPRODUCED RED FIRST**, gate open, one binding registered, two tenants, one locator: the stranger's resolve answered `resolvers: ["r5-credential-broker"]` → `destination_refused`, and that refusal is the **loopback pre-flight**, reached only once a credential has RESOLVED.
- ⭐ **The discriminator is the RESOLVER, not the error kind**, and naming it was the difference between a control that proves something and one that passes. Both paths end at the same loopback refusal; only `resolvers` says which ran. ⛔ My first draft asserted `credential_unavailable` and would have FAILED against the repaired product.
- ⭐ **DECIDED: the selector moves to `reference_registrations`, and `migrations/0069` DROPS the old column.** The shared row keeps the CONTENT; the tenant-bound row keeps the DECISION — this family's own correction for the **fourth** time (citations, assessments, reference reads, now a means of access). Promoted as `docs/knowledge/a-shared-row-may-not-hold-a-tenant-decision.md`.
- ⭐ **The repair is wider than a denial, and that was measured rather than designed.** The binding is a RANKING input: it ranks only the `credential`-class packs, and a binding-less reference ranks only the `none`-class ones. A tenant that named no binding is **not routed to the credential broker at all**.
- ⭐ **And a limit closes in the OTHER direction that nobody had stated.** The pair key meant two tenants citing one URL could not hold two DIFFERENT bindings — the second's value was discarded by the replay. Both may now.
- ⭐ **The strongest part is structural:** the unbound read has no column to read a selector from, so it cannot leak one even by mistake. Only `get_for_tenant` supplies one, and only the asking tenant's own.
- ⚠️ **The backfill is EXACT, which is the only reason a credential may be moved at all.** `registered_by` and `submitted_by` are literally the same value, so the submitter joins precisely to its own registration and every other tenant's gets NULL — §16.4's fail-closed rule for secret access. ⛔ Had no such join existed, the honest backfill would have been NOBODY.
- 🔴 **A defect of my own, caught by the neighbours.** The new control left the opt-in gate open and broke `the_gated_packs_resolve_only_while_the_gate_is_open`. ⛔ Closing the gate at the END of my test was NOT the fix — a test that PANICS never reaches its own end, which is exactly how it was found. The gate is now normalised in `pool()` with the other fixtures.
- **FALSIFIED** — source and migration stashed together, stash verified landed, RED at exactly the defect arm (`r5-credential-broker` where the repair gives `r0-https-fetcher`), restored and re-verified.
- **No regression:** `profiles` 55/55, `migration_upgrade` 4/4, `backup_restore` 1/1, `--lib` 113/113, clippy clean.
- ⚠️ **What is NOT closed, and is owned.** The broker's namespace is global: naming a binding an operator created for someone else is still sufficient. It needs an operator-issued tenant-scoped grant, and `GrantAction` is thread-scoped (10 variants, 0 about a resource or credential) — a §16.4 authorization surface, not a wiring change. `.11.14.3.10.1`.

## 2026-09-17 — The plan checker had no commit-time trigger (`SIGNOFF-REPAIR.11.14.1.2`)

⭐ **The failure class that bit three times today is now mechanical — and building the gate produced a fourth instance of it, caught.**

- **The rule is not new and the checker is not wrong.** The shared runtime checker already refuses a cleanup plan whose declared tables omit a real foreign-key child. ⛔ **What was missing is a TRIGGER**: it only runs when a suite runs, so a plan nobody executes is never checked. Six suites were refused for ~20 commits, and the two omissions came from `migrations/0062` and `migrations/0067` — the second in the session that wrote the sweep-key rule down.
- ⭐ **`.11.6` is satisfied rather than waived.** It forbids proposing a rule before its population is measured; `.11.14.1.1` measured it in the PREVIOUS commit — **16 parent tables, 45 inline foreign keys, 29 declared plans, 6 refused** — and the population is now **0 refused of 29**. The gate ships GREEN, and its value is preventing the class rather than finding a present defect. Said plainly so a green run is not read as evidence of one.
- **It needs no database**, which is what makes it affordable in a pre-commit hook: `ALTER TABLE … ADD CONSTRAINT … FOREIGN KEY` appears **0** times, so every key is inline in a `CREATE TABLE` and the graph is readable from the migrations.
- ⚠️ **IT REFUSES RATHER THAN UNDER-REPORTS.** An unmodelled key form is an ERROR naming that reason; an unreadable `delete_tables(` call shape is an ERROR naming the file. Only two files are excluded — the helper's own definition and its 15 negative-control tests — **by path, with the reason**.
- 🔴 **That defence was earned, not anticipated.** The first draft parsed **29** plans from **45** call sites, and I nearly published the difference as coverage. The 16 skipped were the right exclusions, reached **by accident** because a regex happened not to match them. A control now asserts the exclusion is by path rather than by luck.
- **FALSIFIED at both layers.** Against the instrument: the pre-repair `quota` plan restored → RED naming both omissions → byte-identical restore → green. Against the REGISTRATION: two lines removed from `cards.rs` → `make gate` answers `1 doctrine breach(es) — commit blocked`, naming the file and both tables.
- 🔴 **And the registration's first falsification passed for the WRONG reason.** The scripted edit's indentation did not match, so **nothing was injected** and the gate stayed green — a green run proving nothing, exactly like the `pg_stat` instrument `.11.14.3.7` discarded, one layer out. ⭐ **An injection must be shown to land**: `git diff --stat` before reading the result. The same attempt also wrote its backup to a temporary directory this environment refuses, so the restore never ran; `git checkout --` is the restore that cannot fail that way. Promoted as `docs/knowledge/an-injection-must-be-shown-to-land.md`.
- **Self-test: 8 controls**, two-sided, and the SELF-TEST harness discovers it (30 instruments now carry one).

## 2026-09-17 — Six suites could not start, and the second missing child was mine (`SIGNOFF-REPAIR.11.14.1.1`)

🔴 **Six suites could not start, since REPAIR-0213 — and the second missing dependency was mine, added in the session that promoted the rule against exactly this.**

- **Found because a broad verification run stopped dead** at a suite this session had not run before. Attributed by command rather than by reading: `evidence_citations` landed in `b4d6201` (REPAIR-0213); the `quota` plan was last touched in the **earlier** `40155d1` (REPAIR-0154); and with this session's working changes stashed, the suite fails identically at committed HEAD.
- **The mechanism.** The shared checker validates a plan BEFORE the first deletion and requires dependents to precede parents, cascading ones included. `evidence_citations` declares `tenant_id … REFERENCES tenants ON DELETE CASCADE`, and six plans delete `tenants` without naming it: `cards`, `classification`, `federation`, `mcp_listen`, `quarantine`, `quota`.
- 🔴 **AND I DID THE SAME THING, TODAY.** `migrations/0067` added `reference_registrations` with foreign keys to **both** `resource_references` and `tenants`. I swept the 21 plans naming the first — the table I was thinking about — and not the plans naming the second. So all six were missing **two** children and the second was mine, added in the very session in which I wrote `docs/knowledge/a-census-is-as-wide-as-its-key.md`. ⛔ **Knowing the rule is not applying it: the sweep must be keyed on the NEW TABLE'S OWN constraints, every one of them, computed rather than recalled.**
- ⚠️ **"SIX" is a correction to this leaf's own first number, made inside one session.** It opened saying **TEN**, from a loose census — `grep -l '"tenants"'` matches a row-count tuple in `authority_transaction.rs:346`, a bare list entry in `migration_upgrade.rs:118`, and files with no cleanup plan at all. The tightened instrument parses each `delete_tables(…, &[…])` array: **16 tables carry a direct FK to `tenants`, 27 declared plans delete it, 6 were refused.** ⭐ Catching the wrong number inside the session is the point of the audit; publishing it in a committed leaf first is what it cost.
- ⭐ **The rule, derived before the edit.** The runtime checker already computes the right thing. What is missing is not a rule but a **trigger** — it only runs when a suite runs, so a plan nobody executes is never checked. The census above is that trigger made cheap: it needs no database, because the constraint it depends on is declared in the migrations.
- ⛔ **Whether it becomes a GATE is decided: not yet, and `.11.6` is the reason.** Its population was measured for the first time in this commit, and a rule proposed in the same commit that first measured its population is a rule proposed before the population settles.
- ⚠️ **What this means for a published claim:** the full-checkpoint record's *"40 of 40 database suites, 291 tests, no failures"* predates `migrations/0062`. Six of those suites could not run today. `.11.4.7` re-derives the gate records and now has one more reason to distrust a count taken before the source review.
- **Verification:** all six RUN and pass — **14 tests / 0 failed**, rc=0; the census re-derives to **0 refused of 27**.

## 2026-09-17 — The acquisition path takes the quota (`SIGNOFF-REPAIR.11.14.3.14`)

⭐ **The director delegated this call on 2026-09-17. Taken in full, and the finding is why the two scopes had sat unwired since `0047`.**

- **The measurement.** `resolver` and `destination` ship in `SCOPE_KINDS`, in `migrations/0047`'s CHECK constraint, and in **nothing else**. Meanwhile `POST /v1/resources/{id}/resolve` carries **no quota, no storm control and no breaker** and performs a real network acquisition per call — which is exactly what §16.11 names as "resolver abuse" and "scraping". ⚠️ A raw `grep -c 'pub const SCOPE_'` returns **5** because it matches the `SCOPE_KINDS` array itself; the array's own type is `[&str; 4]`.
- **(1) The gap §16.11 names is the ACQUISITION path, not the thread verbs.** ⛔ The other eleven thread operations do not gain quotas: `.11.14.3.7` measured that the citation amplification is inside ONE request, so a per-hour ceiling bounds arrivals and not per-request work, and bounding the rest would be a rule without a population (`.11.6`).
- 🔴 **(2) WHY THEY SAT UNWIRED — and the deferral never said it.** `check_in_tx` is fail-closed, and `0047` is right that the bound must exist before the surface is usable. That works for the two WIRED scopes because of a property these two lack: **their members are created by a path the server controls, so the bound is seeded at creation** — a tenant by the enroll transaction, a principal by enrolment and card import. The RESOLVER space grows at runtime through `POST /v1/resolvers`; the DESTINATION space is the open internet. ⛔ **A fail-closed bound over a space you cannot enumerate is not a bound, it is an outage.**
- ⭐ **(3) DECIDED: the fail-closed CONTRACT is kept and the MECHANISM changes.** A per-tenant DEFAULT row at the wildcard scope id `*` — seeded by `insert_defaults_in_tx`, backfilled by `migrations/0068` — with a specific row overriding it, and **the absence of BOTH still the typed refusal**. `check_open_scope_in_tx` resolves most-specific-wins with one existence probe and reaches `check_in_tx` unchanged, so the window arithmetic and the recorded `use`/`denial` are the ones already qualified. The property fail-closed exists for — *there is always a bound* — is untouched; the default is now a **row** rather than an **absence**.
- **(4) It counts ATTEMPTS**, after the ranking and before the pack executes, on the ranked resolver and the locator's host. An attempt is what a caller repeats and what reaches the network — the control shows the bound consumed by an attempt the destination policy then refuses. ⚠️ A specific row starts its OWN count, because `quota_events` is keyed by `quota_id`: a narrowed bound is a new bound, not a continuation.
- ⛔ **The ceilings are dev-profile defaults, labelled at the constant.** No acquisition volume has ever been measured, and `.11.6` forbids proposing a threshold before its population. **What was decided is the shape of the bound, not its number**, and no gate may read the ceiling as evidence that abuse is bounded at any level.
- **Verified:** the enrol transaction seeds **one default row per open scope, not one per member**; one resolution records **one use per scope**; a host-specific ceiling **overrides the default** and refuses `429` with a **recorded denial**; removing **both** rows is still the typed fail-closed `503`.
- ⚠️ **The first broad run was BLOCKED by a pre-existing defect and that is recorded rather than worked around**: the `quota` suite — and **nine** others — cannot start, because `migrations/0062` added `evidence_citations` with a foreign key to `tenants` and did not sweep the fixture plans that name `tenants`. Attributed by command (`REPAIR-0213` added the table; the plan was last touched in the earlier `REPAIR-0154`) and reproduced at committed HEAD with this session's changes stashed. `.11.14.1.1` owns it.

## 2026-09-17 — The register asked a question about its reader (`SIGNOFF-REPAIR.13.5`)

🔴 **The blocker register asked a question about its reader, and the director had to point it out.**

- **The defect, named rather than apologised for.** `.13` gave the register an `Ack?` column meaning *"has the director engaged with THIS row"*, and a rule that every stopping-point reply re-surface every row whose value was `no`. ⛔ That is **a field whose value is a fact about the READER, in the maintainer's own register** — the register cannot observe it and the maintainer cannot set it, so the only mechanism available was to repeat the row until the reader reacted.
- **Measured, not felt:** `git log -S"Ack? = yes"` over both copies returns **nothing**. From 2026-09-15 to 2026-09-17 the column was `no` on every open row and never once anything else. ⭐ A column whose value has never changed in its lifetime carries no information — the same shape as a control never seen RED, which this tree refuses everywhere else.
- ⛔ **And it inverted a delegation.** The director had delegated blocker disposition; the column made his attention the precondition for a row to stop being repeated at him. An instrument for *surfacing* had become one for *nagging*, which trains a reader to skip the section the register exists to be read.
- ⭐ **The replacement is a different question, not a renamed column: `Owed here?`** — *is there anything left that THIS REPOSITORY can do about this row?* Set by the maintainer from the owning leaf, beside the named next action. A `yes` row is **work** and belongs in the frontier; a `no` row is surfaced **once per session** with its external party and trigger.
- 🔎 **It paid for itself immediately, and that is the finding.** Under `Ack?`, B1–B3 were reported to the director as *"OUTSIDE"* with nothing owed. Under `Owed here?` all three are **yes** — they share one in-repo prerequisite, `SIGNOFF-REPAIR.14`'s frozen exposure candidate, which `docs/book/src/blockers.md` **had already named**. ⛔ The old column was hiding the maintainer's own work from the maintainer, and the same reply that called those rows external also linked the page that said they were not.
- **The corrected values:** B1 **yes**, B2 **yes**, B3 **yes**, B4 **no** (the only row with nothing owed here), C1 **no** and relabelled *not a blocker — a limit on what may be CLAIMED*, C2 **yes** (four gate records remain).
- ⚠️ **No blocker's substance moves.** B1, B2 and B4 still need outside parties; `.14` remains under its standing prohibition against turning the exposure profile on; G6/G7 remains NOT MET. What changes is which rows the maintainer treats as work.

## 2026-09-17 — The snapshot census was eight, not seven (`SIGNOFF-REPAIR.11.14.3.15`)

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

## 2026-09-17 — What a citation list costs (`SIGNOFF-REPAIR.11.14.3.7`)

⭐ **Three answers, and the most transferable one is about an instrument I threw away.**

- **The amplification, measured.** `.11.14.3.2` made each citation register a §12.1 reference **inside the thread's aggregate transaction**, which holds `FOR UPDATE` on the thread's row. **64 distinct citations register 64 references**, asserted by a live control. ⚠️ The leaf's acceptance asked for a before→after on transaction DURATION; the row count is substituted deliberately and the substitution is stated — duration is the symptom, the count is the thing, and a wall-clock assertion over ~60 round trips measures the machine.
- ⭐ **(1) A windowed quota is the WRONG instrument for this defect**, measured rather than argued: the cost is inside ONE request, so a per-hour ceiling on `thread.contribute` bounds how many requests arrive and says nothing about how long any one of them holds the row. The dimension is request SIZE, not call rate.
- **(2) What bounds a request today**, read from the vendored source rather than recalled: `axum-core-0.5.6/src/ext_traits/request.rs:319`'s `DEFAULT_LIMIT = 2_097_152` — **2 MiB, inherited from a dependency**, since `grep -rn 'DefaultBodyLimit'` returns 0. ⛔ Declaring it explicitly was NOT taken here: one global limit also governs `POST /v1/snapshots`, which legitimately carries base64 evidence bytes, so it is a PER-ROUTE decision and taking it inside this leaf would be the unmeasured policy the leaf exists to avoid.
- **(3) The de-duplication ships**, DERIVED rather than chosen — the `(uri, digest)` pair IS the reference's identity, so a repeat can only fetch back what the first citation wrote. ⛔ It de-duplicates the **work**, never the **record**, and that is structural: the decision is a pure function returning one answer per citation, so the event still carries all 48 repeats in order with their own notes. ⚠️ It is **not a bound** — N distinct citations cost the same as before.
- 🔴 **HOW IT IS VERIFIED, and what that cost.** The de-duplication is **not observable through any product surface**: the row count is **1 either way**, because the pair replay already returns the existing row. ⛔ A `pg_stat_user_tables` scan-counter instrument was written for it and **DISCARDED** — run against the unrepaired handler it reported **the same value**, and the control passed both times. **A control that passes identically either way measures nothing**, and shipping it would have converted an unverified change into one that looks verified. The decision was extracted into a pure function and falsified directly instead: a locator-keyed implementation gives `[0, 0, 0, 0]` against the correct `[0, 1, 0, 3]`. Promoted as `docs/knowledge/a-change-no-surface-can-see-needs-a-seam.md`.
- ⚠️ **A correction to the leaf's own census:** it said *"the one `quota::check_in_tx` call is `OP_INVITE`'s"*. Re-derived, there are **2** — `OP_INVITE` on the tenant scope and the MCP write gate on the principal scope.
- ⚠️ **Two control defects of my own**, both caught by running it and corrected rather than relaxed: an assertion keyed on `len() == REPEATS` matched the DISTINCT contribution's event because both arms used 64, and the events payload is `{"events": […]}` rather than a bare array. Neither was a product defect.
- **Verification:** `profiles` **52 passed / 0 failed**, plus the pure function's unit test falsified by injection.

## 2026-09-17 — A reference's `scheme` is a capability selector (`SIGNOFF-REPAIR.11.14.3.5`)

🔴 **Three mechanisms in one goal line. ONE was a real defect, one was a design I had misread, one was already decided — and the misreading was caught by the product's own controls after I had taken it through a RED/GREEN cycle.**

- 🔴 **(1) `scheme` — REFUTED, not repaired, and the route there is the main deliverable.** The premise came from a test helper's comment (*"caller-supplied and NOT validated against the locator"*) beside the facts that §12.2 ranks resolvers on the field and that `resources::scheme_of` exists with one caller. ⛔ I wrote the check, reproduced RED (`ftp://…` declared `https` registered, `200`), went GREEN on my own control — and **two of this suite's own controls refused it**: `the_r1_resolver_resolves_git_…` and `the_gated_packs_resolve_only_while_the_gate_is_open`.
- ⭐ **Measured instead of inferred.** The consumer is one predicate — `resolvers::resolve`'s `WHERE schemes @> $1::jsonb` — and **two SHIPPED packs pair a non-URI scheme with an `https://*` locator pattern**: `r1-git-fetcher` advertises `["git"]`, the R3 browser pack advertises `["web+render"]`. A Git repository and a rendered page are both reached over HTTPS. **The field is how a caller asks for a CAPABILITY.**
- ⭐ **(1) DECIDED: no check; the contract is STATED** — at the type, in the book, and in a control arm that pins the refutation. The helper comment that produced the wrong premise is corrected **at its source**, because a sentence that produced one wrong repair will produce another. ⚠️ Nothing else validates it either, deliberately (§3.7: accepting a reference is not a promise the core can resolve it), and a second arm pins that. ⛔ A shape check was considered and rejected under `.11.6` — no census of malformed scheme tokens exists, and inventing a rule before its population is how this section went wrong the first time.
- 🔎 **What caught it, because that is the transferable part:** the two refusing controls are not about references — they exercise the PACKS, and **the packs are the consumer**. A repair is falsified by the code that USES the thing, which is the same place its contract lives. Running the affected suites BROADLY rather than only this leaf's own control is what surfaced it; the narrow run was green. Promoted as `docs/knowledge/a-fields-name-is-not-its-contract.md`.
- 🔴 **(2) The dead column defaults — a real defect, repaired.** `migrations/0023` declares `NOT NULL DEFAULT 'network'` / `'low'`; `#[serde(default)]` is `String::default()` — the **empty string** — and the store binds it explicitly, so the column default never applied and `''` is not a risk class. The citation path wrote the declared values, so **the two writers produced different rows for the same omission**. The serde default is now the schema's, and the control asserts it by COMPARING the two writers' rows.
- ⚠️ `low` is the permissive direction and it is **adopted rather than chosen** — it is what the migration already recorded. ⛔ Nothing reads either column for a decision; §12.2's risk filter owns whether `low` may be a default at all, and that is stated at the field.
- ⭐ **(3) The fragment stays in the locator** — three spellings, three references, and the arm pins the behaviour rather than a repair. Splitting IS canonicalization, which §12.1 defers as scheme-specific and which would erase a distinction that section protects. ⭐ The same boundary `.11.14.3.13` drew from the other side, so the two leaves agree and the line is stated rather than felt.
- ⚠️ **The leaf therefore closes with ONE repair and TWO stated contracts**, which is the honest count rather than "three findings, three fixes".

## 2026-09-17 — A snapshot names the locator its reference names (`SIGNOFF-REPAIR.11.14.3.13`)

⭐ **The census decided the disposition, and the new check found a defect in this session's own test data on its first run.**

- **The census, first because the leaf asked for it.** `grep -rn "original_locator" crates/reasonbraid-server/src/*.rs`: every hit outside `resources.rs` is either `reference.original_locator` — the REFERENCE's field, which the resolvers read to fetch — or the snapshot column being written, mapped and listed. ⭐ **Nothing reads `evidence_snapshots.original_locator` to make a decision**: no resolver, no gate, no routing rule. So this is an evidence-integrity defect rather than a routing one, and a refusal is the whole repair — there is no downstream behaviour to correct, only a record that could be false.
- 🔴 **RED, falsified against the exact unrepaired store.** A submission naming `https://example.org/some-other-document`, filed against a reference registered as `https://example.org/the-registered-report`, was **stored**: `200 {"replay":false,"snapshot_id":"snp_01a0af0ef6fe7f028d0e2014a444655b"}`; `49 passed; 1 failed`.
- ⭐ **DECIDED: byte equality with the reference's locator; `final_locator` untouched** (`docs/decisions/2026-09-17_a-snapshot-names-the-locator-its-reference-names.md`, three alternatives rejected).
- ⛔ **The leaf's warning is ANSWERED rather than obeyed.** *"An equality check is a canonicalization decision wearing a different name"* is true of a **normalising** check — one that lower-cases a host or strips a trailing slash to decide two spellings are the same — and false of a **strict** one, which normalises nothing and decides nothing. §12.1's immutable locator is exactly what makes the reference's stored string the identity to compare against.
- ⭐ **`final_locator` stays free, and the control asserts it.** A redirect legitimately ends somewhere else, which is why §12.6 records both. A repair that compared them too would have been WRONG, not merely stricter.
- **The refusal names BOTH values.** It runs after `.11.14.3.11`'s registration predicate, so the caller has proved it registered the reference and may read that locator — quoting it discloses nothing it does not hold, and the diagnosis is worth more than the symmetry.
- ⚠️ **NO REGRESSION, and it corrected a control of my own.** `.11.14.3.6`'s pin control passed the pinned report's locator for BOTH of its references — simply wrong about which document the unpinned snapshot was of, and nothing checked it. It now passes each reference's own locator, with every original assertion intact.

## 2026-09-17 — A snapshot is filed against a reference its own tenant registered (`SIGNOFF-REPAIR.11.14.3.11`)

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

## 2026-09-17 — A resolution says when its evidence was not persisted (`SIGNOFF-REPAIR.11.14.3.12`)

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

## 2026-09-17 — A reference's `expected_digest` names its bytes (`SIGNOFF-REPAIR.11.14.3.6`)

🔴 **A §12.1 field a caller supplied to say "these are the bytes I expect" constrained nothing, and enforcing it is what makes the pair key mean something.**

- **The census, re-derived on the working tree rather than quoted:** `git grep -n expected_digest -- crates/reasonbraid-server/src` → **20** hits (13 at the pinned commit; the growth is the registration path WRITING the column). `snapshots::submit` asked `SELECT EXISTS (SELECT 1 FROM resource_references WHERE resource_id = $1)`. **Zero sites read a STORED pin.**
- 🔴 **RED, falsified against the exact unrepaired store** — the production file reverted with `git stash push`, the control run, the file restored. A reference pinned to `sha256(the audited figure is 41.2 per cent)` accepted a snapshot of `the audited figure is 62.8 per cent`: `200 {"replay":false,"snapshot_id":"snp_01a0aec59be97d93af711df98b146e90"}`; `46 passed; 1 failed`.
- ⭐ **DECIDED: a pinned reference accepts only the bytes it names; an unpinned one is unchanged** (`docs/decisions/2026-09-17_a-pin-names-its-bytes.md`, three alternatives rejected). ⭐ **Enforcement is what makes `.11.14.3.2`'s pair key MEAN something**: that leaf made `(locator, digest)` the reference's identity so §12.6's changed page would be a SECOND reference rather than an erased distinction — and with no checkpoint both rows accepted any bytes, so the distinction the key was created to preserve was preserved nowhere.
- ⚠️ **The leaf's own worry resolves rather than binds.** It warned that *"a pin enforced at acquisition would forbid exactly that [plural]"*. The plural belongs to the UNPINNED reference: `evidence_snapshots` replays on `(reference_id, raw_digest)`, and the control asserts one unpinned reference holding **2** versions.
- ⛔ **The refusal carries NEITHER digest.** The caller already holds the actual one — it hashed the bytes it sent — and the pinned one belongs to a reference this route does not check the caller may read, so quoting it would be an oracle over a `res_…` id. The message carries the next step instead: *register the locator at the new digest and acquire against that.* In the STORE rather than at the resolver, for `.11.14.3.8`'s reason — four call sites reach it.
- ⛔ **CONSEQUENCE, stated rather than discovered.** `api.rs`'s R0 and R5 arms call `let _ = crate::snapshots::submit(…)`, so a pinned reference whose page drifted now returns an acquisition receipt and no snapshot, **silently**. The discard predates the pin (its own comment says so); what changed is that it now has a likely, caller-meaningful cause. Censused to **two** sites — the R2 arm captures its result. `.11.14.3.12` owns it.
- 🔎 **And a gap in MY OWN census one commit earlier — `.11.14.3.11`.** `.11.14.3.4` enumerated the routes under `/v1/resources` and bound the two that read a reference; `POST /v1/snapshots` names a `reference_id` in its **BODY**, so a route-prefix census cannot see it. ⭐ That is `.3.5.3`'s shape exactly — a census correct about its own scope and silent about what fell outside it — committed by the session that had just written the rule down.
- **Verification:** `RB_DEMO=0 bash scripts/run_pg_tests.sh profiles` → **47 passed / 0 failed**; the RED baseline 46/1.

## 2026-09-17 — The §12.1 reference detail read is bound to its registrants (`SIGNOFF-REPAIR.11.14.3.4`)

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

## 2026-09-17 — A corrected claim was corrected in the register and left standing in the corpus (`SIGNOFF-REPAIR.13.4`)

🔴 **Two blockers were measured false on 2026-09-15 and 2026-09-17. The book kept asserting both.**

- `.13.3` (REPAIR-0220) measured *"remote CI has never run"* FALSE — 41 runs, green at `origin/main` — and corrected the register, the blockers chapter, the decision record and the one downstream conclusion inside the task tree. `.13.2` (REPAIR-0195) cleared **B5**, the licence. **Neither swept the live-document corpus**, so both superseded claims stayed readable in the two documents the director actually reads.
- **The census, over live documents only** (`README.md`, `LIVE_STATUS.md`, `MEMORY.md`, `docs/book/src/*.md` — the set `COMMIT.md` holds to CURRENT truth; `CHANGELOG.md`, `DEV_NOTES.md`, `docs/decisions/` and the dated artifacts are ledgers and correct as history): **11 hits, 4 stale, 7 sound.** ⭐ Publishing the raw 11 would have been wrong in 7 cases — three unrelated uses of "never runs", the two passages that STATE the correction, and `MEMORY.md`'s already-corrected line.
- **The four:** `LIVE_STATUS.md:356` gave the false claim as the CAUSE of two invisibly-red supply-chain gates; `:382` described the register's own C1 row; `:1541`/`:1543` and `docs/book/src/qualification-review.md:546` carried the same checkpoint sentence, with BOTH stale claims in it.
- **ROOT CAUSE.** `.13.3`'s acceptance reads *"every place **this tree** calls a gate 'passed'"*. Its scope was the register and the task tree, and it reached both. ⛔ Nothing asked what else in the corpus RESTATED the claim — the defect class `.11.16` owns. A claim corrected in its owner and left in its copies is the same failure as a number corrected in its source and left in its mirrors.
- **FIX.** All four corrected **in place, not deleted**, each naming what it used to say and what measured it false, so a reader who remembers the old sentence finds out what happened to it. `checkpoint-7233122.md` is left **byte-unchanged** — a dated artifact records what was measured on its date — and the book now says so.
- **ADDRESSED.** The narrow probe `git grep -nI -E "remote CI has never run|licen[cs]e decision remain open"` over the same four live documents returns **7** lines, every one a QUOTED correction, with no bare assertion left.
- ⚠️ **What this does NOT do:** it sweeps two corrected claims, not every claim. Whether the sweep should be MECHANICAL is routed to `.11.16`, deliberately unanswered — the population must be classified before a rule is proposed, which is why this leaf publishes 4-of-11 rather than 11.
- **Verification:** `make gate` 18/18; `make book` rc=0.

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated twenty-one times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

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
