# CHANGELOG.md

## 2026-09-13 — The routes censused against the book, and the instrument cross-checked (`SIGNOFF-REPAIR.11.8`)

- The naive census said `111 routes / 24 named / 87 not`, and said in the same breath it was an upper bound wrong in both directions: it counted test fixtures, and accepted a passing mention as documentation.
- Both refinements now have a stated criterion. A **product** route is one registered outside every `#[cfg(test)]` block — which removes the 8 routes `fetcher.rs` registers inside its test module ("a tiny local origin: the routes the refusals and the ceilings target"). **Described** means the book names the route beside an HTTP method, the shape a contract line takes; **mentioned** means the bare path appears. The three are reported separately, because collapsing them is how the first census went wrong twice at once.
- **Refined: 103 product routes — 22 described, 1 mentioned, 80 absent.** The gap is slightly smaller and considerably better understood: of the 23 routes the book touches, 22 carry a contract line. The 80 include every `…/revoke`, the region pairing verbs, and the console's three asset routes.
- 🔴 **The instrument was wrong before it was right, and that is recorded rather than quietly fixed.** Its first version scanned line by line, so a `.route(` whose path sits on the next line was invisible: 53 of `api.rs`'s 92, and 63 overall — a 42 % undercount that would have published a *smaller* gap than the real one. Caught by cross-checking against an independent per-file count taken before the instrument existed. The self-test now carries a multi-line route in both the product and the fixture arm.
- ⭐ The transferable part: **an instrument's first number should be compared against a number obtained a different way before it is believed.** Two counts agreeing is weak evidence; two counts disagreeing is what found this.
- The decision is a measured **backlog**, not a gate. "Every route must appear in the book" would flag the console's static assets and accept a bare mention — wrong in both directions before it is written. What "documented" must mean differs by route family, and the population has not been partitioned that way.
- ⛔ Not claimed: 80 is not 80 defects, and "described" is a proxy — a contract line is not proof the description is correct. ⚠️ And this leaf **documents no route**; it measures, and names what measuring cannot decide.
- Validation: `--self-test` rc=0; gate (18 checks) rc=0, with the new `SELF-TEST` doctrine now running this instrument's own controls; book and link check rc=0.

## 2026-09-13 — A node id owned by another tenant answered 500, one arm from answering with a certificate (`SIGNOFF-REPAIR.3.5.1`)

- The leaf proposed making the one-unused-token index per-tenant. Measuring refused its own repair: **`nodes.node_id` is a GLOBAL primary key**, so a node identity belongs to at most one tenant ever and the global token index is *consistent* with that. Per-tenant uniqueness would allow two live tokens for one id and move the failure from a clean `409` at issuance to a primary-key violation at redemption — after the operator had distributed a token.
- 🔴 **Reading the redemption path to decide that found something reachable today.** The index blocks a second *unused* token, not a second token: once the first tenant enrols, its token is used, so a second tenant's issuance succeeds — and the redemption answers **`500 dependency_unavailable`**. The existence check was `EXISTS(… node_id = $1 AND tenant_id = $2)`, tenant-scoped against a global key, so a node owned by someone else reported false, the insert ran, and the primary key aborted the transaction. ⭐ The code's own comment two lines above says *"a unique-violation probe would abort the transaction"* — the hazard was known; the check asked the wrong question.
- The query now reads the node's **owner**, and a node owned by another tenant is refused — audited, reusing the same-tenant duplicate's wording, so nothing names the owner and the caller learns only what they would about a node of their own.
- 🔴 **The naive version of that fix is a takeover, and it is demonstrated rather than argued.** The replacement branch swaps the node's key and issues a fresh workload certificate with **no tenant check**, and it fires once every certificate is revoked. With the foreign-owner arm removed, tenant B's redemption answers `200` carrying `cert_der` for `CN=node:nod_…09f1` with `SAN=host-b` and the escrowed `key_der` — tenant A's node identity, issued to B.
- Falsified in **both** directions the design space has, because the two failure modes differ and one reversion would have shown only one: the change absent gives the `500`; the query widened without the refusal arm gives the certificate.
- ⚠️ Reproduction hygiene, recorded rather than hidden: the control failed three times before it measured anything and only once was the product at fault — a reversed helper destructuring, a test helper that panics decoding an empty body (hiding the status behind a decode error), and a second issuance correctly declined by the very index under discussion.
- ⛔ Not claimed: the index is **unchanged** — no migration. This repairs the refusal, not the uniqueness. The availability half of the original finding is reassessed rather than repaired: holding a node id out requires two tenants to name the same unguessable UUID, and the global primary key settles ownership once either enrols.
- Validation: `run_pg_tests.sh node_enrollment node_channel node_inbox` rc=0 — **3 suites / 50 tests, zero failures**. Strict lint, fmt, gate (18 checks), book and link check rc=0.

## 2026-09-13 — The enforcer now runs the self-tests, and caught itself recursing (`SIGNOFF-REPAIR.11.4.3.1.7.2`)

- Every check and census instrument here carries a two-sided `--self-test`, and **nothing ran them**. They were author-run: fired once while the change was written, never again. That is how one of them sat broken from the very commit that added it, unnoticed for dozens more.
- Measured before proposing the rule, because a gate people route around is a gate that lies: **17 scripts carry a self-test, they cost 1.01 s in total**, and the enforcer costs 3.15 s. Registered, the gate measures **4.27 s over 18 checks** — a real increase of 1.12 s.
- The check **discovers** the population rather than carrying a list, so a new instrument is covered the day it lands.
- ⛔ **It catches nothing today**, and its own header says so: all 17 pass. Its value is preventing a control from silently stopping, not finding a present defect. ⚠️ 11 of the 28 check/census scripts have no self-test at all, and this does not reach them.
- 🔴 **The gate fell into its own trap while being built.** The check warns at length about self-reference — and *registering* it put the flag's literal text into the enforcer's description, so discovery found the **enforcer**, ran it with the flag, and the enforcer (which ignores unknown arguments) ran every check again. Unbounded recursion on the first `make gate`; the command was killed at 400 s. The gate exists because a control searched for a string it contained, and building it I made a discovery match a string it had just written.
- Defended twice now, the first making the failure impossible rather than unlikely: an exported guard variable makes any nested invocation exit immediately, and the enforcer is excluded by path because it is the runner, not a check. Both are asserted by the self-test, alongside the failing-arm and passing-arm directions.
- ⭐ Falsified **end to end against the real defect**, not a synthetic one: restoring the original literal probe makes `make gate` print `❌ SELF-TEST`, quote the guard's own failure, and refuse the commit. The gate would have caught the defect that motivated it, on the commit that introduced it.
- ⛔ Not claimed: it does not assert the 17 self-tests are *good*, only that they still pass. A weak one that passes is invisible to it — judging a control's strength is the separate accepted rule that a control must have failed on the defect it was written for.

## 2026-09-13 — A revoked boundary left the metrics surface open (`SIGNOFF-REPAIR.3.5.2`)

- 🔴 `GET /v1/admin/metrics` gated on the *grant's* status and never joined the boundary. Revoking a boundary updates only `enrollment_boundaries` — it does not cascade to the grants issued under it — so an administrator whose authority had been withdrawn kept this surface.
- Reproduced against the **live route** before anything changed: enrol an administrator (`200`), revoke the boundary her `tenant_admin` grant hangs from, confirm the boundary reads `revoked` *and* her grant still reads `active`, then call the route — `200`.
- ⭐ Those middle assertions are what make the control durable. Without them a later reader cannot tell whether the gate was repaired or whether revocation merely started cascading to grants; the test now states which mechanism it exercises.
- The gate joins the boundary and requires it live too — `status = 'active'` inside its own validity window, which is what every ordinary evaluation requires. One query; no route, response or counter change.
- ⛔ Deliberately **not** the frozen-boundary exception the nine `/v1/admin/*` inspection reads take. That exception exists so an administrator can inspect *authority* state while a revocation is in flight; process counters are not authority state, and this handler never called it.
- ⛔ **Two things are confirmed unrepaired rather than quietly fixed.** The process-global width stays as it was — deliberate, 7 aggregate counters, no identities — and the read is still **unaudited**, writing no authorization record unlike the nine inspection reads. Both need the route to name a tenant, which is a signature change and a breaking one; making a security repair hostage to a wire decision would have been the wrong trade. `SIGNOFF-REPAIR.3.5.2.1` owns them.
- Validation: `run_pg_tests.sh command_api` rc=0 — **37 passed / 0 failed**. Strict lint, fmt, gate (17 checks), book and link check rc=0. FALSIFIED against the restored unjoined query: `left: 200, right: 403`, and only that control — the pre-existing metrics acceptance passes throughout.

## 2026-09-13 — The tenant-administration surface, censused and split (`SIGNOFF-REPAIR.3.5`)

- The leaf named three surfaces without having measured any. The census moved all three.
- **The token index is still global.** Migration 0018 indexes `(node_id) WHERE used_at IS NULL` with no tenant column, so the cross-tenant collision reproduces unchanged: tenant B's administrator issuing for a node id tenant A already holds a token for gets a `409` naming the outstanding token, and A can hold B out of that identity indefinitely. ⚠️ Node ids are unguessable UUIDs — an oracle for a *known* identity and an availability hazard, not an enumeration path.
- 🔴 **The metrics surface outlives the boundary that authorized it.** `GET /v1/admin/metrics` gates on a hand-rolled query over the *grant's* status and never joins the boundary; revoking a boundary updates only `enrollment_boundaries`, leaving its grants `active`. So **revoking a boundary does not remove access to this surface**. It also writes no authorization record, so the read is unaudited — unlike the nine repaired tenant-admin inspection routes.
- ⚠️ Bounded rather than implied: the payload is **7 aggregate counters with no tenant dimension and no identities**. Process-wide volume, not per-tenant data — an aggregate-volume side channel, not a data leak.
- ⚠️ And read the instrument precisely: the admission census classifies this route as "identity only", which means it reaches none of the *recognised gates*, **not** that it is unauthenticated. It does gate; it simply does so with its own query instead of the evaluator.
- ⭐ **The third surface turned out to be largely closed already** — `.3.3.4.10` put all five node administrative mutations into guarded transactions with tenant-bound targets. The residual is the 42 mutating identity-only routes, which are already owned and recorded as unrepaired; this leaf does not re-own them.
- ⭐ Every census instrument's `--self-test` was RUN before its output was trusted — this session's own lesson from the instrument whose self-test had been failing since its first commit. Two pass; **`census_authority_paths.py` has no `--self-test` at all** and dies with a traceback when given the flag. Routed to the leaf that owns it.
- Split into `.3.5.1` (the token index — a migration *and* the redemption-lineage question `.4.1` shares) and `.3.5.2` (the metrics surface).
- ⛔ **Nothing is repaired in this commit.** The token collision still reproduces; the metrics surface still outlives its boundary and still writes no record.
- Validation: documentation only. Gate (17 checks), book, link check and diff --check rc=0.

## 2026-09-13 — The certificate rotated five minutes after it expired (`SIGNOFF-REPAIR.3.4.3.1.3`)

- `.3.4.3.1`'s census found this while looking for something else and forbade writing it up until a control drove a skewed clock at it. This is that control, and the numbers are worse than the arithmetic suggested.
- 🔴 A workload leaf issued at server-time `T` expires at `T + 600`; rotation is due from `T + 300`. With the node's clock 600 s **behind**, the check does not fire when due, does not fire with **one second** of validity left, and does not fire at expiry. It first fires at `T + 900` — **300 s after the certificate has already expired** — so the channel is dead for five minutes before the node even tries to renew.
- ⚠️ This is the **opposite** direction to the dispatch outage, where the danger is a node running *ahead*. A reader carrying that intuition here gets it backwards, which is why the two were kept in separate leaves and why the warning now sits in the code at the point of use.
- The rotation decision is extracted as a pure function — which is what makes it drivable — and evaluated in the server's terms through the offset. The channel keeps its own copy of that offset because the check runs inside `handshake`, before the journal is reachable; it is `0` until the first handshake answers, which is exactly the previous behaviour.
- Four controls: the reproduction, the opposite direction (early rotation uncorrected, on time corrected — harmless, which is why it was never the defect, but it must not stay wrong), the no-skew boundary unchanged so the correction cannot be satisfied by moving the threshold, and a **wiring** control that issues a real certificate with a chosen expiry and proves the stored offset actually reaches the decision.
- ⭐ That last one earned its place: the falsification drops the offset from the live check and leaves the three pure-function controls **green**, failing only the wiring control. The decision was already right; the defect was what it was fed.
- ⚠️ One test-only dependency added (`time = "0.3"` as a dev-dependency, to issue a leaf with a chosen validity window). The same 0.3 the workspace already resolves, so no second version enters — `cargo deny check bans` confirms and the lockfile gains one line.
- Validation: `cargo test -p reasonbraid-node` over the node's non-provider targets rc=0 — **69 tests, zero failures**. Strict lint, fmt, bans, gate (17 checks), book and link check rc=0.
- ⛔ Not claimed: **no field failure is asserted.** This is reproduced in a driven control, not in a deployment. The certificate's `not_after` is the server's *process* clock while the offset is measured against its *database* clock; those were measured to agree within a declared second, so the correction is sound to that tolerance and no further.

## 2026-09-13 — The node tells the time in the server's terms (`SIGNOFF-REPAIR.3.4.3.1.2`)

- 🔴 The outage this closes: a node whose clock ran more than 60 s AHEAD of the server read every admission decision as already expired and refused **every** dispatch — it did no work at all, fail-closed, with a reason line whose epochs matched so nothing looked wrong.
- Both handshake and poll responses now carry `server_time`, read from `clock_timestamp()` folded into the **existing** revocation-epoch query, so a poll — every few seconds — gains no extra round trip. The node measures `offset = server_time − midpoint(sent, received)`, stores it beside the epoch, and evaluates `decided_at` through it. The comparison is same-clock; a node whose clock is wrong keeps working.
- The midpoint, not either end: measuring against the send instant overstates the server's lead by the full round trip, against the receipt instant understates it by the response leg.
- ⭐ **This supersedes a repair from earlier the same day, and the reversal was measured before anything changed.** `.3.4.3` bounded the hazard without a wire change by running the window from `min(decided_at, received_at)`. With the offset in place that clamp **fights** the correction — the receipt is a *local* instant, and a correctly-corrected node 600 s behind computed `expires 02:56:58` from its own receipt against `server now 03:05:58` and found everything stale. **Bounding a cross-clock comparison and removing it do not compose.** The clamp and its supporting machinery are gone; `.3.4.3`'s record had already named this leaf as owning the change that "would remove the cross-clock comparison rather than bound it".
- 🔴 **A third cross-clock comparison the census missed.** The retry gate filters attempts by `updated_at >= decided_at` — a local journal timestamp against the server's clock. With the node behind, every genuine post-decision attempt reads as older than the decision, is filtered out, and **the retry bound never trips**: unbounded re-dispatch, fail-open. The census missed it because it looked for server instants compared against `Utc::now()`, not against stored local ones.
- ⚠️ A defect caught before it shipped: the first cut passed the corrected instant to the refusal path, which *writes* it. Journal timestamps are local and compared against local ones later, so a corrected value stored among them would corrupt the very comparison being repaired. Two instants are now explicitly named and separated.
- The node **reports** a disagreement of 5 s or more, naming both clocks and the offset — `.3.4.3.1`'s census had found nothing anywhere reported clock disagreement.
- ⚠️ The offset is the server's own statement and is **not** a secure time source. It adds no new trust: the node already accepts `revocation_epoch` from the same response, a stronger claim than the time. ⚠️ `deny_unknown_fields` makes this a coordinated wire change — server and node ship together.
- ⛔ Four controls asserting the superseded clamp are removed, each with a comment naming its replacement. The property they protected is now asserted in **both** skew directions, which the clamp could only do for one; two of them also drove a state production cannot reach.
- Validation: core+node **10 targets / 115 tests** rc=0; `run_pg_tests.sh node_channel node_inbox node_work node_enrollment` rc=0 with **4 suites / 57 tests**. Strict lint, fmt, gate (17 checks), book and link check rc=0. FALSIFIED in both halves — reverting the gate fails the skew controls in both directions, reverting the retry filter fails its control at `left: 0, right: 1`.

## 2026-09-13 — The self-test searched for a string it contained (`SIGNOFF-REPAIR.11.4.3.1.7.1`)

- Before acting on the cluster instrument's census, I ran its `--self-test`. It failed: `citation guard: an absent name reported references`.
- 🔴 The guard's absent-name arm searched the tracked tree for the literal `run-selftest-no-such-cluster-name` and asserted zero hits — and **that literal is written in the instrument's own tracked source**. `git grep -c` returns `scripts/census_pg_test_clusters.py:1`. The control searched for a string it contains.
- **Dated rather than estimated:** the file's add-commit and the string's introducing commit are the SAME one, `cb2f197`. So the arm passed exactly once, while the file was still untracked and `git grep` could not see it, and has failed from the instant it was committed.
- It survived because **nothing runs it** — the only mentions of the instrument anywhere in `scripts`, `.githooks`, `.github`, `Makefile` or `knowledge-map` are inside its own usage docstring. Widened to the enforcer that would be its natural runner: `grep -n "self-test" scripts/check_doctrines.sh` returns **0**, so none of the 17 registered doctrine checks has its self-test arm run either.
- ⚠️ What this did and did not compromise: the guard's other direction never broke, so cited clusters stayed protected. What was missing is the proof that the guard is not simply returning non-zero for everything — a half-verified safety check, which must not authorise a deletion.
- The probe is now generated per run (`uuid4`), absent by construction rather than by luck, and the failure message prints the probe it used. ⛔ The grep was deliberately **not** narrowed to exclude the script: that would blind the guard to the file most likely to name a cluster in a comment.
- ⭐ **The repair immediately paid for itself: the §8 artifact review it gates had been blocked behind it.** Census 34 clusters / 1,797,794,711 bytes → retired 30, freeing **1,591,802,083 bytes**; residue verified at 4 clusters / 205,992,628 bytes with `retirable: 0`. The four survivors are named with why — three are cited by tracked files and are therefore evidence, one is this session's own falsification cluster under the one-hour floor. All four confirmed present afterwards.
- Independent safety checks beyond the instrument's own: no live `postmaster.pid` under `target/pg-tests`, probed per cluster twice; and `target/pg-tests` shares the repository's filesystem id, satisfying the same-volume policy.
- Validation: `--self-test` rc=0 with all six refusal arms and both citation directions firing. FALSIFIED by restoring the literal — rc=1 naming the exact probe, rc=0 again on restore. Gate (17 checks) rc=0; no Rust source changed.
- ⛔ The second half of the finding is **not** closed: nothing runs any `--self-test` here. That is `SIGNOFF-REPAIR.11.4.3.1.7.2`, not something this commit fixed.

## 2026-09-13 — The stored certificate expiry was never read from the certificate (`SIGNOFF-REPAIR.3.4.3.1.1`)

- The leaf asked whether the server's outbound instants could be unified on one clock. Measuring says **no**: `rcgen` signs `not_after` INTO the certificate from the server process clock, and a verifier enforces that and nothing else, so it cannot move to the database clock without changing what the CA signs.
- 🔴 Measuring that turned up a defect the census had not seen. Both call sites computed `Utc::now() + LEAF_TTL_SECS` **independently of the instant baked into the certificate** — two derivations of one quantity. They did not agree: the certificate's instant is truncated to whole seconds and the stored one is not, so the stored expiry was wrong on every issuance, and on the enrollment path the caller's `now` is sampled before the transaction's database work, moving it further.
- ⚠️ Severity as it is, not inflated: nothing enforces the stored value, so **no certificate was ever accepted or refused wrongly**. What it was is a stored claim about an artifact that the artifact did not make, in the row an operator would trust to answer when a node's certificate expires.
- `issue_node_leaf` now returns the instant it signed, both call sites bind it, and a new public `ca::leaf_not_after(cert_der)` extracts the same value from a DER — public deliberately, because it is exactly the extraction the node performs when deciding whether to rotate.
- The clock question is answered as the leaf's second option, since the first is unavailable: the server's two clocks are **declared** coherent within a second and that assumption is now **measured** by a control, which samples the database clock between two process instants and takes only the divergence the round trip cannot explain. Its failure message says to fix the deployment's clocks or re-open `.3.4.3.1.2`, never to raise the bound.
- Recorded for that leaf, and it narrows its job: of the three instants the server sends a node, **only `decided_at` is ever read**. `cert_expires_at` and `lease_expires_at` are declaration-only on the node side, and the node's other cross-clock comparison uses the certificate's own embedded `not_after`, which is not a wire field at all.
- Validation: `run_pg_tests.sh node_enrollment node_channel node_inbox` rc=0 — **3 suites, 49 tests, zero failures**; server lib 102 + mtls 1 rc=0. Strict lint, fmt, gate (17 checks), book and link check rc=0.
- FALSIFIED against the restored re-derivation: **11 passed / 1 failed**, `left: …12.973747Z` against `right: …12Z`. ⚠️ 973 ms of that is the whole-second truncation and the enrollment work is the remainder — under 26 ms on this run, since the two pull in opposite directions. Both are real; the split is stated rather than the single scarier number.

## 2026-09-13 — There are three clocks, not two (`SIGNOFF-REPAIR.3.4.3.1`)

- `.3.4.3` closed the cache's backward-skew hole and opened this leaf for the other direction: a node clock AHEAD of the server refuses every dispatch. The leaf owed a census before proposing any rule, and the census refuted the leaf's own framing — both answers it was opened with are shaped around `decided_at`, and the population is neither one field nor one clock.
- **Three** server instants reach the node: `decided_at`, `cert_expires_at`, `lease_expires_at`. **Two** are compared against the node's own clock; the third is received and never read.
- 🔴 **The two live ones fail in opposite directions.** A node AHEAD refuses every dispatch. A node BEHIND by more than 300 s makes `cert_expires_soon()` first fire at real time `not_after - 300 + S` — *after* the workload certificate has expired — so rotation is too late and the channel breaks. A per-field patch shaped on the dispatch case would have fixed one and left the other.
- 🔴 **And the server does not have one clock either.** `decided_at` comes from PostgreSQL `clock_timestamp()` (22 `database_now` sites); `cert_expires_at` and `lease_expires_at` come from the server process (`Utc::now()`, 8 sites in `node_channel.rs`). Nothing reconciles them, so "the server's time" is not yet a single quantity — and an offset correction built today would silently assume they agree.
- ⭐ The census also found the shape the answer should take, already shipped one file from the defect: the reservation's `wall_clock_seconds` is a server-supplied **duration** the node anchors to its own clock, with no skew exposure at all.
- Split into three children along the measured population: `.1` the server's own clock split (the prerequisite), `.2` the node evaluating server instants in the server's terms (which closes the outage and makes the skew observable), `.3` the certificate's opposite-direction skew.
- ⛔ **Nothing is repaired in this commit and it does not pretend otherwise.** The dispatch outage is open at `.2`; the certificate finding is arithmetic on a source reading, explicitly not reproduced, and open at `.3`.
- Validation: documentation only — no Rust source, behaviour or qualification change. Gate (17 checks), book and link check rc=0.

## 2026-09-13 — ADR-009 is called "chain-in-envelope" and there is no chain (`SIGNOFF-REPAIR.3.4.5`)

- The leaf owed ADR-009 the comparative measurement its withdrawn claim never had. That claim took the envelope's own length `N` and asserted `N < N + 64`: true by construction, encoding no token and comparing no delegation depth.
- The measurement now exists as a re-runnable instrument, `crates/reasonbraid-core/tests/delegation_representation.rs`, needing no new dependency. Both shapes carry the same request and the same scope against a common **308 B** undelegated baseline.

  | Depth | Chain-in-envelope | Token (ES256) | Token (HS256) |
  | --- | --- | --- | --- |
  | 1 | 505 B | 1,066 B | 1,023 B |
  | 2 | 684 B | 1,824 B | 1,738 B |
  | 3 | 861 B | 2,582 B | 2,453 B |

- Per additional hop: **177 B** for the envelope against a constant **758 B** (ES256) / **715 B** (HS256) — roughly **4.3×**. The per-hop figure is precisely what a fixed increment could never have produced.
- ⭐ The gap is structural rather than an encoding detail: a credential must carry an issuer, audience, lifetime, replay id, key id and signature, and an in-request context needs none of the six because the server already knows them. It would not close with a tighter encoding.
- 🔴 **And the finding that matters more than the ratio: the ADR is titled "chain-in-envelope" and no chain exists.** `AuthorityContext` holds one `on_behalf_of` string, so a depth-2 delegation has nowhere to go. A control pins it, so the title cannot keep implying a capability the type does not have. Third independent measurement of the same absence, after `.3.4.1` (the grant-chain flags have no producer) and `.3.4.1.1`.
- ⚠️ Stated rather than left implicit: depth 1 encodes the shipped envelope, depths 2–3 are prototype-vs-prototype; the signature bytes are not a real signature and only their length is load-bearing, which is the algorithm's (32 B HS256, 64 B ES256/Ed25519) — the distinction that makes this different in kind from `N < N + 64`. And size is not the axis that decides the question: tokens buy offline verification and delegation while the delegator is unreachable, which no byte count settles.
- ⛔ No size advantage is offered as the reason for the choice. The ADR now says it rests on the subtraction and the existing revocation lifecycle — which is what it rested on, since the table did not exist when the choice was made.
- ⚠️ A fixture artifact was caught before the numbers were recorded: a short placeholder actor id on the final hop made the depth-3 token 37 B short and the per-hop cost look irregular. Corrected, and recorded — a first run with a spurious irregularity is where a plausible wrong explanation gets adopted.
- Validation: `cargo test -p reasonbraid-core --locked` rc=0, **74 tests, zero failures**. Strict lint, fmt, gate (17 checks), book and link check rc=0.

## 2026-09-13 — A `join` carrying a decline was accepted as a join (`SIGNOFF-REPAIR.3.4.4`)

- 🔴 Measured against the shipped decoder on a route that reads an **untrusted HTTP body**: `{"kind":"join","reason":"I decline"}` was **accepted as a join**, and so was `["join"]`. Same for `observe`. A respondent whose payload plainly says *decline* was recorded as having joined the deliberation panel, with no refusal to notice and the `reason` thrown away.
- Cause: the `.3.3.3.2.2.1` Serde branch. An internally tagged **unit** variant discards the rest of the map and also accepts the sequence form, and `deny_unknown_fields` — which this type declared — switches off neither. The members-carrying responses (`decline`, `defer`, …) were already strict, which is exactly what localises it to the unit shape.
- ⭐ **The census found the live one, and it is not the type the leaf was opened around.** 28 tagged enums across the tracked sources, 8 with the vulnerable shape; a reachability pass found exactly one decoded from untrusted input — `RecruitmentResponse`, which the leaf never mentioned. `Decision::Allowed`, the type it did name, has no untrusted producer at all.
- Repaired with the established pattern (private wire enum with empty-struct markers + object-only decoding) for `RecruitmentResponse`, `Decision` and `CachedDecisionKind`; both halves are needed, since the marker refuses the extra member and the object-only decoder refuses the sequence form. `object_only` is now a public, documented core export rather than a per-crate copy.
- ⚠️ `Decision` is deliberately **tightened** with a `deny_unknown_fields` it never declared. The leniency read a denial and its evidence back as an allowance (`{"decision":"allowed","reason":"the grant is revoked"}`), which for an audit decision is the dangerous direction. It tightens `Denied` too — the honest cost, stated rather than omitted.
- ⛔ The four adapter types with the same shape are **not** repaired, and the reason is measured rather than assumed: none is deserialized anywhere.
- The book gains "Responding to an open call" — the eight-response vocabulary, an example, and the strict rule. ⚠️ The endpoint had **no book coverage at all**; the wider gap (87 of 111 registered routes unnamed in the book, an upper bound) is now `SIGNOFF-REPAIR.11.8` rather than absorbed here.
- Validation: core lib 53, `authorization_evaluation` 6, server lib 101, and `run_pg_tests.sh profiles authority command_api` rc=0 with **3 suites / 96 tests, zero failures**. Strict lint, fmt, gate (17 checks), book and link check rc=0. FALSIFIED twice, each reversion isolating one crate's repair, with the pre-existing codec controls staying green throughout.
- ⛔ Not claimed: no evidence a real client ever exercised the acceptance. The finding is what the endpoint accepts, measured against the decoder.

## 2026-09-13 — A delegation without a scope is now unwritable (`SIGNOFF-REPAIR.3.4.1.1`)

- `CommandAuthz` carried the delegated **subject** and the delegation **scope** as two independent `Option` fields, so a subject with no scope was writable. `selection.rs` read the scope under `if let Some(scope)`, which meant that value would have delegated with the subject's **full grant selector** — the §16.3 widening invariant skipped rather than failed.
- They are one `Option<Delegation>` now, with `Delegation { subject, scope }` holding both non-optionally. 20 construction sites across 11 files; the only producer already set both, so no reachable behaviour changes.
- ⭐ **The leaf was opened calling the state unreachable. That held for production and NOT for the test suite.** `tests/authority.rs` constructed a delegate with no scope in two places, so the widening gate was being skipped inside a test that reads as though it exercises delegation. The compiler surfaced both the moment the state became unwritable — neither was found by reading.
- ⚠️ Giving those fixtures the scope they should always have had makes the invariant RUN where it previously did not. The audited-delegation test keeps `TenantWide`, matching the subject's own grant selector, so it still asserts the allow it always asserted — now through the gate instead of around it.
- The acceptance's "the type admits no delegate without a scope" is proved by the **compiler**, not a test: a temporary `Some(Delegation { subject })` yields `error[E0063]: missing field `scope``. ⚠️ It must be checked with `--all-targets` — a plain `--lib` check does not build `#[cfg(test)]` code and passed the control vacuously the first time.
- `policy_digest`'s subject input was hand-checked as byte-identical, because the digest is a **stored** value and an altered input would invalidate every historical authorization record.
- Validation: `run_pg_tests.sh authority command_api escalation mcp_write` rc=0 — **4 suites, 66 tests, zero failures**; every existing delegation control passes unchanged. Strict lint, check, fmt, gate (17 checks), book and link check rc=0.
- ⛔ Not claimed: no defect repaired and no runtime behaviour changed on a reachable path. This removes a latent state and strengthens two fixtures.

## 2026-09-13 — The cache's freshness window compared two different clocks (`SIGNOFF-REPAIR.3.4.3`)

- The leaf asked what a cached decision dated in the future means. Tracing the clock answered it: `decided_at` is the **server's database clock** (`database_now_in_tx`, and `tx.database_now()` on the replay path — the only two writers, and no request field reaches either), while the freshness comparison runs against the **node's process clock**. A future `decided_at` is therefore backward node skew and nothing else.
- 🔴 The consequence, stated as the bound it is: with the node behind by `S`, a cached allow stood for `S + 60 s` of observed time instead of 60 s.
- **That window is load-bearing, which is what makes the stretch matter.** The revocation epoch is bumped in exactly two places, both explicit revocations; a grant that simply reaches its own `expires_at` bumps nothing. So natural grant expiry is bounded at the node by the freshness TTL **alone**, and a stretched window is a stretched authorization window.
- ⛔ Both options the leaf was opened with were rejected by that measurement. Refusing a future instant refuses the server's own clock and turns skew into an outage; documenting it and moving on publishes a 60 s bound the product does not honour.
- The window now runs from `min(decided_at, received_at)` — the node's own receipt, already in its journal. It binds **iff** the node received a decision before its clock says the server made it, is byte-for-byte the shipped behaviour for any ordinary receipt, and can never refuse work the plain rule allowed: the window always ends at least one TTL after receipt.
- ⚠️ **The opposite skew direction is deliberately unchanged.** A node clock running *ahead* refuses every dispatch past 60 s of skew — fail-closed, journaled, safe. Backward skew failed *open*. Fixing only the fail-open direction is the correct scope; the availability defect is now owned by `.3.4.3.1`, along with whether the wire should carry a duration instead of an instant.
- 🔴 The clamp introduced a hazard the shipped suite does **not** catch: a replay rewrites `decided_at` on a row the journal already holds, so anchoring it to the ORIGINAL receipt would make a command replayed days later stale on arrival. The existing replay control passes either way because it seeds and replays within milliseconds. The receipt is refreshed with the decision, guarded on the decision having actually changed — every poll re-records every delivered command, so an unguarded refresh would re-anchor the window one poll at a time.
- Validation: `cargo test -p reasonbraid-core -p reasonbraid-node` over the affected set rc=0 — **10 targets, 116 tests, zero failures**. Five new controls; the five pre-existing cached-decision controls and the always-standing-deny unit test pass unchanged. Strict lint on both crates, fmt, gate (17 checks), book and link check rc=0.
- FALSIFIED three times, each reversion isolating one part: no clamp → both skew controls fail with the dispatch allowed; no receipt refresh → only the replay control fails, `expires 2026-09-11` against `decided 2026-09-13`; no guard → only the redelivery control fails.
- ⛔ Not claimed: the node journal is not a boundary and this does not make it one. Skew is now bounded, not **detected** — nothing reports that a node's clock disagrees with the server's.

## 2026-09-13 — A revoked delegation was told it had succeeded (`SIGNOFF-REPAIR.3.4.2`)

- 🔴 Measured on the unchanged path: a first request delegating to a LIVE role answered `200`; a second with the **same key and body** delegating to a **REVOKED** role answered `200 replayed=true` with `ok: true` — and wrote **zero** authorization records. A caller presented authority that had been revoked, was told it had succeeded, and left no trace of the attempt.
- Root cause: `request_hash` covered the operation, the actor and `envelope.body`, and `authority_context` is a SIBLING of `body`, not part of it. The idempotency claim is step 1 of the command transaction and authorization is step 2, so a matching hash returns the stored result — success or stored refusal — without evaluating the second request's authority at all.
- ⛔ **No new effect is applied by such a replay**, so this was never an escalation of what was written. What it was: an unauthorized request answered with a success, invisible to the audit. Both halves are stated rather than the scarier one alone.
- The hash now takes the authority context and appends it when present. It is `409 idempotency_mismatch` — the same key describing a different request.
- ⚠️ **The migration answer was chosen, not assumed.** The hash is a STORED value, so changing its inputs is a wire change. A request with no authority context hashes **byte-identically** to before, because the suffix is appended only when there is one, so every historical undelegated key keeps replaying — nearly all of them. A historical *delegated* key now conflicts instead of replaying: the safe direction, since it refuses rather than returning a result decided under a different authority.
- The control asserts both halves in one test: the changed context conflicts, AND a genuine replay — same key, body and subject — still returns the original result marked `replayed`. A binding that worked by breaking replay would fail it.
- Validation: `bash scripts/run_pg_tests.sh command_api` rc=0 — **36 passed / 0 failed**. The affected set `command_api command_ordering escalation mcp_write atomic_transaction invitations` rc=0 with **6 suites, 62 tests, zero failures**. Strict lint, fmt, gate (17 checks), book and link check rc=0.
- FALSIFIED against the exact pre-`.3.4.2` sources: **35 passed / 1 failed**, `left: 200, right: 200`, the body showing `"ok":true,"replayed":true` for the revoked delegation.

## 2026-09-13 — ⛔ Correction: the `delegable` flag governs chains that do not exist (`SIGNOFF-REPAIR.3.4.1`)

- **The previous commit's census was wrong, and this corrects it.** `.3.4` recorded that `evaluate()` never reads `grant.delegable` and called it a defect — a non-delegable grant backing a delegated request. The measurement was right; the inference was not, and acting on it would have broken a shipped, tested feature.
- Three measurements refute it. (1) Every dev-profile grant is issued `delegable: false`, under a boundary that is `delegable: false` with `max_delegation_depth: 0` — asserted from the database in the new control, not assumed — and the shipped `.1.4.2` delegation still succeeds, so the flag cannot have been gating that path or the feature could never have worked. (2) No grant's parent is ever another grant: issuance loads an `enrollment_boundaries` row, and `parent_or_root_authority` is a string on the boundary. The chain machinery has no producer, exactly as its own comment says. (3) §16.3 conditions delegation on invariants — no widening, both permissions evaluated — not on a flag, and those are enforced.
- ⭐ **And the gate the census had not looked for is the one that matters.** A probe had an unrelated human in the same tenant, holding the same full action set, act on behalf of a role: `403 … is not a participant of this thread`. Participation is enforced, and the refusal names the ACTOR rather than the subject — naming a well-placed subject does not launder an outsider in, which is the confused-deputy shape §16.3 exists to refuse.
- This leaf therefore ships no behaviour change. It ships two controls pinning the measured behaviour — the first ASSERTS the non-delegable premise before exercising the delegation, so it fails loudly if the dev profile ever starts issuing delegable grants — the two core fields annotated where they are discarded, and a book section with the four gates a delegated request must pass and an explicit "what `delegable` and `max_delegation_depth` do not do".
- ⛔ What genuinely is not required, with its bound: the subject's **consent** to a particular actor. The consequence is attribution only — the actor must already hold a grant covering this action *and this target*, and must already be a participant, so it gains no reach, only the audit naming the subject alongside it. Changing that is a wire and policy question no ADR currently answers, and it is recorded rather than invented.
- ⚠️ The leaf closes by refusing its own opening acceptance, which asked for the delegated request to be DENIED. The superseded wording is kept in the leaf rather than edited into agreement, because what the census measured was right and what it inferred was wrong, and a reader needs to see which was which.
- Validation: `bash scripts/run_pg_tests.sh command_api escalation authority` rc=0 — **3 suites, 61 tests, zero failures**. Strict lint on both crates, fmt, gate (17 checks), book and link check rc=0.
- `.3.4.2` is unaffected and still owns a live measured defect: the replay hash does not bind the authority context.

## 2026-09-13 — The delegation and cache surface, censused and split (`SIGNOFF-REPAIR.3.4`)

- The leaf's own goal line named four mechanisms — delegability, bounded depth, replay-hash binding, cache freshness — plus two inherited representation follow-ups. Split into five children before implementing, as `.10` and `.11` were, with the census command recorded so it re-runs at any commit.
- 🔴 **`evaluate()` — the function that decides every command — never reads `grant.delegable`.** The flag is read in exactly one place, and only in the direction "a grant claiming to be delegable must sit under a delegable boundary". Nothing asks, when a request arrives on behalf of someone, whether that subject's grant permits being delegated at all. A grant issued with `delegable = false` backs a delegated request exactly as a delegable one does. `.3.4.1` owns it.
- 🔴 **The replay hash does not bind the authority context.** `request_hash` hashes the operation, the ACTOR's identity and `envelope.body` — and `authority_context` is a sibling of `body`, not part of it. Two requests identical except for `on_behalf_of` therefore produce the same idempotency key, and the second replays the first's stored result without its own authority ever being evaluated. `.3.4.2` owns it, starting by measuring whether the store holds denials, which decides whether this is only an audit defect or also a refusal that can be replayed away.
- **Bounded depth is deliberately not enforced, which is not the same as missing**: `let _ = boundary.max_delegation_depth; // dev profile: only direct grants exist; chains are Phase 2`. A depth bound with no chains to bound is machinery without a population, so `.3.4.1` owns saying so in the book rather than repairing it.
- **A structural hazard that is NOT reachable today, recorded so it is not rediscovered as a defect**: the §16.3 widening check sits inside `if let Some(scope)`, so a `CommandAuthz` with a delegate and no scope would delegate with the subject's full selector. The only caller that sets a delegate always sets the scope too. `.3.4.1` owns making the pair unconstructible rather than merely unused.
- ⭐ **The cache's two halves fail in opposite directions.** `is_invalidated` uses `!=`, so a decision claiming a FUTURE epoch reads as stale and fails safe. `is_fresh` is `now < expires_at` and never reads `decided_at`, so a decision claiming to have been decided in the future is fresh for its whole stated window. `.3.4.3` owns deciding it — and the leaf notes that where the clock comes from settles it: if `decided_at` is server-produced, refusing a future value turns clock skew into an outage; if a node can influence it, refusing is the only safe answer.
- Task-tree and index only: no production source changed, and no repair is claimed. The frontier advances to `.3.4.1`.

## 2026-09-13 — The guard census, reconciled with instruments (`SIGNOFF-REPAIR.3.3.4.13`)

- `.3.3.4` closes here, and the closure's job is as much to say what is NOT covered as what is. Both censuses are tracked scripts a later leaf can re-run, because a reconciliation that re-reads the source proves only that someone read it twice.
- **The named-call census** reproduces its own recorded baseline exactly before reporting anything (`1ba6184`: 101 files, 1,749,975 bytes, its corpus SHA-256, 42 locations). At this commit: **118 files, 2,135,430 bytes, 27 direct named-call locations** — down from 42 while the corpus grew by 17 files. ⭐ Two numbers moving opposite ways is the integration's own signature: callers stopped naming an authority function and started entering a guarded service that names it once.
- **A new route-admission census** (`scripts/census_admission_paths.py`, `--self-test` 5/5) classifies each of **118 registered routes** by the gate it actually reaches: **18 guarded transactions, all mutating**; 9 pool inspections, 0 mutating; 11 pool tenant-admin, 3 mutating; 6 pool authorize, 1 mutating; 74 identity-only, 42 mutating.
- ⛔ **46 mutating routes do not run on a guarded transaction, and every one has a named owner** — policy 15 (`.9.1`), evaluations and routing 9 (`.8.2`), evidence 5 (`.7.4`), recruitment 3 (`.5.2`), resolvers and resources 3 (`.7.1`), deployment 3 (`.9.3`), regions 3 (`.3.2`), adapters 2 (`.10.1`/`.10.2`), workflow 1 (`.8.1`), directory matching 1 (`.5.1`), and `PUT /v1/profiles/{role_id}`, the one deliberately excluded because it is gated on identity and has no admission for a guard to order. **Being owned is not being repaired**, and this closure certifies none of them.
- ⚠️ **The instrument was wrong twice before it was right, and both are recorded in it.** A non-transitive version called five routes unadmitted, including `/v1/admin/grants/{grant_id}/revoke`, whose admission is one level down through `run_revocation`. A version that followed calls across the whole crate — to reach the SQL that now lives in the service modules — over-approximated to uselessness: 53 of 118 routes on the thread-command path, 41 on a guarded transaction, both false. The mutation column is therefore taken from the route's declared HTTP verb, a fact the router states rather than one a script infers.
- **No obsolete bypassing executor remains**: 84 functions across the authority modules, 10 never referenced, all 10 `#[test]` functions the harness reaches by name — confirmed by running them.
- ⚠️ That check was also wrong first, in the dangerous direction: it reported `insert_grant_row` and `load_boundary_by_id_in_tx` as dead. Both are live and generic, so `fn name<E>(` never matched a call-shaped pattern. An instrument that reports live code as dead invites someone to delete it, so the correction is a permanent note rather than a silent fix.
- Validation, the parent's selected BROAD step: **19 live suites, 262 tests, zero failures** (`authority authority_issuance authority_transaction administrative_effects enrollment_transaction command_api command_ordering node_channel node_enrollment node_inbox node_work node_replacement cards profiles federation quarantine site_authority site_registry_http bootstrap_recovery`), plus **99 library unit tests**. Strict lint, fmt, gate (17 checks), book and link check rc=0. ⛔ A whole-workspace `make check` was deliberately not run: the remote run is the authoritative pre-push gate, and the broad step this leaf owns is the wide LIVE suite run.
- promotion: `docs/knowledge/a-census-is-an-instrument-not-a-table.md` is new — reproduce the recorded baseline before reporting; find the scope at which a lexical closure stops working by running it and asking whether the answer is credible; and record an instrument's wrong answers inside it, especially the ones that ran in the direction of calling live code dead.

## 2026-09-13 — The card import now declares both tenants' guards (`SIGNOFF-REPAIR.3.3.4.12.1`)

- `.11.3` predeclared a limit before it could be closed: moving the federation-agreement read inside the import's transaction bought one consistent snapshot and NOT an ordering, because the direction verbs took no guard at all. `.12` gave them one and closed the importing half. This closes the other half.
- The import declares its guard set as the importing tenant EXCLUSIVE — it issues a grant — plus the ORIGIN tenant SHARED, because it reads the origin's agreement row and the guard contract says to declare every tenant whose domain state is used. A direction revocation takes its own tenant's key exclusively, so shared is exactly what gets fenced by it.
- ⛔ Declared to the RUNNER as one set, never two acquisitions and never a later upgrade: the sorted acquisition is the whole anti-inversion mechanism, and the guard API deliberately offers no upgrade on an existing context.
- An origin id that does not parse as a tenant id declares no second key, and the reason is ordering rather than tidiness — the set is built before the transaction opens, so turning a parse failure into a refusal there would move a card check AHEAD of the admission, the same wire-ordering mistake `.11.3` declined to make with the pure rungs. Self-import needs no special case: the runner normalizes a duplicate key to its strongest mode.
- Validation: `bash scripts/run_pg_tests.sh cards` rc=0 — **6 passed / 0 failed**. The affected set `cards federation profiles administrative_effects authority_transaction` rc=0 with **5 suites, 89 tests, zero failures**. Strict server lint, fmt, gate (17 checks), book and link check rc=0.
- FALSIFIED against the exact pre-`.12.1` source: **5 passed / 1 failed**, at `the import waits on the ORIGIN tenant's guard`. ⚠️ It does not fail slowly — the old import does not block AT ALL, because that transaction never declared the origin's key. The cleanest discrimination in this family: the fixture holds a key the old code had no reason to want.
- ⚠️ The fixture is an EXCLUSIVE holder on the ORIGIN key, a third shape after `.9`'s shared holder and `.11.3`'s injected fault, and deriving it took the same two questions: the repair moves the origin tenant from NO key to a shared one, so the holder must take that key in a mode excluding shared.

## 2026-09-13 — The three federation directions, each in one transaction (`SIGNOFF-REPAIR.3.3.4.12`)

- All three verbs were the `.9` shape exactly: an admission in an already-committed shared-guard transaction, then a mutation ON THE POOL — `propose` and `revoke` as bare statements, `accept` in its own untenanted transaction — with no guard, no effect record and no receipt on any of them. ⭐ As at `.11`, their tenant predicates were already correct: every statement matches on `(tenant_id, remote_tenant_id)` with the caller's own admitted tenant, which is what the both-sides pairing is for.
- 🔴 **The enumerate-the-constraints check promoted one commit earlier found its next instance on first use.** `federation_agreements.remote_tenant_id REFERENCES tenants` with the value caller-supplied, so proposing to a tenant that does not exist RAISED — `PROBE propose to an unknown tenant -> 500 {"code":"dependency_unavailable"}`. It is a typed `404 not_found` now, recorded. A `500` is not a contract.
- All three now run ONE transaction under the LOCAL tenant's **exclusive** guard, holding the admission, the mutation, the acceptance's cross-domain receipt and the effect record together.
- ⭐ The mode is derived from `.9`'s FIRST reason rather than its second: none of these advances the revocation epoch, so none is a revocation in the sense `.10.2` is — but all three are a classify-then-write over a row that **may not exist**, deciding `applied` against `no_op` or a refusal from how many rows they touched, and a row lock cannot cover an absent row.
- ONE guard key, not two: each verb writes only the local tenant's row — the pairing exists so neither side mutates the other's — and the proposal's counterparty check is a read of `tenants`, the minimal foreign-ID existence probe the guard contract permits.
- Three distinct bodies, as `.10.3` established for a three-verb family. The proposal's conflict arm carries `WHERE … IS DISTINCT FROM`, so re-proposing identical terms touches no column and records `no_op` while answering exactly as before. The acceptance tells its two idle states apart on the refusal path only: already accepted is `no_op`, never proposed is `refused`/`invalid_transition`, and both keep the single `409`. The revocation's idle state is the `revoked: 0` the caller already received.
- `accepted_at` is now the transaction's own database time rather than `now()`, which is BEGIN time — before the waits the acceptance queued behind.
- ⛔ Preserved deliberately: re-proposing with DIFFERENT terms still resets an accepted direction to `proposed`, which is how terms change and also means a direction can stop being effective without anyone calling revoke. What consumers are guaranteed across that stays `.5.3`'s.
- ⚠️ **Half a race is closed, and the leaf says which half.** A card import holds the IMPORTING tenant's guard, so it is now ordered against that tenant revoking its own direction — and still not against the ORIGIN tenant revoking its side, which takes a key the import does not hold. `.3.3.4.12.1` is opened for the both-tenant guard set, which the guard contract already implies: the import reads the origin's agreement row, and the contract says to declare every tenant whose domain state is used.
- Validation: `bash scripts/run_pg_tests.sh federation` rc=0 — **4 passed / 0 failed**. The affected set `federation cards profiles administrative_effects` rc=0 with **4 suites, 72 tests, zero failures**. Strict server lint, fmt, gate (17 checks), book and link check rc=0.
- FALSIFIED against the exact pre-`.12` sources: **1 passed / 3 failed**. The ordering control fails at `the revocation is waiting` — the old shape completed while a SHARED holder held the guard; the counterparty control reports `left: 500, right: 404`.
- ⚠️ The fixture is a SHARED holder, and choosing it required asking rather than reusing `.11.2`'s answer: that leaf took shared and no holder could discriminate it, while this moves shared → EXCLUSIVE, the transition a shared holder detects.

## 2026-09-13 — The profile and card chapter, with its claims checked by running them (`SIGNOFF-REPAIR.3.3.4.11.4`)

- The `/v1/profiles` surface had **no book chapter at all** — sixteen chapters in `docs/book/src/` and a grep for `profile` across them returned only workflow-profile and dev-profile prose, for seven shipped handlers. `docs/book/src/profiles.md` now covers all of them: the gates, the refusals, worked examples, the four reader classes and the per-field visibility defaults, the content-addressed history, the attestation, the export gate, the import ladder in rung order, and a closing section naming what is not implemented with the leaf that owns each gap.
- ⭐ **A documentation leaf that changes nothing should be cheap. This one produced two defects and three corrections, and that is the result worth recording.** Stating a behaviour forces you to enumerate behaviours; implementing one only exercises the path you built. Four drafted claims were checked by RUNNING them and three were wrong:
  - "importing the same card twice creates a second local role" — it answered `500` and recorded nothing. Repaired as `.3.3.4.11.5`.
  - "an unknown field is a typed `400`" — it is **`422`**: the body never deserializes into the typed profile, so the request never reaches the handler. The chapter now distinguishes the two, because a client treating every rejection as `400` mis-handles the typed ones.
  - "`expires_at` … no route filters on it" — true but far too soft. The directory's eligibility check compares a claim's `taxonomy_id` and `confidence` against the requirement and never reads the expiry, so **an expired claim still satisfies a requirement**. Stated plainly, with `.5.1` named as its owner.
- ⚠️ One documented claim had no control anywhere, and the chapter would have been its only assertion: that only the `full` class exports the portable card. `only_the_full_class_exports_the_portable_card` now covers it — both owner-class readers export, a same-tenant sibling refuses `403` while still reading the `tenant` view of the profile (the card is not a filtered form), and a stranger refuses identically. Every other status the chapter publishes was traced to an existing control rather than duplicated.
- promotion: `docs/knowledge/writing-the-documentation-is-a-verification-pass.md` is new. Implementing asks "does my path work"; documenting asks "what happens for each thing a reader might do", and only the second enumerates. A sentence you are about to publish is a hypothesis — run it. ⛔ With one exception: when the enumeration finds a defect, the control asserts the REPAIRED behaviour, never the measured one, or the test enshrines the defect.
- Validation: `bash scripts/run_pg_tests.sh cards profiles` rc=0 — **2 suites, 43 tests, zero failures**. Strict server lint, fmt, gate (17 checks), book and link check rc=0; the rendered chapter was inspected for its examples and cross-references.
- `SIGNOFF-REPAIR.3.3.4.11` closes with this. It grew from three children to five: the transactional writer, the guarded attestation, the atomic import, this chapter, and the colliding-label defect the chapter's own drafting uncovered — plus `.3.3.4.7.4`, opened along the way because the import's allowlist rung refuses an ADMITTED caller and the refusal vocabulary had no word for it.

## 2026-09-13 — Re-importing a card answered 500 and recorded nothing (`SIGNOFF-REPAIR.3.3.4.11.5`)

- 🔴 Measured on the current route while drafting the next leaf's documentation: `PROBE first import -> 200`, `PROBE second import -> 500 {"code":"dependency_unavailable"}`, and **no administrative effect recorded** for the failed attempt. An admitted request that failed was invisible in the evidence the whole `.7`/`.8` family exists to produce.
- Root cause from the schema, not the error text: `agent_roles` carries `UNIQUE (tenant_id, name)` and `enrollments` carries `UNIQUE (tenant_id, kind, name)`, while the import writes the card's `display_label` into both. The `agent_roles` insert runs first and RAISES.
- ⚠️ **It is not only a re-import.** Any card whose display label matches an identity already in the importing tenant collides — two different origin tenants exporting a role called "schema reviewer" is enough.
- ⭐ **Making the import atomic one leaf earlier is what turned this from a survivable mess into an unrecordable one.** Once the admission and the effect record share the transaction, a raised constraint takes them down with it. `docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md` said exactly this at `.10.1`, and `.11.3` did not apply it to its own inserts.
- Both inserts now use `ON CONFLICT … DO NOTHING RETURNING`, so zero rows returned IS the refusal with no abort. The answer is `400 invalid_command` naming the label, and the effect record says the same sentence.
- ⛔ It says the label is TAKEN. It deliberately does not decide what a repeated import ought to do: the ordinary enrollment route answers a `(tenant, kind, name)` collision with a replay, returning the original principal id, and whether a card import should do the same is card replay semantics owned by `.5.3`. Inventing replay here would be deciding another leaf's question on this leaf's finding.
- Wire: `500 dependency_unavailable` becomes `400 invalid_command`. A `500` is not a contract, so this is a correction rather than a break.
- Validation: `bash scripts/run_pg_tests.sh cards` rc=0 — **4 passed / 0 failed**. The affected set `cards profiles federation administrative_effects identity_store` rc=0 with **5 suites, 72 tests, zero failures**. Strict server lint, fmt, gate (17 checks), book and link check rc=0.
- FALSIFIED against the exact pre-`.11.5` sources: **3 passed / 1 failed**, `left: 500, right: 400`, with `duplicate key value violates unique constraint "agent_roles_tenant_id_name_key"` in the log.
- ⚠️ **The probe that found this was not a test anyone set out to write.** The chapter needed a "what is not here yet" section, its draft said "importing the same card twice creates a second local role", and running it rather than publishing it returned `200` then `500`. Writing documentation is a verification technique.

## 2026-09-13 — A failed card import left the role, its grant, its quota and its receipt behind (`SIGNOFF-REPAIR.3.3.4.11.3`)

- 🔴 Measured on the unchanged route, with every new profile version row made to fail and nothing else touched: the import answered **`500`** and the imported role was still there — `left: 1, right: 0` — along with its grant, its per-principal quota row, its enrollment and its cross-domain receipt. An identity the directory cannot describe, created by a request that told its caller it had failed.
- Root cause: the route committed the grant, the identity, the quota, the enrollment and the receipt as one transaction and then wrote the profile ON THE POOL. It also read its admission, its active boundary and its effective federation agreement OUTSIDE the transaction that used them, so the boundary the grant was checked against could have been revoked in between.
- The whole ladder now runs in ONE transaction under the importing tenant's **exclusive** guard. The mode is structural rather than stylistic: the import ISSUES a grant, and `create_grant_in_guard` asserts the exclusive scope on the connection it is handed, so a shared acquisition could not create the grant at all.
- ⭐ **The split's own plan lost to a re-reading, for the third time in this parent.** `.11.3`'s opening note said the pure rungs should move before the transaction. The superseded route admitted FIRST and verified the card second, so moving them would have told a caller who is not this tenant's administrator that their card is malformed instead of refusing them. They stay after the admission.
- ⚠️ **An existing test caught a wire change that was not intended**, and the fix is the interesting part. `cards.rs` asserts the grant refusal begins `the imported role's grant exceeds the importing boundary: `; routing that refusal through `GrantCreateError`'s own `Display` would have replaced it. There is now ONE renderer used by both the HTTP response and the effect record, so the two descriptions of the same refusal are the same string by construction rather than by care.
- The effect's target is the digest the server RE-DERIVES from the submitted card, not the one the caller presented. They differ on exactly one path — when the digest rung refuses, the presented value is by definition not a digest of that card — and the re-derived one is always a usable target id, which an arbitrary caller-supplied string is not.
- The allowlist rung is what `.3.3.4.7.4` was opened for: it refuses an ADMITTED caller with body code `unauthorized`, and the record now says the same word instead of being unrecordable.
- Three superseded bridges are DELETED rather than left beside their replacements: `create_grant_unordered_in_tx` (whose own comment named this leaf as the one that would migrate its last caller), `load_active_boundary_for_tenant` (which called itself "an explicit temporary bridge until its complete integration"), and the pool-taking `profiles::write_profile` that `.11.1` introduced for exactly the two callers `.11.2` and `.11.3` have now moved. Every remaining profile writer runs inside a transaction its caller owns.
- ⛔ What this does NOT close, stated because it would otherwise be assumed: moving the agreement read inside buys one consistent snapshot with the writes and nothing more. `federation::propose`, `accept` and `revoke` take no tenant guard, so no guard set here fences a concurrent revocation — `.3.3.4.12` owns that ordering.
- Validation: `bash scripts/run_pg_tests.sh cards` rc=0 — **3 passed / 0 failed**. The affected set of **9 suites, 156 tests** passes with zero failures; it includes every suite covering a `grant_creation` caller, because the shared renderer touches that path. Strict server lint, fmt, gate (17 checks), book and link check rc=0.
- FALSIFIED against the exact pre-`.11.3` sources: **1 passed / 2 failed**, the discriminating one reporting the orphan verbatim.
- ⚠️ The discriminating fixture is a FAULT, not a lock holder, and the transition is why: the superseded failure mode is not a race but a second transaction, so no guard mode change and no contention fixture could expose it. What exposes it is making the second transaction fail and looking at what the first one left.
- `SIGNOFF-REPAIR.3.3.4.11` now has all three transaction children landed; `.11.4` writes the profile/card chapter the family still lacks.

## 2026-09-12 — The fourteenth handler refuses an admitted caller, and the vocabulary had no word for it (`SIGNOFF-REPAIR.3.3.4.7.4`)

- `.7.3` derived the administrative refusal vocabulary by parsing the fourteen handlers for `ControlApiError::` constructors and classifying them by name, concluding that `unauthorized` is always "the admission's own denial and already the admission record's job". Building the card import showed that is true **thirteen times out of fourteen**.
- The fourteenth is `import_profile_card`'s ALLOWLIST rung: it refuses a caller who WAS admitted — an administrator of their own tenant — because that tenant holds no effective federation agreement with the card's origin. That is a precondition about the two tenants, not about the caller's grant, and the response body carries `code: "unauthorized"`.
- ⚠️ **This is the seventh instance of the pattern `.11.6` is censusing, and the second one inside `.7`.** `.7.3` wrote "the reasoning was sound and the set was wrong" about its own predecessor; the same sentence applies to it, one rung down. A census that classifies by a constructor's NAME rather than by what each site MEANS will be right wherever the name and the meaning agree, and silently wrong where they do not.
- The gap is measured to be one value rather than assumed to be one — the census re-runs across all fourteen operations and returns `handlers not found: none` and exactly one domain-refusal site. That is the check `.7.3` itself insisted on when it refused to patch the §9.8 registry with a single value.
- `AdministrativeRefusal::Unauthorized` is added, its wire name literally the `code` the response body carries, so the invariant that the record and the response cannot disagree keeps holding by construction. Without it the allowlist refusal could not have been recorded at all.
- ⛔ Not a widening of what writes an effect: a DENIED ADMISSION still writes no effect record, so a `refused` outcome carrying this code always means the request was allowed and the operation was not.
- No migration — migration 0058's `CHECK` pins only the outcome object's `kind`, and this field lives beneath it. `.7.2`'s layering pays for itself a second time.
- Validation: `cargo test -p reasonbraid-core --locked` rc=0, **68 tests**; `bash scripts/run_pg_tests.sh administrative_effects` rc=0, **25 passed / 0 failed**, so the stored `kind` vocabulary and migration controls are untouched. Strict lint on both crates, fmt, gate (17 checks), book and link check rc=0.
- FALSIFIED: leaving `"unauthorized"` on the control's fail-closed list makes it fail at `` `unauthorized` decoded as an administrative refusal `` (**9 passed / 1 failed**), so the control discriminates the change rather than passing either way.

## 2026-09-12 — Two owners attested two claims and one attestation vanished (`SIGNOFF-REPAIR.3.3.4.11.2`)

- 🔴 The census predicted this from source; the control measured it. Two administrators attesting two DIFFERENT capabilities of one role, against the unchanged route: **both received `200`**, and the profile the role then published carried `code_review` upgraded to `owner_attested` and `schema_design` still `self_asserted`. One administrator's audited attestation was gone, with no error reported to anyone and a perfectly ordinary version 3 to show for it. An audit trail that silently drops an entry is worse than one that refuses — nothing in it says a record is missing.
- Root cause: `attest_capability` read the current profile ON THE POOL, edited one claim in memory, and wrote through a different transaction. Both attestations started from version N and each wrote a profile carrying only its own upgrade.
- ⭐ **Serializing the writers would not have fixed it, and that is the part worth keeping.** `.11.1` already made the writers serialize at the role's anchor; the losing attestation's version NUMBER would have been correct while its CONTENT was stale. The READ had to move under the same lock as the write. A lost update and a version collision look like one problem and are two.
- The verb now runs ONE transaction under the role's tenant SHARED guard: database time after the wait, the admission on that connection, the role's tenant re-read inside, the anchor taken `FOR UPDATE`, the read, the single claim's upgrade, the new version, and the final effect record.
- ⭐ The mode is derived, not inherited: an attestation writes no authority and advances no revocation epoch, so it is not a revocation in the sense `.10.2` is — what it needs is to be fenced BY one, which shared gives. Exactness against a concurrent attestation comes from the anchor lock, at the granularity that actually conflicts.
- The anchor is taken only when one already EXISTS, so a refusal creates nothing. An absent anchor means no profile at all, since the writer always creates the anchor before any version row in the same transaction.
- Wire: unchanged. The `404` still answers "no profile" and "no such capability" with one message, and the effect record distinguishes them — both `refused`/`not_found`, against an upgrade's `applied`. One addition: every answer that reached an admission carries `x-reasonbraid-authorization`; the pre-admission `404` for an unknown role deliberately does not. A repeat attestation still writes a new version and records `applied` rather than `no_op`, because a new version row IS protected state changing.
- The superseded pool-taking `attest_capability` is deleted rather than left beside its replacement.
- Validation: `bash scripts/run_pg_tests.sh profiles` rc=0 — **38 passed / 0 failed**. The affected set `profiles cards federation administrative_effects` rc=0 with **4 suites, 65 tests, zero failures**. Strict server lint, fmt, gate (17 checks), book and link check rc=0.
- FALSIFIED against the exact pre-`.11.2` sources: **36 passed / 2 failed**, the discriminating one reporting `left: Some("self_asserted"), right: Some("owner_attested")`.
- ⚠️ The mode does not change in this repair (shared before, shared after), so — as `.10.1` established — no lock-holding fixture discriminates it in either mode. The discriminating properties are the lost update and atomicity. The third new control passes at the baseline and is labelled a regression control in the suite.

## 2026-09-12 — Two people editing one profile at once lost a write to a 500 (`SIGNOFF-REPAIR.3.3.4.11.1`)

- 🔴 Reproduced before it was repaired, and worse than reading the source suggested: four concurrent writers for one role, twice, against the unchanged writer — **five of the eight answered `500`**, each logging `duplicate key value violates unique constraint "profile_versions_role_id_version_key"`. An ordinary role rewriting its own profile from two places at once lost writes to an internal error, and the version ledger the content addressing exists to keep is where the loss landed.
- The writer computed its version as a read-then-write: read `current_version`, add one, insert under migration 0019's `UNIQUE (role_id, version)`. Two writers both read N, both wrote N+1, and the constraint RAISED for the second rather than queueing it.
- ⭐ **The obvious fix is wrong, and the reason is the durable part.** Locking the anchor row with `SELECT … FOR UPDATE` protects every write except a role's FIRST — over a row that does not exist yet the lock matches nothing, locks nothing and returns immediately. The anchor has to be created INSIDE the acquisition (`INSERT … ON CONFLICT DO NOTHING`, then `SELECT … FOR UPDATE`), which is the same two-step the tenant authority guards already use for first-use acquisition. Promoted as `docs/knowledge/serializing-writers-at-a-row-that-may-not-exist.md` at the parent's split, one commit before it was needed.
- ⛔ The `UNIQUE` constraint is kept and its role changes: it is now the backstop that proves the lock works and should never fire. It also could not have stayed the serialization story — a violation aborts the transaction, so once `.11.2` and `.11.3` must record an effect in the same commit, a raised refusal cannot be recorded at all.
- ⭐ **Guard mode chosen: none, and that is derived rather than skipped.** `PUT /v1/profiles/{role_id}` is gated on identity — only the role itself writes its own profile — evaluates no grant and produces no authorization record, so there is no authority decision for a tenant guard to order it against, and taking one would block every concurrent operation in the tenant to buy no invariant. What it needs is serialization against itself: a row lock at the role, not a guard at the tenant. `.11.2` and `.11.3` do take guards, because theirs are admitted.
- `write_profile_in_tx` now runs on a caller-owned transaction — the form `.11.2` and `.11.3` need to reach one commit — and `write_profile` is a thin wrapper over it, so there is one writer rather than two. `put_profile` opens that transaction itself, so its role-existence gate, its lineage probe and its write share one snapshot.
- Wire: one value changes, `written_at` is the write transaction's own database time rather than a process clock read taken before the anchor wait. Status codes, response fields and the content addressing — identical content still hashes identically — are unchanged.
- Validation: `bash scripts/run_pg_tests.sh profiles` rc=0 — **35 passed / 0 failed**. The affected set `profiles cards federation administrative_effects` rc=0 with **4 suites, 62 tests, zero failures**. Strict server lint, fmt, gate (17 checks), book and link check rc=0.
- FALSIFIED against the exact pre-`.11.1` sources: **34 passed / 1 failed**, `left: 500, right: 200` at `round 1 writer 0`.
- ⚠️ **The discriminating control is probabilistic, and saying which of your controls is which is part of the evidence.** No lock-holding fixture can discriminate this repair: the superseded upsert and the new `FOR UPDATE` contend on the same anchor row, so a held lock blocks both. What changed is where in each sequence the contention happens, and its only external consequence is the outcome under real concurrency. An earlier two-writer version of the control passed once against the defect by luck — which is why it is now four writers over two rounds. The lineage-refusal control passes at the baseline too and is labelled a regression control in the suite.
- ⚠️ Found while updating the book and TASK-TREE OWNED rather than reported: **the `/v1/profiles` surface has no book chapter at all** — a grep across `docs/book/src/` returns only workflow-profile and dev-profile prose for seven shipped handlers. This commit added the transaction-discipline section its own change required; the chapter is opened as `.3.3.4.11.4`, to be written after `.11.3` settles the wire rather than written twice.

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated eight times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

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
