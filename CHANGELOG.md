# CHANGELOG.md

## 2026-09-15 — Take the rustls advisory, and enumerate the four lines it moved (`SIGNOFF-REPAIR.11.4.7.2.2`)

✅ **One of the three red things is now green.** `cargo deny check` returns `advisories ok, bans ok, licenses ok, sources ok`, rc=0.

- `cargo update -p rustls` — `0.23.43 → 0.23.45`, for **RUSTSEC-2026-0285**. ⭐ The lock diff is **four lines**, enumerated in the leaf rather than summarised: a lockfile bump is a supply-chain change, and "just a patch update" is a claim about a file nobody read. Nothing transitive moved; the 37 other dependencies behind latest are deliberately untouched, because this leaf owns one advisory and not a refresh.
- ⭐ **The exposure at its real width:** rustls accepted TLS 1.3 handshake messages sent at the wrong encryption level after a key-changing message in the same record. ⛔ The transcript stays authenticated, so this is not handshake forgery — the effect is that a peer may send in plaintext what should have been encrypted without rustls rejecting it.
- **No other advisory was masked**: `cargo deny` reports all four sections every run, and the failing run carried exactly one `error[vulnerability]` and zero warnings.
- 🔴 **NO REGRESSION, measured with the strongest run this project has had:** `cargo test --all --locked --no-fail-fast` reaches **94 test binaries** — **102 suites ok / 1 failed, 815 tests passed / 1 failed**. The single failure in the entire workspace is `.11.4.7.2.4`'s known browser control, which predates this change.
- ⭐ **That also prices `--no-fail-fast`, which `.11.4.7.2.4` asked for as a measurement rather than a preference.** Default fail-fast reached **11** binaries and left `cli`, `core`, `node` and `server` unknown; `--no-fail-fast` reached **94**, at **319.3 s** of test execution (one suite is 135.23 s of that). The cost of knowing was run time, not extra failures.

## 2026-09-15 — Re-derive G1–G2's sixteen claims, and find three things red right now (`SIGNOFF-REPAIR.11.4.7.2`)

🔴 **The verdicts are the smaller half. The finding is that `make deny`, `make secret-scan` and the workspace test suite all FAIL today — and nothing has been running the first two since Phase 1.**

- **3 stand, 4 narrow, 9 must be re-earned**, each with the command that produces it. Record: `docs/decisions/2026-09-15_g1g2-sixteen-claims-re-derived.md`; the original is byte-unchanged since it was written.
- ⛔ **`grep -c 'deny\|secret-scan\|gitleaks'` returns 0 for `check_doctrines.sh`, 0 for the project slot and 0 for `.githooks/pre-commit`.** Both supply-chain gates live only in `.github/workflows/supply-chain.yml`, which runs in remote CI — and remote CI has never run (blocker C1). `origin/main` is at 2026-09-12; the gitleaks finding entered on 2026-09-13, inside the unpushed range, so even a CI run would not have caught it. ⭐ C1 is not only a limit on what may be CLAIMED; it is why two gates have been red with nobody able to see it.
- **`cargo deny check` rc=1** — RUSTSEC-2026-0285, `rustls 0.23.43`, fixed in ≥0.23.45. ⚠️ Dependency drift, not advisory drift: rustls was not in the lock at the gate commit (positive control: `tokio` was). Owner `.11.4.7.2.2`.
- **`gitleaks detect` rc=1** — one `generic-api-key` hit on a test fixture named `idempotency_key`. A false positive, and the gate is red anyway. Owner `.11.4.7.2.3`.
- 🔴 **`cargo test --all --locked` rc=101, aborting at the 11th test binary** — `cli`, `core`, `node` and `server` never execute. The failing control asserts `cleanup_confirmed == true` while the assertion above it passes, which means REPAIR-0089's fix is working: that repair deliberately made the product report `cleanup_confirmed: false` rather than claim an unobserved termination. ⛔ Reproduced on an idle machine (81.30 s, `elapsed_ms` 40022) after the first run was under load (41170) — two load states, one result, so not a flake. Owner `.11.4.7.2.4`.
- ⛔ **The deferral count is SIX, not five** — the record says five twice and lists six, and `git log -S` puts the sixth row in the same commit as the sentence, so it was inconsistent the day it was written.
- 🔴 **Deferral #5's revisit trigger fired and nothing revisited it.** The fuzz baseline was deferred until "the first untrusted parser — Phase 4's resource packs". Phase 4 closed; `fetcher.rs`, `git.rs`, `reasonbraid-extract` and `-browse` now parse untrusted input; `git ls-files | grep -ic fuzz` returns **0**; the word appears in exactly one decision record — the one that deferred it. Owner `.11.4.7.2.1`.
- ⭐ **A deferral with a trigger nobody checks is an omission with extra steps.**
- **Also opened `.13.1.1`, found answering a question from the director rather than by a leaf**: `verify_ladder` — `PHASE-8.4.4`'s five-rung fail-closed adapter-load ladder — has **no production caller**. Six of its eight `git grep` hits are inside its own `#[cfg(test)]` module; `pub use` hides it from `dead_code`. And `AllowedCapabilities::dev()`, the crate's only ceiling, sets `tool_support: true`. ⛔ Nothing unverified loads today — the adapters are compiled in — but the control is inert for the third-party case it exists for.

## 2026-09-15 — Grant the licence the manifests have been declaring (`SIGNOFF-REPAIR.13.2`)

✅ **Blocker B5 is closed, and with it the last row that named the director.** The repository declared `MIT OR Apache-2.0` in every manifest and contained no licence text at all. A licence expression is metadata; the operative default for a **public** repository without the texts is ordinary copyright, so readers held none of the rights the expression appeared to offer.

- **Shipped:** `LICENSE-APACHE` and `LICENSE-MIT` at the root, a four-line `## License` section in `README.md`, and `scripts/check_licence_grant.sh` registered as the **9th project-specific check** inside `PROJECT-SPECIFIC` (the enforcer's top-level count stays at 18). Copyright holder: **Richard DJE** — the one input not derivable from the tree. ⚠️ `git log` shows a committer, which is evidence of authorship and not a statement of ownership; inferring one from the other is exactly the guess a legal document must not contain.
- ⛔ **The EXPRESSION is untouched.** The choice was made when the first manifest was written. Narrowing or broadening it here would have been a relicensing act wearing the clothes of a completion.
- ⭐ **Neither text was typed.** `LICENSE-APACHE` is byte-identical (`cmp -s`) to the copy **104 crates** in this workspace's own registry ship; `LICENSE-MIT`'s body is whitespace-identical to the copy **127 crates** ship. A licence's operative content is its exact words, and a *plausible* paraphrase is the dangerous kind — nothing in a fluent reconstruction signals which clause drifted.
- ⚠️ **Correction inside the same commit: "ten manifests" was never measured.** This leaf, the blocker register, `LIVE_STATUS.md` and the book all said ten. The census says **13** — the workspace root plus 12 crates, 5 literal and 8 inherited. Named at every site rather than silently swapped. ⭐ It survived because a wrong count that changes no conclusion is the kind nobody re-checks; it was caught only because the new gate had to enumerate the manifests to gate them.
- ⛔ **487 commits of project history passed under every gate set this project has ever had, and not one of those sets contained a licence check** — `git log --oneline -- 'scripts/check_licence*'` returns nothing before this commit, and the current 18-doctrine set has only been in force for 67 of them (since `86dd272`). They governed documents, code, tables and claims, and none governed the grant. `LICENCE-GRANT` closes both directions: a declared identifier must have its text, **and a licence file must be named by the declared expression**, since a file left behind after an expression changes still reads as an offer.
- 🔴 **Falsifying the new gate found two defects in it, and its self-test was green through both.** Sentinels were literal substrings while the real MIT text is hard-wrapped at ~55 columns, so the gate **failed a perfectly valid licence**; and all four Apache sentinels sat in the first five lines, so a five-line stub passed. Both because the fixtures were hand-written and therefore tidier than the shipped files. Fixtures are now the real files mutated, sentinels span the whole document with a length floor, and both holes are pinned as named cases. Promoted to `docs/knowledge/a-self-test-cannot-be-tidier-than-the-real-input.md`.
- ✅ **A1 closes in the same pass**: its register row still called it "the only item genuinely awaiting a director decision" after REPAIR-0196 had dissolved the question — a stale row on the table built to stop rows going stale.
- ⛔ **B4 is unaffected.** A licence grants permissions in the work; it says nothing about the name on it, and `ReasonBraid` remains an uncleared working name.
- Record: `docs/decisions/2026-09-15_licence-granted-mit-or-apache-2.md`. Public register: `docs/book/src/blockers.md`.

## 2026-09-15 — Correct a security finding I overstated the same day (`SIGNOFF-REPAIR.11.4.7.1`, `.7.2`)

⛔ **The §16.12 line-(4) finding published four hours earlier was broader than the truth, and this corrects it at every site rather than editing it away.** The defect is real and still open; it is **narrower and sharper** than I wrote.

- **What I published:** that an IP-literal destination — "the initial URL's or any of the five auto-followed redirect hops'" — never reaches `ClassifiedDns`, and that the DNS hook is R1's "ONLY destination control".
- 🔴 **Both halves overstate it.** `GitFetcher::classify` is a genuine PRE-FLIGHT second control, and it handles the literal case explicitly: `if let Ok(ip) = host.parse::<IpAddr>() { return allow_ip(&self.policy, ip); }`. The initial URL is classified, literal or not.
- ⭐ **So the defect is the REDIRECT HOPS ALONE.** The pre-flight checks the first destination and does not run again; `Policy::limited(5)` then follows up to five hops with only the DNS hook behind them, and that hook never fires for a bare address. A request the caller makes directly is checked; a hop an origin sends it to is not.
- ⚠️ **That changes the exposure's shape, which is why it mattered enough to correct promptly.** It requires a Git origin that REDIRECTS into a private or link-local range — not a caller naming one, which the pre-flight already refuses.
- Corrected at all four tracked sites plus the book: the `.7.2` annotation, `.11.4.7.1`'s verdict, `docs/decisions/2026-09-15_g6g7-shipped-lines-re-derived.md` and `LIVE_STATUS.md`. Each NAMES the superseded claim rather than replacing it silently — the practice this tree applies to inherited numbers, applied to my own.
- ⭐ **Found by reading the source I was about to repair.** The overstatement survived a decision record, a leaf, a status entry and a book page, because every one of them was written from the same reading. The only thing that caught it was opening `git.rs` again with a different question — *what validates the initial URL?* — rather than re-reading what I had already concluded.

## 2026-09-15 — Audit the metrics read against the tenant its caller already has (`SIGNOFF-REPAIR.3.5.2.1`)

The director asked for the blockers to be unblocked with rationale. This one was held for two days on a question that measurement dissolves.

- 🔴 **All three shapes the leaf considered shared a false premise: that the route must NAME a tenant.** It does not. `migrations/0007_identity_store.sql` declares `human_principals.principal_id` and `agent_roles.role_id` as **PRIMARY KEY**, each with one `tenant_id` — so a principal belongs to exactly one tenant, structurally, and the tenant is DERIVED from the authenticated caller. The same argument `.3.5.1` used about `nodes.node_id`.
- **The fourth shape costs none of what the other three cost.** No `?tenant_id=` parameter, so no caller breaks. No new authority-selection path, because the tenant is known before anything is selected. No behaviour that starts failing when a caller gains a second grant. ⛔ And **no stored-format change**: the route uses the ORDINARY `authorize_guarded`, recording `boundary_checked`, so `TenantAdminInspection` gains no variant and no older reader meets a discriminant it cannot decode.
- ⭐ **That also honours `.3.5.2`'s prohibition by construction rather than by luck.** `authorize_tenant_admin_inspection` applies the frozen-boundary carve-out, which exists so an administrator can inspect AUTHORITY state during a revocation; process counters are not authority state. Choosing the ordinary path is what keeps the prohibition intact.
- ⚠️ **One declared narrowing**: the gate asked `subject_id = $1` with no tenant predicate — any `tenant_admin` grant in ANY tenant. It now requires an administrator of your OWN tenant. Nothing binds a grant's tenant to its subject's (that is `R-85-1` clause 2's shape, owned at `.3.3`), but both producers — development enrolment and card import — create the principal in the grant's own tenant, so no reachable caller loses access. The deliberate half of the width is intact: an admin still sees ALL the process's counters.
- ⭐ **Deleting the hand-rolled gate removes a SEVENTH spelling of grant-liveness**, and the loosest: it omitted `subject_kind`, and its `(expires_at IS NULL OR expires_at > now())` branch was dead — the column is `NOT NULL`, so the disjunction read as leniency that was never there.
- **Verified:** `command_api` **38 passed** (37 + this control) and `authority` **22 passed**, rc=0, cluster removed. ⭐ **FALSIFIED** against the exact superseded handler restored from `HEAD`: **37 passed / 1 failed**, the one failure being this control at `the admitted metrics read carries its admission receipt`. The other 37 pass throughout, so the neutralization discriminates rather than breaks.
- ⚠️ The neutralization was made **self-reversing** — the restore was chained to the wait that consumed its result, and the repaired file copied aside first. A neutralized working tree is the one state a handoff must never be left in, and that should not depend on the session surviving to undo it.

## 2026-09-15 — Re-derive G6–G7's seven shipped lines, and find the one still open (`SIGNOFF-REPAIR.11.4.7.1`)

**Three must be re-earned, two are narrowed, two stand.** ⛔ The gate's conclusion is UNCHANGED — G6–G7 remains NOT MET for Internet exposure — and `2026-09-08_phase7-gate-record.md` is byte-unchanged: `docs/decisions/` supersedes rather than mutates, so the new record ADDS to it.

- ⭐ **"Since" is exact, and one command settles it.** The gate record closed **2026-09-08**; the earliest corrective repair is **2026-09-09**. Comparing every repair's timestamp against the record's returns **zero** that predate it — so all 110 code-touching repairs are "since" and no verdict has to argue about ordering.
- **(2) enrollment / rotation / revocation / tenant-isolation — MUST BE RE-EARNED.** The line names FOUR things and the corpus falsified all four: `.4.1.1` and `.3.5.1` (enrollment), `.4.2.1` and `.4.2.9` (rotation), `.4.1.3` and `.3.1` (revocation), and five separate cross-tenant reads and writes.
- **(3) non-escalation / confused-deputy — MUST BE RE-EARNED.** `.9.3.1` is the confused-deputy shape itself — five sites asking whether an authority EXISTS where the question is whether the caller HOLDS it, over grant ids derivable from principal ids.
- 🔴 **(4) SSRF / rebinding / redirect / archive-bomb — NARROWED, and the only one of the seven still an OPEN defect.** Two packs, opposite designs: `fetcher.rs` sets `Policy::none()` and re-classifies every hop manually; `git.rs` sets `Policy::limited(5)` with a DNS hook as its only control — and hyper-util 0.1.20 says in its own source, *"If the host is already an IP addr (v4 or v6), skip resolving the dns and start connecting right away."* The R1 header meanwhile claims "every dial (redirect hops included) passes the destination policy". ⚠️ Source-measured; runtime reproduction pending. Owner `.7.2`, whose goal line names it verbatim; record `R-44-45-1`.
- **(6) supply chain — STANDS**, narrowed by the dependency ledger's still-empty `tested_versions` for MCP and A2A. **(7) backup restore — NARROWED**: the exercise runs, `R-59-2`'s fixture defects are open. **(8) breaker / storm — MUST BE RE-EARNED**: `.3.3.4.9` found the breaker's two verbs mutating on the connection pool with no transaction, no guard and no record. **(10) runbooks / disclosure — STANDS.**
- ⛔ The three EXTERNAL gaps are restated unchanged with their triggers and remain out of local scope (`.13.1`).
- ⚠️ **NOT claimed: that re-earning is repair work.** The defects are repaired; what is missing is the coverage measurement that would let each line be counted again — a different activity, owned per line.

## 2026-09-15 — Census the five gate records, and find a wrong number inside a release gate (`SIGNOFF-REPAIR.11.4.7`)

Can this project still state its own release-gate position? The leaf was opened by the director's question about Internet exposure and widened by his challenge from one gate record to five. This commit censuses them and splits; it re-derives nothing yet, deliberately.

- **The population, measured before anything was proposed: 35 verdict claims and 19 deferrals** across G1–G2, G3, G4, G5 and G6–G7. ⚠️ The mechanical regex finds 7 verdict-table rows and 19 deferral items and scores **zero** on Phase 7, whose seven shipped §16.12 lines and seven-row unsupported matrix are PROSE — so the regex is a locator and the population is hand-classified, which the leaf says out loud rather than publishing an incomplete number.
- **The repair corpus to map against them: 110 of 198 commits touched product source or a migration.** ⛔ The first pathspec returned **8** and was wrong — `git show -- 'crates/*/src'` matches a path that IS `crates/<x>/src`, not the files beneath it, the trap `MEMORY.md` already warns about. The corrected form is proved in BOTH directions: two `.rs` files for a commit that has them, zero for a docs-only one. ⭐ A pathspec returning a small number is indistinguishable from a small population until the negative control runs.
- 🔴 **The census found a defect before any re-derivation ran, and it is arithmetic.** `2026-09-07_phase1-gate-record.md` says "**Met** — with **five** named deferrals" and "the **five** numbered deferrals above", and its own table lists **SIX**. `git log -S` over the sixth row's text returns exactly one commit — the gate package itself — so the count was **wrong when written**, not overtaken. ⭐ This is `R-80-82-1`'s "four places" one level up: a count written while reading, in a RELEASE GATE. A deferral is a named limitation with a revisit trigger, so an undercount is one limitation not carried forward. The book repeated it and is corrected to six with a note; the record is superseded rather than edited, owned by `.11.4.7.2`.
- 🔴 **And a gate cannot see the claim it exists to stop.** `scripts/check_visibility_policy.sh` was written because the superseded private-repository instruction "had already leaked past two" reviews. All four of its sentence shapes anchor on the full word `repositor(y|ies)`; **two live instances written with the abbreviation are invisible to it** and neither is in the exceptions file. ⚠️ Both are correct HISTORY — dated before the 2026-09-09 correction — which is exactly why nothing ever failed and the gap survived. New owner `.11.2.4`. ⭐ **The gate then proved PRECISE by refusing the leaf that documented it**: writing its shapes out longhand tripped it, correctly.
- ⛔ **The same blindness caught the instrument measuring it.** The census's first search used `\brepo\b`; `git grep -E` does not support `\b`, so it returned no match — blind exactly as the gate is, and caught only because a known line existed to test the tool against. **A search that returns nothing is a claim about the SEARCH until a positive control says otherwise.**
- **Split four ways on the record boundary**, because re-deriving a gate line needs that phase's suites: `.11.4.7.1` G6–G7 (8 claims, first — it has reproduced counter-evidence in hand), `.11.4.7.2` G1–G2 (16, the largest, carries the undercount), `.11.4.7.3` G3 (7), `.11.4.7.4` G4+G5 (4 — one DECLARED fold: both discharge their gate by SUBTRACTION, so both re-derive by asking whether the absent claim is still absent).
- ⛔ The three EXTERNAL G6/G7 gaps are unchanged and out of scope for all four children: the externally reviewed threat model, the prompt-injection suite, and the penetration test. `.13.1` owns them as blocker rows.

## 2026-09-15 — Delete the superseded direction services, and make the book stop contradicting itself (`SIGNOFF-REPAIR.3.3.4.12.2`)

⚠️ The book said, in ONE chapter, both that a card import "is now fenced by a revocation from **either** side" and that nothing there "can fence them" — the second in the present tense, naming a leaf that had already landed. Nothing unsafe follows: the stale half UNDERSTATES what is protected. What follows is that the project's primary review surface disagreed with itself about a property two leaves spent commits proving.

- 🔴 **`.3.3.4.12` put the three federation direction verbs on the guarded shape and left the three unguarded ones standing.** `git grep -c "federation::propose\|federation::accept\|federation::revoke" -- crates` returns rc=1, no match — no caller anywhere — and `cargo check -p reasonbraid-server --all-targets --locked` returns rc=0 after the deletion, which quantifies over every test, bench and example target at once. ⭐ The build is the stronger instrument: a grep can miss a re-export, and the compiler cannot.
- 🔴 **Why the discipline did not fire on its own.** `.3.3.4.8` deleted its two superseded revocation services and `.3.3.4.11.3` deleted three unguarded bridges — both times the compiler flagged them. These three were `pub` on a `pub mod`, and `dead_code` does not apply to a public item by construction. "The compiler would have told me" is FALSE for anything exported, and the only instrument there is an explicit caller census.
- **DECISION: delete, with the rule named rather than assumed.** `reasonbraid-server` is `0.1.0` and unpublished, ADR-001 forbids assuming the crate name is even obtainable, and §21 says crates follow SemVer only **after their public contract is declared**. No contract is declared, so `pub` here is a visibility keyword rather than a promise.
- **Three present-tense sites corrected**, located by enclosing construct rather than by line number: the doc on `federation::has_effective_recruitment_agreement_in_tx`, the comment above `profile_admin.rs`'s import transaction, and the book's "Importing a portable agent card" section. Each RECORDS the superseded sentence rather than silently replacing it, because that sentence had propagated to three files and stayed true-sounding in all of them for two leaves.
- ⛔ **`LIVE_STATUS.md`'s two mentions and the tree's four are deliberately untouched** — they are dated records of what `.3.3.4.11`/`.11.3` measured, correct as history, and `tests/federation.rs` already writes it in the past tense. Editing them would destroy the provenance that shows when the limit was real.
- **Promoted to `docs/knowledge/a-repair-owns-every-sentence-that-states-its-limit.md`**: a repair that falsifies a limitation owns every sentence that states it, in the same commit — and the instrument is `git grep` for the limitation's OWN WORDS, not for the leaf id, because a sentence that becomes false almost always quotes itself. The record also carries the `pub`-item blind spot and the split-arithmetic check from `.11.9.1.3.2`.
- 🔴 **The blocker register ships: `SIGNOFF-REPAIR.13`.** The director instructed, three times in one session and each time stronger — a blocker is handled promptly or at minimum put front and center and discussed explicitly; it is **reminded until he actively interacts with it**; it is **listed in the mdBook and task-tree owned/tracked**. Eight rows, each with who it is blocked on, its owner, what it does NOT block, and an `Ack?` column starting at `no`. The public face is the new **[blockers chapter](docs/book/src/blockers.md)**, in `SUMMARY.md` after the qualification page. ⭐ Three of the eight had no leaf at all: the three external G6 preconditions lived as prose inside a CLOSED phase's gate record, the licence decision was tracked by nothing anywhere, and "remote CI has never run" was a cadence note in `COMMIT.md`. New leaves `.13.1`–`.13.3`; rows already owned point at `.3.5.2.1` and `.11.4.7` rather than being duplicated.
- ⛔ **The defect the register repairs is not that blockers were unowned — it is that being owned made them invisible.** `.3.5.2.1` sat correctly held, correctly acceptance-bearing and correctly at frontier row 8 for two days. Every rule was followed and the effect was burial on a slower timescale than an unowned finding would have suffered.
- 🔴 **`SIGNOFF-REPAIR.11.4.7` opened, then WIDENED at the director's challenge** from the G6/G7 record alone to **all five gate records**. Four of the five make countable "shipped / Met" claims, every one counted BEFORE the full source read — and the review has since reproduced cross-tenant defects inside G6/G7 line (2)'s own subject, tenant isolation (`.6.1.1`, `.3.5.3`, `.3.3.4.10.3`, `.6.1.2`, `.9.2.1.2`). ⛔ The records are not dishonest: each claims the evidence its suites then carried and could not know their coverage. Frontier row 1b.

## 2026-09-15 — Reconcile tranche 4b, and find the two mechanisms `.3.4`'s own split dropped (`SIGNOFF-REPAIR.11.9.1.3.2`)

Eight census records, 40 clauses, every one read at the source. The largest child this activity has executed, and the one carrying `.11.9.1.3`'s single declared deviation — the `.3.5` singleton folded in rather than run alone.

- 🔴 **The finding is about `SIGNOFF-REPAIR.3.4`, the leaf whose rarity ranked seven of the eight records.** It is `active`, all FIVE of its children are `done`, and **two of its own goal line's mechanisms have no child**. The line reads "actor/subject participation **and consent**; bind replay hashes **to target** and authority context"; `.3.4.1`–`.3.4.5` carried delegability, the authority-CONTEXT half of the hash, cache freshness, the codecs and ADR-009. ⛔ How it stayed invisible is arithmetic: the split's own bullet says "five children along the **four** mechanisms the goal line names", and the goal line names five.
- ⭐ **A third shape of lost ownership, named rather than absorbed.** `.11.9` found leaves that never cited a routed record; `attach` is for a goal line that does not make a clause visible. This is neither — the goal line names the mechanism VERBATIM, so no census reading that text can find the gap. ⛔ The ledger's closed state set is deliberately not extended, because the required action is `unowned`'s; the check the shape earns is written into the ledger's vocabulary instead: at every split, count the goal line's mechanisms against the children and make the split's own sentence reconcile. New owners `.3.4.6` (the hash does not bind the target) and `.3.4.7` (whether a delegated subject consents — `grep -i consent` over ADR-009 returns rc=1, the word is nowhere in it).
- 🔴 **A role can auto-initiate exactly ONCE per tenant, for ever.** `create_thread_auto`'s idempotency key is `format!("auto_{}_{}", role, tenant_id)` — the role and the tenant alone — against an `idempotency` table keyed `(tenant_id, idempotency_key)`, so a second initiation replays the first thread when the body matches and answers `idempotency_mismatch` when it does not. ⭐ `R-76-77-3`'s coverage clause becomes its cause: the named test makes one success and then only POLICY refusals, so no arm of it is a second VALID initiation. Annotated at `.5.2`, whose goal line already says "make repeated legitimate auto initiation possible".
- 🔴 **The R1 Git acquisition opens its repository with gix's DEFAULT permissions, over an untrusted remote.** `gix::init_bare` is `ThreadSafeRepository::init(…, create::Options::default())`, and gix 0.87.1 ships the remedy it does not use — `open::Options::isolated()`, documented as "prohibiting accessing the environment or spreading beyond the git repository location". The operator's global and system git configuration and the git environment variables are therefore honoured while fetching a caller-supplied URL, against §12.5's default refusal of hooks, filters and alternates. ⚠️ Distinct from `.7.2.1`, which bounded where gix WRITES. Attached to `.7.2`.
- 🔴 **The quarantine gate does not exist anywhere in the product**, measured as a zero-hit grep rather than by reading call sites: `git grep -c "quarantine_status" -- 'crates/*/src'` returns rc=1, no match, so migration 0028's column is written by its `DEFAULT 'none'` and read by nothing. ⭐ The §9.8 code that would report it, `evidence_quarantined`, is constructed nowhere either — and the book's errors chapter already lists it among the eleven registered codes this build never emits, without anyone noticing the schema was waiting for it. Attached to `.7.4`.
- 🔴 **`.3.3.4.12` left the three superseded federation direction services standing, and the book now contradicts itself about them in ONE chapter.** `git grep -c "federation::propose\|federation::accept\|federation::revoke" -- crates` returns rc=1, no match — no caller anywhere — yet they remain `pub` on a `pub mod`, which is why the compiler's dead-code analysis cannot see them and why `.3.3.4.8` and `.3.3.4.11.3`, which DELETED their superseded bridges, set no precedent that fired. Meanwhile `authority.md` says an import "is now fenced by a revocation from either side" under one heading and "take no tenant authority guard at all … That ordering arrives with `SIGNOFF-REPAIR.3.3.4.12`" under another — a landed leaf, in the future tense. ⛔ The error is conservative, so nothing unsafe follows. New owner `.3.3.4.12.2`.
- ⭐ **Two clauses were refuted at HEAD and both were RIGHT when written**, told apart by `git show` at the review baseline. At `9c2d2ba` the grant loader was `ORDER BY valid_from DESC LIMIT 1`, so a newer narrow grant genuinely shadowed an older broad one; `select_authority_in_tx` now pages every active candidate and returns the first ALLOWED one.
- ⚠️ **One clause narrowed, and the narrowing is the useful half.** `expired_and_revoked_grants_are_denied` exercises no revocation — `git grep -c "GrantStatus::Revoked"` over that suite returns rc=1, no match — but the invariant IS covered, by `.3.3.2`'s pure evaluator controls. The defect is a test name and a doc comment that overclaim, and a reader who trusts the name stops looking where the coverage is.
- **40 clause rows**: 16 `handled`, 11 `owned`, 7 `attach`, 4 `unowned`, 2 `declined`. All seven `attach` clauses written into `.5.3`, `.7.2` and `.7.4` in this commit. `--classified` rc=0 at 244 rows with all 47 `attach` clauses named by their leaves; `--self-test` 49 controls pass. No product code, schema, test or script changed.

## 2026-09-15 — The action set extends, the target selector does not (`SIGNOFF-REPAIR.9.3.4`)

Five administrative surfaces can now say **who** an authority belongs to and none can say **what it is authority over**. `.9.3.4` owned that question. The decision is taken and recorded; the implementation is decomposed.

- 🔴 **The leaf's own premise was false, and it is the premise that would have declined the extension.** It says `GrantAction` and `TargetSelector` are "closed wire vocabularies and §9.8 publishes a stable registry", and reads across to `.11.7.1`'s frozen-roadmap hold. §9.8 is the **reason-code** registry — a different vocabulary. `git grep -c "GrantAction\|TargetSelector" ROADMAP.md` returns **0**. The freeze does not reach these types, and the coupling between the two leaves dissolves with the premise.
- ⭐ **The project had already ruled on how to do it**, in `docs/decisions/2026-09-06_authority-boundary.md`: *"Add an action/ceiling by extending the checker FIRST — a grant that exceeds the boundary must be unrepresentable, not merely unapproved."* Prescribed, with an ordering constraint the children inherit.
- ⛔ **`TargetSelector` does NOT extend, measured rather than preferred.** `Threads { threads: Vec<ThreadId> }` enumerates its objects at grant time. That works for threads, which exist before anyone is authorized over them; it cannot work for the case that matters, because the ordinary flow publishes a **new** publication whose id does not exist when the grant is issued. A `Publications { … }` variant would be unusable for exactly the operation it was added for. The residual is accepted and published: narrowed by **verb**, still tenant-wide by **object**.
- 🔴 **The extension is a MIGRATION, not an addition.** `migrations/0004_authority.sql:15` stores `permitted_actions` as a JSONB array of **wire names**, so no stored boundary contains a name that postdates it. The moment a verb requires a new action, every already-enrolled tenant's boundary fails to permit it and that verb stops working for them — fail-closed, which is the safe direction, and still live breakage.
- ⛔ **The leaf's site census was stale and this session staled it.** It says "the three sites"; re-derived, `git grep -n "grant_held_by(\|grant_is_live(" -- ':(glob)crates/reasonbraid-server/src/**'` returns **6**, the fourth added by `.9.2.1.2` two commits ago. Pinned rather than silently refreshed.
- **Decomposed into `.9.3.4.1`** (the vocabulary, the boundary checker, the default set, the stored-row disposition) **and `.9.3.4.2`** (the coverage check at all six sites), each with its own acceptance and its own red-first control. ⚠️ Not implemented here, and said plainly: `git grep -n "GrantAction::"` returns **207 references across 19 files**, and the change is three things at once with a live-breakage edge — the same reason REPAIR-0189 split `.9.2.1`.
- ⛔ **Holding is not covering.** `.9.3.1` found "names a grant" and "holds a grant" conflated; scope is the third term, and a site that checks holding while claiming scope is the same defect one level up.

## 2026-09-15 — The build-cost finding did not survive its own grading (`SIGNOFF-REPAIR.9.2.1.1.1`)

The director asked whether the findings had legs. One of them did not. Graded on `docs/CLAIM_VERIFICATION.md` §4.1's three axes, separately:

- **PROSE — fails.** *"This build is I/O-bound against the external repository volume"* and *"the mechanism is not CPU contention at all"* were never established.
- **NUMBER — one fails.** *"About 9 % CPU utilisation"* is true of the lib-only arm and false by a factor of **22** of the `--tests` arm it was published beside (**207 %**).
- **NAMED INSTANCE — fails, and that axis has no tolerance band.** *"The completed runs say the same thing"* is false; they disagree by more than an order of magnitude.

🔴 **Re-derived as an interval, and the interval refutes the correction as well as the mechanism.** Three runs of the same command, same work each time, ratio computed by the tracked function rather than by hand:

| run | wall | user | sys | cpu % |
| --- | --- | --- | --- | --- |
| 1 (coldest) | **627.0 s** | 49.7 s | 290.3 s | **54.2 %** |
| 2 | 148.9 s | 37.1 s | 287.4 s | **217.9 %** |
| 3 | 140.7 s | 34.7 s | 274.3 s | **219.7 %** |

- ⛔ **The CPU work is constant; the wall clock is not.** Total CPU per run 340.0 / 324.5 / 309.0 s — a 31 s spread — against a wall clock moving **4.5×**. The build does the same work every time; what changes is how long it waits. Run 1 spent about **472 s not computing**.
- 🔴 **Run 1 exceeds ten minutes, so `.9.2.1`'s original note was right and my correction of it was wrong.** It recorded ">10 minutes" under measured contention; I contradicted it from one warm run. Both are true, of different cache states, and neither is "the" build cost. A point estimate for a quantity whose own spread is 4.5× reads as precision and is luck — `CLAIM_VERIFICATION`'s stochastic rider, exactly.
- ⛔ **The mechanism is withdrawn, not replaced.** Run 1's ~472 s of non-CPU wall time is real and unexplained here. It is *consistent* with `docs/decisions/2026-09-12_checkpoint-cost-model.md`'s adjudicated cost (~21.9 s per newly-written executable; 472/21.9 ≈ 22, the right order for this crate's proc-macro dylibs) — but consistency is not evidence, and that ruling's signature is `user 0.00 sys 0.00` while run 1 burned 340 s of CPU. **The earlier ruling stands as the leading candidate.**
- 🔴 **The real failure was leg 2, and the standard names it.** *"The cheapest oracle is your own project's history… if you cannot name a difference, the earlier ruling wins."* That ruling had already adjudicated this exact signature and names `syspolicyd` — which I observed at 74 % and did not look up.
- ⭐ **The leg-3 gap is closed where it belongs.** `scripts/measure_check_phases.py` now records `user_seconds`, `system_seconds` and `cpu_percent` per phase, so the ratio comes from the instrument instead of a reader's division — and it gains a `--self-test` whose arms **are** the three measurements that were got wrong, so a change making a busy phase indistinguishable from an idle one now fails a control. It had none before, so it was one of the eleven the `SELF-TEST` gate did not reach.
- **What survives, with its conditions instead of as a fact about the crate:** warm, **141–149 s at ~218 % CPU**; cold, **627 s at 54 %**.

## 2026-09-15 — Publishing requires an authority the caller holds, not merely enrolment (`SIGNOFF-REPAIR.9.2.1.2`)

`publish_publication` and `mark_publication_effective` each checked `reader_tenant(…).is_some()` and nothing else. **Enrolment in any tenant was the whole predicate** for writing a publication into a Git repository and for declaring one effective. This closes the last of `.9.2.1`'s three children, and with it the parent.

- **One definition, two verbs.** `api::held_publication_authority` reads an `owning_authority` from the request and decides with `authority::grant_held_by` — `deployments::register_target`'s shape and `.9.3.1`'s predicate. The two verbs share no core to put the check in, so the shared thing is the function itself, and a later change cannot move one verb without the other. It runs **before** the path is resolved and before the publication is loaded, so an unauthorized caller reaches neither the filesystem nor the database.
- ⭐ **Naming a grant is not holding one, and that is the load-bearing leg.** Grant ids here are derivable (`grt_<principal_id>`), so a check that asked only whether an active grant *exists* would be no check at all — the exact conflation `.9.3.1` found on three other surfaces. The control names another principal's real, active grant and is refused, beside the matched pair where **only the holder differs** and the same request is admitted.
- **The positive arm differs by verb, and the difference is stated rather than smoothed over.** `effective` completes for the holder (200, state `effective`). `publish` is admitted and then stopped by the projection the seeded row names — a *later* refusal, asserted to be `invalid_command` and not to carry the authority message, which is what proves the gate let it through; the end-to-end success already lives in `the_publish_verb_drives_the_git_half`.
- ⚠️ **The limit is RECORDED, not papered over** (`.9.3.4`). No grant action and no target selector can *name* a publication, so a held grant is effectively **tenant-wide** for these verbs. Holding is strictly stronger than enrolment and is the best today's vocabulary expresses; the book says so in its own words rather than implying the verb is narrowly scoped, and `.9.3.4` is promoted to frontier row 1b because this repair has now had to write that caveat in two places.
- **The wire change's blast radius was measured, not assumed:** `git grep -rn "policy-publications"` over crates, docs, scripts and deploy returns route registrations, handler docs, `rb-server`'s argument help, two book lines and the phase records — and **no client outside `tests/policy.rs`**. Neither the CLI nor the MCP surface drives these verbs.
- **Verified:** `bash scripts/run_pg_tests.sh policy` → `14 passed; 0 failed`, rc=0.
- ⭐ **`.9.2.1` is closed, and the decomposition paid for itself**: three separate red-to-green cycles, three bisectable commits, each child's acceptance met on its own evidence — instead of one commit carrying three unrelated repairs, which is what REPAIR-0189 split it up to avoid.

## 2026-09-15 — A declared Git object id is now looked for before it is recorded (`SIGNOFF-REPAIR.9.2.1.3`)

`publications::mark_effective` took `git_object_ids: Vec<String>`, refused only the empty list, and wrote them into the row. Nothing opened a repository. The publication's Git provenance was whatever the caller said it was.

- ⛔ **The bypass was exercised by the suite meant to qualify it**, which is the sharper half of the finding. `git grep -n '"git_object_ids": \["abc123"' f3d77c9 -- crates/reasonbraid-server/tests/policy.rs` returns **3 sites**, and that suite was green — `13 passed; 0 failed`. Three fixtures proved the staged → effective transition using ids that resolve to nothing, and one of them is the deployment fixture.
- **The check is per id, in the core.** `publisher::missing_objects` answers which declared ids name nothing; `publications::mark_effective` calls it. `git grep -n "publications::mark_effective" f3d77c9 -- crates/reasonbraid-server/src` returns **2 production callers**, both in `api.rs`, so a handler-level repair would have left the sibling open and its own suite green — `.6.1.2`'s rule applied rather than re-derived.
- ⭐ **Containment became a type.** `resolve_repository` now returns a `PublicationRepository` newtype with no other constructor, so a verb that takes one cannot be reached with a path that has not passed `.9.2.1.1`'s check. A convention the next caller could forget is now one they cannot.
- ⛔ **An unparseable id and a well-formed absent one are the same answer**, and both are asserted: a check that validated only the shape would refuse `abc123` and accept forty zeros. A repository that fails to open is reported as a repository failure, deliberately not as a verdict about the ids — an unreadable store must not read as a forged publication.
- ⛔ **Per id, not per request:** one real id beside one fabricated id is refused and the message names only the fabricated one. "Does any declared id resolve" would let a real id launder a forged one.
- ⚠️ **`repo_path` is now required on `POST /v1/policy-publications/{id}/effective`** — a change to a shipped request shape. The design question was recorded rather than assumed: the publications table has no repository column, the standalone verb exists for a Git half performed out of band where no column could have been filled, and `.9.2.1.1` had already shaped the root as a container. The caller names the repository; the configured root constrains which ones it can name.
- **The three fixtures are re-seeded, not relaxed** — each asserts the same transition it always did, now with ids read back from a real bare repository under its own configured root.
- 🔴 **I mapped one of the three sites to the wrong test and the compiler caught it.** The third is in `the_drift_corrections_and_outcomes_ride_the_records`; the reviews test never drives the transition. Seeding the wrong function produced `cannot find value \`object_ids\`` and `unused variable: \`object_ids\`` in one build — two errors naming both halves of the mistake. A line number in a 3,600-line file is not a location; the enclosing function is.
- ⚠️ **A limit stated rather than implied:** an object is proved to exist, not to be this publication's own. A sibling repository inside the configured root holding a matching id would satisfy the check; binding the ids to the publication's own refs needs the repository recorded with the row, which is a migration and a different leaf.
- **Verified:** `bash scripts/run_pg_tests.sh policy` → `13 passed; 0 failed`, rc=0; `cargo test -p reasonbraid-server --test publisher` → `7 passed; 0 failed`, rc=0.

## 2026-09-15 — The server says where a publication may be written, not the caller (`SIGNOFF-REPAIR.9.2.1.1`)

`POST /v1/policy-publications/{id}/publish` read `repo_path` out of the request body and handed it to `gix::open`. Any enrolled principal named any path on the server's filesystem, and the module's own header said the publication went "into the LOCAL bare repository".

- **The deployment now declares one root** (`rb-server --publication-repo-root`), the request names a location inside it, and `publisher::resolve_repository` is the single predicate that decides. ⭐ **Both sides are canonicalized before they are compared**, which is what lets one question answer two escapes: `..` is a walk the caller spells, a symlink is one the filesystem spells on the caller's behalf — and in the symlink case every component the caller named *is* inside the root, so a string containment test refuses the first and admits the second.
- **Fail-closed, and that is the contract rather than a detail.** `api_router(pool)` keeps its signature and declares no root, so an unconfigured deployment answers the new `503 publication_repository_unconfigured` instead of opening whatever arrived in the body. A declared root that does not resolve to a directory refuses the **boot**, before the migrations run, so a typo cannot leave a changed database behind with no server on it. ⛔ `.11.12` is untouched: the secret-store ordering and the `--host` gate remain its to decide.
- 🔴 **The repair exposed a pre-existing fixture defect, in the very leg it was told to preserve.** `the_publish_verb_drives_the_git_half`'s "a publish into a NON-repository path refuses (the store contract's open failure)" ran **after** the publication had been driven to `effective`, so the STAGE check answered it and `gix::open` was never reached — green for the wrong reason for as long as it has existed. Found by pointing the leg inside the configured root and watching it fail with ``is at stage `effective` `` instead of "does not open". It now runs first, while the publication is staged, and asserts the state afterwards so a reordering fails loudly.
- **Verified:** `bash scripts/run_pg_tests.sh policy` → `13 passed; 0 failed`, rc=0; `cargo test -p reasonbraid-server --test publisher` → `6 passed; 0 failed`, rc=0. The new live control carries **6 legs**, the offline one **8 arms**.
- ⚠️ **Falsified by a matched pair rather than by neutralizing the shipped predicate, and the reason is recorded rather than hidden**: the harness refused the source weakening as a security change. The contrast is stronger anyway because it is **durable** — the same three locations, against a server whose declared root is the directory above, are now *accepted* and reach the record exactly as a legitimate location does. One knob moves; the legs flip. A neutralization would have proved the same thing once and left nothing behind.
- ⚠️ Two limits stated rather than implied, in the leaf and in the book: containment is decided once per request against the filesystem as it then stands, so a symlink swapped before `gix::open` would not be seen; and the verb is still authorized by **enrolment alone** — that is `.9.2.1.2`, unchanged here.
- ⛔ **A new wire code, with the population it moves pinned.** `publication_repository_unconfigured` is an `ext` code, the documented forward-compatible path, and touches no frozen document. The census now reads **19 emitted, 10 unregistered, 0 undocumented**; `.11.7.1` owed a verdict on nine and now owes one on ten, pinned at `2e72571` so the correction is visible rather than silent.
- ⛔ **SUPERSEDED — the bullet below is preserved as written and is wrong in three ways; see the 2026-09-15 `.9.2.1.1.1` entry at the top of this file.** Its duration is one warm run of a 4.5×-spread quantity, its "9 %" belongs to a different arm than the one beside it, and its mechanism is withdrawn.
- ✅ **`.9.2.1`'s build-cost note was re-measured, as it asked.** `cargo check -p reasonbraid-server --tests` finished in **2 m 39 s**, rc=0 — not the ">10 minutes" the decomposition recorded. ⛔ The two numbers answer different questions rather than contradicting each other, exactly as that note predicted: it measured a contended machine with a cold cache. 🔴 **And the re-measurement found the real mechanism, which is not CPU contention at all**: the first attempt sat at **3.84 s of CPU across 6 minutes**, blocked with all four threads asleep and its dep-graph file byte-identical, and cleared only after 3.8 GB of `.rmeta` was pulled into the page cache. The completed run spent **2 m 39 s wall for 46 s user**; the lib-only check, **3 m 01 s wall for 5.8 s user** — about 9 % CPU. This build is I/O-bound against the external repository volume. Evidence for `docs/decisions/2026-09-12_checkpoint-cost-model.md`, not a change to it.

## 2026-09-14 — Three repairs were sharing one leaf (`SIGNOFF-REPAIR.9.2.1`)

`.9.2.1` carried three acceptance clauses over the publication verbs. They are three different repairs: one invents server configuration and a path-containment predicate, one changes the **wire contract** of two shipped verbs, and one adds a repository read to a database transition. They shared a leaf because one reviewer found them together, not because they land together.

- **Decomposed into `.1` (repository root), `.2` (authority binding) and `.3` (object existence)**, each with its own acceptance and its own control observed RED before its fix — `.11.13`'s precedent, which converted eight containers into leaves that can actually be finished.
- ⚠️ **Two facts settled it rather than taste.** The acceptance requires each control RED first, and `cargo check -p reasonbraid-server --tests` does not complete inside ten minutes on this machine — so one commit would mean three unrelated repairs sharing a single red-to-green cycle, and a bisect could not separate them.
- ⚠️ The children are **ordered**: `.1` invents the configured root and `.3` needs a repository to look objects up in. `.2` is independent.
- **Measured while writing the children, not asserted:** `git grep -n '"git_object_ids": \["abc123"' crates/reasonbraid-server/tests/policy.rs` → **3** sites, one of them the deployment fixture. The suite meant to qualify the transition exercises it with ids that do not exist, which is why `R-73-74-3`'s framing is preserved as the stronger claim.
- Pending rose 53 → **55**, which is the healthy direction: three things carried as one unfinishable leaf are now three with acceptance attached.

## 2026-09-14 — The derived map was generated from the working tree and committed against the index (`SIGNOFF-REPAIR.11.4.5.4`)

`.githooks/pre-commit` regenerates `KNOWLEDGE_MAP.md` and stages it, so *"map-drift is structurally impossible"*. But the generator enumerated its sources with `ls docs/{tasks,decisions,knowledge}/*.md` — the **working tree** — while the map it staged was committed against the **index**. An untracked file in any of those directories entered the committed map as a link the commit does not contain.

- ⛔ **Found in this session's own commit.** `git show 991bbf8:KNOWLEDGE_MAP.md | grep -c 'a-control-is-calibrated…'` → **1**; `git ls-tree --name-only 991bbf8 docs/knowledge/` → **0**. A checkout of that commit has a map whose link does not resolve.
- ⛔ **Widening the census changed the answer.** `docs/knowledge/` alone gave 2; all three families the generator actually lists gave **3 of the 139** commits touching the map — the third, `0558ed6`, is a `docs/decisions/` record the narrow scan could not see. `MEMORY.md`'s standing *"scope a census whole"* warning names this failure and it still caught me, one leaf after the last time.
- ⭐ **All three share one shape**: a commit sequenced BETWEEN the work that creates a source file and the commit that adds it. Two are changelog rotations — and `.11.4.1.2` *requires* a rotation to be its own commit, so the spine's correct sequencing rule is exactly what exposed the generator's wrong source.
- **Fix:** derive from the commit — `git ls-files -- ':(glob)docs/<dir>/*.md'` to enumerate, `git show ":$f"` to read content. Removing the failure mode rather than detecting it, per `.11.4.5.3`'s ruling. It also stops an **unstaged** `answers:` edit being published into the map, the same root cause reached by the same reasoning.
- 🔴 **The first cut of the fix would have published 54 broken links.** A git pathspec wildcard **matches `/`** by default, so `git ls-files -- 'docs/tasks/*.md'` returns **69** paths where `ls` returns **15**, reaching into `docs/tasks/artifacts/`; with the link built from `basename`, every extra would have rendered as a link to a file that is not there. Caught by diffing the fresh render against the committed map — the one comparison that could see it — and corrected with `:(glob)`, fnmatch with `FNM_PATHNAME`. ⚠️ `MEMORY.md` warns that `*.rs` does **not** recurse; this is the same confusion in the opposite direction, with the two tools disagreeing on one glob.
- **Verified:** the corrected generator reproduces the committed map **byte-for-byte**, so the repair changes the failure mode and not the output. New `--self-test`, 7/7 arms in a throwaway `git init` repo under the repository-derived scratch, **falsified three ways**, each variant reddening only its own arms. `make gate` 18 checks green; `SELF-TEST` now covers **24** instruments at 1.59 s.
- ⛔ The three historical commits are **not** rewritten; each self-corrected within one commit. The repair stops the next one.

## 2026-09-14 — The arity gate modelled the opposite of the renderer, and its self-test agreed (`SIGNOFF-REPAIR.11.2.3`)

`scripts/check_table_arity.sh` flags Markdown rows whose cell count disagrees with their header — the defect matters because GFM **silently** drops the extras and pads the missing, so the page looks fine and the reader loses a column. Its splitter treated a pipe inside an inline code span as part of the cell. GFM does the opposite: a row is split into cells **before** inline parsing, so only a backslash escape protects a pipe.

- ⛔ **A green whole-corpus scan was measuring the wrong predicate.** Two rules run side by side over the same 323 tracked files at `dd4151b`: the shipped rule finds **0** arity-defective rows, the renderer's rule finds **1**. The leaf recorded 2; re-measured rather than inherited, because `RECONCILIATION.md:111` was genuinely repaired by REPAIR-0175 — so the one that remains is exactly the retained falsification target.
- ⭐ **Asked the renderer nine times before asserting anything.** A throwaway mdbook 0.5.2 book under the repository-derived `TMPDIR` (§13), cell counts read out of the emitted `<tr>`/`<td>`: `` | `x | y` | 2 | `` → **2 cells**, backticks LITERAL, the `2` **discarded**; `` | x \| y | 2 | `` → 2 cells; `` | `x \| y` | 2 | `` → `<code>x | y</code>` and `2` — literal pipes *and* the code span, which is the repair form.
- 🔴 **The `--self-test` was why it survived, not what would have caught it.** Its arm *"a pipe inside a code span is not a separator"* asserted `want=0` for the exact shape the renderer scores as a defect. Written from the same reading as the code, it could not disagree with it — `TOOLBOX.md`'s measured blind-spot rule, reproducing exactly.
- ⛔ **A wrong model is wrong in both directions.** The old parser also scored an unpaired backtick run as a defect; the renderer emits 2 cells and no defect. That arm was asserting a **false positive**, and correcting the rule removed it rather than preserving it.
- ⚠️ **The damage was on this project's own doctrine page.** The `INDEX-FRONTIER` registry row wrapped `` | — | `` in a code span, so the renderer cut the cell at *"8 completed trees write `"* — **516 of 1,027** characters — and published a bare `—` where `scripts/check_tree_index_frontier.sh` belongs. ⛔ The leaf's own note said the cut was at *"fixed this exact"*; measured, it is later, and the record is corrected rather than repeated. Repaired to `` `\| — \|` `` and re-rendered: **3 cells**, enforcer named, raw HTML `<code>| — |</code>`.
- ⭐ **The leaf's warning that the ratchet would block its own fix did not materialise**, and the reason is recorded instead of the relief: the ratchet pipes both `git show ":$f"` and `git show "HEAD:$f"` through the *same* working-tree parser, so a rule correction moves `now` and `before` together — here 0 vs 1, a fall, promoted silently. Same-commit repair was still right, for the published page rather than for the gate.
- **After:** 9/9 self-test arms, every one the renderer's verdict named in the arm's own title; `--all` → **0**; `make gate` → 18 checks green.
- **Promoted:** `docs/knowledge/a-control-is-calibrated-against-the-renderer.md` — a control over a format you did not implement is calibrated against the consumer, never the spec.

## 2026-09-14 — Graded on challenge: the findings stand, three decorations did not (`SIGNOFF-REPAIR.11.9.1`)

The director asked *"do you stand by your findings?"* — `docs/CLAIM_VERIFICATION.md` §4.1's grading question. Graded on all three axes, separately.

- **PROSE — stands, all three repairs.** The MCP reads ran a copy that omitted the authorization; the policy registry is site-global; citing an authority was the same as holding one on three surfaces; the call-response defect is in the shared core. Each was reproduced live and separately falsified.
- **NUMBER — stands.** 8→0, 6→0 and 5→0 legs are test-binary output, deterministic and re-derivable.
- **NAMED INSTANCE — three failed, and a named instance has no tolerance band.**

1. ⛔ **A carried line count I staled myself.** "the 728-line `policy.rs`" was measured, then the file was edited by REPAIR-0184 in the same session; it is now **731**. Removed at all **6** sites rather than refreshed — §5B: derive or delete, never re-carry. The claim it decorated (`git grep -n "tenant"` → rc=1) was re-checked and still holds after the repair.
2. ⛔ **A classified count published against a command that does not yield it.** "five sites" was correct as a classification and was published beside a raw `git grep`, so a reader re-deriving gets a different number — and the population itself was wrong, because `src/*.rs` is a glob that does **not recurse** and missed `src/authority/selection.rs`. Pinned at all **4** sites to `aee579a`: **9 sites, 5 of which ask the grant-validity question in four spellings**. ⚠️ `MEMORY.md`'s standing "scope a census whole" warning names this exact failure, and it still caught me.
3. ⛔ **An over-count assembled while writing.** "the third header asserting a check nobody wrote" is **two**. The third, `lifecycle.rs`'s approval comment, is accurate about its SQL — the check existed and matched a subject; the defect is that the subject was an unverified claim. Folding it in made the generalization fit a population it does not describe.

⭐ **The lesson is the shape, not the three instances**: every one of them is a number attached to a finding as *decoration* rather than as the finding's evidence. The evidence legs were re-derived, falsified and durable; the ornaments were none of those, and nothing was watching them.

## 2026-09-14 — The seam was named; the core was the defect (`SIGNOFF-REPAIR.6.1.2`)

- **Measured live before the repair, on BOTH surfaces:** a role enrolled in one tenant recorded a `decline` and a `recuse` against another tenant's recruitment call — through the MCP seam **and over plain HTTP**. A new seven-leg control reported **5 breaching**.
- 🔴 **The leaf named the MCP seam; the HTTP verb over the same core takes no tenant at all** and was equally open. ⛔ A repair at the seam would have left that verb wide open **and the seam's own suite green** — which is the argument for where the binding went: into `respond_to_call_core`, derived from the call, so both callers get it at once.
- ⛔ **The fixture was wrong first and three legs were green for the wrong reason.** The control posted to `/v1/calls/{id}/responses`; the route is `/respond`, so every HTTP leg was reading a 404 as a refusal — the positive leg included. The corrected fixture moved two breaches onto the surface the leaf had not implicated.
- ⛔ **One leg was removed rather than kept green.** The acceptance said "every response kind"; `join` is refused earlier, by the absence of an enrolled node, so it would pass without the binding ever being reached. The two kinds that skip the eligibility gate carry the claim.
- ⭐ **The proposal half was settled by measurement, as the leaf directed**: `register_proposal` has no grant check and no audit row — and neither does the HTTP verb behind it. The tool is not weaker than the HTTP surface, so the seam's header was describing a control neither has. The header now states what each of the three verbs actually brings, separately, because they are not uniform.
- **Promoted:** when two surfaces share a core, the binding belongs in the core — and the giveaway is a seam-level repair that leaves the seam's own suite green (`TOOLBOX.md`, `docs/knowledge/`).
- After: **0 of 7 legs breach**, 5/5.

## 2026-09-14 — Citing an authority is not holding one (`SIGNOFF-REPAIR.9.3.1`)

- **Measured live before the repair:** a principal in one tenant recorded a policy **retraction** under another tenant's grant, registered a deployment target owned by it, recorded a correction under a grant whose `valid_from` was tomorrow, and filed a policy **approval as another principal**. A new nine-leg control reported **6 breaching**.
- ⛔ Reachable rather than theoretical: the dev enrolment mints `grt_<principal_id>`, so naming another principal's grant needs nothing but their id.
- **The census took the family whole** — `git grep -n "FROM authority_grants" aee579a -- crates/reasonbraid-server/src/` (pinned to the pre-repair commit, because this repair CHANGED the population) returns **9 sites**, of which **5 ask the grant-validity question** — `corrections`, `deployments`, `lifecycle` and `policy` twice — **in four different spellings**; the other 4 ask different questions. ⛔ The population number was first taken with `src/*.rs`, a glob that does NOT recurse, and it missed `src/authority/selection.rs` — and `git grep -n "valid_from"` over those modules returns **rc=1**: not one of them consulted the column, though it is `NOT NULL`. The `expires_at IS NULL` arm four of them carried was dead for the same reason.
- ⭐ **The one site that bound a subject was the one that had had the attention, and it was still half-bound** — it matched the grant to `input.approver`, a string off the wire that nothing tied to the caller. A third instance of the same mechanism, which no leaf had named; repairing the two the leaf named would have shipped half a repair.
- **The fix is one definition, two shapes**: `authority::grant_is_live` and `authority::grant_held_by`. Three surfaces take the held-by form and receive the principal their handlers had already resolved and were discarding.
- ⭐ `.9.1`'s measured asymmetry is settled — registration no longer accepts a grant the resolver in the same module would refuse — while `.9.1`'s semantic question, whether a policy's owner must be the registrar's own grant, is deliberately left to it.
- ⚠️ **One acceptance leg was not expressible and is corrected rather than quietly weakened**: `GrantAction` and `TargetSelector` cannot NAME a publication, a deployment target or a correction, so "the grant's action covers the target" has nothing to compare. Opened as `.9.3.4`. This is the second such leg this session.
- **Falsified one arm at a time**: subject binding → legs A and B; `valid_from` → leg C; approver-is-the-caller → leg G alone.
- ⚠️ The falsification found a defect in the control itself: two approval legs shared a proposal, and a successful approval advances it, so each reported the other's outcome. Each leg now has its own.
- After: **0 of 9 legs breach**, 12/12, with the eleven pre-existing policy tests unchanged under a stricter registration path.
- Pending leaves 55 → 55: one closed, one opened. ⭐ Written 56 from arithmetic first; the tree's own re-derivation said 55.

## 2026-09-14 — A claim of sameness is worth its call graph (`SIGNOFF-REPAIR.6.1.1`)

- **The severest defect this reconciliation produced, repaired.** `reasonbraid-mcp` carried a private re-implementation of the server's three inspection reads while its own header claimed *"the SAME queries + the SAME authorization as the HTTP handlers"*. The copy ran neither: `list_inbox` declared a `principal` it never read, `get_policy_bundle` never read one either, and `get_thread` computed the foreign-reader class and returned the full projection beside it.
- **Measured live before the repair, on a real cluster:** a principal in tenant A received tenant B's ENTIRE thread projection — subject, objective, participants, creator, budget, ceiling id — with `"visibility":"network"` printed next to it. A new fourteen-leg control reported **8 legs breaching**.
- **The repair removes the failure mode instead of patching it.** A new `mcp_read` seam in the server crate (`mcp_read_internal`, beside `mcp_write_internal`) calls three halves now shared with the HTTP handlers — `authorize_inspection`, `thread_inspection`, `inbox_inspection` — and the MCP crate's copy is DELETED. Sameness is a call, not a sentence.
- ⚠️ **One part of the original finding was corrected by measurement rather than confirmed.** It read the policy bundle as a cross-tenant leak. `git grep -n "tenant"` over the registry's schema and its whole module returns **rc=1**: there is no tenant column and no site filters by one. The registry is site-wide on BOTH surfaces. What stood: no `WHERE`, no principal check, and a bundle labelled with the caller's tenant. What fell: the framing. The tool now runs the HTTP surface's enrolment gate and returns no tenant label; the schema question is opened as `.6.1.5` rather than answered by a filter on one read.
- **Falsified one arm at a time**, each neutralization proved landed by a gate-call census and each rc read directly: removing the thread gate → 3 breaches, leg A only; the inbox gate → 2, leg B only; the enrolment check → 1, leg C only. No arm carries another's proof.
- ⛔ `git diff` was useless as the landing proof — `mcp_read.rs` is a NEW file, so an unstaged edit to it produces no diff at all, and the first falsification's proof printed nothing while succeeding.
- Side effects, each deliberate: the MCP thread read now applies the `.1.3.1` derived view and the inbox read the `node_inbox_state` view, matching HTTP; `get_policy_bundle` loses its tenant parameter and the conformance golden asserts that absence; `chrono` becomes a dev-dependency.
- Pending leaves 54 → 54: one closed, one opened.
- **Promoted:** a claim of sameness is worth exactly the call graph that enforces it (`TOOLBOX.md`).

## 2026-09-14 — Routing is not ownership (`SIGNOFF-REPAIR.11.13`)

- **The director's correction.** This session surfaced six findings and reported an owner for each. Audited mechanically, **8 of the 10 named leaves carried no `- Acceptance:` line of their own and 6 had no children** — they could be read, not finished.
- ⭐ **The one properly-owned finding got that way by accident of vocabulary**: `.11.12` exists because the clause ledger classified its clauses `unowned`, and that state forces a new leaf. `owned` and `attach` measure whether a clause is *visible* to a leaf; neither asks whether the leaf is *executable*.
- **Eight containers censused and split into eleven bounded leaves**, each with its own reproduce, owns and acceptance, and each acceptance naming a control to be observed red before its repair: `.6.1.1`–`.6.1.4` (the MCP reads, the write seam's enrolment-as-authority, the untyped body index, the quota semantics), `.9.2.1` (the publish verbs' unbound path and fabricated Git ids), `.9.3.1`–`.9.3.3` (citing an authority equals holding one, a publication reviewable once for ever, the declared-versus-deployed digest), `.11.2.3` (the table-arity parser and the row still publishing broken), `.11.4.2.2` (the `MEMORY.md` cap).
- **Frontier re-ranked by severity** — the MCP cross-tenant read leak leads.
- ⚠️ The re-run audit reported two leaves as lacking acceptance and the audit was wrong; both were written `- Acceptance, …`. Fixed in the leaves, not by loosening the matcher.
- Pending leaves 47 → 54: the count rises because routing became ownership.
- **Promoted:** a leaf that cannot be picked up and finished is not an owner (`TOOLBOX.md`).
- No product code, schema, test or script changed. 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0.

## 2026-09-14 — Correcting a number this activity published (`SIGNOFF-REPAIR.11.9.1.3.1`, tranche 4a)

- **Tranche 4 sized at 10,519 characters — 4.12x the proved-executable size — and split five ways** on each record's own narrowest candidate, with one declared deviation: a 423-character singleton folded into its sibling in the same tree family.
- 🔴 **A number this reconciliation itself published was wrong when written.** The clause ledger said a fixture accepts an unbound verdict digest "in four places"; `grep -c` says seven, across seven distinct test functions, and `git show` at the original commit proves the corpus did not move. Both sites corrected, the superseded figure named. **Promoted:** a count written while reading is not a count (`TOOLBOX.md`).
- 🔴 **Any enrolled principal can act as any authority whose grant id they know.** The corrections module's authority check takes the grant id and nothing else — no caller, action, selector, boundary, publication or valid-from — behind an endpoint that admits on enrolment alone. The deployments module repeats the shape.
- 🔴 **No second review can ever be scheduled for a (publication, trigger) pair**: the review id is deterministic and is the primary key, and the colliding insert's error is discarded. The same discard makes a storage failure return a successful empty schedule.
- 🔴 **`rb-server` migrates the database on the line before it validates the profile it refuses to boot without**, and its bind address carries no predicate at all. New leaf `SIGNOFF-REPAIR.11.12`, with the two halves to be decided separately.
- ⚠️ Two fixtures declare digests bound to nothing, and a metrics test's comment claims a counter assertion the test does not make.
- ⭐ The fifth two-records-one-finding pair: one waiver satisfying a trigger named "repeated", reached independently by two records.
- Validation: `--classified` 204 rows clean with all 40 `attach` clauses named by their owners, `--self-test` 49 controls, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema, test or script changed.

## 2026-09-14 — Measure a deadlock as a ratio (`SIGNOFF-REPAIR.11.9.1.2.3`, tranche 3 complete)

- **Tranche 3c reconciled five records into 22 clause rows**: 16 `owned`, 6 `attach`, 0 `unowned`. Tranche 3 is complete at 15 records and 69 clauses.
- 🔴 **The browse worker waits for its child to exit before reading the child's piped stdout**, and the request it writes allows 4 MiB of output against an OS pipe buffer of at most 64 KiB — so the deadlock covers most of the intended range, not an edge case, and surfaces as a timeout. Its error path drops a live `Child` without killing or waiting; its timeout path kills the worker but not the browser beneath it.
- 🔴 **Four adapter defects, each at two sites**, because `codex.rs` and `claude.rs` share a shape: a byte slice of a `String` that panics off a char boundary, an unbounded line read under a bound checked before the append, a child map that is never removed from, and a success path that returns without awaiting the drain or waiting the child while the failure path does both.
- 🔴 **A deadline is computed, shipped, and read by nobody** — one grep hit across every adapter source and the supervisor, and it is the struct field's own declaration.
- 🔴 **A completed item that reaches the retry gate is dead-lettered and auto-quarantined**, and that report never acknowledges its own journal row while its only sibling call site does.
- ⚠️ **One clause was narrowed rather than confirmed** — `OutcomeUnknown`'s propagation is real and documented as deliberate by the type itself.
- ⚠️ **The narrowest candidate was the owner of only one of the five records**, the clearest instance yet that a record's candidate list is a suggestion.
- **Promoted:** prefer a ratio, a single-hit grep or a two-site contrast to a reading (`TOOLBOX.md`).
- Validation: `--classified` 179 rows clean with all 38 `attach` clauses named by their owners, `--self-test` 49 controls, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema, test or script changed.

## 2026-09-14 — The sentence at the top of the MCP module (`SIGNOFF-REPAIR.11.9.1.2.2`, tranche 3b)

- **Tranche 3b reconciled five records into 27 clause rows**: 22 `owned`, 3 `attach`, **1 `none`** — the ledger's first, which completes the closed set of six states — and 0 `unowned`.
- 🔴 **The MCP read surface leaks across tenants on all three read tools while the module's first paragraph claims the same authorization as the HTTP handlers.** `policy_bundle` takes its tenant as an unused `_tenant_id` over a `SELECT` with no `WHERE` clause and returns every tenant's policy clauses labelled with the caller's own tenant id; `list_inbox` declares a `principal` argument and never reads it; `get_thread` classifies, gets `Network` for a foreign tenant, and returns the full projection anyway. ⛔ **CORRECTED by `.6.1.1` (REPAIR-0182):** `policy_versions` has NO tenant column — `git grep -n "tenant" -- migrations/0038_policy_registry.sql crates/reasonbraid-server/src/policy.rs` returns rc=1 — so the registry is SITE-GLOBAL and HTTP `api.rs::list_policies` returns the same unfiltered set to any enrolled caller. The missing `WHERE`, the unchecked principal and the false tenant label all STAND; the cross-tenant FRAMING does not. The schema question is `.6.1.5`.
- 🔴 **`join_call` is the third instance of the two-caller-identifiers family**: gated on the caller's tenant, then the call is fetched by id alone and the two are never compared. A decline, recommendation or recusal skips the eligibility gate entirely.
- 🔴 **`policy.rs` queries the grants table two ways twenty-seven lines apart** — registration on status alone, resolution on status and expiry — so an expired grant registers a policy version the resolver in the same file would refuse. Its selector default is fail-open where the owning goal line says fail-closed.
- ⚠️ **Two measurements narrowed the record rather than confirming it**, and both are recorded as narrowings: the `body["tenant_id"]` panic is real but unreachable through the typed MCP tool, and the quota-per-retry behaviour is documented verbatim in the module header.
- ⭐ `ATTACH-LANDED` fired on this leaf's three unattached clauses before they were written — the second consecutive tranche it has caught.
- 🔎 **`MEMORY.md` stands at 7,161 bytes against a 7,168-byte cap**, with three consecutive leaves forced to evict a standing warning to land. Routed to `.11.4.2` with its measurement.
- Validation: `--classified` 157 rows clean with all 32 `attach` clauses named by their owners, `--self-test` 49 controls, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema, test or script changed.

## 2026-09-14 — One missing predicate at three sites (`SIGNOFF-REPAIR.11.9.1.2.1`, tranche 3a)

- **Tranche 3 sized at 5,041 characters and split** on the same narrowest-candidate boundary that formed it — 1,624 / 1,655 / 1,762, all inside the 1,405–3,006 range this activity has proved executable in one commit.
- **Tranche 3a reconciled five records into 20 clause rows**: 15 `owned`, 3 `attach`, 1 `handled`, 1 `declined`, 0 `unowned`.
- 🔴 **A lookup keyed on `operation_id`/`command_id` with the owning node absent from the predicate lives at three sites**, reached by two different records and connected by no leaf: `node_channel.rs::event_id_for_operation` (the reconciliation lookup the handshake performs), the receipt insert's `ON CONFLICT (event_id) DO NOTHING` over a primary key that does not include `node_id`, and migration 0021's `delivery_state` view marking a row `consumed` from another node's result. `SIGNOFF-REPAIR.4.3`'s goal line already demands the proof that would have caught all three.
- 🔴 **The cursor high-water mark is derived from the table prune deletes from.** `enqueue` writes `MAX(cursor)+1` and `current_cursor` reads `COALESCE(MAX(cursor), 0)` over `node_inbox`; prune everything and it rewinds to 0, re-issuing cursor 1 for a node that already acknowledged it. The prune test leaves a partial prefix that keeps the highest cursor, so `MAX` never has to answer for an empty inbox.
- ⭐ **A dated answer to a question the record left open:** the hardcoded `expires_at` of `2026-09-15` does NOT break the two `policy.rs` tests tomorrow — neither the writer nor the reader compares a waiver's expiry to `now()`. The clause is declined as a present defect, and the same measurement found that `repeated_waiver` fires on long-lapsed waivers and on the first waiver, which is written into `.9.3`.
- ⭐ **`ATTACH-LANDED`, registered one commit earlier, fired on this leaf's own three unattached clauses before they were written**, naming each row and its owner, and went green once the sentences landed. Its first production use caught the exact failure it was built for.
- **A sharper shed rule at the `MEMORY.md` byte cap:** a warning a GATE now enforces is the cheapest of all to shed, because the machine states it at the moment it matters and names the offending row.
- Validation: `--classified` 130 rows clean with all 29 `attach` clauses named by their owners, `--self-test` 49 controls, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema, test or script changed.

## 2026-09-14 — `ATTACH-LANDED`: gating the one ledger state that cannot check itself (`SIGNOFF-REPAIR.11.11`)

- **The reconciliation ledger's `attach` state is the only one of six whose next action is not "none"** — it requires a sentence written into a leaf the classifier does not own — and every other property of a row is visible in the row itself. `--classified` could therefore validate everything except the thing that mattered.
- **Registered as `ATTACH-LANDED`**, the registry's 24th named doctrine (the enforcer's top-level count stays 18; it runs inside `PROJECT-SPECIFIC`, 7 sub-checks -> 8): an `attach` clause whose owning leaf's own section does not NAME its record now blocks the commit.
- **The leaf had published a prediction before any instrument could test it, so the census was reproduced at four points by two independent routes** — the new refusal copied into a detached worktree at each tranche's closing commit, and a throwaway re-implementation over `git show`. Both report **3 breaching of 3** at tranche 1's close (`5862837`), then **0 of 7**, **0 of 15** and **0 of 26**. The three named are exactly the set found by hand: `R-53-4` clauses 2 and 3 on `.7.1`, `R-63-1` clause 4 on `.11.4`.
- **Zero today, every historical instance caught, one bounded file** — the `REASON-CODE-DOC` shape. `SIGNOFF-REPAIR.11.9`'s rejected gate would have fired on 114 of 131. The same rule family, measured twice, decided oppositely both times.
- The rule is the record id in the owner's own section, never a phrase, and it does not require the literal words `ATTACHED CLAUSE`. The id carries a right-hand boundary because `R-53-4` is a prefix of `R-53-41`.
- Falsified twice and observed red both times, through the instrument and through the registered enforcer; both restored to rc=0.
- Validation: `--self-test` 49 controls (42 before), `--classified` 110 rows clean, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema or test changed.

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated seventeen times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

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
