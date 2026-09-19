# CHANGELOG.md

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

## 2026-09-19 — Three of the thirty-six advertised policy lines are enforced (`SIGNOFF-REPAIR.7.3.6.1`)

A resolver pack publishes six policy fields to every caller that reads the §12.2 capability registry, and a caller choosing a pack reads all six. `SIGNOFF-REPAIR.7.3.5` found two of them advertised as `deny` and enforced by nothing. This is the census of the other thirty-four.

- 🔴 **36 advertised lines across 6 packs — and 3 are enforced with a control that has been observed refusing.** `enforced` **3** · `unverified` **5** · `vacuous` **14** · `misdescribed` **7** · `undefined` **7**. Every line's verdict and its evidence are in `.doctrine/advertised_policy_verdicts.tsv`; re-derive the population with `python3 -B scripts/census_advertised_policies.py`. ⛔ The two populations — 18 Rust literals, 18 migration rows — are counted separately and never summed into one claim: a Rust literal is traceable to a call graph and a database row is not.
- 🔴 **Four of the six policy fields are consumed by nothing.** `redirect_policy`, `archive_policy`, `subresource_policy` and `javascript_policy` occur **34** times in tracked Rust — on 32 lines, since the INSERT's column list names three of them at once — and every occurrence is a declaration, a write or a comment. ⭐ The claim carries its enumeration in both directions: the classifier's default for an occurrence no rule explains is `read`, so it can only under-state the finding. Only `egress_class` and `sandbox_level` are consulted — by the ADR-018 vocabulary check and the resolution filter.
- ⭐ **So `.7.3.5` repaired the behaviour of two lines without connecting the advertisement to the enforcement.** The R3 worker denies subresources because a person read the advertisement and wrote the code, not because anything compares them; flipping the advertised word today changes what callers are told and not what the browser does. Owned by `.7.3.6.3`.
- ⚠️ **`vacuous` is 14 of the 36 and is not a synonym for safe.** R0 advertises `javascript_policy: "deny"` and runs no script engine — true today, guarded by nothing, false the day R0 gains one. That is exactly how `.7.3.5`'s two lines behaved until the pack they described started executing pages.
- 🔴 **`archive_policy` is defined nowhere in the repository**, and its sharpest instance needs no argument: R2 advertises `archive_policy: "deny"` two fields above a `media_types` list containing `application/zip` and `application/x-tar`, both of which its worker expands.
- 🔴 **The instrument carried the exact defect it was built to find, and falsification is what caught it.** Keyed by pack and field alone, R3's `subresource_policy` flipped `deny` → `allow` **in the producer** left the census **GREEN**, still reporting `enforced — .7.3.5` for a line advertising the opposite of what that leaf repaired. The advertised value is now part of the verdict's key. Measured in situ, restored byte-identical, replayed against the repair.
- ✅ Registered as a doctrine gate (0.044 s; its 26-arm self-test 0.067 s). ⚠️ **Calibrated over all 594 commits, not a window** — 300 returns a survivorship `0 of 300`, because the instance is older than that. Over the full history: **4 (0.7%)**, every one a commit that ADDED a pack. No commit has ever moved a value.
- ⛔ **No product code changed, deliberately** — a census that also fixes things cannot be re-run against the tree it changed. Four defects routed OUT with their own acceptance: `.7.3.6.2` (the registry's upsert writes 6 of its 18 columns, so a corrected advertisement never reaches an existing row — and `media_types`, whose own accessor's doc comment — `crates/reasonbraid-server/src/resolvers.rs:211`, citing `docs/decisions/2026-09-12_r2-acquisition-accept-set.md` — promises that *a deployment that narrows a pack's advertisement narrows what that pack may acquire, in the same act*, is one of the 11 it drops), `.7.3.6.3`, `.7.3.6.4` (the egress ladder is compared in the direction that admits a wider pack than the caller asked for) and `.7.3.6.5` (the undefined and misdescribed terms, G4's open strand among them). Decision: `docs/decisions/2026-09-19_an-advertised-line-carries-an-adjudicated-verdict.md`.

## 2026-09-19 — The debug build tree is retired, and `cargo clean` would have taken the evidence with it (`SIGNOFF-REPAIR.11.4.3.1.8`)

**Option (b), taken by the director**: *"if `target/debug/deps` is taking way too much space please delete, I don't mind long build time from time to time."* The leaf had left "measured and not worth acting on" open as a legitimate third outcome; the instruction forecloses it by accepting the rebuild cost explicitly.

- 🔴 **Executing option (b) literally would have destroyed cited evidence, and that is the finding.** The leaf writes (b) as *"a whole-tree `cargo clean`"* — and `cargo clean` removes the entire `CARGO_TARGET_DIR`. Three of its subdirectories are retained evidence, not build output: `target/pg-tests` (**309,444 KiB / 8,583 files**, named by **26** tracked files), `target/ci-browser` (**554,732 KiB / 397 files**, **7**) and `target/browser-lifetime-controls` (**384 KiB / 96 files**, **5**) — including `run-9_ueev0t`, which the fixture census holds back *because* it is cited. ⭐ The census that protects those directories from the fixture reaper does not protect them from cargo's own broom. The retirement was scoped to `target/debug`.
- ✅ **Frozen manifest → retirement → residue census**, the shape `.11.4.3.1.6` set. Before: **182,302,640 KiB logical / 1,826,894 files**. After: `target/debug` absent, `find target -maxdepth 1 -name 'debug*'` returns **0**, and all three evidence directories report their pre-retirement byte and file counts unchanged.
- ⚠️ **The `CLAIM_VERIFICATION` caveat is inherited verbatim and it bites**: `du` figures are logical bytes and do not establish physical space recovered on a cloning filesystem. The published recovery figure is the volume's own — **≈174 GiB by `df`** — against 173.9 GiB logical.
- ⛔ **NOT claimed: that builds get faster.** The leaf forbade that causal claim without an experiment and none was run. What the session did measure is that the build-time cause is a HOST property: `syspolicyd` at 44% CPU during a 68m 16s rebuild, with swap at 6,151 of 7,168 MB.
- ⭐ A no-cost prophylactic shipped alongside: `target/.metadata_never_index` and `.project-data/.metadata_never_index`, after `mdfind -onlyin target` measured **573,057** indexed items. ⚠️ Not asserted to have taken effect — the marker is documented for volume roots and the reliable mechanism is the Spotlight Privacy list.
- ⛔ Gatekeeper's Developer Tools exemption was **described wrongly** in the session that proposed it and is corrected here: it takes **applications**, not volumes or paths, so it exempts everything spawned by that terminal. It is a real defence-in-depth reduction, it cannot be set from a CLI, and it remains the director's decision.

## 2026-09-19 — G4 and G5 re-derived, and blocker C2 closes (`SIGNOFF-REPAIR.11.4.7.4`)

✅ **All five gate records now re-derive, line by line, each verdict carrying the command that produces it.** G1–G2 (REPAIR-0198), G6–G7 (REPAIR-0195), G3 (REPAIR-0264), G4+G5 here. **3 stand · 1 must be re-earned.** ⛔ No record's CONCLUSION changed: G6–G7 remains NOT MET for Internet exposure, G3 remains blocked as binding use.

- 🔴 **G4 must be re-earned, and unlike G3's three it is NOT fully discharged.** Its clause is *unsupported, denied, mutable or non-reproducible resources fail explicitly rather than becoming fabricated evidence*, and three defects since the gate land inside those words: REPAIR-0224 (`expected_digest` was read by nothing, so a **pinned** reference accepted a snapshot of entirely different bytes), REPAIR-0227 (a snapshot recording an acquisition of a document its reference did not name, stored at `200`), `.11.14.3.12` (a receipt with no snapshot, silently). All repaired.
- ⚠️ **The open strand, named rather than folded in.** G4's deferral 1 says the R3 `vm_container` requirement *stays gated*; `resolvers.rs:303` emits `"container_required": true`, a declaration **about the deployment** rather than a property the product enforces. `.7.3.6` is **open** over it and two further advertised-but-unverified R3 lines, after `.7.3.5` found the pack advertising `redirect_policy: "deny"` and `subresource_policy: "deny"` and enforcing **neither**. An advertised `deny` that is not enforced is G4's clause from the inside — it is now frontier row 1.
- 🔴 **4 of 6 evidence pointers no longer resolve.** `profiles.rs` went **23 → 56** tests, and `profiles 23` — **G4's only test citation** — now names an unrelated tenant-binding test. ⭐ The notation is settled as an **index**, by the records themselves: Demonstration B's step 1 gives a number *and* a name, and the count reading is impossible there.
- ⛔ **The instrument that produced those numbers was wrong first and was rebuilt before anything was concluded.** Counting every `fn` made `routing 2` resolve to `pool`, a helper. The corrected instrument counts only test-attributed functions and **self-checks against a known answer** — `policy.rs` = 14, matching the 14 that passed. Third near-miss of the session from a key wider than the thing it named.
- ✅ **G5's three claims all stand, and its subtraction census carries a positive control** because a zero-hit grep is an absence claim. The first pattern returned **0** and was **wrong** — `quality lift` with a space, where the corpus writes `quality-lift`. Corrected it returns **3**, and all three are withdrawals (`LIVE_STATUS.md:1719`, `qualification-review.md:12`, `roadmap.md:99`). The README's mechanism claim is verbatim at line 9, the **H1 null** is preserved, and no new benchmark evidence exists, so no cross-run quality claim can.
- ✅ **Each of G4's five deferrals re-derived individually**: (1) narrowed, `.7.3.6` open; (2)–(5) hold, with (5)'s worst case repaired by `.11.14.3.12`.
- NO REGRESSION: `profiles` 56/56 · `evaluation` 3/3 · `routing` 2/2 — **61 tests, 0 failures** — and all six tests the records name pass individually. No product code, schema, migration, test or script changed. Decision: `docs/decisions/2026-09-19_g4-g5-four-claims-re-derived.md`; both gate records byte-unchanged.

## 2026-09-19 — G3's seven claims re-derived, and its evidence pointers no longer resolve (`SIGNOFF-REPAIR.11.4.7.3`)

**3 stand · 1 narrows · 3 had to be re-earned** — blocker C2's third of five gate records, measured against **38 tests, 0 failures** across every suite the record cites.

- 🔴 **The finding came before any verdict: G3 cites its evidence by test POSITION, and positions are not identities.** `policy.rs` went **11 → 14** tests, and **2 of the 10** cited pointers now name a different test — including `policy 10`, the **correction clause's only evidence**, which today resolves to a publication-authority test. The other eight survive by accident: all three inserts landed after the highest index they use.
- ⭐ **Three more pointers are unresolvable from the document at all.** When the record was written `publisher` held exactly **2** tests, `reconciler` **3** and `compiler` **8** — each equal to the highest index cited against it — so `publisher 2` is ambiguous between *the suite's two tests* and *test number two*. The readings have diverged: `publisher` now holds **7**.
- 🔴 **The AUTHORITY clause is the strongest verdict, and its own citations could never have caught the defect.** `.9.3.1` measured `corrections.rs::authority_holds(pool, grant_id)` taking the grant id **and nothing else** — asking whether a grant EXISTS, never whether the caller HOLDS it — at **5 sites in 4 spellings**, the approval surface among them. `policy 1` and `policy 4` are green against both the defective and the repaired code. ⚠️ Residual OPEN: `GrantAction` cannot express a publication or correction target.
- ⭐ **On PUBLICATION the sharp point is where a defect sat, not how many there were:** `.9.2.1.1` found a vacuous third leg inside `the_publish_verb_drives_the_git_half` — one of the two tests that clause cites. Part of the evidence was hollow when the gate was taken.
- ⛔ **A concern RAISED AND REFUTED rather than published:** `run_pg_tests.sh publisher reconciler` answers `unknown suite(s)`, which read as cited evidence no gate can execute. All **six** files absent from `SERVER_SUITES` are **offline** and CI's `cargo test --all` runs them; `server_boot` looked database-backed only because a grep matched its prose. No gap, no leaf.
- ⚠️ **The blocker register's C2 cell was stale by two** — it said *four remain* while three of five records had been re-derived. Corrected, with the command to re-derive it rather than read it. `.11.4.7.4` (G4 and G5) is the last open child; closing it closes C2.
- ⛔ The 2026-09-07 gate record and `docs/evidence/2026-09-07_demonstration-b.md` are **byte-unchanged**, and no doctrine gate is proposed (`.11.6`: four hand-written records is not a population). Decision: `docs/decisions/2026-09-19_g3-seven-claims-re-derived.md`.

## 2026-09-19 — The lease was written by one clock and read by another, so the published 60 s TTL was nominal (`SIGNOFF-REPAIR.4.2.3.1`)

🔴 **`node_leases.lease_expires_at` was WRITTEN `Utc::now() + LEASE_TTL` by the server process and COMPARED against `now()` by the database.** With the two apart by `S`, a lease was live for `60 s ± S` — and nothing anywhere would have noticed them parting.

- 🔴 **REPRODUCED by driving the writer's own parameter ten minutes ahead** — the one seam that can separate two clocks on a single host. The unrepaired product granted **660.0 s of lease on the database clock**, on BOTH writers: **38 passed / 2 failed**, only the new controls. ⭐ Each asserts `last_seen_at` first, because it is written from the same parameter and compared by nothing — a free witness that the injection LANDED.
- ⭐ **The argument that decided it was in the WIRE, not the store.** One `HandshakeResponse` already carried `lease_expires_at` (process clock) beside `server_time`, which is `clock_timestamp()` — the database's — and `server_time` is the anchor `.3.4.3.1.2` gave the node to correct every server instant against. The field was expressed in a clock the product does not publish.
- ✅ **The database clock now owns the column** at all five sites: three writers produce it in SQL, and both admission checks compare it in SQL. ⛔ **One clock is not one function** — `now()` is `transaction_timestamp()`, right where the statement IS its transaction, wrong inside a caller's: `events` can block on the tenant guard's exclusive mode first, so `verify_fencing_in_tx` and the replacement writer use `clock_timestamp()`. Spelling all five `now()` would have re-admitted a lapsed lease — `.4.2.3`'s revival defect re-entering through the clock.
- 🔴 **The census corrects `.4.2.3`'s own: three writers, not two.** That census was right when run; `.4.1.5` (REPAIR-0167) added the replacement's afterwards. A count inherited from a leaf is a claim about a moment.
- ⚠️ **NOT claimed: three of the five sites are unfalsifiable here and are labelled, not covered.** The replacement writer and both admission checks sample their clock internally, and a fixture host has one clock; two hosts with a driven offset is what would test them. `.3.4.3.1.1` measured this server's two clocks agree within a second.
- ⛔ **Blast radius measured before choosing, not assumed:** `git grep -n "\.lease_expires_at" -- crates/reasonbraid-node` returns **zero** uses against two struct-field declarations, so changing what the handshake returns changes no node behaviour.
- 🔴 **FALSIFIED TWICE, one writer at a time** — neutralize `issue_lease` and only `an_issued_lease_lasts_one_ttl_of_database_time` fails, at 660.0 s, while the renewal control holds at 60.0 s (**39/1**, rc=101); neutralize `renew_lease` and the mirror happens (**39/1**, rc=101). Each neutralization was read in the source before the run and the restore proved byte-identical with `cmp`.
- ✅ **VERIFIED:** `bash scripts/run_pg_tests.sh node_channel` → **40 passed, 0 failed** in 53.08 s; both controls print **`60.0 s of lease left on the DATABASE clock`**, and the three controls the acceptance named pass unchanged.
- 🔎 **Opened while rewriting the frontier: `.11.22.1`.** The table named `.4.2.3.1` at rows **1 and 5**, both `pending`, while the caption below asserted the duplicate had been removed — true when written, false after a later commit promoted the leaf to row 1. Censused at **1 unfinished duplicate across all 15 tracked trees**. `FRONTIER-STATUS` is silent because both rows AGREED with the leaf.
- Decision: `docs/decisions/2026-09-19_the-database-clock-owns-the-lease.md`, with four rejected alternatives.

## 2026-09-19 — The acceptance gate read one checklist per file, and drifted out of its own corpus (`SIGNOFF-REPAIR.11.2.6`)

🔴 **`check_task_acceptance.sh` stopped at the FIRST box matching each label and exited. This tree carries 194 `ROOT CAUSE` boxes; the gate read one, at line 388, from a leaf closed long before — on every code commit for roughly 200 commits.**

- ⭐ **One cause, two defects: an inert control does not hold a line, it hides that the line moved.** While the gate was vacuous its vocabulary drifted out of the corpus unnoticed. It hard-gated `ADDRESSED` (**14** uses) and `ROOT CAUSE` (**13**) while the project wrote `NO REGRESSION` (**203**), `FIX / LOCKSTEP` (**192**) and `REPRODUCE / ISSUE` (**142**).
- ✅ **Scope**: the owner is the leaf whose `Status:` turns `done` in the staged diff, taking the DEEPEST — a commit closes **2.75 leaves on average**, because a parent lane closes with its child and carries no checklist. Read from the diff's hunk headers, never the commit message: a pre-commit hook runs before the message exists. ⚠️ And from the STAGED BLOB, not the working tree — line numbers indexing one and content read from the other misalign exactly where a pre-commit hook invites it.
- ✅ **Vocabulary**: label families move to `.doctrine/acceptance_labels.txt`, a project seam, so the portable check keeps no project's words. Derived from the census, never invented. Calibrated over 300 commits: every closed leaf refuses 57.7 %, the deepest with the derived families **20.6 %**.
- ✅ **The gate gained the `--self-test` it never had**, six arms, whose NEGATIVE arm is the extractor it replaced: a first-box scan that must pick the wrong leaf, or the positive arm proves nothing. `scripts/tests/test_task_acceptance.py` rewritten to the new semantics — **13 tests, OK** — including the regression test for this defect.
- ⛔ **The debt is COUNTED, NOT BACKFILLED.** `--debt` reports **73 of 281** closed leaves answering fewer than every question. Writing `NO REGRESSION: …` into a leaf closed weeks ago asserts a suite was run when nobody ran one — manufacturing evidence after the fact, which is the exact offence the gate guards against. The softer "re-run today, record it there" is rejected too: a true statement about today attached to a past change is a false record. Decision: `docs/decisions/2026-09-19_acceptance-debt-is-counted-not-backfilled.md`.
- ⚠️ **NOT a quality claim about those 67 changes** — most were verified; what is missing is the record of it in the leaf.
- 🔴 **The in-situ falsification earned its keep: the first repair PASSED with its own checklist deleted.** This leaf *discusses* `NO REGRESSION`, so five prose bullets naming it satisfied the question — incidental-prose leakage walking back in through the widened vocabulary, and a self-referential leaf is where it bites. Fixed by anchoring the family as the bullet's **lead-in**; re-priced 17.5 % → **20.6 %**, three points paid for correctness. ⭐ The `--debt` census had been written separately and disagreed with the gate (67 vs **73**); there is one definition now.
- 🔎 **The repair tripped a neighbouring gate's documented residual, now closed.** `check_self_tests.sh` discovers instruments by grepping for the flag's text; the new test case runs the checker with it, so discovery found a unittest module — which does not *ignore* an unknown argument but exits `error: unrecognized arguments`. `scripts/tests/` is excluded by path: tests of instruments are not instruments.
- 🔎 **FOUR instances of *a key too loose* in one session** are now in that note: `list_runs` matched as a path tail, `tenant_id` blind to `authored_by_tenant`, an added `Status: done` line matched by TEXT when it is byte-identical for every leaf, and — in the falsification harness itself — a bullet regex that deleted a DIFFERENT leaf's identically-opening line and reported PASS. ⭐ All three keys *looked* specific; what they shared is being a value the domain repeats.

## 2026-09-18 — The acceptance gate enforces a vocabulary this project does not use (`SIGNOFF-REPAIR.11.2.6`, calibration)

🔴 **`check_task_acceptance.sh` reads ONE checklist per staged tree file — this tree has 194 `ROOT CAUSE` boxes and the gate validates line 388, a leaf closed long ago, on every commit that touches code. Calibrating the repair found a second defect larger than the first.**

- 🔴 **The vocabulary census:** `NO REGRESSION` **203**, `LOCKSTEP` **147**, `REPRODUCE / ISSUE` **142**, `FIX / LOCKSTEP` **45** — against the gate's own hard-gated `ADDRESSED` **14** and `ROOT CAUSE` **13**. Two of its three blocking labels are the project's rarest spellings, and the two it does not know at all are how the checklist is actually written. ⭐ One root cause for both holes: the gate has been inert for ~200 commits, so nothing kept its scope or its words honest — `.11.2.4`'s defect, in the gate that polices the other gates.
- ✅ **Four candidates priced over 300 commits** (231 touched code and staged a tree): added-lines boxes **23.8%** refused; the same accepting prose **25.5%**; every closed leaf **57.7%** (a commit closes **2.75 leaves on average, up to 5** — parent lanes close with their child and carry no checklist); **deepest closed leaf + corpus-derived vocabulary 20.9%**.
- ⭐ **20.9% is PAST DRIFT, not a forecast.** The gate is staged-scope-aware and only examines the leaf being closed now, so a leaf written to the enforced standard passes. Classified rather than assumed: **four of five sampled refusals carry no regression evidence anywhere** — real omissions the inert gate never asked about.
- 🔴 **My own third measurement was unsound and is recorded, not quietly replaced.** It matched added `Status: done` lines by TEXT; that line is byte-identical for every leaf, so one commit appeared to close dozens and the candidate scored 64.7%. Re-derived from the diff's hunk headers it scores 57.7%. ⭐ Third `a-key-too-loose-returns-the-wrong-instance` in one session.
- ⚠️ The gate is NOT yet repaired — this commit records the calibration and the leaf stays `pending`. The debt the 20.9% represents is named for an explicit decision rather than silently inherited.

## 2026-09-18 — One node presence read asked nobody who was calling (`SIGNOFF-REPAIR.3.5.5`)

🔴 **`GET /v1/nodes/presence` took no `HeaderMap` at all.** It never learned who was calling, never asked whether they may, and then selected `FROM node_presence WHERE node_id = $1` — on a view that carries `tenant_id`. Meanwhile `GET /v1/admin/nodes/presence` gated the SAME view behind a `tenant_admin` grant, on the same port, in the same process.

- ⚠️ **Reproduced, and the measurement covers more than one caller:** against the unrepaired handler the control failed `left: 200 / right: 401` with the other 37 tests passing. Because that handler read no headers, **every** caller took the path the anonymous one was measured on — the cross-tenant case is the same execution, not an extrapolation.
- ✅ **Authenticate, then DERIVE the tenant from the principal.** A principal belongs to exactly one tenant structurally, so there is no second identifier to name and none to bind: **no wire change**. The parser is shared (`crate::api::resolve_principal` is now `pub(crate)`) rather than copied, because two copies of that rule would drift — which is the disagreement this repair exists to end.
- ⛔ **A foreign node answers `unknown_node`, the SAME answer an absent node gives.** The existence oracle is closed, not the payload withheld — §9.8's `scope_hidden`, and the stance this file already took for the handshake ("or whose node has no key — the same refusal: no existence leak").
- ⭐ **Cost measured at ZERO for the only caller.** `web/app.js` already sends `x-reasonbraid-principal` on every GET through one `api()` helper, and its own comment says *"so the server's gates apply exactly as they do to the CLI"* — they did not, here. No node client calls the route.
- ⭐ **Three tests failed the first repaired run for the RIGHT reason**, which is the repair demonstrating itself: their node lives in a bootstrapped admin's tenant, not the suite's seed tenant, so the seed reader was correctly a foreign caller. `node_channel` 38/38, `node_replacement` 2/2, clippy clean.
- ⛔ **NOT claimed:** that this settles the node channel's authentication generally. `presence` was the only GET on `node_router`; the POST verbs authenticate on fencing and enrolment tokens in their bodies, a mechanism this leaf did not audit.
- Decision: `docs/decisions/2026-09-18_node-presence-is-read-by-its-own-tenant.md`, with both rejected alternatives.

## 2026-09-18 — The read census could not see 34 of its own routes, and its population had no producer (`SIGNOFF-REPAIR.3.5.4`)

🔴 **`.3.5.3` published "24 `get(…)` routes in `api.rs`". At its own commit there were 54 — and `api.rs` is not the surface: `rb-server.rs` merges three routers, for 58.** The leaf was opened to follow 10 delegating handlers; it found the population itself was wrong by more than half.

- ⛔ **The number had no producer.** `24` was prose in a leaf; nothing derived it, so nothing could contradict it. `scripts/census_get_route_binding.py` now does — enumerating every product GET across the three mounted routers, reporting what the caller supplies, whether the handler authenticates, which authorization primitive it reaches **transitively through the local wrappers**, and following each `crate::module::fn` delegate one level for its SQL and tenant parameter.
- ✅ **All 58 classified, `24 + 6 + 24 + 4`.** 24 authorize (five of them only via the generic `inspect<F, Fut>` wrapper — the hop the old per-handler scan could not take); 6 derive the tenant from the principal and bind it; **24 are site-global BY SCHEMA** — every one of their 24 tables has no tenant column, so there is nothing to bind and a column-grep would have reported 24 false defects; 4 do not authenticate.
- 🔴 **One product defect, routed to `.3.5.5`: `GET /v1/nodes/presence` asks nobody who is calling.** No `HeaderMap`, no principal, no authorization, and `SELECT … FROM node_presence WHERE node_id = $1` with no tenant predicate — on a view that carries `tenant_id`. The product already gates the same view at `/v1/admin/nodes/presence` (a frozen-admin inspection scoped by `?tenant_id=`), and both are served by one process on one port. ⚠️ Bounded: a read, observability payload, needs a known unguessable node id — but it needs **no principal at all**, and `200` vs `unknown_node` is an existence oracle.
- 🔴 **The instrument found five defects in itself before it found one in the product**, each the very failure the leaf is about: an `impl` method shadowing the route handler of the same name; generic `fn name<F, Fut>(` invisible to the resolver, so the whole `/v1/admin/*` family read unauthorized; `crate::evaluation::list_runs` walking into `api.rs`'s own `list_runs` (*a key too loose*, a false positive on a security census); `resolve_principal` counted as a gate, conflating authentication with authorization; and anchoring on the literal `tenant_id`, which called the correct `authored_by_tenant` and `{CITED_BY_TENANT}` unscoped. Each is now a two-sided self-test arm.
- 🔎 **A second finding, unrelated to the product and opened as `.11.2.6`:** `check_task_acceptance.sh` reads **one** checklist per staged tree file. This tree carries **194** `ROOT CAUSE` boxes and the gate validates line 388 — a leaf closed long ago — on every commit. That is a third leakage hole in a check whose own header documents two.

## 2026-09-18 — Correction: REPAIR-0259's severity claim, and a reproduction that modelled a path nobody uses

⚠️ **The previous entry said *"a project that pulled this spine could not make its first commit"*. That is wrong, and the reproduction behind it was a fixture rather than the product.**

- **How projects are actually created, from the director and confirmed in this repository's own history:** `rdje/bedrock` is a **GitHub template repository**, not a syncable remote. A new project is instantiated through GitHub's *Use this template*, which copies the WHOLE repository, and is then cloned and bootstrapped locally — `b932c05 "Initial commit"` followed by `823c2bc "bootstrapped from bedrock"`. The remotes confirm it: **reasonbraid's remote set contains no bedrock entry** — `origin` is `rdje/reasonbraid` and nothing else. ⚠️ bedrock of course has its OWN `origin` (`rdje/bedrock.git`), which is how the template itself is maintained; what is absent is a link FROM an instantiated project BACK to the template.
- ⛔ **So a fresh project receives every script and commits fine.** Copying only the `NEUTRAL` paths into a bare `git init` was a model of a creation path nobody uses, and its `10 doctrine breach(es)` measured my own fixture.
- ✅ **What remains true, and why the repair stands.** `update_scaffold.sh` is an UPDATE mechanism and only copies listed files — it deletes nothing. `NEUTRAL` carried the registry and not the 7 scripts it names, so syncing into a project instantiated from an OLDER snapshot installs a registry naming checks that never arrive and breaks that project's gate. A real hazard at update time for an older project; not at creation.
- ⚠️ **Corrected severity:** the defect is *the update mechanism cannot deliver the doctrines the registry names*, not *the spine ships broken*. `SCAFFOLD-COVERAGE` keeps its 0.02 s on that basis — registry-versus-list drift is silent and bites the project being updated — as a consistency guard rather than a shipped-broken guard. The root-cause finding is untouched: `NEUTRAL` is a second copy of the registry that nothing derives, and it had drifted by seven.
- 🔎 **The method failure, which is the useful part.** I built a fixture, reproduced a failure inside it, and reported the fixture's behaviour as the product's. **A reproduction is evidence only if the path it models is one that occurs.** The falsify leg asks what would have to be true for the claim to be false — here, *"projects are created some other way"* — and I never checked how a project is created before asserting creation was broken.

## 2026-09-18 — The scaffold shipped an enforcer registering checks it did not carry (`SIGNOFF-REPAIR.11.23`)

🔴 **A project that pulled this spine could not make its first commit.** `scripts/update_scaffold.sh` carried `check_doctrines.sh` — the registry of 18 doctrines — and **not the 7 check scripts that registry names**. Copying only the `NEUTRAL` paths into a bare `git init` repository printed seven `?? (missing/not executable: …)` rows and `10 doctrine breach(es) — commit blocked`.

- The seven: `HEADING-DEPTH`, `TASK-STATUS`, `LOCKSTEP-CLAIM`, `INDEX-FRONTIER`, `FRONTIER-STATUS`, `RUST-FORMATTING`, `SELF-TEST`. **Every one was earned here by a measured defect** — and the list they were missing from is the only mechanism that moves a doctrine anywhere, so the spine's improvements were accumulating in the instance and could not leave it.
- ⭐ **Fourth instance of one shape:** `NEUTRAL` is a second copy of a fact the registry owns and nothing derived it — as `INDEX-FRONTIER` is the index's copy of the frontier, and `FRONTIER-STATUS` a row's copy of a leaf's status. Each drifted the moment the first copy grew.
- ✅ **Fixed and re-reproduced:** the seven join `NEUTRAL`, `SCAFFOLD-COVERAGE` is registered at **0.02 s**, and the bare-repo run now prints **zero** `??` rows. Four breaches remain there, none a scaffold defect — three are empty-project content checks, and the fourth is below.
- 🔴 **The gate failed its own reproduction, which is the useful part.** `update_scaffold.sh` is deliberately absent from its own `NEUTRAL` (a sync script overwriting itself mid-run is its own hazard), so a project may legitimately have no scaffold at all. That is **NOT CHECKED** — never a pass, never a breach.
- 🔴 **And falsification found the gate's own `$EXEMPT` guard DEAD.** Mutating it away changed nothing, because the parser anchored on the array literal and never saw the conditional `DOCTRINES+=("…")` form that registers `KNOWLEDGE-MAP` and `PROJECT-SPECIFIC`. Both forms are parsed now. ⚠️ Widening the parser then over-matched the registry's own explanatory comment — the third *key too loose* in one session.
- ⛔ **Backflow is still manual.** `update_scaffold.sh` pulls downstream only and `rdje/bedrock` is the root template with nothing to pull from. What changed is that the seven are now **transportable** and the class cannot silently recur; carrying them up belongs to a `BEDROCK-MAINTENANCE` leaf there.

## 2026-09-18 — A gate blind to the abbreviation of the very noun it is built on (`SIGNOFF-REPAIR.11.2.4`)

🔴 **`check_visibility_policy.sh` exists because the superseded private-repository instruction "had already leaked past two" reviews. All four of its clauses were anchored on the full word, so two live instances written in the abbreviated copular form sat in the corpus invisibly and the gate returned rc=0 over them.**

- ✅ **The widening is DERIVED, not invented** (`.11.2.2`'s rule). A census of every tracked-Markdown line carrying a repo word beside `privat` — about 60 lines — shows the abbreviation is the ONLY uncovered spelling that *states* a visibility. The many `private-repository instruction` lines put `private` BEFORE the noun and are corrections of the superseded policy, so no clause matches them and none should. One substitution in three clauses; **delta 3 → 11 matches, `comm` confirms none lost**.
- 🔴 **Six of the eight new matches were the project documenting its own blind spot.** Writing down what the instances read puts the trigger into the corpus, so the widened gate flagged `MEMORY.md`, `docs/TASK_TREE.md` and three places in the tree. That is `check_self_tests.sh`'s founding incident — *a control searched for a string it contained* — reached from the documentation side.
- ✅ **Dispositioned by a rule rather than case by case: except what must stay VERBATIM, reword what is PROSE.** Five live sentences now NAME the abbreviated form instead of instantiating it, following `check_self_tests.sh`'s own precedent of rewording rather than path-excluding, so those surfaces stay policed. Three preserved-evidence lines are excepted with reasons: a dated decision record (decisions supersede rather than mutate), a dated `Done (2026-09-08)` entry whose correction is the line above it, and the leaf's own reproduction command, where the literal sentence *is* the evidence.
- ✅ **Falsified four ways**: the pattern reverted (three positives red), a bare `privat[a-z]*.repo` clause added (**all four** negatives red, including two real corrections), a genuinely new sentence injected into the book (refused by file and line — what the gate is *for*), and a bogus exception added (`stale exception` by name).
- ⭐ **And `FRONTIER-STATUS`, registered one commit earlier, caught this leaf leaving a stale frontier row behind** — the defect it was built for, firing on its author within the hour.
- ⛔ Not claimed: that the pattern is complete. It covers the spellings this corpus contains today; the census command is recorded so the next reader re-derives rather than trusts.

## 2026-09-18 — Small-swarm orchestration recorded as the existing objective, unscheduled (director direction)

The director asked for ReasonBraid to orchestrate 2–5 agents working collectively on any given problem, scaling later, prompted by the reported OpenAI Navier–Stokes swarm and xAI's Grok 4.20 multi-agent model.

- **It is not a pivot.** `README.md` already states the objective, and the shipped step vocabulary in `workflows.rs` — `solicit`, `blind_solicit`, `critique`, `revise`, `synthesize`, `assess`, `vote`, `adjudicate`, `decide`, `approve`, `moderate`, `present`, `retrospect`, `evidence_request`, `quick_advice` — contains **no programming-specific verb**. A profile is versioned configuration over existing verbs, so changing domain is writing a profile rather than changing code; real `claude.rs`/`codex.rs` adapters already dispatch to agent harnesses.
- ⛔ **The binding constraint is the ACCEPTOR, not orchestration.** Output stays untrusted until deterministic rules accept it. Programming has acceptors — compilers, suites, this repo's own gates. "Any problem" frequently has none, which is exactly what the Navier–Stokes episode illustrates: a swarm produced a result that remains unverified and whose attribution is disputed. Those two difficulties are what this platform is for.
- ⚠️ **2–5 agents is in range; thousands is a re-architecture**, not an extrapolation — the current design serialises administrative acts on one site guard row under per-tenant transactions.
- ⛔ **Unscheduled, and no leaf is opened** — deliberately, because the LAN bar was defined hours earlier and a leaf would put this above it. Much of the capability sits inside that bar regardless: `blind_solicit`, rounds, quorum and adjudication are G3/G5 surfaces and G5 is currently withdrawn.
- ⚠️ The two external results are recorded as **reported, not established** — the Navier–Stokes claim is explicitly unverified — because a direction may rest on them but no claim may.

## 2026-09-18 — Internet exposure deferred; the LAN path is the priority (director instruction)

> *"The internet exposure is not high priority right now. It needs to fully work on the local network first."*

- **B1, B2 and B3 become `yes (deferred)`** in the blocker register — a third state the register did not have. The in-repo work is still owed and `SIGNOFF-REPAIR.14` still owns it; what changes is that it leaves the frontier and no commit is spent on it until the local-network path is complete.
- ⛔ **`yes (deferred)` is deliberately not `no`.** `.13` was built around the observation that an owned-but-unsurfaced blocker is harder to notice than an unowned one, so these rows stay visible and are surfaced once per session with their ruling and trigger.
- ⛔ **Priority moved; claims did not.** Phase 7's G6/G7 stays **NOT MET** and `.14`'s standing prohibition against turning the exposure profile on is untouched. Deferring work is not permission to describe the system as Internet-ready.
- ✅ **The trigger is DEFINED, and the roadmap already encoded it.** The director clarified it in two steps: behaviour matches the roadmap's objective on the LAN, and G7 is earned on the LAN before the Internet. The roadmap's header separates *initial: private LAN* from *target: arbitrary Internet hosts*, and its gate table's *unlocks* column makes **G6 the only gate whose unlock is a transport**. So the bar is **G0–G5 genuinely met plus G7 on the LAN**, G6 out of scope.
- ⛔ **The first candidate is WITHDRAWN as wrong in KIND**, not detail: it was phase arithmetic, and repairs closing can diverge from behaviour matching in both directions.
- ⛔ **G7 splits, recorded now so it cannot become a false claim later.** Structural legs — restore, survival of induced failure, instrumentation — are earned once. Load thresholds and SLO targets are anchored to a LAN and get re-derived under the Internet posture.
- ⚠️ **Measured before accepting the bar:** G7 is the least-built LAN gate — backup/restore real, metrics without objectives, load dev-scale only, and chaos/game-day returning **0** hits repo-wide. G5 is *withdrawn* rather than unverified, so re-earning it is inside the bar too.
- Recorded in `docs/decisions/2026-09-18_lan-completeness-precedes-internet-exposure.md`; the register, the book's blockers chapter, the frontier row for `.14` and `MEMORY.md` all carry it.

## 2026-09-18 — The frontier table gains a row per state change and never retires the old one (`SIGNOFF-REPAIR.11.22`)

🔴 **DOC-0049 reported a stale frontier pointer. Censusing it found the cause is structural, and worse than the symptom.**

- **The census, run before any rule (`.11.6`): 66 rows naming 49 distinct leaves — 13 leaves carry 2 to 4 rows each, and 10 rows' status columns contradict their leaf.** A leaf gets a row when it opens (`pending`) and another when it closes (`done`), and the opening row is never removed. `.11.14.3.10` had four rows, three stale.
- ⭐ **It caught its author in the act.** `.11.21` had a `done` row at `1a18` and a `pending` row at `1a16` — both written by me two commits earlier, in this same session.
- 🔴 **A third historical firing, found by the census rather than remembered:** `.11.15` records `.7.4.1` sitting at row 2 for seven commits after closing. With REPAIR-0253's five rows and row 1's own pointer, the founding population is three separate firings.
- ⚠️ **Two false starts in the census itself, recorded because they are one lesson twice.** `s.index("## Current Frontier")` matched an earlier prose mention and censused a table 400 KB away, returning `0 rows`; then the slice ran to the next `## ` heading and swept in unrelated tables. Anchoring to `^## Current Frontier$` and bounding to the contiguous pipe lines fixed them. Both are `a-key-too-loose-returns-the-wrong-instance` — in the leaf immediately after the one that repaired a key defect of the same shape.
- ✅ **The rule, with the history case explicitly permitted:** every row's status column equals its leaf's own status, and row 1's leaf is not finished. ⛔ It is NOT "no finished leaves in the table" — most of the table is closed rows kept as recent history. A self-test arm asserts the legal `done` row so the rule cannot drift into the stricter one.
- ✅ **Both status forms read.** 350 leaves, 28 carry no `Status:` line and 27 of those state it in an `Opened:` bullet; reading one form would report 28 false disagreements. `active` is a real third value and is not finished.
- ✅ **Reconciled:** 9 stale rows removed, 1 corrected (`.9.3.4` `pending` → `active`). **`FRONTIER-STATUS` registered** as doctrine 16 of 20 at **0.06 s**, falsified five ways and against the real defect in both shapes.

## 2026-09-18 — A frontier row pointed at a leaf closed forty commits earlier (`SIGNOFF-REPAIR.11.22`)

🔴 **Row 1 of the frontier — the one row a fresh session resumes from — named `.7.4.5`, which has been `done` since REPAIR-0216.** It was found only by opening the leaf in order to work on it.

- **This is the second firing in two days.** REPAIR-0253 reconciled **five** rows whose status column said `pending` while their own leaves said `done` (`.11.18`, `.11.17`, `.11.14.3.7`, `.11.14.3.5`, `.11.4.7.2`). Two hand reconciliations is a population, not an anecdote.
- **Censused rather than asserted:** `git grep -lni "frontier" -- scripts .githooks knowledge-map` returns **10** scripts, and **0** relate a row's status column to its leaf's `Status:` line. ⭐ Two apparent hits were false positives of a loose key — `check_task_status.sh` matched its own self-test FIXTURES, `bootstrap.sh` matched a heredoc it WRITES — a key defect of exactly the shape repaired one commit earlier in `.11.8.1`.
- ⛔ **Three gates look adjacent and every one of them correctly misses it.** `INDEX-FRONTIER` compares `docs/TASK_TREE.md`'s Frontier column to the tree's own row 1 — both named `.7.4.5`, and two files agreeing is precisely what it asks for. `TASK-STATUS` requires one `Status:` line per leaf and never reads a table. `TABLE-ARITY-RATCHET` reads cell counts, never cell meaning. The defect lives in the seam between three checks each doing its job.
- ⚠️ **The rule is not "no closed leaves in the table".** Rows `1a11`–`1a18` deliberately carry `done` leaves as history and should. What is wanted is that a row's status column AGREES with its leaf's own `Status:`, and that row 1 names a `pending` leaf.
- ✅ Owned by `.11.22` with its acceptance: census the whole table before proposing a rule (`.11.6`), permit the `done`-as-history case explicitly, and if a gate is registered, see it RED against both a stale `pending` row and a row-1 pointer at a closed leaf. The pointer itself is corrected to `.11.2.4`, the tree's true next-ranked pending leaf.

## 2026-09-18 — Unformatted Rust reached `main`, because nothing checked per commit (`SIGNOFF-REPAIR.11.21`)

🔴 **`crates/reasonbraid-server/src/git.rs` sat on `main` failing `cargo fmt --all -- --check` at two sites, and was found only because a LATER leaf happened to run the check while verifying a file it had not touched.**

- ⭐ **A policy gap, not a lapse.** `CLAUDE.md` §16 routes the full CI to pre-push and the per-commit gate to the doctrine enforcer, whose 18 registered checks contained no formatting check; `cargo fmt --check` appeared only in `COMMIT.md`'s pre-push list. Pushes go out in batches of ~300 commits, so unformatted code could sit on `main` for as many commits as that — and did, from REPAIR-0251/0252.
- ✅ **Measured before the rule was proposed (`.11.6`).** Three runs each, warm: `cargo fmt --all -- --check` at **0.81 / 0.66 / 0.66 s**, the enforcer at **12.57 / 12.52 / 13.00 s**. About **5 %**, and `.11.5`'s constraint holds — nobody routes around two-thirds of a second.
- ⛔ **Leaving it in the pre-push list was rejected on evidence, not taste**: that is exactly where it already was, and it is the arrangement that let the defect ship. A check running once per ~300 commits cannot say WHICH commit broke formatting, so the cost of finding out is the thing being economised on.
- ✅ **`RUST-FORMATTING` is now doctrine 16 of 19**, whole-workspace rather than staged-scope — deliberately, because the defect was in a file no staged change touched. ⚠️ That is affordable only because the workspace is formatted as of this commit; a strict check over pre-existing debt would have needed a ratchet like `TABLE-ARITY`.
- ✅ **Seen RED, then green.** `let  _deliberately = 1 ;` injected into `git.rs::expired` produced `❌ RUST-FORMATTING … 1 doctrine breach(es) — commit blocked`, naming the file by repository-relative path; restored byte-identical, the gate returns to green over 19 checks. Its own self-test reaches the TOOL rather than just the verdict, and FAILS rather than skipping when `rustfmt` is absent.
- 🔴 **The registry's own guard caught me while registering it.** My first description spelled the command in backticks — and each registry entry is a bash double-quoted string, so that is command substitution the driver would EXECUTE on every commit and in CI. `DOCTRINE-REGISTRY` refused it by name. `DOCTRINE_ENFORCEMENT.md` documents this exact trap from a previous occurrence; the gate worked, and so would have reading the section first.
- ⚠️ A number corrected in passing: the enforcer costs ~12.6 s, not the 3.15 s of `.11.4.3.1.7.2`. That figure is anchored to its own leaf rather than wrong — the registry has grown — so the current one is named beside it rather than edited into history.

## 2026-09-18 — A route key that was too loose and too tight in one expression (`SIGNOFF-REPAIR.11.8.1`)

🔴 **The route-documentation census was wrong in BOTH directions, and its own self-test asserted the defect rather than catching it.**

- **The mechanism, one function.** `stem()` DELETED every `{param}`: `re.sub(r"\{[^}]*\}", "", route).replace("//", "/")`. That is too loose and too tight at once.
- ⛔ **Too loose — a false `described`.** `/v1/snapshots/{snapshot_id}` and `/v1/snapshots` collapse onto one key, so a contract line for one credits the other. Re-derived by hand: `/v1/calls` was counted documented because `GET /v1/calls/{call_id}` appears at `authority.md:833`, while a grep for a bare `… /v1/calls` contract line returns nothing. `/v1/policy-publications` was credited the same way by `POST /v1/policy-publications/{id}/publish`.
- ⛔ **Too tight — a false `absent`.** A parameter in the MIDDLE of a path leaves a stem no book writes: `/v1/snapshots/{id}/derivations` became `/v1/snapshots/derivations`, though the book names it at `deployment.md:584`.
- ✅ **21 of 104 routes misclassified** — 18 wrongly `absent`, 3 wrongly credited as documented. Corrected today: **104 routes — 49 described, 3 mentioned, 52 absent**. And apples-to-apples, at `.11.8`'s own commit `0fbb85f` in a throwaway on-volume worktree with only the key replaced: **103 routes — 30 described, 3 mentioned, 70 absent**, against its published **22 / 1 / 80**.
- ⚠️ **A correction to this file.** REPAIR-0254's entry attributed `35 described / 68 absent` to `.11.8`. Those are what the DEFECTIVE instrument reports over today's corpus; `.11.8` published `22 / 1 / 80` over 103 routes. A number needs its producer *and* its moment.
- ✅ **The repair is to normalise, not delete** — every `{whatever}` becomes `{}` on both sides, so a key matches the book's own spelling and two routes stay two keys — plus a right match edge so `/v1/snapshots` cannot match inside `GET /v1/snapshots/{}`.
- ⛔ **No LEFT edge, measured rather than assumed.** It guards a hazard this surface has **0** of, and it would cost a real answer: `curl -i "$RB_URL/v1/admin/grants"` would read `absent`. Instead `tail_collisions()` prints the count every run, so the day the hazard appears the reader is told — the assumption is checked, not commented.
- 🔴 **The old self-test's third arm asserted `stem("/v1/calls/{call_id}/respond") == "/v1/calls/respond"`** — codifying a key no book writes as the correct answer. Not a control that could not see the defect: one that demanded it. Falsified five ways, each red by name; a sixth attempt was a bad mutant of mine (`return [] or sorted(...)` evaluates to the sorted list) and is recorded rather than counted.
- ⛔ Unchanged: 52 is not 52 defects, `described` is still a proxy, and no gate is proposed — a better key does not touch `.11.6`'s reasoning.

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated twenty-eight times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

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
