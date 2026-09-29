# DEV_NOTES.md

## 2026-09-29 — Verdict target and synthesis sources narrowed, check deferred (`SIGNOFF-REPAIR.11.63`)

`REASONBRAID-REPAIR-0572`.

- 🔍 **Measured:** `target_digest` only written into the event; `SynthesisInput.sources` read nowhere; the target's meaning unsettled (book: a claim; ADR-029: the proposal); 19 fixtures send `sha256:00`.
- ✅ **Narrowed:** `decision-rules.md`, ADR-029/030 dated notes, qualification review; `.11.63.1` deferred. Docs only.

## 2026-09-29 — Reconciler claim narrowed; the out-of-band scan deferred; the lone-channel case pinned quiet (`SIGNOFF-REPAIR.11.61`)

`REASONBRAID-REPAIR-0571`.

- 🔍 **Refuted before building:** alerting on `git.effective` in the `db = None` arm would alarm on every repository in use (the channel is repository-wide); the arm is right, its unreachability is the gap. Control added; that mutant caught.
- ✅ **Narrowed:** `publication-store.md` row + paragraph (no record → not looked for; manifests unsigned), qualification review; `.11.61.1` deferred (trigger: a second writer, or Internet exposure).

## 2026-09-29 — RX narrowed: call published, answer received nowhere; `original_not_inspected` required; six-shape round trip (`SIGNOFF-REPAIR.11.59`)

`REASONBRAID-REPAIR-0570`.

- 🔴 **RED:** the "every shape" test covered 3/6; an omitted flag read `false`.
- ✅ **Fix:** book (`roadmap.md`, `adapter-boundary.md`, qualification review), G4 record deferral #2 and PHASE-4 `.5` corrected; `#[serde(default)]` removed; exhaustive-match completeness; `.11.59.1` deferred (trigger: first `AcquisitionAnswer` consumer). profiles 102 green.

## 2026-09-29 — `record_decision` is one transaction holding its proposal; proposal and decision inserts map only a unique violation to Duplicate (`SIGNOFF-REPAIR.11.64`)

`REASONBRAID-REPAIR-0569`.

- 🔴 **Measured on unchanged code** (leg 9, assertions turned into observations for one run, restored by sha): 9a 500 but 1 decision left; 9b a CHECK(false) → `400 "decision … already exists"`; 9c two concurrent → 200/200, 2 decisions. RED rc=101 at 9a.
- ✅ **Fix:** `FOR UPDATE` read, insert + `AND status = 'draft'` update on the tx, commit; `taken_or_storage` for both inserts. Hand mutants (SQL-in-strings): unlocked read, insert outside the tx, both mappings, no commit caught; the unconditioned update survives, as predicted (the lock serializes).

## 2026-09-29 — Unknown token counts settle at the hold on the completed path; LocalBudget stated per process (`SIGNOFF-REPAIR.11.62`)

`REASONBRAID-REPAIR-0568`.

- 🔴 **Root cause:** server `as_u64()` → `None` → settled `null` → held sum adds nothing; node `as u64` wraps a negative and a `None` adds nothing. RED: node `(Some(0), Some(0))` vs 5,000; server `null` vs 2,000.
- ✅ **Fix:** core `unknown_charged_at`; node `completed_charge` (`try_from`) on both completed paths; server `settle_charging_in_tx` + `settle_completed_in_tx` from the ledger row; known failure untouched (control). cargo-mutants 2 caught + 2 unviable; 5/5 hand mutants on `.or`/cast/settle lines.

## 2026-09-29 — `projections::load` is typed and verifies its bytes; the publish verb's store faults are 500 (`SIGNOFF-REPAIR.11.60`)

`REASONBRAID-REPAIR-0567`.

- 🔴 **Root cause:** `load` mapped every query error and a missing row to `Duplicate`, `.expect`ed the unrepresentable list, dropped a bad resolved set, and never hashed `bytes`; publish mapped all its store errors to `400`. RED: 2 controls (missing → "already exists"; renamed table → 400).
- ✅ **Fix:** `NotFound`/`Corrupt` variants, hash by the writer's `digest_sha256_hex`, `projection_refusal` (500 / 409 `publication_conflict`), `publication_refusal` on the verb's other store calls. Two legs changed and recorded. 8/10 hand mutants caught; the 2 missed (`publications::load`, `mark_effective`) unreachable live, predicted.

## 2026-09-29 — Tranche 6c: `.11.64` opened; tranche 6 closed (`SIGNOFF-REPAIR.11.9.1.5.3`)

`REASONBRAID-DOC-0200`.

- 🔍 **6c:** 9 clauses: 5 handled (.7.4.3, .7.4.4, .11.55.1, REPAIR-0398, REPAIR-0397), 1 attach (`.11.4`: `threads.rs` header still says auto-accept), 1 unowned → `.11.64` (class 2: `record_decision` autocommit insert + unconditioned update, no uniqueness on `proposal_id`, any insert error → `Duplicate`), 2 none. 12 records left (tranche 7).
- ✅ **Tranche 6 closed:** 48 clauses, six leaves; frontier → `.11.60`.

## 2026-09-29 — Tranche 6b: 19 clauses, `.11.63` opened, two attaches; broad run green (`SIGNOFF-REPAIR.11.9.1.5.2`)

`REASONBRAID-DOC-0199`.

- 🔍 **6b:** 6 handled, 5 owned (`.8.1`: challenge tracking, expired re-invite), 2 attach (`.8.1` reviser not bound to author; `.11.4` node_enrollment comment "second token unissuable"), 2 unowned → `.11.63` (verdict `target_digest` and synthesis `sources` unchecked), 2 declined, 2 none. `--classified` clean, 15 left.
- ✅ **Broad run at REPAIR-0565:** 51 suites, 581 passed, 0 failed, demo green (`target/r11_57/broad_0565.log`).

## 2026-09-29 — Tranche 6 sized on text and split; 6a opens .11.59–.11.62; tranche 5's sizes corrected (`SIGNOFF-REPAIR.11.9.1.5.1`)

`REASONBRAID-DOC-0198`.

- 🔍 **6a:** 20 clauses: 6 handled, 1 owned (`.8.1`), 9 unowned, 2 declined, 2 none; `--classified` clean, 21 records left. New: `.11.60` (class 2: `projections::load` maps any error to `Duplicate`, bundle never hashed before write), `.11.62` (class 2+3: unknown/negative usage settles as `None` = nothing spent; `LocalBudget` in memory), `.11.59` (class 3: RX answer consumed nowhere), `.11.61` (class 3: no-record row unreachable).
- ⛔ **Own error:** DOC-0194's sizes were `len(str(record))` via a silent fallback. Calibrated measure reproduces 1,405: tranche 5 is 7,299 (3,757/2,168/1,374). Also fixed in passing: the resource row called `.7.1.5` open and blocking (closed REPAIR-0499).

## 2026-09-29 — LIVE_STATUS's table names no leaf, and PLAN-STATES-TARGETS refuses one there (`SIGNOFF-REPAIR.11.44`)

`REASONBRAID-REPAIR-0566`.

- 🔴 **Census:** 9 blocking leaves open, all under `.11`; the table was false in 8 rows (the *Corrective review* log and the Phase 0–6 "repairs are `.3`–`.4`" rows). Gate RED on the old table: 9 rows.
- ✅ **Fix:** the table states standing facts with derived pointers; the gate reads `## Current status` to the next `## `, refuses a missing heading, and gains backticked one-level paths. 5/5 hand mutants caught. Book `roadmap.md` states the rule for both. Opened `.11.58` (deferred): 3 stale `active` parents.

## 2026-09-29 — A self-test's real-process arm tolerates a listed pid that exits before its probe (`SIGNOFF-REPAIR.11.57`)

`REASONBRAID-REPAIR-0565`.

- 🔴 **Root cause:** `census_retained_fixtures.py --self-test` took the first root pid from `ps -axo pid=,uid=` and failed if signal 0 read it absent. The hook hit it on pid 419, a short-lived root process from after the pid wrap, and REPAIR-0564 was refused once. RED by injection reproduces the hook's exact message.
- ✅ **Fix:** `foreign_liveness_check` — absent is a failure only if `ps -o pid= -p` still lists the pid after the probe; else try the next candidate; none left → NOTE. Four injected arms (race, liar, no survivor, the second look itself). 6/7 hand mutants caught; the real arm's `append` is unreachable without a lying kernel.

## 2026-09-28 — The network view names roles by their ids; the book stops calling it a pseudonym (`SIGNOFF-REPAIR.11.46`)

`REASONBRAID-REPAIR-0564`.

- 🔍 **Measured:** a foreign tenant holding a role id: invite 400, presence 404, profile 200 at the network view, card 403 — no reach beyond the listing, so class 3 only. The profile leg's unfiltered mutant caught.
- ✅ **Narrowed:** "network pseudonym" → "network view" in the book, code and test comments; `.11.46.1` deferred (per-reader pseudonyms).

## 2026-09-28 — R2 reads RSS 2.0; a nested archive is known by its content (`SIGNOFF-REPAIR.11.54`)

`REASONBRAID-REPAIR-0563`.

- 🔴 **Root cause:** every feed → `atom_syndication` (refuses `<rss>`); `is_archive_name` by extension only. RED: 2 worker controls.
- ✅ **Fix:** root-element dispatch + `extract_rss` on quick-xml 0.41 (entities arrive as `GeneralRef`; predefined + char refs resolved); `is_archive_bytes` (zip×3, gzip, tar, bzip2 with block magic, xz, 7z) checked before the ratio brake. `cargo mutants`: 6 missed in the signatures → a table test → 13/13 caught.

## 2026-09-28 — Retention class is one of three; tombstones carry the given time (`SIGNOFF-REPAIR.11.53`)

`REASONBRAID-REPAIR-0562`.

- 🔴 **Root cause:** free-text `retention_class`; sweep matched two; `deleted_at = now()`. RED: 4 failures.
- ✅ **Fix:** submit refusal + `migrations/0117` (convert unknown → standard, VALID CHECK); `expire_due`/`tombstone_in` take `at`. ⚠️ First draft used `NOT VALID` + a widened sweep: an UPDATE still checks a NOT VALID constraint, so one legacy row fails the whole sweep — caught by the control. 5/5 mutants caught.

## 2026-09-28 — `make showcase`: live system, answering agents, page, status CLI, feedback file (`SHOWCASE.1`)

`REASONBRAID-SHOWCASE-0001`.

- ✅ `scripts/showcase.py` (`up` | `status` | `--self-test`): ephemeral PG under `target/showcase/`, rb-server :4310, two fake-adapter rb-nodes, loopback page :4320 (ask via the CLI, derived progress, evidence links, feedback → `target/showcase/feedback.jsonl`). Skips its cargo build while another runs.
- 🔴 **Its own first run failed:** tokens bound to `showcase-host`, nodes presenting `dev-host` → agents exited unseen; now `--host-claim` matches, the start waits for "enrolled", the page shows agent state. ⚠️ SIGINT from a non-interactive background start is inherited ignored; the script installs `default_int_handler`.

## 2026-09-28 — A replay of an earlier incarnation's work needs the possible-duplicate authorization (`SIGNOFF-REPAIR.11.52`)

`REASONBRAID-REPAIR-0561`.

- 🔴 **Root cause:** the duplicate path admitted only `retry_requires_authorization` rows, a report a lost journal cannot send; RED live: plain replay `200`, provider invoked on both machines.
- ✅ **Fix:** `possibly_run_by_an_earlier_incarnation_in_tx` (`offered_at` < current incarnation's `valid_from`); plain replay `409`; duplicate path admits it and holds the original reservation. R2–R4 caught; R1 survived and removed the "no result" clause (a returned result is a certain re-run).

## 2026-09-28 — The call's initiator is re-authorized at close (`SIGNOFF-REPAIR.11.47`)

`REASONBRAID-REPAIR-0560`.

- 🔴 **Root cause:** `close_call`'s `is_initiator` was a string comparison; RED live: grant revoked, close `200`.
- ✅ **Fix:** named AND `authorize_guarded(ThreadInvite, thread)`. ⚠️ The first non-initiator leg did not discriminate (the joiner lacked `thread_invite`); with the grant seeded, the drop-the-name mutant is caught.

## 2026-09-28 — Imported claims land self_asserted; the schema control sees its rung (`SIGNOFF-REPAIR.11.45`)

`REASONBRAID-REPAIR-0559`.

- 🔴 **Root cause:** `profile_admin` wrote `card.profile` verbatim (RED: forged `certified` landed); the schema control sent `sha256:00000000`, so it passed with the rung deleted (measured, 8 passed).
- ✅ **Fix:** cap every imported claim at `SelfAsserted`; recomputed digest + asserted message in the control. Mutants S and C2 caught live. "byte-identical regeneration" and "origin's own records" claims corrected in `cards.rs`, `receipts.rs`, `profile_admin.rs`, `profiles.md`.
- ✅ **Batch broad run** at `f66ff90e`: 577 passed.

## 2026-09-28 — Runs name the incarnation at fold time; the book narrowed (`SIGNOFF-REPAIR.11.48`)

`REASONBRAID-REPAIR-0558`.

- 🔍 **Measured:** `a_run_names_the_incarnation_current_when_its_result_folds` — the run names the post-offer incarnation. `offered_at` is write-once and a result carries only `attempt_id`, so no server-side anchor is right; `.11.48.1` deferred (node reports its incarnation).
- ✅ **Decided and written down:** unusable results are events with no effect (§10.6 receipt semantics); reported usage is trusted as the tenant's own (`budget.md`). Bar revised class 2 → 3; the pin's mutant (oldest incarnation) caught.

## 2026-09-28 — An approval is one transaction holding its proposal (`SIGNOFF-REPAIR.11.55.1`)

`REASONBRAID-REPAIR-0557`.

- 🔴 **Root cause:** `record_approval` read the stage unlocked, then INSERT + UPDATE on the pool; any insert error became `Duplicate`. RED live: two concurrent approvals both `200`; failed stage write left the row; CHECK-refused insert said "already exists".
- ✅ **Fix:** `pool.begin()`, `FOR UPDATE`, every read and write on the tx, `is_unique_violation()` only for `Duplicate`. M1/M3/M4 caught live; M2 (`AND status = 'decided'`) survives, equivalent under the lock. `.11.55.2` deferred: policy verbs write no authorization record.
- 🧹 12 retained failure clusters retired via `census_pg_test_clusters.py --retire --confirm` (642 MB).

## 2026-09-28 — The reconciler judges the head's effective channel (`SIGNOFF-REPAIR.11.56`)

`REASONBRAID-REPAIR-0556`.

- 🔴 **Root cause:** `reconcile`'s `Effective` arm compared only the immutable ref; RED live: moved channel → `Consistent`. ⚠️ The leaf's first text blamed the quiet-pair unit test; it is the superseded case (no superseded state exists) and correct.
- ✅ **Fix:** `expected_channel` for the head only (`git_object_ids[1]`, unless another staged/effective publication in the repository recorded it as `expected_effective`); live legs for moved, deleted, superseded and failed-does-not-supersede. `cargo mutants` on the matrix: 6/6 viable caught; driver hand mutants D1 (never superseded), D2 (failed supersedes) caught live.

## 2026-09-28 — The grant checks require a live boundary (`SIGNOFF-REPAIR.11.55`)

`REASONBRAID-REPAIR-0555`.

- 🔴 **Root cause:** `grant_is_live`/`grant_held_by` read `authority_grants` alone; RED live: approval accepted and recorded under a not-yet-valid and under a revoked boundary.
- ✅ **Fix:** one `LIVE_GRANT_ACTIONS` predicate joining `enrollment_boundaries` (status + window) for all 7 call sites; three refusal texts corrected. H1–H3 hand mutants each caught by its own leg. `authority.md`'s stale "not holding" sentence (settled by REPAIR-0503) corrected.
- 🔴 **Opened:** `.11.55.1` (class 2): `record_approval` is an unlocked read and two pool statements.

## 2026-09-28 — IPv6 literal hops classified; hops keep scheme and port; `classify_v6` allow-lists (`SIGNOFF-REPAIR.11.50`)

`REASONBRAID-REPAIR-0554`.

- 🔴 **Root cause:** `git.rs` parsed `url.host_str()` (IPv6 stays bracketed) as `IpAddr`, so `[::1]` and `[::ffff:127.0.0.1]` hops were followed unclassified; RED live, both `Ok(200)`. `classify_v6` ended `_ => Public`.
- ✅ **Fix:** `literal_ip` on the typed `url.host()` for pre-flight and hops; `hop_leaves_origin` (scheme + `port_or_known_default`); `classify_v6` public only in `2000::/3` minus `2001::/23`, `2001:db8::/32`, `2002::/16`, `3fff::/20`. `cargo mutants --in-diff`: 18/19 viable caught; the `fec0` arm's survivor caught after the control asserted the class `private`.

## 2026-09-28 — Tranche 5c: 20 clauses; grant checks never read the boundary, the reconciler never judges `refs/rb/effective` (`SIGNOFF-REPAIR.11.9.1.4.3`)

`REASONBRAID-DOC-0196`.

- 🔍 **Census:** 7 handled, 7 attach, 1 unowned, 3 declined, 2 none. Tranche 5 closed: 104 clauses, 22 records, `.11.45`–`.11.56` opened.
- 🔴 **Opened:** `.11.55` (class 2/3: `grant_is_live`/`grant_held_by` have no `enrollment_boundaries` join; 7 call sites) and `.11.56` (class 2/4: `reconcile`'s `Effective` arm ignores `git.effective`; `the_consistent_pairs_are_quiet` asserts a moved channel as consistent).
- ⚠️ **Fixing `.9.2`'s stale `active` Status** made `QUALIFICATION-CURRENCY` read its two book rows: the wall-clock one was repaired at REPAIR-0401, and the reconciler one was a false ✅ in DOC-0166's census, corrected beside it.

## 2026-09-28 — Tranche 5b: 33 clauses; retention class unchecked, R2 refuses RSS (`SIGNOFF-REPAIR.11.9.1.4.2`)

`REASONBRAID-DOC-0195`.

- 🔍 **Census:** 7 handled, 7 owned, 13 attach, 5 unowned, 1 none. Attached: the IPv6 hop's record and the dead `numeric_ambiguous` arm to `.11.50`; fetcher latent limits (u8 hops, default port, deflate-as-raw, `text/*` trusted, R1 pre-flight DNS) to `.11.51`; sync `thread::sleep` extraction on the async handler to `.7.3.4`.
- 🔴 **Opened:** `.11.53` (class 3: `retention_class` free text, sweep knows two; `deleted_at = now()`; bytes kept, unstated) and `.11.54` (class 3: `rss+xml` advertised, Atom parser refuses `<rss>`; nested archive by extension only).

## 2026-09-28 — Tranche 5a: 51 clauses; an IPv6 redirect escapes the destination check (`SIGNOFF-REPAIR.11.9.1.4.1`)

`REASONBRAID-DOC-0194`.

- 🔍 **Census:** `.11.9.1.4` split three ways by narrowest candidate (`.5.3` 10 records, `.7.3` 7, `.2.2` 5). 5a: 24 handled, 26 unowned, 1 declined; evidence gathered by three read-only passes, every live clause re-read by hand.
- 🔴 **Opened:** `.11.50` (class 1/3: `git.rs` hop policy parses `host_str()`, bracketed for IPv6, so an IPv6 literal hop is unclassified and hyper-util dials it; `classify_v6` default-public), `.11.45` (cards: claimed confidence verbatim, origin unauthenticated, schema control rung-blind), `.11.46` (presence raw `role_id` vs "network pseudonym"), `.11.47` (close by name), `.11.48` (run → current incarnation; unusable results acked), `.11.52` (lost-machine replay without `allow_possible_duplicate`); deferred `.11.49` (quota denial rows), `.11.51` (R1 file walk).

## 2026-09-28 — The ownership gate sees D, R, migrations and hooks; one code definition (`SIGNOFF-REPAIR.11.40`)

`REASONBRAID-REPAIR-0553`.

- 🔴 **Root cause:** `--diff-filter=ACM` plus a private glob (no `migrations/`, `.githooks/`, `scripts/`), a second copy of `.doctrine/code_paths.txt`. RED in a throwaway index: migration, hook, deletion each rc=0. History: 1007 commits, none used the gap.
- ✅ **Fix:** `check_task_acceptance.sh --code-paths` is the one definition (seam read from the index only; seam and default gain migrations, hooks, Cargo files); both gates `--no-renames`, all statuses, a tree must be a present `docs/tasks/<TREE>.md`; `SPINE_ALLOW_UNOWNED` removed. New `--self-test`, 10/10 hand mutants killed. ⚠️ The self-test cannot see its own `GIT_INDEX_FILE` isolation; an outside control does.
- 🔴 **Opened:** `.11.44` (class 3): `LIVE_STATUS.md`'s *Corrective review* row says work "follows" that closed at REPAIR-0544.

## 2026-09-28 — A replaced recruitment response moves its time (`SIGNOFF-REPAIR.11.42`)

`REASONBRAID-REPAIR-0552`.

- 🔴 **Root cause:** `ON CONFLICT … DO UPDATE SET response_kind, payload` left `created_at`; the inspection's `"at"` is that column. RED: decline then defer, both at `08:50:14.601048`.
- ✅ **Fix:** `created_at = now()` in both upserts; the comment states replacement. Mutant caught live; `mcp_write`/`federation` pass.

## 2026-09-28 — `ROADMAP.md` states targets; `PLAN-STATES-TARGETS` keeps it so (`SIGNOFF-REPAIR.11.43`)

`REASONBRAID-REPAIR-0551`.

- 🔴 **Root cause:** a hand-kept frontier narrative in the frozen plan (13 leaf references, stale since at least `.3.3.4.3.3.3.3.2.3.2`), while the book's roadmap page refused the same copy.
- ✅ **Fix:** stable pointers plus a deviations-go-to-the-qualification-page sentence; `scripts/check_plan_states_targets.py` (reads the index; RED 13 → OK), registered; the two pattern mutants caught.
- ⭐ The director's lockstep rule, mechanised where it can be: a status narrative cannot be kept true by hand, so the gate keeps it out rather than checking it.

## 2026-09-28 — `POST /v1/snapshots` refuses a wrong `byte_length` (`SIGNOFF-REPAIR.11.41`)

`REASONBRAID-REPAIR-0550`.

- 🔴 **Root cause:** `snapshots::submit` verified the digest and stored `byte_length` as declared; a true resubmission REPLAYED onto the wrong row (RED: `21` stored for 20 bytes).
- ✅ **Fix:** `SnapshotError::LengthMismatch` after the digest check, refused 400 naming both. The four server packs already pass the true length. Disabled-check mutant caught live.
- 📋 **Opened:** `.11.43` (the director's lockstep rule: `ROADMAP.md`'s status preamble is weeks stale).

## 2026-09-28 — Tranche 4e: 17 clauses; `byte_length` unchecked, a changed response keeps its first time (`SIGNOFF-REPAIR.11.9.1.3.5`)

`REASONBRAID-DOC-0193`.

- 🔍 **Census:** 10 handled, 5 owned, 2 unowned. Four handled by REPAIR-0545/-0515 from this week.
- 🔴 **Opened:** `.11.41` (class 3/2: `POST /v1/snapshots` stores the declared `byte_length`; `evidence.md` says it is the stored length) and `.11.42` (class 2: `record_response` upsert keeps `created_at`, and the inspection's `"at"` pairs the new answer with the old time).

## 2026-09-28 — The invite checks enrollment and bounds its TTL (`SIGNOFF-REPAIR.11.39`)

`REASONBRAID-REPAIR-0549`.

- 🔴 **Root cause:** `threads.rs` parsed the role id's format only and computed `now + ChronoDuration::seconds(secs)` on the caller's integer: `i64::MAX` panics (`TimeDelta::seconds out of bounds`), and a negative is stored as expired on arrival.
- ✅ **Fix:** `agent_roles` tenant check (one message for elsewhere/nowhere), `1..=INVITATION_TTL_MAX_SECONDS` (365 days). RED live; H1–H3 hand mutants caught. `decline_expiry_and_reinvitation` used `-1` as a shortcut and now waits out a 1-second TTL.
- ⭐ A test built on a defect is part of the defect's reach; changing it is part of the repair, not collateral.

## 2026-09-26 — Tranche 4d: 23 clauses; the ownership gate is blind, the invite trusts its body (`SIGNOFF-REPAIR.11.9.1.3.4`)

`REASONBRAID-DOC-0191`.

- 🔍 **Census:** 17 handled, 3 unowned, 1 owned, 1 declined, 1 none. Two measured live with a throwaway probe (reverted): foreign-role invite `200` / accept `403`+`404`; TTL `i64::MAX` → the request's connection drops.
- 🔴 **Opened:** `.11.40` (class 4: `--diff-filter=ACM` plus a narrow code list; a second copy of `.doctrine/code_paths.txt`) and `.11.39` (deferred; the trigger fired).

## 2026-09-26 — `.8.1.1.6` closed by census: every narrow reading holds (`SIGNOFF-REPAIR.8.1.1.6`)

`REASONBRAID-DOC-0190`.

- ✅ **Census:** no recusal code (`git grep -il recus` → 0); ballots filed under the AUTHENTICATED caller (`crates/reasonbraid-server/src/api.rs:9069`/`:9706`), delegation included; finality refused and controlled (`crates/reasonbraid-server/tests/profiles.rs:18291`). Mechanisms deferred as `.8.1.1.6.1`.
- ⭐ When the book already states the limit, a class-3 leaf asks whether the stated fallback is TRUE, not whether the feature exists.

## 2026-09-26 — `role_weighted`/`human_committee` stay refused until designed (`SIGNOFF-REPAIR.8.1.1.4`)

`REASONBRAID-REPAIR-0548`.

- ✅ **Decided:** the leaf's false claim was already gone (`.8.1.1.2` refuses both at creation). What remained was the decision, recorded; the design is `.8.1.1.4.1`, deferred with a trigger. Both refusals now name the deferral (RED first on the message).
- ⭐ `.12.2`'s instrument found this leaf: its state was only `- Opened: pending`, and my Status-only census had never listed it.

## 2026-09-26 — `census_open_leaves.py` derives the exit count; REPAIR-0546's "no class 1–3 open" was false (`SIGNOFF-REPAIR.12.2`)

`REASONBRAID-REPAIR-0547`.

- 🔴 **Root cause:** a hand count from `- Status:` lines misses 13 open leaves recorded as `- Opened: \`pending\``, 7 of them blocking class 3. My MEMORY census used that method, and REPAIR-0546 published the false sentence.
- ✅ **Fix:** the instrument (three state forms, bar classes, `--check` refusing an unbarred open leaf) in the project gate; 4 blocked leaves classified deferred on their named triggers.
- ⭐ Its first version treated a shell `# comment` inside a code fence as a heading and read three closed leaves as open. Reading the closing commits before "repairing" the records caught it, and repairing them would have produced a second Status line. Verify before acting on an instrument's first output.

## 2026-09-26 — `SemanticLosses` loses its `Default`; `.6.3` closes (`SIGNOFF-REPAIR.6.3.1`)

`REASONBRAID-REPAIR-0546`.

- 🔴 **Root cause:** `#[derive(Default)]` on a loss record means "nothing lost"; both mappers used it, and two tests asserted `== SemanticLosses::default()` under messages saying all five were lost. `response_message` never set `task_id`.
- ✅ **Fix:** no `Default`, `SemanticLosses::ALL_LOST`, and `task_id` set. RED 4/5; `cargo mutants` made only 3 unviable mutants, so 4 hand mutants cover the literals.
- ⭐ A test compared against the value under test (`== SemanticLosses::default()`) cannot fail when that value is wrong. The fix spells the expected record field by field in the test.

## 2026-09-26 — `.6.3` censused: the A2A facade's loss record lies, and so does its test (`SIGNOFF-REPAIR.6.3`)

`REASONBRAID-DOC-0187`.

- 🔍 **Census:** `reasonbraid-a2a` has no consumer in the workspace. `map_message`/`map_task_request` record `SemanticLosses::default()` (all false) against docs saying all five are lost, and the test asserts that default under the message "all five dimensions lost". `response_message` drops `task_id`.
- 📋 **Opened:** `.6.3.1` (blocking class 4/3) and `.6.3.2` (the build half, deferred until a product surface accepts A2A).

## 2026-09-26 — The freshness horizon moves to the citation; a replay refreshes it (`SIGNOFF-REPAIR.7.4.9`)

`REASONBRAID-REPAIR-0545`.

- 🔴 **Root cause:** `snapshots::replay` wrote `refreshed_at` only, and `fresh_until` lived on the SHARED `evidence_snapshots` row, so the first acquirer decided every tenant's stale list. The replay also discarded its refresh error.
- ✅ **Fix:** migration 0116 moves the horizon to `evidence_citations` (backfilled, shared column dropped); `record_citation` upserts it, and the reads take the reader's. RED 4/5, four live hand mutants, and an upgrade control.
- ⭐ The bound-instants census had judged `evidence_snapshots.fresh_until` "written and never read back", yet the stale list had always read it: the right verdict for a false reason. A reason is a claim too, and only moving the site made anyone read it again.

## 2026-09-26 — A real SIGKILL between the CLI's writes, qualified; `.3.3.4.3` closes (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.3.3`)

`REASONBRAID-REPAIR-0544`.

- ✅ **Qualified:** `kill_point::reached`, a `debug_assertions`-only self-SIGKILL at `publish:<checkpoint>#n` and `bootstrap:{pending,outcome}-published`. Seven cases, every recovery exact, no repair needed. Release carries no hook (`strings target/release/rb` → 0).
- 🔴 **Survivor:** `bootstrap_flow.rs:108:58 && → ||` (resume matching after cleanup) was uncovered, because every resume control ran while a request was pending. Now controlled.
- ⭐ A structural cascade closed five leaves up to `.3.3.4.3`. No class-2 blocking leaf remains open.

## 2026-09-26 — Abrupt writer death with a surviving descendant, qualified; the lock refusal corrected (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.3.2`)

`REASONBRAID-REPAIR-0543`.

- ✅ **Qualified:** the test binary re-executes itself as an embedded writer (an `#[ignore]`d test gated by an env var), hands its lock description to a python descendant (witnessed by dev:ino), and is SIGKILLed. Successors refuse in under 1 s, bytes unchanged, then proceed once the descendant exits; the pending key survives.
- 🔴 **Fixed:** the refusal told the operator to "retry after it finishes" when no writer exists. It now names "a writer, or a process a writer started".
- ⭐ Two harness lessons. A release signalled by a file inside a fixture directory that is removed right after needs an acknowledgment, or the descendant misses it (three orphans, ended by verified path). And `pgrep -f "os.fstat(2)"` never matches: the parentheses are an ERE group.

## 2026-09-26 — Keyed bootstrap recovery qualified across a server restart (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.3.1`)

`REASONBRAID-REPAIR-0542`.

- ✅ **Qualified:** `bootstrap_recovery_survives_a_server_restart`: outage resend fails and keeps the key; a fresh `ApiState` on the same pool and address replays the original IDs; counts `(1, 1, 2)` unchanged. Server and harness hand mutants each caught.
- ⭐ My own limit text one commit earlier said filesystem failure was untested; `publication_failures_keep_complete_snapshots_and_recover_reserved_work` injects faults at all five replacement checkpoints. A census that lists controls by name is what caught it; a limit written from memory drifted within one commit.

## 2026-09-26 — `.3` censused: 6 of 9 clauses held; restart, inherited descriptor and mid-publication kills open (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.3`)

`REASONBRAID-DOC-0186`.

- 🔍 **Census:** each clause mapped to named controls. Three residuals: a server restart (measured only within one process), abrupt death with a surviving descendant (only normal release controlled), and a real kill between publications (held only by composition).
- 📋 **Opened:** `.3.1`–`.3.3`. `.3.3` needs a deterministic pause where nothing external happens; the candidate is a `debug_assertions`-only failpoint.

## 2026-09-26 — The CLI chapters describe the recovery the CLI has; `.2.4` and `.2` close (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.4.2`)

`REASONBRAID-REPAIR-0541`.

- 🔴 **Root cause:** five book statements outlived their repairs (REPAIR-0031, -0101, `.9.3.4.1`), and nothing bound the chapter to the code. `git grep -ln "cli-bootstrap-state" -- crates` was empty.
- ✅ **Fix:** present-tense rewrite, examples checked against the printing code, and `state_store::book_examples`, which `include_str!`s the chapter and decodes each JSON block through `codec::decode` / `bootstrap_flow::decode_outcome`. An unknown shape fails. Two falsifications bite.
- ⭐ The interruption limit was about to be written as "not tested", which is false in the pessimistic direction: two kill controls exist. A limit is a measurement too.

## 2026-09-26 — The ordinary CLI writers bind the reply to the request (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.4.1`)

`REASONBRAID-REPAIR-0540`.

- 🔴 **Root cause:** `run_enroll_with_recovery` (ordinary branch) and `run_thread_create_in_store` read the reply with `unwrap_or_default()`; the store validates shape, never the binding. RED 8/8: 3 recorded silently, 5 refused as LOCAL state after a 200.
- ✅ **Fix:** `enrollment_binding` (kind, name, canonical principal of that kind, canonical tenant equal to the one asked for) and a canonical `thread_id`, both checked before `state_mut()`; `malformed server response`. 13/14 mutants caught via `cargo mutants --in-diff … --in-place`, 1 unviable.
- ⭐ `cargo mutants --in-diff <git diff>` scopes mutants to the leaf's own lines, and `--in-place` reuses the warm target dir (6 s baseline build). Both suit a leaf whose change is a few functions in a large file.

## 2026-09-26 — `.2.4` censused: no obsolete recovery path; two ordinary writers take the reply on trust (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.4`)

`REASONBRAID-DOC-0185`.

- 🔍 **Census:** 3 request consumers, 3 state writers (4 open sites), 0 dead or deprecated CLI paths. `StateFile::save` has no production caller. The unkeyed server bootstrap is live (load harness), so it is not obsolete.
- 🔴 **Opened:** `.2.4.1` (class 2): `crates/reasonbraid-cli/src/lib.rs:683/687/857` record `principal_id`/`tenant_id`/`thread_id` via `unwrap_or_default()` and never compare the tenant with `--tenant`. `.2.4.2` (class 3): `cli.md` says keyed recovery is "not implemented yet"; it has been since REPAIR-0031. `.11.38` (deferred): 19 display-only defaults.
- ⭐ DOC-0162 classified this leaf from its Owns line ("obsolete recovery paths … remain"). Measured, that half was empty and the real residue was elsewhere. A classification made from a leaf's wording is a hypothesis until its census.

## 2026-09-26 — Every malformed request now gets a reason code (`SIGNOFF-REPAIR.11.36`)

`REASONBRAID-REPAIR-0539`.

- 🔴 **Root cause:** 52 handlers destructured axum's `Json<T>`, whose rejection is plain text with no `code`.
- ✅ **Fix:** `api::ApiJson` / `node_channel::NodeJson` extractors over one `api::json_rejection` (axum's status kept, `{code: invalid_command, message}`); a source guard over every file; `json_body`'s lone `400` becomes `422`.
- ⭐ My first cut flattened `422` to `400`, reversing `.9.2.1.2.3`'s recorded two-level convention — the knowledge note that already records this mistake. The corpus refused it at once.

## 2026-09-26 — Tests no longer leave empty folders behind, and the scripts review is complete (`SIGNOFF-REPAIR.11.3.6`)

`REASONBRAID-REPAIR-0538`.

- 🔴 **Root cause:** a hand-made `DirBuilder` directory, only its dump removed on success (`remove_file(…).ok()`).
- ✅ **Fix:** a `Fixture` (removed on pass, retained on failure) plus an assertion that the directory is gone after the drop; 46 empty leftovers removed with an empty-only `find … -empty -delete`.
- ✅ `.11.3` CLOSED with a closing census.

## 2026-09-26 — The demonstration's "this did not happen" checks can no longer pass by accident (`SIGNOFF-REPAIR.11.3.7`)

`REASONBRAID-REPAIR-0537`.

- 🔴 **Root cause:** `! cmd | grep -q x` judges only grep; `curl -s > file` ignores the status; `probe_poll` accepted any answer.
- ✅ **Fix:** `scripts/lib/demo_checks.sh`: `absent` (command must succeed with output) and `fetch` (2xx or fail, body kept); 3 negative checks, 12 captures and the poll probe converted; a shape guard keeps the old forms out.
- ⭐ A mutant ignoring the command's status survived: the test's failing command was silent, so the no-output rule masked it. A failing-but-printing command is now its own control.

## 2026-09-26 — The load test measures exactly what it was asked to (`SIGNOFF-REPAIR.11.3.4`)

`REASONBRAID-REPAIR-0536`.

- 🔴 **Root cause:** a rounded-up per-worker count and no validation; no curl timeout; `wait $PIDS`; percentiles over every line; fixed port/output, any-listener readiness, relative bin root, unreaped server.
- ✅ **Fix:** `scripts/load_summary.py` (unit-tested summary); exact split; `--timeout`; per-pid waits; own-server readiness; per-run output; URL off argv; setup requires 200 and ids. Closing census of `.11.3`: argv and readiness guards over all five scripts.
- ⭐ My mutant driver hung 5 h 47 min: `pg_ctl start` with captured output leaves the pipe with the postmaster. Log to a file, bound the call, check long jobs.

## 2026-09-26 — The development environment checks its own server and cleans up honestly (`SIGNOFF-REPAIR.11.3.3`)

`REASONBRAID-REPAIR-0535`.

- 🔴 **Root cause:** any-listener `curl` readiness; `pg_ctl stop || true; rm -rf`; no `cd "$ROOT"` before a bare `cargo`; a fixed CLI-state path; a residue census over every run's directory.
- ✅ **Fix:** `server_ready` (own pid + own startup line + answering); `teardown` removes only when no server runs (status → stop → status), reports and exits 1 otherwise; `cd "$ROOT"` + `project_env.py`; per-run CLI state; URL off argv.
- ⭐ Scenarios first shown holding on the unmodified script; 5 of 5 mutants caught. My harness stalled once on `kill -INT` to a background job (bash ignores SIGINT there); SIGTERM now.

## 2026-09-26 — The demonstration checks it is talking to its own server, and its two-host mode works (`SIGNOFF-REPAIR.11.3.5`)

`REASONBRAID-REPAIR-0534`.

- 🔴 **Root cause:** `curl -s -o /dev/null` readiness (any listener); `env.txt` echoed `$DATABASE_URL`; URL on `rb-server`/`psql` argv; ids pasted into SQL; remote words single-quoted. Remote mode had never run: `~` unexpanded, `./rb-node` one level too deep, `--fake-script` unquoted, the backgrounded `mkdir` pid race, a local certificate check.
- ✅ **Fix:** `server_ready <n>` (own pid + n-th post-bind startup line + port answers, else stop); `redact-env`; libpq environment; `lease_field` binds via psql variables on stdin; `remote_quote` (`%q`, leading `~/` kept); positional probe args; `node_files_present`; deterministic reaping (`read` not `$(cat)` after a kill).
- ⭐ Harnesses: a decoy listener, a stand-in ssh/scp with a separate home. 34/34 local and remote; 7 of 7 mutants caught (one judge written inverted, corrected).

## 2026-09-26 — A backup is never half-written and never readable by others (`SIGNOFF-REPAIR.11.3.1`)

`REASONBRAID-REPAIR-0533`.

- 🔴 **Root cause:** `pg_dump --file "$FILE"` wrote the final name directly, under the caller's umask, with the URL as its connection argument.
- ✅ **Fix:** `umask 077`; dump to a `mktemp` name, trap-removed on failure, `ln` into place (refuses an existing name); libpq environment via `pg-env`; `write-backup` by variable name.
- ⭐ Controls use a stand-in `pg_dump` on PATH that records argv and environment; the permission test was blind until the child got a permissive umask. 7 of 7 mutants caught.

## 2026-09-26 — The restore test can no longer overwrite the live database (`SIGNOFF-REPAIR.11.3.2`)

`REASONBRAID-REPAIR-0532`.

- 🔴 **Root cause:** `restore.sh` gave `RESTORE_DATABASE_URL` to `pg_restore --clean` after checking only the dump; the URL rode argv.
- ✅ **Fix:** `guard-restore-target` (live host/port/db refused), a zero-relations emptiness check, and `backup_receipt.py pg-env`/`pg-names` exporting libpq's variables (fail closed on anything libpq cannot carry). No override. `redact` no longer raises on a bad port.
- ⭐ Controls: a populated target and the live database refused; a hostile ambient `PGSSLMODE`/`PGPASSWORD` passes only because they are cleared; a source guard keeps URL variables off `restore.sh`'s command lines. 7 of 7 mutants caught.

## 2026-09-26 — The operational scripts reviewed: six problems to fix (`SIGNOFF-REPAIR.11.3`)

`REASONBRAID-DOC-0183`.

- 🔎 **Census:** 6 artifacts (backup.sh, restore.sh, backup_receipt.py, dev.sh, load_harness.sh, demo_two_host.sh) + `tests/backup_restore.rs`; every goal-line and attached clause mapped to lines. Split `.11.3.1`–`.6` by script.
- 🔴 Live: `restore.sh` `--clean`s any target; the DB URL is argv everywhere and verbatim in the demo's `evidence/env.txt`; `backup.sh` writes the final name directly under the ambient umask; four readiness probes accept any listener on the port; `PER_WORKER` rounds up; 34 residue directories from the backup test.

## 2026-09-26 — The review page no longer lists problems that were already fixed (`SIGNOFF-REPAIR.11.37`)

`REASONBRAID-REPAIR-0531`.

- 🔴 **Root cause:** nothing compared `qualification-review.md`'s owner cells with the tree's statuses; 24 open rows had only `done` owners.
- ✅ **Fix:** each row read against its owner's record and the code (21 repaired, 1 refuted, 1 repaired+decided, 1 partial → `.4.4.11` deferred); `QUALIFICATION-CURRENCY` (`scripts/check_qualification_currency.py`, project slot, 0.03 s, `--self-test`) refuses an open row whose owners are all done or name no leaf.
- ⭐ The first hand census said 11: its regex read a leaf's Status only within four lines of the heading and silently skipped every other leaf.

## 2026-09-26 — Deployment records no longer blame database failures on the caller (`SIGNOFF-REPAIR.9.3.3.6`)

`REASONBRAID-REPAIR-0530`.

- 🔴 **Root cause:** 10 sites in `deployments.rs`/`corrections.rs` discarded the `sqlx::Error` (`map_err(|_| …)`, `Err(_) => Duplicate`, `.is_err()`), and the three correction handlers answered everything `400`; an unparseable expiry was `MissingExpiry`.
- ✅ **Fix:** `write_failure` (only `is_unique_violation()` is a duplicate) and `storage` (unrepresentable input vs store fault) in both enums; `CorrectionError::{Storage, UnrepresentableInput, MalformedExpiry}`; `api::correction_refusal`. Source guard `deployments::store_fault_classification` refuses the three discarding shapes in both modules. 12 of 12 viable mutants caught; the guard caught a planted line.
- ✅ `.9.3.3` CLOSED with a closing census (`.2.1` and `.4.1` deferred with triggers).

## 2026-09-26 — A drift record compares against what was actually assigned (`SIGNOFF-REPAIR.9.3.3.5`)

`REASONBRAID-REPAIR-0529`.

- 🔴 **Root cause:** `record_drift` checked the assignment's existence and stored the caller's `desired_digest` verbatim.
- ✅ **Fix:** the existence read returns the assignment's `desired_digest`; `CorrectionError::DesiredDigestNotAssigned` refuses a mismatch by name. Six fixtures re-seeded from `desired_pair`.
- ⭐ A mutant unbinding the lookup from the publication survived: no control had ever asserted the assignment is the (target, publication) pair. Added; 4 of 4 caught.

## 2026-09-26 — Only a target's own authority can decide what it runs (`SIGNOFF-REPAIR.9.3.3.7`)

`REASONBRAID-REPAIR-0528`.

- 🔴 **Root cause:** `assign` asked whether the target existed, never whose it was; the tenant binding is on the publication and targets are site-wide.
- ✅ **Fix:** `assign` takes the principal and asks `grant_held_by(target.owning_authority, principal, DeploymentTargetRegister)` before any publication lookup; `NotTargetAuthority` → `403 unauthorized` via `deployment_refusal`; the target lookup classifies store faults. Decision record added. 7 of 7 viable mutants caught, including the check's order and verb.
- 🔎 The qualification review's *citing an authority* row was stale since REPAIR-0184/0343; corrected. A scan finds 11 candidate rows owned only by done leaves: `.11.37` (row + gate).

## 2026-09-26 — The book no longer implies deployments roll out in stages (`SIGNOFF-REPAIR.9.3.3.4`)

`REASONBRAID-DOC-0182`.

- 🔎 **Census:** `wave` is declared, bound and selected back, never compared; nothing acts on `deployment_assignments` beyond recording and reading.
- ✅ **Decided:** a LABEL (`docs/decisions/2026-09-26_the-deployment-wave-is-a-label.md`); the sequencer is `.9.3.3.4.1`, deferred until the first code path acts on an assignment. The book's *canary wave* became *wave* with the limit stated.
- 🔎 Found `R-33-35-1` clause 2 (`assign` checks no authority over the target) attached to `.9.3` and owned by no child: opened `.9.3.3.7` (blocking, class 2).

## 2026-09-26 — A target's reports are kept, not overwritten (`SIGNOFF-REPAIR.9.3.3.3`)

`REASONBRAID-REPAIR-0527`.

- 🔴 **Root cause:** `record_receipt` overwrote `deployment_assignments.observed_*` in place, outside a transaction; there was nowhere to keep a second receipt.
- ✅ **Fix:** `migrations/0115` `deployment_receipts` (tenant copied from the publication, `BEFORE UPDATE` trigger refuses rewrites, FK to the assignment); `record_receipt` is one transaction under `FOR UPDATE OF a` (insert the row, `UPDATE … RETURNING` the pair; `load_assignment` retired); `GET /v1/deployments/{t}/{p}/receipts`; `DeploymentError::storage` + `UnrepresentableInput`.
- ⭐ The first ordering control was blind: without the lock the receipt queued on its INSERT's FK share lock, not on a statement naming the assignment. Watch widened; the lock mutant now fails on the ordering assertion. 13 of 13 viable mutants caught.

## 2026-09-25 — Only the principal a target names can report what it runs (`SIGNOFF-REPAIR.9.3.3.2`)

`REASONBRAID-REPAIR-0526`.

- 🔴 **Root cause:** `record_receipt`'s only gate was the tenant join, and `deployment_targets` had no column naming a reporter.
- ✅ **Fix:** `migrations/0114` adds `deployment_targets.reporter`; `TargetInput.reporter` is required, parsed by the new shared `api::parse_principal` and required enrolled AFTER the authority check (no enrolment oracle); `record_receipt` reads the reporter in its tenant join and refuses `NoReporter` / `NotTheReporter` by name. `DeploymentError::Storage` + `api::deployment_refusal` make the reporter lookup's store failure a `500`. Decision record added; renaming/replacing a reporter deferred (`.9.3.3.2.1`).
- ⭐ RED first live; 9 of 9 viable mutants caught (1 of the tool's unviable), including the check ORDER and the store-fault arm (withholding `agent_roles` for one request).

## 2026-09-25 — A deployment must deploy what its publication published (`SIGNOFF-REPAIR.9.3.3.1`)

`REASONBRAID-REPAIR-0525`.

- 🔴 **Root cause:** `deployments::assign` validated `desired_digest`'s shape only and never read the projection digest or `git_object_ids`; ADR-021 defines the desired pair as the publication's.
- ✅ **Fix:** the existing publication read LEFT JOINs `policy_projections`; `DeploymentError::{DesiredDigestNotProjection, DesiredRefNotRecorded}` refuse by name. Six fixture sites re-seeded through a `desired_pair` test helper, not relaxed. RED first live; 5 of 5 mutants caught (3 from `cargo mutants --list`, 2 hand SQL).
- 🔎 Opened `.9.3.3.5` (`record_drift` stores a caller-declared desired digest) and `.9.3.3.6` (14 store sites in `deployments.rs`/`corrections.rs` answered as the caller's).

## 2026-09-25 — A commit check no longer mistakes a setting's name for a waiver (`SIGNOFF-REPAIR.11.2.10`)

`REASONBRAID-REPAIR-0524`.

- 🔴 **Root cause:** `WAIVER_RE`'s `[A-Z][A-Z0-9_]*_WAIVER` had no right boundary, so `REPEATED_WAIVER_THRESHOLD` matched and WAIVER-ROUTING refused REPAIR-0523.
- ✅ **Fix:** `…_WAIVER([^A-Z0-9_]|$)`, and the gate's first `--self-test` (5 must-match, 5 must-not-match, 3 owner verdicts), discovered and run by `check_self_tests.sh`. RED against the old pattern; the drop-the-alternative mutant caught.

## 2026-09-25 — A policy can be reviewed more than once (`SIGNOFF-REPAIR.9.3.2`)

`REASONBRAID-REPAIR-0523`.

- 🔴 **Root cause:** `review_id = rev_{publication}_{trigger}` was the primary key, so the second review of a pair collided and `inserted.is_ok()` swallowed it (and every other insert error); the waiver trigger had no count, window or expiry.
- ✅ **Fix:** per-pair latest occurrence vs the pair's latest review; own ids; `policy_reviews_one_due` partial unique index (`migrations/0113`) named in `ON CONFLICT`; `REPEATED_WAIVER_THRESHOLD = 2`, `REPEATED_WAIVER_WINDOW_DAYS = 90`, in force; insert errors propagate. Decision record added.
- ⭐ The old control asserted the defect; rewritten with relative clocks, a backdated waiver and a trigger-forced insert failure. 8 of 8 mutants caught; broad run (migration).

## 2026-09-25 — A database failure is no longer reported as "that record does not exist" (`SIGNOFF-REPAIR.9.2.3`)

`REASONBRAID-REPAIR-0522`.

- 🔴 **Root cause:** 15 lookups (`lifecycle.rs` 11, `publications::stage` 4) were `.map_err(|_| <not-found>)`; `record_approval` answered a `grant_held_by` store error as an invalid proof; one stored-projection decode answered as a missing thread.
- ✅ **Fix:** `LifecycleError::{Storage, UnrepresentableInput, UnknownDecision}` + `LifecycleError::storage`; `api::lifecycle_refusal` (`Storage` → `500`); `publication_refusal` on `stage`; a source guard (`lifecycle::store_fault_classification`) refuses the pattern in both modules.
- ⭐ RED by renaming a table away for one request (sequential suite, restored before asserting); 5 hand mutants caught.

## 2026-09-25 — A policy publication can only be finished once (`SIGNOFF-REPAIR.9.2.2`)

`REASONBRAID-REPAIR-0521`.

- 🔴 **Root cause:** `mark_effective`/`mark_failed` read the stage, checked it in Rust, then wrote `WHERE publication_id = $1`; measured, both racing answers were `200` and the row ended `effective` with a `failed_reason`.
- ✅ **Fix:** `AND state = 'staged'` on both writes, zero rows → `lost_transition` (`WrongStage` with the found stage); store errors on the path (`owned_by`, reads, writes, `load`) → `storage`; `UnknownPublication`; `api::publication_refusal` (`Storage` → `500`) for both transitions and publish.
- ⭐ Ordered race control (queue one verb, observe it, then the other) so each verb's check is load-bearing; the original and 5 hand mutants caught. `.9.2.3` opened for 14 more store-fault-as-missing sites.

## 2026-09-25 — A refused conversation leaves no trace in the routing log (`SIGNOFF-REPAIR.8.2.7`)

`REASONBRAID-REPAIR-0520`.

- 🔴 **Root cause:** `create_thread` wrote the `create_boundary` resolution on the pool before `run_thread_command`, so neither the authorization nor the idempotency replay could undo it.
- ✅ **Fix:** `CommandTarget::Create { resolution }` + `routing::record_resolution_in` on the command's transaction, after the two committed-denial refusals. The autonomous handler's routing branch was dead (`routing_class` always `None`) and is removed.
- ⭐ RED live; 2 tool + 3 placement mutants caught, the before-authorization and before-claim placements by the new control alone. `.8.2` closed.

## 2026-09-25 — An experiment can no longer list the same option twice, and its errors say what is wrong (`SIGNOFF-REPAIR.8.2.6`)

`REASONBRAID-REPAIR-0519`.

- 🔴 **Root cause:** `validate_trial` checked emptiness and cohort kinds only; the draw indexes into `arms`, so each listing is a share, and the assignment map keeps one entry per case id.
- ✅ **Fix:** `first_repeat` + two named refusals before any lookup. 🔎 On the way: 15 of 17 `EvaluationError::MalformedDigest` uses were not digests and read as *"digest `…` is not a 64-hex string"*; a new `Invalid(String)` carries them (site classifier unchanged in effect).
- ⭐ 13 mutants lib-scoped: 11 caught, 2 unviable, 0 missed after closing four pre-existing lib-scope gaps with unit tests. `.8.2.8` deferred (a corrupt stored baseline classified as the caller's refusal).

## 2026-09-25 — Changing an assessment is refused instead of silently ignored (`SIGNOFF-REPAIR.7.4.8`)

`REASONBRAID-REPAIR-0518`.

- 🔴 **Root cause:** `claims::submit`'s replay pre-check selected only `assessment_id` by the six-column key and returned it; the body was never compared.
- ✅ **Fix:** the pre-check reads the seven body fields; identical replays, different is `ReplayMismatch { assessment_id, differs }` → `409 idempotency_mismatch` on the route, a named `400` in the `assess` step (whose command has its own key).
- ⭐ RED on both writers; 9 tool-listed mutants (hand-applied through the live suite) + 1 hand mutant (a field dropped from the comparison) caught. Stated gap: concurrent first submissions still race the pre-check (pre-existing, `.9.2.2`'s shape).

## 2026-09-25 — An assessment's author is whoever submitted it (`SIGNOFF-REPAIR.7.4.7`)

`REASONBRAID-REPAIR-0517`.

- 🔴 **Root cause:** `claims::AssessmentSubmission` was both the route's `Json` body and the store input, so the body's `author`/`verifier` went into the row; `threads::AssessmentInput` carried `verifier` the same way.
- ✅ **Fix:** `AssessmentRequest` (wire, no author/verifier, `deny_unknown_fields`) → `authored_by(principal.id_string())`; the store input is not deserializable and has no `verifier`; `api::json_body` maps a body rejection to `400 invalid_command` with serde's sentence, keeping `413`/`415`.
- ⚠️ A live `413` arm was flaky (the server closes a connection it did not read; the next pooled request broke) and became `api::json_bodies` unit tests.
- ⭐ RED live; 2 tool + 5 hand mutants caught. Opened `.11.36` (53 bare-`Json` routes) and `.7.4.13` (verification act, deferred).

## 2026-09-25 — The two-host demonstration passes again (`SIGNOFF-REPAIR.4.4.2.2.1`)

`REASONBRAID-REPAIR-0516`.

- 🔴 **Before:** since the web console's inbox panel was fixed on 2026-09-24, the demonstration's check of the console still looked for the panel's old, broken query and failed. Nothing noticed because everyday test runs skip the demonstration, but the next publication to GitHub would have failed its automated checks on it.
- ✅ **Now:** the check looks for the corrected query and says what it actually checks, and the whole demonstration passes. The book's description of that step was corrected the same way.

## 2026-09-25 — Deleted evidence can no longer be built on, and fetching it again makes fresh evidence (`SIGNOFF-REPAIR.7.4.6`)

`REASONBRAID-REPAIR-0515`.

- 🔴 **Root cause:** four reads of `evidence_snapshots` decided reliance without `deleted_at`: `derivations::submit`'s parent probe, `claims::submit`'s byte read (both assessment writers), `snapshots::submit`'s replay lookup and `external_identity`, the last backed by `0080`'s unique index, which covered tombstoned rows too.
- ✅ **Fix:** the probe and the byte read return the tombstone and refuse (`ParentTombstoned`, `SnapshotTombstoned`; `400 invalid_command` naming the reason) before any replay lookup; both replay lookups read live rows; `migrations/0112` re-creates the external identity index partial on `deleted_at IS NULL`, and the `ON CONFLICT` inference names the same predicate.
- ⚖️ **Decision** (`docs/decisions/2026-09-25_a-tombstone-retires-an-acquisition-not-content.md`): a tombstone retires the row; re-acquired content is a new row. Refusal was rejected because the retention sweep writes the same tombstone.
- ⭐ RED on every path by reverting its own site; 2 tool-listed + 5 hand mutants caught; broad live run (a migration is a shared primitive; it also closes the three-leaf batch).

## 2026-09-25 — The web console no longer shows a late answer under the wrong view (`SIGNOFF-REPAIR.11.1.2`)

`REASONBRAID-REPAIR-0514`.

- 🔴 **Root cause:** `render()` cleared the one `#output` element and awaited the renderer, which appended into that same element after its fetch; nothing told an earlier fetch that the operator had moved on. The presence and inbox buttons appended after their own fetch with the same race between two clicks.
- ✅ **Fix:** each render creates its own container and swaps it into `#output` at once (the heading still shows while loading); a stale renderer draws into a detached container. The container's identity is the generation, so no counter can drift from it. `latestOnly()` gives each panel a click counter and drops an answer that is not the latest click's.
- ✅ **Control:** the test router gains a middleware that HOLDS one named request until the test releases it, and `received()` waits for the page's resource-timing entry before reading, so the race is produced on demand, not timed. Four controls: view switch, identity change, presence, inbox (a seeded queued command tells the two answers apart). The node router joins the test server for presence.
- ⭐ RED 3 of 3 mechanisms; GREEN 7/7; 6 hand mutants caught, each by exactly its controls.

## 2026-09-25 — The web console's Timeline works again, and a real browser now checks the console (`SIGNOFF-REPAIR.11.1.1`)

`REASONBRAID-REPAIR-0513`.

- 🔴 **Root cause:** `app.js`'s `el()` converted only STRING children to text nodes and passed everything else to `appendChild`. `viewEvents` hands it `aggregate_version`, a number, so every thread with an event threw `TypeError: Failed to execute 'appendChild' on 'Node': parameter 1 is not of type 'Node'` and `render()` showed *client error*. The console's only controls read `app.js` as text.
- ✅ **Fix:** `el()` appends a `Node` as itself and anything else as a text node, a structured value as its JSON (`textOf`).
- ✅ **Control:** `crates/reasonbraid-server/tests/console_browser.rs` drives the pinned Chrome over CDP (`chromiumoxide`, a new dev-dependency already in the lock) against `api_router` + `ui_router` over a `run_pg_tests.sh` database. The browse worker could not be used: its deny-policies refuse `app.js` and every `fetch`. Chrome runs in its own process group with its stderr pipe as the exit detector, and a resolver rule that resolves no hostname, since `--disable-background-networking` still let it open six connections to Google in 20 s. A failed run keeps its workspace with a `browser.json` receipt, which `census_retained_fixtures.py` now reads as a fourth population.
- ⭐ RED in the real browser with the exact operator-visible error; GREEN 3/3; 3 hand mutants of the fix (JavaScript is outside `cargo-mutants`) all caught. CI: `pg-tests` runs its suites through `ci_browser.py`.

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

The entries before those above were rotated into reachable Git history at the
**nineteenth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show a02b7e914cdaae786515a833173a2949da8e20f5:DEV_NOTES.md
```

That snapshot is 74617 bytes and 756 lines, and contains 94 dated
entries; its Git blob is `0b97e62d505b104d405b4c21ae86196c415d2588` and its SHA-256 is
`b180421360ffcd2924a02b6e9d02ec9b84644bc72646ed1ecde8c002046c2c78`. It carries the eighteenth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **12 record(s) rotated out, 83 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
