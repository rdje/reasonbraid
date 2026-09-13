# CHANGELOG.md

## 2026-09-13 — A token does not outlive the authority that issued it (`SIGNOFF-REPAIR.4.1.2`)

- **THE DECISION, and it was genuinely open.** Redemption validated only the token's own fields, so a token issued by an administrator whose grant or boundary was revoked before it was redeemed still enrolled the node. A bearer credential surviving its issuer's revocation is defensible — it is how most bootstrap tokens work. This project's own answer is the opposite, and it has been given twice already: `.3.5.2` (a revoked boundary left the metrics surface open) and `.4.1.3`/`.4.1.3.1` (a revoked node kept running and kept being fed). **Withdrawn authority stops producing effects**, and a redeemed token produces an enrolled node.
- **censuses run before arguing, because a decision whose check is unimplementable is not a decision.** The token table records *no issuer at all*; but `authorization_records` already stores the `grant_id` and `boundary_id` of every allowed decision, and issuance writes that record in the SAME transaction as the token — so the link is one column away and resolves to the exact authority, not to a principal. And there is **no route that cancels an outstanding token**: after withdrawing an administrator the operator could only wait out the TTL.
- ⭐ **The harder decision was WHERE, and redemption-time re-checking is REJECTED on this codebase's own repair history.** `enroll` takes no tenant authority guard, so re-reading a grant's status there would read authority state outside the guard a revocation holds exclusively — the class `.3.3.4.5` repaired. Voiding at REVOCATION time needs no new guard: that transaction already holds the exclusive guard, and the token row is already redemption's declared serialization point, so the row lock alone orders them.
- ⛔ **Tenant-wide voiding was rejected as too broad** — it needs no schema change at all, which is what makes it tempting, but revoking node X would void a pending token for node Y.
- 🔴 **The interaction caught before the migration was written:** a voided token would still satisfy the one-live-token index predicate, so it would hold its node's slot for ever — the operator revokes a compromised administrator and can never issue a replacement token. That is `.4.1.1`'s lockout re-entered through a different door. The index now keys on `voided_at`, and a control proves the slot is freed; neutralising that key returns `.4.1.1`'s message **verbatim**.
- ⛔ `voided_at` is a NEW column rather than a reuse of `superseded_at`, whose own `COMMENT` means something else entirely. Redemption answers a DISTINCT reason — *the authority that issued this token has been revoked* — because re-issuing under that authority will not help and neither will waiting, which is what "expired" would have implied.
- 🔴 **The checked fixture-cleanup plan caught the new foreign key's cost before any test did, and the census behind the fix was wrong TWICE.** It first found only lists holding both tables, missing three that purge the parent without naming the child — of which only one was a real cleanup plan. It was then scoped to two test directories, and a cleanup list lives in `crates/reasonbraid-mcp/src/lib.rs`. **A census scoped to the directories you expect is not a census.** Both misses were caught by an instrument, and both would have survived a selective run: the MCP one is the 44th suite of 44.
- Validation: falsified in three parts for three properties, each failing on a different assertion — the voiding disabled hands the withdrawn administrator's token a full certificate; the index key removed returns the lockout message; the selection widened voids a colleague's token in the same tenant. ⚠️ The third neutralisation's first attempt did not compile and was redone rather than reported. **Broad PostgreSQL run: 44 suites, 371 tests, 0 failed**; clippy `--all-targets --all-features -D warnings` rc=0; `mdbook build` ok.

## 2026-09-13 — The token's lifetime is validated before anything computes with it (`SIGNOFF-REPAIR.4.1.2.1`)

- Opened while measuring the PREMISE of a neighbouring decision. `.4.1.2` argues about a bearer token's exposure from "it expires" — and the expiry was `req.ttl_seconds` straight off the wire, an unvalidated `i64`. A decision whose premise is false is not a decision.
- 🔴 **Reproduced against the live route first, and it was THREE different failures where one was suspected.** `i64::MAX` from an **unauthorized** caller **panicked** the handler and dropped the connection — `TimeDelta::seconds` panics above `i64::MAX / 1_000`, and it ran BEFORE any authorization, because the principal header is only parsed at that point. `1_000_000_000_000_000` from an admin panicked at a different site, `at + ttl` leaving chrono's date range, **inside the tenant's authority guard**. And a century was simply issued: `200`, `expires_at: 2126-09-14`.
- ⭐ **The source reading predicted the wrong one.** Two of the three values answered a correct `403` to the unauthorized caller, because the addition happens inside the guard after the admission. Only the extreme magnitude crossed the panic threshold, and only from outside the gate. Reading said "could panic"; only driving it said which value, which caller, which of two sites.
- Fix, one placement answering all three: a range check beside the existing node-id shape check, before any arithmetic, refused as a typed `400 invalid_command`. The bounds are named constants — 3 600 s default, 86 400 s maximum — because `.4.1.2` reasons from this bound and a bound worth reasoning about is worth naming.
- The cap is a judgement, recorded as one: one day, so an operator can prepare a node ahead of a working day and no further. ⚠️ It **narrows a wire contract** deliberately, and the book says so. `ttl_seconds <= 0` is refused too: a token born expired can never be redeemed, and before `.4.1.1` it was exactly the row that locked its node out for good.
- ⚠️ **Three controls used `ttl_seconds: -1` to mint a lapsed token; the fixture is COMPLETED, not deleted.** `.4.1.1` wrote down why it used the route rather than a test-side `UPDATE` — the lapse must happen through the product's own expiry, not through a write to the row under measurement — and the replacement keeps that property exactly, issuing at the shortest legal lifetime and waiting on the database's own clock. The resulting row state is identical to production's, so what those controls prove is unchanged.
- 🔎 **The original source census already had this, and the split dropped it.** `census-1.md` record `R-31-32-5` routes to `.4.1` and names three things; `.4.1`'s census carried exactly one into its split. The mechanism is not carelessness — `.4.1` censused its own GOAL LINE rigorously and never re-read the record pointing at it, so a rigorous census of the wrong list looks exactly like a rigorous census. Owned as `.11.9`, with the still-unopened third clause as its first routing output.
- Validation: falsified in two arms that DISCRIMINATE — the whole check disabled fails the control on a dropped connection (the panic); the panic range guarded but the cap widened fails it instead on a `200` expiring in 2126. `node_enrollment` 16/16, `node_replacement` 1/1, `node_result_ordering` 6/6, `node_work` 8/8, `profiles` 38/38, `cli_end_to_end` 5/5; clippy `-D warnings` over server and CLI rc=0; `mdbook build` ok.

## 2026-09-13 — A revoked node is handed no new work, and the work is withheld (`SIGNOFF-REPAIR.4.1.3.1`)

- **THE DECISION: a node whose workload credential is not currently usable is handed NO work, and the work is WITHHELD rather than dropped.** `.4.1.3` bounded a revoked node's session to its remaining lease but left the server handing it newly enqueued work for that tail. Continuing to dispatch to an identity the operator has just withdrawn is not a courtesy to a running session — it is fresh authority granted after the withdrawal.
- **census of the delivery ladder BEFORE a rung was proposed**, because a filter at the wrong rung either drops work a healthy node should get or leaves a second path open. **2 writers** into `node_inbox` (one with no production caller); **7 readers**, of which exactly **one delivers rows to a node** — two are tenant-admin inspection surfaces, three are result-folding lookups, one is the cursor read, one the cursor allocator.
- ⚠️ **The census found the second path and it was already gated.** The delivery read has **two** callers, not one: `poll` AND the handshake, whose response carries a replay tail. The handshake's is gated by the certificate proof, so a filter written into `poll` alone would have been correct today and silently incomplete. The predicate went into the shared read instead, where it is provably vacuous on the handshake path.
- ⛔ **The enqueue rung was rejected on that census, not on taste.** Refusing at dispatch would abort the thread transaction that produced the work and, worse, destroy it for a node about to be REPLACED — the replacement ritual re-enrols the same node id and replays the tail. Withholding preserves it; a control proves the replacement receives both withheld rows in cursor order.
- Fix: the delivery read asks the question the lease renewal asks, asked the same way (`revoked_at IS NULL AND expires_at > now()`, on the database clock). It is a **filter, not a refusal** — `poll` still admits on its fencing check and answers the true cursor, `ack` and `events` are untouched — which is what keeps `.4.1.3`'s seam intact.
- 🔴 **A durable claim this project had recorded three times turned out to be stale, and a control is what caught it.** `.4.1.3`, `MEMORY.md` and this changelog all said `node_presence.suspended` means *ever revoked* (migration 0012) and would starve every replaced node. **Migration 0017 supersedes 0012** — the view reads *revoked exists AND no unrevoked exists*, written for exactly that hazard. The rejection of `suspended` stands, on a narrower fact measured here: 0017 never asks whether the surviving certificate is IN DATE. Promoted to `docs/knowledge/a-schema-object-is-its-latest-migration.md`.
- ⚠️ **A fixture was completed, not deleted.** `the_zero_concurrency_wake_gate_holds_the_delivery` seeded a role with no certificate and called the delivery read directly — a poller that could never have existed, since a fencing token comes only from a certificate-proof handshake. The fixture was incomplete, not the rule wrong; its two-sided shape still fails if the concurrency filter is removed.
- Validation: falsified in three separate neutralizations, each verified to have landed — whole predicate removed (2 failed), the revocation half alone (2 failed), the expiry half alone (1 failed, only the lapsed arm), so both halves are load-bearing and the controls discriminate. The first neutralization attempt REFUSED to apply because its anchor string also occurs in the lease renewal. `node_channel` 32/32, `node_replacement` 1/1, `node_work` 8/8, `node_inbox` 7/7, `node_result_ordering` 6/6, `quarantine` 1/1, `command_api` 37/37; clippy `-D warnings` rc=0; `mdbook build` ok.

## 2026-09-13 — A revoked node renewed its own lease for ever (`SIGNOFF-REPAIR.4.1.3`)

- 🔴 **Revoking a node did nothing to a node that was running, and kept doing nothing.** Measured, not read: every channel surface driven with the same still-live fencing token after a real `POST /v1/nodes/revoke` answered `heartbeat ALLOWED · poll ALLOWED · ack ALLOWED · events ALLOWED`, with only `rotate` and `handshake` refused — the two operations that re-present a certificate. `heartbeat` verified the fencing token and renewed, and the fencing check reads no ledger fact by design, so a node heartbeating inside the 60 s lease TTL never needed the handshake that would have refused it.
- ⭐ **The prior decision was not wrong; it was unfinished.** `.1.3.1` chose deliberately that "suspension gates re-entry, it does not pretend the running session never existed". That reading is bounded — the session runs out its lease, then re-entry is required and refused. What shipped was the unbounded version, because re-entry was never required. The difference between the two is one verb.
- Fix, and the SEAM is the decision: the credential check goes into the lease RENEWAL and deliberately not into the fencing verification. Poll, ack and events keep their pure fencing check — that is the bounded tail `.1.3.1` intended — and only the extension now requires a certificate that is neither revoked nor expired. The condition is folded into the `UPDATE` so no window exists between the decision and the write, and the certificate's liveness is read on the database clock, the same question the handshake asks asked the same way.
- ⛔ **`node_presence.suspended` is the WRONG predicate, rejected on measurement rather than taste.** Migration 0012 defines it as *ever revoked*; after the replacement ritual a node holds both a revoked old certificate and a fresh active one, so gating renewal on it would have left every replaced node silently unable to hold a lease.
  - **Correction, 2026-09-13 (`.4.1.3.1`):** the rejection stands, the reason above does not. **Migration 0017 supersedes 0012** — `suspended` reads *a revoked certificate exists AND no unrevoked one does*, so a replaced node reads NOT suspended and would not have been starved. What actually disqualifies the column is that 0017 never asks whether the surviving certificate is still IN DATE.
- The node is told which refusal it got: a withdrawn credential is distinct from a fenced token, because re-handshaking fixes the second and cannot fix the first.
- ⚠️ **Two limits stated rather than implied.** The remaining tail is up to a full lease, and during it the server still hands the node NEWLY enqueued work — measured (`commands delivered 1`) and deliberately not repaired here; `.4.1.3.1` owns that decision, and the control asserts nothing about it in either direction so today's answer is not enshrined. And nothing at the transport re-checks a certificate: `mtls::build_server_config` is referenced only by its own test and `rb-server` serves plain HTTP, which `PHASE-7` and the book's Honest Boundaries already name.
- Registered as **R-REVOKE** in the live risk register.
- Validation: falsified by neutralizing the predicate (`30 passed; 1 failed`, exactly the one control) and restored (`31 passed`). `node_channel` 31/31 — including `.1.3.1`'s own control, whose `online == true` after revocation still holds because the repair refuses the extension and never touches the lease — `node_replacement` 1/1, `node_work` 8/8, `node_inbox` 7/7, `node_result_ordering` 6/6; clippy `-D warnings` rc=0.

## 2026-09-13 — A token that expired unused locked its node out for good (`SIGNOFF-REPAIR.4.1.1`)

- `.4.1`'s goal line named five mechanisms, so the leaf was censused before it was split. **One confirmed defect, one worry REFUTED outright, three left open and named as unmeasured.**
- 🔴 **The defect.** Migration 0018 narrowed the token table's unique index to `(node_id) WHERE used_at IS NULL` — "one unused token per node". An EXPIRED token still has `used_at IS NULL`, so a token issued and never consumed occupied that index permanently: every later issuance for that node id answered `409`, and the lapsed token itself enrolled nothing. The node's ordinary enrollment path was closed for good.
- ⭐ **The 409's own message named the recovery, and the recovery was the defect**: *"consume or expire it before issuing another"*. Expiring it is exactly what had happened, and it is what shut the door. Reproduced against the live routes before anything changed, with the response body captured verbatim.
- ⛔ **The obvious repair is unavailable**: the index predicate cannot become `AND expires_at > now()`, because a partial index predicate must be IMMUTABLE and `now()` is not. So the liveness the index cannot evaluate is written down — migration 0059 adds `superseded_at`, the index keys on it, and the issuance transaction stamps a lapsed row before its insert, at the same database time the admission was evaluated with.
- ⛔ Three alternatives were checked and rejected, each for a stated reason: not DELETE (an audit row would name a token id resolving to nothing), not REUSE the row in place (an audit row would resolve to a *different* token's facts — and it is the option needing no migration, which is what makes it tempting), and never stamp `used_at` (the token was never redeemed; recording it as used would make the store lie).
- ⚠️ **Stated bound:** the supersede carries `AND tenant_id = $2`, so another tenant's lapsed token still blocks. Superseding it would be a write into another tenant's rows under this tenant's guard. That is the global-index territory `.3.5` owns; this leaf neither widens nor repairs it.
- ⚠️ **A control was written, then rewritten, because the first version broke a recorded decision.** It had asserted the foreign tenant's `409` — precisely the control `.3.3.4.10.1` refused to commit, "rather than commit a control that would enshrine the defect". It now asserts nothing about that status and everything about the row, so it holds under the current index and under whatever `.3.5` does to it.
- Validation: reproduced `409` → repaired `200`; falsified by neutralizing only the supersede predicate with the migration and index left standing (`13 passed; 1 failed`, identical body) and restored (`15 passed; 0 failed`). 0018's invariant is falsified in the same test rather than argued — a third issuance while the replacement is live must still be `409`. `node_replacement` 1/1, `node_channel` 30/30, clippy `-D warnings` rc=0, gate 18 checks, book rc=0.

## 2026-09-13 — The census partly refuted its own pattern, and the statement says so (`SIGNOFF-REPAIR.11.6`)

- The leaf had collected ten instances of a rule, threshold or severity changed by the first measurement of its population — and forbade proposing its own gate before measuring whether that generalises beyond one session by one author.
- ⚠️ The general population is **not** mechanically countable. `grep -c superseded` over the tree returns 49, nearly all about superseded *designs* rather than a leaf reversing its own rule. Counting by vocabulary would have produced a confident number measuring nothing — the exact failure the leaf is about.
- So the census runs where "a rule was proposed" *is* countable: the gates this repository has shipped, dated per check with `git log --diff-filter=A`. **11 checks across 9 leaves** (18 doctrines registered; the rest arrive with the spine).
- **Result: 4 of the 8 assessed leaves had their rule revised by the census that preceded it** — twice the obvious rule was rejected outright (wrapping `git diff --check` would have flagged nine legitimate files; regenerating the index's frontier column would have destroyed accurate prose in 13 of 14 rows), once the rule's key moved, once the census itself was revised three times.
- ⭐ **One was measured and shipped unchanged**, and is counted as such: the cost census for running every `--self-test` found 1.01 s against a 3.15 s enforcer, so the rule proceeded as proposed. A measurement that confirms is not a wasted one.
- ⛔ **It is 4 of 8, not a law**, and the ten-instance list counts something different — severity reversals inside repair leaves that ship no gate. The two are not conflated. One leaf whose *census* was corrected but whose *rule* was not is counted as not revised, rather than folded in to help the total.
- Decision: a **`TOOLBOX.md` method statement, no gate**. A gate is not available — "proposed a rule without measuring its population" is a judgement over prose — and the one narrow shape that is gateable already is (`GAP-CLAIM-CENSUS`). The statement carries the refutation alongside the pattern.
- Validation: the dating command re-runs and reproduces the 11/9 split; gate (18 checks), book and link check rc=0. Documentation only.

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

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated nine times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

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
