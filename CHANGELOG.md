# CHANGELOG.md

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

## 2026-09-17 — The standalone assessment route is bound to the citing tenant (`SIGNOFF-REPAIR.11.14.3.8`)

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

## 2026-09-17 — The adapter ladder is ahead of its caller, and the ceiling permitted what nothing declares (`SIGNOFF-REPAIR.13.1.1`)

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

## 2026-09-17 — The assessment namespace is part of the row (`SIGNOFF-REPAIR.11.14.3.3`)

✅ **The leaf was opened on a namespace tidiness question. The control found a live aliasing defect, and that is what shipped.**

- **The population, measured TWO ways**, because "this project has no production data" is an assumption and only one of these survives the project acquiring some. **Writes:** `git grep -n "INSERT INTO claim_assessments" -- .` → **1** statement, reached by **2** callers; no migration, seed, fixture or deployment artifact inserts a row; every invented identifier (`clm_budget`, `clm_beta`, `clm_g4`) lives only in `profiles.rs`. **Durability — and the correction that made it honest.** The first census ran `find . -name PG_VERSION -not -path "./target/*"` and reported no cluster anywhere. ⛔ The EXCLUSION is why that check could not fail: the runner destroys a cluster on success and **retains it under `target/` as failure evidence**. Without it: **32 PG data dirs, 3 retained clusters**, one of them this leaf's own RED run. ⭐ The defensible claim is narrower and rests on the WRITE census, not on `find`: no row is carried in **tracked** state (`target/` is gitignored), and the GREEN run's own line is `pg-tests: stopped and removed target/pg-tests/run-wowo355p`.
- 🔴 **RED, and it is not the defect the leaf was opened on.** `claim_assessments_replay_idx` was `(claim_id, snapshot_id, assessment, author)` and `claims::submit` pre-checks those same four columns. On the standalone route **`author` is a caller-supplied label** — `.11.14.2` established the authorization never reads it — so a caller naming a real thread's minted claim digest, its snapshot, its kind and its author matched the whole key and the pre-check returned **the deliberation's own `assessment_id`**: `assertion \`left != right\` failed … left: "asn_01a0ac9b3bbb7d3293dcd08c20f9fc07" right: "asn_01a0ac9b3bbb7d3293dcd08c20f9fc07"`, `43 passed; 1 failed`.
- 🔴 **The width was published WRONG, and correcting it found a second defect.** The first version called the oracle "bounded to one tenant by `.11.14.2`'s authoring gate". The authoring gate binds the two READS; the replay pre-check carries **no tenant predicate at all**, and that width had been published by READING the SQL rather than measuring it. Measured afterwards, with the namespace column already in place: `a SECOND TENANT was handed the first tenant's assessment_id … asn_01a0ace7727c7b31bea32938bf807721` on both sides, `43 passed; 1 failed`. ⭐ So `authored_by_tenant` joins the key too — which is what makes `migrations/0064`'s own sentence true (it claimed the key's `author` column already separated two tenants, and said eight lines later that `author` is an unauthenticated caller string). `0064` is left byte-unchanged; `docs/decisions/` supersedes rather than mutates. ⚠️ Still open and stated: two principals inside ONE tenant can alias each other — not a disclosure, because the authoring gate already admits both to that row.
- ⭐ **The decision** (`docs/decisions/2026-09-17_the-assessment-namespace-is-part-of-the-row.md`, three alternatives rejected): the namespace is part of the row's **identity**. `migrations/0066` adds `claim_namespace` (`thread`|`external`) and rebuilds the replay index to carry it, `NULLS NOT DISTINCT` so the new key is not weaker than the one it replaces. The namespace comes from the CALL SITE, never the submission, and joins the **pre-check** as well as the index — ⛔ an index alone would not have closed it, because the pre-check short-circuits before the insert runs.
- **The rows: left, no backfill — the third time in this family** (`.11.14.1`, `.11.14.2`), and the reason is not "no data": a pre-existing row's namespace is unrecoverable **in principle**, because the only evidence would be the SHAPE of its `claim_id` and a caller can always type the minted shape. That is the collision itself. NULL means "unattributed", the disposition `0064` already took on this table.
- ⭐ **Minting a digest on the standalone route is the plausible WRONG answer, and rejecting it is the substance of the decision.** What makes the thread identifier trustworthy is `claim_exists_in_thread` — the membership check — not the hashing. A standalone route has no thread to be a member of, so a digest there is a hash of a caller's own string: still invented, now **indistinguishable from a thread's by construction**.
- ⛔ **Filtering the claim read to `thread` rows was also rejected:** it makes the standalone route write-only, worse than the removal already declined, and arrived at by accident rather than decided. Both namespaces ride the read, labelled.
- **Why disclosure is the whole repair, measured:** nothing reads `claim_assessments` to make a decision — `git grep` finds the two list routes and nothing else, and "evidence gate" appears here only in doc comments. An ungated row cannot change an outcome; it can only mislead a reader, and a labelled row does not.
- 🔎 **One residual OWNED, not reported** — `.11.14.3.8`: `POST /v1/assessments` has **no citation gate**, while the `assess` step refuses an uncited snapshot with the stated reason that "an assessment would be a way to learn that a snapshot exists". Both call the same `claims::submit`.
- `the_claim_assessments_validate_the_citation` updated deliberately: the route's contract moved, and it now asserts its own row is `external`.
- **Verification:** `profiles` 44, `command_api` 39, `migration_upgrade` 4 (the new `0066` applies), `mcp_write` 5, `cli_end_to_end` 5 — **97 / 0**, every cluster stopped and removed. `cargo clippy -p reasonbraid-server --all-targets --locked -- -D warnings` exit 0 with zero diagnostics; `cargo fmt --all` rc=0; `make gate` 18/18; `make book` rc=0.

## 2026-09-16 — A contribution's citation registers the evidence reference it names (`SIGNOFF-REPAIR.11.14.3.2`)

✅ **DOC-0037's second join, executed. ROADMAP §13.2 step 2 is wired, and taking the decision found a cross-tenant denial that made the obvious repair unshippable.**

- **RED:** `41 passed; 2 failed`, and the two failures are the two defects. `the_contribution_citation_registers_a_resource_reference` — *"a contribution's `EvidenceRef` resolves to nothing … : []"*, 0 rows where 1 was required. `a_reference_submits_typed_and_the_locator_digest_pair_is_the_key` — the second defect printing itself: `{"code":"locator_digest_conflict","message":"the locator's digest is immutable — the same locator with a different digest conflicts"}`, 409 where 200 was required.
- **The decision** (`docs/decisions/2026-09-16_a-citation-registers-the-reference-it-names.md`, five alternatives rejected): a citation REGISTERS the §12.1 reference it names, on the contribution's own transaction, and the event carries the `resource_id` it resolved to. §13.2 *registers* at step 2 and *acquires* at step 6, so refusing a citation for naming something unacquired inverts the flow.
- 🔴 **The second defect.** `resources::submit` pre-checked on the **locator alone** and refused a differing digest as `locator_digest_conflict`. On the contribution path that becomes a cross-tenant denial: the first principal to pin a locator makes it uncitable by every other tenant *in any form*, because a digest-less citation takes the same branch (`None != Some(D)`). On a public network, pinning the popular URLs is one cheap loop.
- ⛔ **And it was already a §9.8 breach** — *"cross-tenant existence is not leaked"*. A 409 on `(L, D2)` told the caller that somebody else had registered `L` at a digest they were never shown.
- ⭐ **Three roadmap clauses and the schema all said the pair before the code did:** §12.1 (canonicalization "must not erase security-relevant distinctions"), §12.6 ("a live Web page or branch can change"), §9.8, and `migrations/0023:21`'s `UNIQUE (original_locator, expected_digest)` — a constraint the pre-check had made unreachable for a differing digest.
- **`migrations/0065`** rebuilds that constraint `NULLS NOT DISTINCT`, so a digest-less citation is ONE row rather than one per citation. No backfill and none possible to need.
- **Two validations a citation never had:** the scheme is parsed from the locator (RFC 3986 §3.1) rather than claimed beside it, and the digest is ADR-011's `sha256:<64 hex>`. ⭐ The shipped `command_api` fixture cited `sha256:abc123` — six characters where ADR-011 asks for sixty-four — which is what a field nothing reads looks like after two phases.
- **Four residual findings OWNED, not reported** — `.11.14.3.4` (the reference read is unbound, and `.11.14`'s per-table verdict answered the column question), `.11.14.3.5` (the route's `scheme` is unvalidated, its omissions bypass the column defaults, the fragment stays in the locator), `.11.14.3.6` (`expected_digest` is a pin nothing enforces), `.11.14.3.7` (registering turns an unbounded citation list into O(n) work inside the thread's locked transaction; `thread.contribute` carries no quota, and a cap was deliberately not invented).
- 🔎 **Two documents were restating their own numbers wrongly.** `DOCTRINE_ENFORCEMENT.md` said the product emits 18 codes while the census said 19 — and retiring `locator_digest_conflict` made it true again **by accident**, which is exactly why it is written down (`.11.16`). The book's introduction said the changelog had "rotated four times" against its own footer's nineteen, two paragraphs above a sentence warning readers not to trust copied facts; that one is fixed here.
- **Verification:** `profiles` 43, `command_api` 39, `authority` 22, `policy` 14, `node_inbox` 8, `invitations` 6, `cli_end_to_end` 5, `mcp_write` 5, `escalation` 4, `migration_upgrade` 4 — **150 / 0**, every cluster stopped and removed. Strict focused clippy and `cargo fmt --all --check` rc=0; `make gate`; `make book`.

## 2026-09-16 — The `assess` step records an assessment against the thread (`SIGNOFF-REPAIR.11.14.3.1`)

✅ **DOC-0037's decision, executed. The deliberation and the evidence chain meet at the point §13.2 names.**

- **RED was the error message:** `unknown field \`assessment\`` — the contribution body had a payload for `verdict` and one for `synthesis`, and none for an assessment. The control drives the SHIPPED `evidence_review` profile, so the RED is a tenant's real path. `41 passed; 1 failed` → **42 / 0**.
- A contribution of kind `assessment` on the `assess` step records a §12.7 row whose `claim_id` is the digest **the server computed** and membership-checked against that thread, over a snapshot the tenant **cited**, authored by the tenant.
- ⭐ **Every gate already existed** — the membership check, `.11.14.1`'s citation gate, `.11.14.2`'s authoring binding, `claims::submit`'s excerpt validation. **No migration.** The repair added a body, a kind and an ordering, and no new rule.
- ⭐ **Atomicity needed the new code:** `claims::submit` and `is_cited_by` became executor-generic so the contribution event and the assessment row commit in one transaction.
- Six refusals each name their gate: wrong step, foreign claim digest, uncited snapshot, absent excerpt, misplaced payload, unknown assessment word.
- 🔎 The diagnostic was re-run over all thirteen `STEP_KINDS` and classified: 6 gate a kind, 3 are terminals, 2 are default-profile only, 2 are pure sequencing — and the two zeroes are explicitly NOT claimed as defects, because no store sits behind them.
- **Verification:** `profiles` 42, `command_api` 39, `command_ordering` 7, `escalation` 4, `node_channel` 37, `evaluation` 3, `mcp_write` 5, `mcp` 6, `cli_end_to_end` 5, `migration_upgrade` 4; strict server clippy; `cargo fmt --all --check`; `make gate`; `make book`.

## 2026-09-16 — The deliberation flow owns the evidence chain (`SIGNOFF-REPAIR.11.14.3`)

🔎 **The decision was held for the director on the grounds that the frozen roadmap settled it neither way. That was wrong: §13.2 settles it, and had not been read.**

- **§13.2 "Rigorous deliberation reference flow"** is step 2 *register context and resource references*, step 5 *normalize claims … and requested evidence*, step 6 *acquire/assess evidence within the allowed plan*. The canonical flow contains both halves. The earlier claim was written from §12.7 and §5.2.2 — two sections of twenty-five.
- 🔴 **The product already encodes it, and the encoding is inert.** `STEP_KINDS` carries `assess`; `evidence_review` and `policy_proposal` — 2 of 8 shipped built-ins — declare it; and `git grep -n '"assess"' -- crates/reasonbraid-server/src` returns **one hit, the vocabulary constant**. A tenant running the built-in profile for reviewing evidence advances onto a step that records nothing.
- ✅ **Decision:** the deliberation flow owns the evidence chain, joined where §13.2 names it — `assess` writes a `claim_assessments` row keyed by a membership-checked claim digest over a CITED snapshot; a contribution's `EvidenceRef` resolves to a `resource_references` row (the same key). Four alternatives rejected, including a `thread_id` column on the content-addressed tables.
- ⭐ Every part the join needs already exists — the claim digest and its membership check, `.11.14.1`'s citation gate, `.11.14.2`'s authoring binding, `claims::submit`'s excerpt validation, `resources::submit`'s replay. This is wiring, not design.
- Split into `.11.14.3.1`–`.3`, checked against the leaf's four mechanisms so none is dropped.
- No code changed; decision and task-tree only.

## 2026-09-16 — The site authorization skeleton is one copy again (`SIGNOFF-REPAIR.7.4.5`)

✅ **A duplication created deliberately two commits earlier, and closed on schedule rather than left to be found.**

- `.7.4.3` restated ~35 lines of the authorize/apply/audit transaction rather than refactor an authorization path inside the commit that repaired a hole in it. This leaf merges them into one `site_authority::authorized(...)`.
- **What the compiler cannot check is enumerated and tied to named controls:** a denial still commits its record; a domain refusal keeps its grant, boundary and its own 400; an audit failure rolls back an allowed effect (`audit_failure_rolls_back_the_registry_effect`); the effect runs only after a live grant, on the transaction's connection; the clock is the post-lock `clock_timestamp()`.
- **Verification:** `site_authority` 11, `site_registry_http` 8, `site_operator_cli` 3, `regions` 3, `allowlist` 2, `profiles` 41 — **68 / 0**, every count identical to the pre-refactor run. Strict server clippy and `cargo fmt --all --check` rc=0.
- No migration, route, wire shape or qualification category changed, and the book is deliberately untouched: it describes the contract this preserves, not the shape of the code.

## 2026-09-16 — An assessment is read by the tenant that authored it (`SIGNOFF-REPAIR.11.14.2`)

🔴 **A guessed claim identifier returned every tenant's assessments. Reproduced RED, closed, and it corrected the decision record this family rests on.**

- **RED:** three assessments returned to a tenant that authored one of them, through the identifier `clm_budget` — the shipped control's own, because nothing mints the namespace. `40 passed; 1 failed`. GREEN: **98 / 0** across six suites.
- 🔎 **`.11.14`'s per-table verdict was wrong for this table.** It grouped `claim_assessments` with the content-addressed evidence tables and concluded no column was possible. `claim_assessments_replay_idx (claim_id, snapshot_id, assessment, author)` carries the author, so two tenants asserting the same thing already hold two rows: an assessment is an authored opinion, not a shared receipt. The record gains a Correction section and one superseded verdict; the other eleven stand.
- **The binding** is a server-recorded `authored_by_tenant` (`migrations/0064`), read by both assessment surfaces. `author` stays a caller label and the authorization never reads it — that clause is `.7.4`'s.
- ⭐ **It closed `.11.14.1`'s co-citation residual for assessments:** two tenants citing one shared snapshot no longer read each other's positions on it. ⛔ Open for `derivations`, which are content-addressed.
- 🔴 **New owned leaf `.11.14.3`:** the product has a real claim identity — a server-computed digest membership-checked inside a thread — and the evidence graph does not use it, so an assessment may cite a claim no contribution ever made.
- **Verification:** `profiles` 41/41, `migration_upgrade` 4, `command_api` 39, `evaluation` 3, `mcp` 6, `cli_end_to_end` 5; strict server clippy; `cargo fmt --all --check`; `make gate`; `make book`.

## 2026-09-16 — The retention sweep becomes a site-operator act on the server's clock (`SIGNOFF-REPAIR.7.4.3`)

🔴 **One enrolled principal, naming the year 3000, tombstoned every tenant's live evidence. Reproduced RED, then closed on both of its two independent defects.**

- **RED:** `an enrolled principal without site authority swept the site: {"tombstoned":2}` at HTTP **200** — the response body counts both tenants' rows. `39 passed; 1 failed`. GREEN after: six site-touching suites **67 / 0**.
- **Two defects, either sufficient.** The gate was enrolment over a sweep with no tenant predicate; the cutoff was an unbounded `at` read from the request body, so the caller chose which rows were due.
- ⛔ **Not a tenant verb, on the schema:** `retention_class` is a column on the SHARED row, so which rows are due is a site fact no citation owns. It takes the new `evidence_expire` site capability, with the tombstones and their audit committing as one guarded transaction — `migrations/0063`, `site_authority::retention`.
- ⛔ **The caller's clock is removed, not bounded.** A row stamped "the retention expired" is a factual claim; the cutoff is the database's own post-lock `clock_timestamp()`, and `at` is now a **400**. The TTL-boundary controls drive `snapshots::expire_due` directly and keep every assertion they made through the wire.
- 🔎 **Fixed a defect in REPAIR-0213's own control, one commit old:** an enrolment-gate leg asserted 401 while presenting a malformed principal id, so it measured the header parser. The gate answers **403** `unauthorized`; 401 `unauthenticated` is the parser's answer. The book carried the wrong number too. Both corrected; both controls now present a well-formed stranger.
- ⚠️ **Narrowed before writing code:** the shared-row deletion question is split to `.7.4.4` with its own acceptance and a derived candidate answer. **Duplication named, not hidden:** `.7.4.5` owns the second copy of the site authorization skeleton, created deliberately rather than refactoring an authorization path inside the commit that repairs a hole in it.
- **Verification:** `profiles` 40/40, `site_authority` 11, `site_registry_http` 8, `site_operator_cli` 3, `regions` 3, `allowlist` 2; `migration_upgrade`, `rls`, `command_api`, `evaluation`, `mcp`, `cli_end_to_end`; strict server clippy; `cargo fmt --all --check`; `make gate`; `make book`.

## 2026-09-16 — The evidence reads are bound to the citing tenant (`SIGNOFF-REPAIR.11.14.1`)

🔴 **Any enrolled principal could enumerate every tenant's evidence trail. Reproduced RED against a two-tenant fixture, then closed on all five surfaces.**

- **RED:** `the_evidence_reads_are_bound_to_the_citing_tenant` failed at `profiles.rs:4790` — `tenant A enumerated tenant B's evidence trail: [...]`, two rows on A's staleness surface. `38 passed; 1 failed`. GREEN after: **39 passed, 0 failed**.
- 🔎 **The leaf's own prediction — derive the citing tenant "through the reference" — was refuted by measurement.** `submitted_by` is `Uuid::new_v5(NAMESPACE_OID, subject.describe())`, joining to no identity table, and `resources::submit` replays on the **locator alone**, so it names the first citer forever. `git grep -ln snapshot_id -- migrations` returns 3 files, all evidence tables.
- ⭐ **A citation is many-to-many and no scalar holds it.** `migrations/0062_evidence_citations.sql` — `PRIMARY KEY (snapshot_id, tenant_id)` — written by `snapshots::submit` on the fresh insert **and on the replay**. The replay write is what keeps `.6.1.5`'s trap closed: the shared row carries **2** citations and both tenants read it.
- **Bound:** `GET /v1/snapshots/stale`, `/{id}`, `/{id}/derivations`, `/{id}/assessments`, and `DELETE /{id}` — the fifth beyond the leaf's four, because it shares a route with the read. A foreign tenant gets **404**, not 403.
- **Unbound reads are no longer representable:** `snapshots::get`/`stale` are replaced by `get_for_tenant`/`stale_for_tenant`; the 25-field row mapping and the column list each collapse from two copies to one.
- ⛔ `stale` stays a TENANT read on a census — nothing operator-shaped consumes it.
- ⚠️ **Published limits:** no backfill is possible, so a pre-migration snapshot is read by no tenant until cited again; a shared row can still be tombstoned by any one citer.
- 🔴 **Two new owned leaves:** `.7.4.3` — `expire-due` is enrolment-gated with an unbounded caller `at` and no tenant predicate, so one request tombstones every tenant's live evidence, irreversibly; `.11.14.2` — `GET /v1/claims/{claim_id}/assessments` is an oracle over a namespace the server never mints.
- **Verification:** `profiles` 39/39; `migration_upgrade` 4/4, `rls` 1/1, `command_api`, `evaluation`, `mcp`, `cli_end_to_end`; strict server clippy; `cargo fmt --all --check`; `make gate`; `make book`.

## 2026-09-16 — Conform the resume pointer to the template that governs it (`SIGNOFF-REPAIR.11.4.2.3`)

⛔ **DOC-0032 rewrote layer A without re-reading `MEMORY_ARCHITECTURE.md` §6, which specifies the file's exact template.**

- **Five deviations:** the title dropped its cap reminder; `How to resume` did not name `MEMORY_ARCHITECTURE.md`; the `Current state` block used free-form bullets instead of the five named fields and **omitted `in_flight_uncommitted`**; the block heading dropped its overwrite rule; and three sections were invented.
- ⛔ **The invented "Traps" section was a layer violation:** environment quirks belong to layer C by §4's write path, and `2026-09-09_repository-local-command-environment.md` already carried `project_env.py`. The commit that removed durable facts from the pointer put two more back in.
- **Three measurements:** 26 lines / 7,168 bytes → 35 / 3,583 → **16 / 1,395**, with 34 lines and 5,773 bytes free. No cap raised.
- ⚠️ §6 also prefers a DERIVED current-state block. Four of five fields are mechanical; the census is owed before proposing anything, and `.11.4.5.3` rejected its own generator on the inverse ratio. New owner `.11.4.2.4`.
- No product code, schema, test or script changed.

## 2026-09-16 — The pointer stops being a log (`SIGNOFF-REPAIR.11.4.2.2`)

✅ **`MEMORY.md` was at exactly 7,168 of 7,168 bytes and every commit had to evict. The cause was shape, not size.**

- The `- **Next action:**` bullet had become a ~6,000-byte SINGLE LINE carrying sixteen lessons — one line of twenty-six. An append-only log inside one bullet of a pointer.
- 🔴 **`.11.4.2.1` had measured the answer eleven commits earlier and the framing hid it.** It asked "would anything be LOST if a warning were evicted?", found 0 of 26 orphans, concluded the worry was refuted — and never asked why anything durable elsewhere was in the pointer at all. ⭐ A census answers the question it was given; the framing is the part to falsify.
- **Partition, with the count on each side: 14 leave, 2 stay.** Every leaver is recorded in its leaf, `docs/knowledge/`, `TOOLBOX.md`, `docs/decisions/`, or is gate-enforced. The two that stay are pointer-shaped, not lesson-shaped.
- **Headroom, named:** 35 lines of 50, 3,583 bytes of 7,168 — no cap raised. The census now reports 0 standing warnings, 0 uncited, against 16 before.
- ⚠️ The eviction ritual is dissolved, not improved: three sharpenings of a selection rule for a choice that should not have existed.
- ✅ **The vulnerability channel is live** — GitHub private vulnerability reporting was enabled by the owner, so `SECURITY.md` states it plainly and names no fallback.
- No product code, schema, test or script changed.

## 2026-09-16 — Own the advertised policy lines `.7.3.5` did not check (`SIGNOFF-REPAIR.7.3.6`)

- `.7.3.5` found the R3 pack advertising two deny-policies it did not enforce, and repaired exactly those two. The other four lines of the same advertisement were never checked, and a caller choosing a pack reads all of them.
- **Population, by command:** 18 policy-field literals across the three gated packs in `resolvers.rs`, plus the four non-gated packs' rows in migrations `0024`–`0027` (19 `'deny'`, 5 `'follow-classified'`, 5 `'listed'`, 4 `'none'`, 1 `'process'`). ⚠️ Two populations, not one sum — one traces to a call graph, the other to a database row nothing traces.
- **The three R3 lines named:** `javascript_policy: "allow-bounded"` has no stated meaning and `.7.3.5` made it MORE ambiguous; `egress_class: "listed"` implies an allowlist where the code classifies; `sandbox_level: "vm_container"` describes the operator's obligation as though it were the product's behaviour.
- ⚠️ The answer is not assumed to be "enforce them" — at least one line may be honestly corrected instead, and that is the leaf's decision to take.
- No code changed.

## 2026-09-16 — Take the four Internet-qualification decisions and open the exposure-candidate lane (`SIGNOFF-REPAIR.14`)

⭐ **Three of the four outside blockers share one prerequisite, and it is work inside this repository.**

- **The structural finding:** the Phase-7 subtraction record has the exposure profile (S-1) waiting on the qualification gaps while B2 waits on "the exposure profile's candidate freeze" and B3 on "its first action-bearing surface". That circle is an artefact of S-1's stated reason — ROADMAP §16.12 blocks Internet-capable **deployment** and §25.1 blocks **exposing**. Neither forbids building a candidate and leaving it off.
- **Decision 1 — build the candidate, un-deployed.** `SIGNOFF-REPAIR.14` owns it under a standing prohibition: no leaf in the lane may deploy the profile or claim any part of G6/G7. First slice measured: `mtls.rs` builds a complete `rustls::ServerConfig` whose only callers are `tests/mtls.rs:42` and `:72`, and `grep -c mtls` over `rb-server.rs` returns 0.
- **Decision 2 — OSS audit programme first, commercial fallback twelve weeks after the freeze.** §2.6 requires independence, not procurement; the repo is public and the software unreleased, which is the profile those programmes exist for. The fallback is dated because "we applied" is not a plan.
- **Decision 3 — GitHub private vulnerability reporting.** `SECURITY.md` said in its own words that no dedicated endpoint existed. The channel publishes no personal address, creates a draft advisory that is the evidence-preservation machinery §16.12 asks for, and needs one owner-side switch, which the policy names and marks pending. `.well-known/security.txt` publishes the pointer and states why it carries no `Expires`.
- **Decision 4 — clearance covers the US, the EU and the holder's jurisdiction.** Deferrable because the rename cost is measured: 67 source files, 191 lines, and every shipped binary already neutral.
- ⛔ **None of this advances the gate.** G6/G7 is NOT MET and B1–B4 remain open. Three now have a named next action here instead of a wait.
- No product code, schema, test or script changed.

## 2026-09-16 — The evidence chain is shared by design; the authorization decision is what must name the tenant (`SIGNOFF-REPAIR.11.14`)

✅ **The decision on twelve tenant-less tables is taken — and taking it found a live cross-tenant enumeration path.**

- ⭐ **The measurement changed the question:** `migrations/0023:21` declares `UNIQUE (original_locator, expected_digest)`, so two tenants citing one URL share one row by construction. The evidence chain is content-addressed by design; a `tenant_id` column would break the dedupe the digest exists for.
- ⭐ **§16.8 asks for tenant id in every *authorization decision*, not every table** — and that is exactly the layer where it is absent.
- **Verdict: none of the twelve gains a column.** Three gain a tenant-bound read, the six `evaluation_*` gain a site-operator grant gate, `deployment_targets` is operator infrastructure, two defer to `.6.1.5` by name. Record: `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`.
- 🔴 **The live path:** `snapshots::stale` carries no tenant predicate and `GET /v1/snapshots/stale` admits on enrolment alone, returning every tenant's `original_locator`, `auth_class` and `provider_receipt`. A disclosure path and an enumeration, not a write path. New owner `.11.14.1`, reproduce-first.
- No product code, schema or test changed.

## 2026-09-16 — Measure the split-coverage gate, decline it, and write the table that finds the defects (`SIGNOFF-REPAIR.11.15`)

🔴 **Two splits had dropped a mechanism their own goal line names. The obvious gate is measured unsound; the practice it would have enforced found two live defects on its first use.**

- **The census:** of 299 leaves, 14 declare a split and **1** carried a mechanism-to-child mapping (`scripts/census_split_coverage.py`, tracked, 6 self-test controls).
- ⛔ **Gate declined, twice over.** Requiring the mapping fires on 13 of 14 — `.11.9`'s rejected shape. Comparing a prose child-count to the real one gives false positives by construction, because most such sentences are about other leaves. Counting mechanisms in a goal line is not mechanizable.
- ⭐ **The method paid immediately:** `.7.2`'s table found `max_time` declared in `GitLimits`, defaulted to 120 s and read by nothing, while R0 enforces the identical field — and three ref-verification clauses owned by the lane with no child. New owners `.7.2.7`, `.7.2.8`.
- 🔴 **The instrument's first number was wrong** and a `grep -c` disagreeing with it is what caught the indented-table blind spot. A second count was a substring artefact (`configu-ratio-n`). Both kept, not edited away.
- Promoted to `TOOLBOX.md`: "Enumerate a goal line's mechanisms against the children a split produces — the act, not the count."
- No product code, schema or test changed.

## 2026-09-16 — Own the browser suite's retained-fixture accumulation (`SIGNOFF-REPAIR.7.3.2.1`)

- **Measured during §8 artifact cleanup, not reported as a worry.** `du -sk target/browser-lifetime-controls/*` returns **12 fixtures, 515,424 KiB** across four sessions. The browser harness retains a fixture on a FAILED run deliberately — the evidence is the point — and nothing retires one.
- ⭐ `.11.4.8`'s `target/ci-browser/` retirement is the shape to copy: keep the receipt, drop the payload any run reproduces. This session applied that precedent by hand to its own four failure artefacts, reclaiming 1.7 GB, and opened the leaf so the rule exists rather than the habit.
- ⛔ Not a locality defect: the fixtures are exclusively created under `target/` on the repository volume. This is accumulation without a retirement rule.
- New owner `.7.3.2.1`, with acceptance. No code changed.

## 2026-09-16 — Every configuration refusal happens before the first mutation (`SIGNOFF-REPAIR.11.12`)

✅ **`rb-server` used to migrate the database and then discover it could not run.**

- **The ordering, which was never a decision:** `sqlx::migrate!` ran before `SecretStore::resolve` and before the bind-address parse, both pure functions of the arguments. Both now run before `PgPool::connect`.
- ⭐ **The control needs no PostgreSQL:** point the server at a port nothing serves, and a boot that reaches the connection reports `PoolTimedOut` while a boot that refuses first reports the configuration it refused.
- 🔴 **The first draft of those controls passed against the defect**, because they asserted on a string the failure never prints. The neutralization is what found it — a control's discriminator has to be the string the failure actually prints.
- **The bind is reported, not refused**, decided separately: `rb-server --host 0.0.0.0` is the supported trusted-LAN profile the book documents, and what bounds Internet exposure is G6/G7 with blockers B1–B3, not a predicate on an argument. The startup line now names its reach instead of printing `(Phase 0 dev profile)` for every bind. Record: `docs/decisions/2026-09-16_rb-server-bind-exposure.md`.
- **Also repaired:** a typo'd `--host` reported `Error: AddrParseError(Socket)` — the debug form, with no argument and no value; it now names both.
- **Verified:** 3 passed in the new boot suite, 112 passed / 1 ignored in the server lib; clippy, fmt, gate, book and links rc=0.

## 2026-09-16 — The R3 pack enforces the deny-policies it advertises (`SIGNOFF-REPAIR.7.3.5`)

🔴 **The resolver registry told every caller the browser pack denied redirects and subresources. It denied neither — proved with a real browser.**

- **Reproduced first:** the page's `<img src="http://0.0.0.0:{port}/…">` was dialed and the origin served it, `left: 1, right: 0` on the origin's own counter, under the pinned Chrome for Testing runtime.
- **Why it was possible:** `resolvers.rs::gated_advertises` publishes `redirect_policy: "deny"` and `subresource_policy: "deny"`, while the worker subscribed to Chrome's request events purely to write the network log.
- **Fix:** the CDP `Fetch` domain pauses every request before it leaves the browser; a document request for a URL the caller asked to navigate to continues, everything else fails with `BlockedByClient`. Each refusal is named on the receipt in `refused_requests`, and the network log still records the attempt.
- 🔴 **Behavioural consequence, stated rather than discovered:** a page that assembles its text from an external stylesheet or script renders less text than in a desktop browser. That is what "deny" means. Changing the advertisement to `allow` was the rejected alternative.
- **The interception task is owned** by `BrowserOwner::intercept`, joined under the same deadline as the other owned tasks and aborted in `Drop`.
- ⭐ **Falsified twice with a real browser**, the second time into the blackout almost-fix, which fails with `net::ERR_BLOCKED_BY_CLIENT` — the control discriminates a policy from a prohibition.
- **Verified:** 18 passed / 0 failed across the browser suite with every pre-existing control unchanged; 111 passed / 1 ignored in the server lib; clippy, fmt, gate, book and links rc=0.

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated twenty times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

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
