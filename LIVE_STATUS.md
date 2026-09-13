# LIVE_STATUS.md — authoritative live progress tracker

Rows use only **Done · Mostly Done · In Progress · Not Started**. This is a current
snapshot. Historical implementation and verification records live in the phase
task-trees and git; the pre-review snapshot is `9c2d2ba:LIVE_STATUS.md`.

## Qualification correction

🔴 **Three doctrine ENFORCERS were writing their own scratch off the repository volume, and the gate written for §13 could not see them; repaired and the gate extended under `.11.2.2` (REPAIR-0170).** Seven sites, all shell: `check_task_acceptance.sh`, `check_waiver_routing.sh` (two), `check_self_tests.sh`, `update_scaffold.sh`, and the two probe scripts that CLONE throwaway git repositories into their scratch. Locality MEASURED, not asserted: the checkout and `target/doctrine_scratch` are device `16777244`, the ambient directory `16777232`. 🔴 **`STORAGE-LOCALITY` enumerated `git ls-files -z -- "*.rs"`**, so the shell family was outside the scan by CONSTRUCTION rather than by exception — invisible, not waived. 🔴 **And this leaf's own published number was WRONG: 7 sites, not the 5 it opened with.** The opening census ran `git grep … | grep -v '^docs/'`, and that convenience filter excluded the directory `R-84-1` was pointing AT — the record names "task-acceptance and waiver-routing **probe scripts**" and both spell `WORK=$(mktemp -d)` under `docs/tasks/artifacts/`. ⚠️ `MEMORY.md` carries that exact warning ("scope a census WHOLE", after a glob silently matched nothing) and it was breached anyway; corrected here rather than quietly used, with the superseded 5 left in place. **DECISION: the gate IS extended** to tracked `*.sh`/`*.py` — it fires on **0** today and would have fired on all **7**, the `REASON-CODE-DOC` shape and not `.11.9`'s rejected 114-of-131. ⛔ **The rule is about the ARGUMENT, not the call**: both tools name by exclusive creation, which is the half this doctrine wants, so a pattern flagging the call itself would condemn 15 conforming sites and teach bypass. ⭐ The one Python hit is a deliberate CONTROL — a probe that measures where a child's temporary file lands, which pinning `dir=` would turn into an assertion — and is a reviewed exception. 🔴 **The extended gate flagged ITSELF on the first run**, its diagnostic spelling the command it matched: the `SELF-TEST` founding incident repeating in a file whose header cites it, fixed by RE-WORDING rather than a path exclusion so it self-polices. FALSIFIED against the exact pre-repair sources: all **7** named, rc=1; restored rc=0. **265 files scan clean, 22 self-test classifications** (8 before), both probes green (10/0 and 5/0). ⛔ NOT claimed: a Makefile recipe, a CI step or a Rust `tempfile` call reaches none of the three patterns.

🔴 **A tenant administrator READ another tenant's node inbox — command ids, thread ids, delivery state and payloads; repaired under `.3.5.3` (REPAIR-0169).** REPRODUCED over the supported HTTP surface before anything was changed: an administrator of tenant A, naming its OWN tenant and tenant B's node id, received **both** of B's rows in full. ⭐ **Why it survived a repair that named its three siblings**: `R-31-32-1` named FOUR verbs, `.3.3.4.10`'s census scoped itself to node administrative MUTATIONS and measured "four of the five mutations carry no tenant predicate", and `.3.3.4.10.3` then bound the three inbox WRITES. The inspection is a READ, so it fell outside a census that was correct about its own scope and silent about the record's. ⛔ Found by `.11.9.1`'s clause reconciliation, not by a test — no leaf had ever cited that record. ⭐ **The defect stated so it generalises: the caller supplies TWO independent identifiers and the handler checked one of them.** The admission proved the caller administers the tenant it NAMED; the select asked only for the node id. **census of all 24 GET routes before repairing, and its 3 hits CLASSIFIED down to 1**: one is this defect, one is `admin_metrics` (already `.3.5.2.1`, deliberately process-global), and one — `inspect_call` — is a FALSE POSITIVE that produced the sharper rule, because it loads the call FIRST and authorises against `call.tenant_id`, deriving the tenant from the target instead of accepting it. ⚠️ **Census limit stated, not hidden: 10 of the 24 handlers hold no SQL of their own** and delegate to a module the scan cannot follow — unmeasured, not clean, and now owned by `.3.5.4`. Fix: `AND tenant_id = $2` in the select, the sibling mutations' shape; ⛔ deriving the tenant from the node was REJECTED here because removing a required parameter is a wire change on a public route, the class `.3.5.2.1` already holds for a director decision; ⛔ no transaction and no guard, because a read has no check-then-act. ⭐ **FALSIFIED in the strongest form available — the control was written BEFORE the fix, so the pre-repair run IS the neutralized build** (`7 passed; 1 failed`), with nothing reverted or reconstructed and therefore none of `.4.2.8`'s "broke a different thing" hazard. The control asserts the absence TWICE (empty rows AND no victim command id anywhere in the text) and carries a positive arm, so a fix that merely emptied the result fails it. **4 suites / 55 tests** rc=0; strict server lint rc=0. ⚠️ **A third finding, in prose rather than code**: the sibling control's own comment claimed it covered "read, move or destroy … all three" and asserted two, and that same sentence had propagated into `cli.md` ("all three verbs" under a block showing four) and `authority.md` ("Three operator verbs"). All three corrected — a description of a check is not its coverage.

🔴 **A leaf's own ranking instruction was REJECTED by its own census, and classifying seven records found TWO live defects; closed under `.11.9.1` (REPAIR-0168).** `.11.9.1` said to start from the SMALL-fan-out records, "there the routing was a genuine assignment". ⭐ **Measured, the two orders are ANTI-correlated**: all five fan-out-1 records point at a CONTAINER leaf (`.11.4` is named by **99 of 131** records, `.3.3` 41, `.4.1` 37), while every record with fan-out 9 or more reaches a leaf named by 10 or fewer. ⭐ **`R-6-27-1` settles it** — its entire body is the artifact's boilerplate, it carries **no finding at all**, and it still received a candidate leaf. Fan-out measures how sure the REVIEWER was; what an uncited routing being a real miss depends on is whether the TARGET needed the record. ⚠️ **Auditor's asymmetry honoured**: `.11.9`'s statement is not called wrong, its question is NAMED as a different one. **Adopted ranking: the narrowest candidate's frequency** (`--rank`), and the 107 remaining records split into six tranches at that distribution's natural gaps. 🔴 **Tranche 1's 26 clauses produced two LIVE findings, each with a new owner.** (1) `inspect_node_inbox` — the FOURTH verb `R-31-32-1` named — admits on the caller's tenant and then selects from `node_inbox_state` by `node_id` alone, so a tenant administrator reads another tenant's command ids, thread ids and payloads; `.3.3.4.10`'s census scoped itself to MUTATIONS and repaired the other three. Owner `.3.5.3`. (2) Three doctrine ENFORCERS write scratch to ambient `TMPDIR`, and `STORAGE-LOCALITY` enumerates with `git ls-files -z -- "*.rs"` so it cannot see them; owner `.11.2.2`, opened with the raw 19 hits CLASSIFIED to 5 breaching (12 conforming Python pass a repository-derived `dir=`). ⭐ **The vocabulary gained a state the acceptance did not have — `attach`**: a clause whose owner's OWN TEXT does not make it visible, which is `.11.9`'s defect caught BEFORE it costs anything. 3 rows carry it. ⚠️ `declined` carries 0 and says so: it is invisible to every search, so it exists before an instance does. 🔴 **The instrument was wrong a FIFTH time and this leaf's own prose exposed it** — the elided-citation rule matched any `:N`, so source line numbers written after a census citation inherited its filename, silently moving 114 to 111; now anchored to its opening backtick, with two controls. 🔴 **The frontier's own pending-count command was also wrong**: it matched `- Status: pending` and missed the **11** leaves carrying only `- Opened: pending`, publishing 36 where the tree holds **46**. **42 controls** rc=0, ledger clean. No product code touched.

🔴 **A REPLACEMENT handed the machine it was replacing its session back, reverted its certificate's host, and never closed its incarnation; all three repaired under `.4.1.5` (REPAIR-0167).** Each REPRODUCED in its OWN run before being fixed: `heartbeat after the replacement: 200`; `nodes.host_id … names: host-a` after moving to host-b; `incarnations: 2 open of 2`. 🔴 **The severe one PARTIALLY UNDID `.4.1.3`**: that leaf's renewal guard asks *"does a usable certificate exist for this node id?"*, and a replacement issues one **for that same id**, so the REPLACED process's predicate becomes true again. ⚠️ Bounded to a replacement inside `LEASE_TTL` of the revocation — the ordinary operational case. 🔴 **The host**: `rotate` reads the SAN's host from `nodes.host_id`, so the first AUTOMATIC rotation (within half a leaf lifetime) silently reverted the certificate to the machine the node no longer runs on; the control drives a real rotation and reads `SAN: [DNSName("host-b")]`. 🔴 **The incarnation**: ⛔ the writer's own comment (*"re-enrollment cannot duplicate — the `nodes` primary key refuses a second enroll"*) is FALSE for a replacement and is the sentence that hid it; corrected in place. ⚠️ Its consequence MEASURED and narrower than implied: both selectors already `ORDER BY valid_from DESC LIMIT 1`, so the defect is the **LEDGER**, not the selection. ⛔ **No property traded**: `.4.1.3.1`'s withheld tail still replays — the control asserts the replacement takes its own lease in the same run, and the ritual passes unchanged. ⭐ The epoch is BUMPED not deleted, because deleting resets it to 1 and breaks the monotonicity `.4.2.3`'s argument rests on. FALSIFIED each part SEPARATELY (`1 passed; 1 failed` ×3, each naming its own clause). **72 tests / 5 live suites** rc=0. 🔎 **Carry this**: a guard asking *"does a usable credential exist for this id?"* is satisfied by ANY such credential, including one issued after the event it defends against.

⚠️ **The wake gate is a DRAIN SWITCH, not a concurrency limiter — stated for the first time, and every state it has now has a control; closed under `.4.2.10` (REPAIR-0166).** census taken from the FILTER rather than the tests: **5** reachable states, of which the existing control covered **2**. ⭐ **The three uncovered ones all mean "deliver" and each reaches that answer by a DIFFERENT SQL route** (missing `concurrency` key → `NULL = 0` is NULL; missing `availability` block → the whole path NULL; a negative number → simply not zero), so one does not stand for the others. ⭐ **The drain state is re-asserted LAST**, so four positive arms cannot all be vacuous — `.4.2.6`'s lesson applied to a filter. 🔴 **The substantive answer is a CLAIM NARROWED, not a defect repaired**: the gate tests for exactly zero and never compares a declared concurrency against an ACTIVE count, so `concurrency: 2` does not cap a node at two; `presence_state` already names zero `Draining`, so this is the design and the book now says so plus what an operator must enforce themselves. ⛔ **My own hypothesis MEASURED AND REFUTED**: the `::bigint` cast at **6** query sites would raise on a non-numeric value, but the field is typed `Option<i64>` and the writer stores the RE-SERIALIZED struct, not the raw body — unreachable. FALSIFIED by removing the gate (`36 passed; 1 failed`, only this control). **37/37** rc=0. ⛔ No production code changed.

🔴 **A node that rotated its certificate and then stayed DOWN past its old one's expiry was permanently locked out; repaired under `.4.2.9` (REPAIR-0165).** CONFIRMED and LIVE: `handshake` rotates automatically when the leaf nears expiry and `install_identity` wrote only memory; census of the identity-file writers finds ONE, `save_workload_identity`, which takes an *enroll response* and runs once at enrollment. ⭐ **The BOUND is the work, and a guess would have been wrong BOTH ways**: a restart usually RECOVERS (stale cert loads, rotation window still open, next handshake rotates again) — but a node down past that certificate's expiry (≤ `ROTATE_REMAINING_SECS` = 300 s on a 600 s leaf) cannot rotate out, because `rotate` REQUIRES a usable certificate. Operator-issued token needed: **`.4.1.1`'s lockout class through a different door**. **DECISION: persistence belongs to the CALLER** — the channel does no file I/O and does not start; `Node::open` (which has the journal path) writes the exact two files the loader reads, and the control asserts the FILENAMES, because "a sink fired" would pass with a sink writing anywhere. ⭐ **PERSIST FIRST, then swap memory**: a crash between them leaves the NEW identity on disk, which the next start corrects; the reverse would reintroduce this very defect. A sink failure is reported, NOT fatal. FALSIFIED by removing the sink call (`29 passed; 3 failed`, exactly the three new controls, the node-level one reporting `cert.der … NotFound`). **76 node tests + 46 live channel/replacement/work tests** rc=0.

🔴 **The node carried a fencing token from one generation with the epoch of another; repaired under `.4.2.8` (REPAIR-0164).** REPRODUCED with a driven interleaving over self-describing generations: **`242 torn of 40,000`** reads, first `("fnc_444", 455)`. ⭐ **Why it looked safe: the WRITE was atomic** — `handshake` took both locks in one scope, so two writers could never interleave; but all **4** fenced read paths took two SEPARATE acquisitions, and a reader straddles a complete write. ⚠️ **Bound stated BEFORE the work and unchanged: an AVAILABILITY defect, not a fencing bypass** — the server refuses the mismatched pair, so the cost is a spurious `401` and a reconnect; repaired because a request that cannot possibly succeed should not be constructible. ⭐ **0.6 % of reads under maximal contention** explains why it never surfaced: it presents as an unexplained `401` followed by a successful reconnect, indistinguishable from a network blip. Fix: ONE field `lease: Arc<Mutex<Option<Lease>>>`, one acquisition — the mixed state is now **UNREPRESENTABLE**. ⛔ A "take both locks" helper and accepting the window were both rejected with reasons. 🔴 **The FIRST falsification was WRONG and is recorded**: it split the WRITE (which the old code never did) and failed ten times louder (`4193 torn`); redone against the true superseded mechanism, the split READ, `242 torn`. A neutralization that exaggerates the defect is not evidence for the repair. **72 node tests + 53 live channel tests** rc=0; no wire field, route or documented behaviour changed.

🔴 **`.4.2`'s split read all 15 routed records and STILL dropped six clauses; parent closed under `.4.2` (REPAIR-0163).** ⭐ **The citation claim is MEASURED for the first time: 15 routings, 15 cited, 0 uncited** — against `.4.1`'s **36 uncited**, the leaf whose dropped clause started the reconciliation question. 🔴 **But citation is not accounting**: a clause-by-clause pass over the record BODIES found **SIX clauses the split did not carry**, in the very leaf that had avoided the citation form of the defect — the strongest possible evidence for `.11.9`'s reframing that the unit is *a clause with an owner*, not *a record with a citation*. **All six now owned**: ⚠️ **two CONFIRMED at the source** — the node reads its fencing token and lease epoch under **TWO separate mutexes** (`.4.2.8`; an AVAILABILITY defect, not a fencing bypass — the bound stated BEFORE the work), and `install_identity` writes only in memory so a rotation may not survive a restart (`.4.2.9`); ⛔ **four recorded as the REVIEWER's claims** under the reproduce-first prohibition — the wake gate's positive-concurrency coverage (`.4.2.10`) and three replacement clauses grouped as ONE question about what a replacement ENDS (`.4.1.5`, flagged against `.4.1.3.1`'s deliberately preserved tail). Every remaining clause of all 15 records is dispositioned. ⛔ **The pending count ROSE 36 → 40, deliberately** — a reconciliation that finds work makes the tree larger, and hiding that would be the defect. Documentation only; no code touched.

⛔ **The reason-code "drift" is mostly REFUTED by the code's own doc; the real gap was that the book had NO error page — closed under `.11.7` (REPAIR-0162).** ⛔ **The leaf's own numbers were wrong: 18 emitted and 9 unregistered, not 19 and 10** — `unknown` is a CLIENT-side sentinel built when a body will not parse and never travels the wire; a search's hits published as a defect count, in a leaf about a published set nobody re-derived. ⛔ **The 11 unemitted registry codes are DELIBERATE and already documented** ("the *complete* §9.8 list, not a Phase-0 subset") — **all 11 KEPT**. ⛔ The 9 unregistered emitted codes are preserved verbatim by `ReasonCode::Unknown`, the designed forward-compatibility path. 🔴 **The real gap: nothing published what the product emits.** DELIVERED `docs/book/src/errors.md` — all 18 with status, registry status and meaning, derived from the sources; the 11 with their reason; four client rules (incl. `commit_outcome_unconfirmed` means UNKNOWN, not failed). ⭐ **A gate IS registered here — `REASON-CODE-DOC` — one commit after rejecting one of the SAME RULE SHAPE**: it fires on **0** breaches today and would have fired on all 9, so it catches the NEXT drift; `.11.9`'s would have fired on 114 of 131 and was rejected on exactly this test. ⛔ Registry NOT extended and no emitter changed (its contract mirrors §9.8, which is in a roadmap FROZEN at v0.4.1); the §9.8 question is routed to `.11.7.1` WITH the measurement, because the freeze says a version bump must cite this kind of evidence. FALSIFIED (an undocumented code -> named, rc=1). **19 checks / 14 self-test controls** rc=0. No product code changed.

🔴 **An AGENT ROLE granted `tenant_admin` can issue node enrollment tokens and revoke nodes — the documentation said "an authorized human" and the code never checked; DECIDED under `.4.1.4` (REPAIR-0161).** MEASURED by driving the live route: `200`, with the ledger recording `authority_grants.subject_kind = 'role'`. **DECISION: narrow the CLAIM, not the code**, on four independent lines — §16.2 names no issuer restriction, §16.4 specifies authorization over typed actions/resources, §16.3 says "a human, service, or agent may delegate"; and across **117** `resolve_principal` sites authorization NEVER depends on principal KIND (every branch selects which identity TABLE to read). ⭐ **The decisive evidence was an ORACLE NOBODY BUILT for it**: adding the kind check fails **3** controls, **2 of them pre-existing**, which already drive an agent role here and require GRANT-based adjudication — and one fails because a kind check refuses `401` BEFORE the authorization that writes the denial record, destroying the audit evidence. ⚠️ **Scope widened by the leaf's own census**: 6 sites, not 2, including `POST /v1/nodes/revoke` with the same sentence and the same gate. ⚠️ **The governance consequence is now STATED in the book**: granting `tenant_admin` to an agent lets it extend the node population; a deployment that does not want that must WITHHOLD THE GRANT — narrowing the route would be a grant-model change, not an endpoint check. FALSIFIED (`15 passed; 3 failed`, naming the change). **5 suites / 91 tests** rc=0. ⛔ No product behaviour changed. Recorded as `docs/decisions/2026-09-13_issuance-is-a-grant-not-a-kind-of-principal.md`.

⛔ **The record-reconciliation gate is REJECTED on its own census, and the leaf's own number was wrong; closed under `.11.9` (REPAIR-0160).** ⛔ **131 review records, not the 53 the leaf claimed** — 53 is the artifact DIRECTORY's file count, and `MEMORY.md` has carried 131 correctly since `.1`, so two live documents disagreed unnoticed in the leaf whose subject is exactly that. **Census: 131 records / 614 routings / 301 to a SPLIT leaf / 16 cited / 285 uncited; at RECORD granularity 17 of 131 cited, 114 by none.** 🔴 **The gate would fire on 114 of 131 on day one** — a backlog wearing a gate's clothes. ⭐ **The census changed the QUESTION**: fan-out is a MEDIAN of 4–5 candidate leaves per record, so a reviewer naming five was writing a SUGGESTION LIST, not five assignments; "a leaf must account for every record routed to it" is unsound at the population level. 🔴 **Which reframes the original defect**: not a missing citation but one record holding THREE findings of which ONE found an owner — the unit is a clause with an owner, and "accounted for" includes DELIBERATELY DECLINED, which no search sees. 🔴 **The instrument was WRONG FOUR TIMES**, each caught by a different control (70 of 131 — two id shapes; parent sections swallowing children; 3 of 19 citations — heading line vs line RANGE; and the elided `` `:N` `` form, caught only against the REAL tree). One control was itself wrong first. Discharged: the "no human restriction" clause now owns **`.4.1.4`** — `resolve_principal` returns `Human` OR `Role` and nothing asks which, while two doc sites say "an authorized human". The 114-record backlog is ROUTED to `.11.9.1` with its size. No product code touched.

⛔ **The `MEMORY.md` eviction worry is REFUTED by its own census; instrument delivered under `.11.4.2.1` (REPAIR-0159).** **26 standing warnings, ZERO existing only in `MEMORY.md`** — all 26 are recorded in a durable layer, so the implied rule ("every warning must first exist in a durable layer") would fire never. 🔴 **The instrument was WRONG TWICE first, and the sequence is the evidence**: 31 (split at every marker — over-count), 21 (split at a sentence terminator, which here sits INSIDE `**…**` — under-count), 26 (correct). ⚠️ **Its own `--self-test` passed throughout** because the fixtures shared the bug's idiom; reading the OUTPUT against the file is what caught it. ⛔ **Then the tool's verdict was itself a false claim** — it printed `ORPHAN (only in MEMORY.md): 13` and hand-classification found **13 of 13 recorded**; a search population published as a defect count, renamed `UNCITED` with the classification printed beneath it. 🔴 **The real finding is narrower: the POINTER is missing, not the record** — 13 of 26 name no leaf, so an eviction costs findability, not fact. ⛔ **NOT mechanized**: "a warning must name its leaf" would flag 3 legitimate ones (the derived frontier note, the `docs/knowledge/` pointer, the `project_env.py` rule) — `.11.4.5.3`'s shape a second time. ⭐ Ships instead: `scripts/census_memory_warnings.py`, tracked, 16-control `--self-test`, in `TOOLBOX.md`, run BEFORE evicting so the choice is derived. **Seven recorded cap crossings, four in this session — the cap was NOT raised.** No product code touched.

⛔ **The secret store's published scope was wider than the code in two of three sentences; narrowed under `.4.2.5` (REPAIR-0158).** ⛔ **The census REFUTED the leaf's own opening sentence** — exactly 2 `SELECT`s of key material exist in the workspace, one IS the store and the other a test fixture, so node keys are not read in production at all and the ONE key read IS routed. 🔴 **But it found the real defect: the seam is READ-ONLY** — the CA bootstrap `INSERT`s into `server_ca` DIRECTLY and then reads back through the store with an `expect`, an invariant only while both ends are one database. ⭐ So the boldest published sentence is the false one: *"the external store arrives as a configuration change, not a code migration"* is backwards — the first external profile ABORTS the server on first boot. *"every secret read goes through the declared profile"* is also false (the enrollment token is read straight from its table). census of published copies: the book carries **none** — measured, so there is no book drift. **DECISION: narrow, not widen** — a write path for a backend that does not exist is unfalsifiable behind a one-profile registry (`.4.2.6`'s lesson). ⭐ **The narrowing is a TRIPWIRE, not a comment**: a test pins the profile count with the reason in its assertion message, so a second profile cannot be added without confronting the unrouted write. ⛔ No runtime change; the decision record gains a Correction that PRESERVES the original wording. FALSIFIED by declaring a second profile (`3 passed; 1 failed`). **103 lib tests** rc=0.

🔴 **A malformed server response ABORTED the node process where the signature promised an error; repaired under `.4.2.7` (REPAIR-0157).** REPRODUCED with a driven decode: `end byte index 2 is not a char boundary; it is inside 'é' (bytes 1..3 of string)` — from `fn from_hex(&str) -> Result<…>`. ⭐ **The condition is NOT "non-ASCII"**: it needs an EVEN byte length (clearing the odd-length guard) whose chunk boundary falls INSIDE a character, so the control asserts both structural facts about its input before decoding. ⭐ **The call site already handled the error the decoder never produced** (`map_err(ChannelError::Malformed)?`, one line away) — a `Result` the body did not honour, not missing error handling. 🔴 **census, and the grep found what the reasoning did not: 7 hand-rolled hex decoders, 6 panicking, and the 1 SAFE one is the only one on the untrusted wire** — attention went exactly where someone was worried. ⛔ **The server's `pub` copy is DELETED, not repaired**: no caller by `git grep`, confirmed independently by making it private (`error: function from_hex is never used`); a repaired dead path is one no control reaches. ⛔ **4 TEST-local copies measured and deliberately unchanged**, reason recorded so a later census does not re-raise them. ⚠️ Trust boundary stated, not inflated: needs a malicious/faulty control plane, not a network attacker — but the cost is the whole node process. FALSIFIED by reverting only the indexing (`24 passed; 2 failed`, the third control green). **Node 70 + rotation/enrollment 55 tests** rc=0. Promoted → `docs/knowledge/a-signature-is-a-promise-the-body-must-keep.md`.

🔴 **A fenced session's `ack` marked the NEW session's delivery acknowledged — and an acknowledged row is one the retention prune DELETES; repaired under `.4.2.4` (REPAIR-0156).** `ack` ran as three separate pool statements, so the admission decision and the write took different snapshots; `events` closed the identical window in `.2.2` and `ack` was never given the same treatment. REPRODUCED with the lease row held: the ack `COMPLETED — it never asked about the lease it was writing under`, the session was fenced, and it still marked **3 rows** (`left: 3, right: 0`). 🔴 **The severity is the PRUNE, measured not asserted:** 9 sites touch `acknowledged_at`, classified to exactly ONE mutating consumer — the retention `DELETE … acknowledged_at <= cutoff` — so a stale ack makes work the live session still holds eligible for deletion. Fix: ONE transaction calling `verify_fencing_in_tx`, the shape `events` already has. ⛔ **`poll` DECIDED, not assumed — NO**: all three of its statements are reads, a fenced poll leaves no trace, and wrapping it would lock the channel's most frequent call for no invariant. ⛔ **No tenant guard on `ack`**: that would cut the tail `.4.1.3.1` preserved. ⭐ The in-transaction cursor read ships with its limit NAMED — no control discriminates it; its justification is not taking a second pool connection per acknowledgement. FALSIFIED with the transaction kept and only the re-verification removed; arm 2 proved red SEPARATELY under the same neutralization. **8 suites / 117 tests** rc=0.

🔴 **A heartbeat whose check had passed REVIVED a lease that lapsed before its write landed; repaired under `.4.2.3` (REPAIR-0155).** The renewal's `UPDATE` named the node, the epoch and the certificate — never `lease_expires_at` — so the expiry was the one condition still owned by a pre-check. REPRODUCED against the live route with the write stalled at the lease row (`blocked_on` proves it WAITING before the lapse commits, so the ordering is measured, not hoped for): renewal **ALLOWED**, fresh expiry, **1 live lease** on a dead session — and presence, delivery and every fenced write are functions of that clock. Fix, two parts: the expiry joins the epoch (`.2.2`) and the credential (`.4.1.3`) INSIDE the write, and the classifier names the third refusal. ⭐ **The WORDING is a repair part** — the same request answered `lease_expired` sequentially and `fencing token refused` when its write was overtaken, so the wire answer was a function of the scheduler. **The clock was DERIVED:** the DATABASE clock, because one statement must not mix two and because `node_presence.online` reads this column that way — so a renewal can never succeed for a node the API reports `offline`. ⚠️ **Residual NAMED, not bundled:** `lease_expires_at` is process-WRITTEN and database-READ, so the 60 s TTL is nominal and carries the skew; pre-existing, not widened here, owned by `.4.2.3.1` and stated in the book's honest limits. ⛔ The pre-existing expiry control does NOT discriminate this repair (it only drives the sequential order) — labelled a regression control. FALSIFIED twice, each part alone (`35 passed; 1 failed` both times, only this control). **7 suites / 79 tests** rc=0.

🔴 **A captured handshake FENCED THE LEGITIMATE NODE OUT OF ITS OWN SESSION, and a captured rotation returned a second PRIVATE KEY; repaired under `.4.2.2` (REPAIR-0154).** REPRODUCED byte for byte with the node crate's own helpers: replayed handshake `200` + the real node's next heartbeat `401`; replayed rotation `200` + two distinct `key_der` values from one captured request, 3 live certificates. **DECISION: a per-request nonce consumed once, NOT a server challenge** — no extra round trip, **no clock** (a skew-window design would put a clock on the auth path of a repo that just fixed three cross-clock defects), and it is enrollment's existing shape. ⛔ Consuming the CERTIFICATE was rejected: a lost-response retry is indistinguishable from a replay and the rule would break §17.4 recovery — a control asserts this refuses replays, NOT reconnects. ⚠️ The nonce is consumed only AFTER the signature verifies, so forged traffic cannot burn one. 🔴 **A DEADLOCK I introduced, found by the suite HANGING**: an FK to `nodes` made the rotation wait on the row it already holds — and would have queued every handshake behind any in-flight revocation. FK dropped, consume moved in-transaction. ⚠️ **A hanging suite is a third failure mode beside red and green**; diagnosed from the process table, not the code. ⚠️ Wire narrowed: `nonce` required (`422` if absent). FALSIFIED. **Broad run: 44 suites / 374 tests, 0 failed** rc=0.

🔴 **The bad-proof control covered the WRONG rung, and a disabled signature check would have gone unnoticed; closed under `.4.2.6` (REPAIR-0153).** The existing negative presents `cert_der: "00"`, which fails the chain check and stops — coverage of "an unparseable certificate is refused", not of "a bad proof is refused". ⚠️ Every rung answers the same `401` BY DESIGN (no existence leak), so the six new negatives prove their rung by CONSTRUCTION, each satisfying every rung but its own; ⭐ a POSITIVE control differing in exactly one factor (the signing key) is what proves the signature negative reached the signature rung. 🔴 **FALSIFIED decisively:** with the signature check neutralised a forger receives a live lease and fencing token, and **only the new control fails** — the pre-existing one stays green; the chain rung behaves identically. ⚠️ Scope correction recorded: the captured-proof fixture is `.4.2.2`'s, because the canonical bytes are not exposed and re-deriving them would duplicate an unchecked wire contract. ⛔ **No production code changed** — control coverage only. **34/34** rc=0.

🔴 **A rotation in flight outlived a revocation and left the withdrawn node a LIVE certificate; repaired under `.4.2.1` (REPAIR-0152).** REPRODUCED against both live routes: with the rotation stalled after its proof check, a complete revocation answered `200`, the rotation resumed ALLOWED, and the node finished with **1 live certificate** — which restores the lease renewal and the work delivery that `.4.1.3`/`.4.1.3.1` gate on exactly that certificate. ⭐ **The first reproduction was an ARTEFACT** (`500 canceling statement due to lock timeout`) and chasing it found the mechanism: **the `nodes` foreign key does not prevent the race, it makes it deterministic in the WRONG direction** — the insert is forced to land after the revocation commits. Fix: rotation is ONE transaction taking the node row FIRST, then re-reading the certificate; ⭐ no new lock or guard — the same row the revocation already takes, so the two serialize. ⛔ Folding the liveness into the INSERT (the `.4.1.3` shape) is REJECTED: READ COMMITTED snapshots at statement start, so the window between two statements needs a lock held across both. 🔴 **The first control did NOT discriminate the repair** — removing the lock left it green — so a third arm asserts the lock directly in `pg_stat_activity`. FALSIFIED separately for each part. **8 suites / 117 tests** rc=0.

🔴 **`.4.2` censused and split into seven children (REPAIR-0151); NOTHING is repaired yet.** ⭐ The census read the **15 routed source-census records** as well as the goal line — `.11.9`'s lesson applied the day it was written — and names the clauses belonging to other leaves rather than dropping them. 🔴 **A rotation can outrun a revocation and leave a LIVE certificate behind** (`census-2.md:56`, `.4.2.1`): `rotate` is three unguarded statements, the revocation updates unrevoked certs under the exclusive guard, and an insert landing after it is unseen — which **undoes `.4.1.3` and `.4.1.3.1`**, both of which gate on exactly that certificate. ⛔ NOT the claim `.4.1` refuted (that was whether a revoked node can rotate; this is whether an in-flight rotation can commit across a revocation). 🔴 **The rotate proof is entirely STATIC** (`{channel_version, node_id, cert_der}`), so a captured request replays verbatim and **each replay returns a fresh PRIVATE KEY**; the handshake proof has no nonce/timestamp/challenge (`.4.2.2`). ⚠️ Bounded: both die with the certificate; the wire is plain HTTP so capture is cheap. **Four more confirmed at source**: `renew_lease` can revive a lapsed lease (`.4.2.3`), `ack` writes on a pre-check (`.4.2.4`), the `secret_store` claim is wider than the code (`.4.2.5`), the node's `from_hex` panics on non-ASCII (`.4.2.7`). **The bad-proof control refuses before the signature** (`.4.2.6`) and is a PREREQUISITE for `.4.2.2`. ⛔ No runtime change; all seven carry the reproduce-before-writing-up prohibition.

✅ **A node enrollment token no longer outlives the authority that issued it; decided and repaired under `.4.1.2` (REPAIR-0150).** Redemption validated only the token's own fields, so a token issued by a since-revoked administrator still enrolled the node. **censused before arguing:** the token table records NO issuer, but `authorization_records` already stores each decision's `grant_id`/`boundary_id` and the issuance writes it in the SAME transaction — the link is one column away and EXACT. There is also **no route that cancels a token**, so the only prior recovery was waiting out the TTL. ⭐ **Redemption-time re-checking is REJECTED on this codebase's own history**: `enroll` takes no tenant guard, so reading a grant's status there is the class `.3.3.4.5` repaired. Voiding at REVOCATION needs no new guard — that transaction holds the exclusive guard and the token row is already redemption's serialization point, so the row lock orders them. ⛔ Tenant-wide voiding rejected (revoking node X would void node Y's token). 🔴 **The one-live-token index had to key on `voided_at`** or the repair would re-enter `.4.1.1`'s lockout — neutralising it returns that defect's message verbatim. 🔴 **The checked fixture plan caught the new FK's cost, and the census behind the fix was WRONG TWICE** (missed parents-without-child; then scoped to two directories while a list lives in `crates/reasonbraid-mcp/src/lib.rs`) — **a census scoped to the directories you expect is not a census**. FALSIFIED in three discriminating parts. **Broad run: 44 suites / 371 tests, 0 failed** rc=0.

🔴 **An unauthorized caller could PANIC the token-issuance route, and an authorized one could mint a century-long bearer credential; repaired under `.4.1.2.1` (REPAIR-0149).** Found by measuring the PREMISE of `.4.1.2`, whose argument rests on "the token expires" — the expiry was an unvalidated `i64` off the wire. **REPRODUCED, and it is three failures, not one:** `i64::MAX` from an UNAUTHORIZED caller panicked `TimeDelta::seconds` and dropped the connection, **in front of the authority guard** (`resolve_principal` only parses the header); `1e15` from an admin panicked at `at + ttl` **inside** the guard; a century returned `200` with `expires_at: 2126-09-14`. ⭐ **The source reading predicted the wrong one** — the two middling values answered a correct `403`, so only driving the route said which value, which caller, which site. Fix: a range check (`1 … 86 400`) beside the node-id shape check, before any arithmetic, typed `400`. ⚠️ **A deliberate wire narrowing**, stated in the book; `ttl_seconds <= 0` refused too, because a token born expired is `.4.1.1`'s lockout row. ⚠️ Three fixtures were **completed, not deleted** — `.4.1.1`'s reason (lapse through the product's expiry, never a write to the row under measurement) is preserved exactly. 🔎 **`census-1.md`'s `R-31-32-5` already named this and `.4.1`'s split carried 1 of its 3 clauses** — owned as `.11.9`. FALSIFIED in two DISCRIMINATING arms (dropped connection vs a 2126 expiry). **6 suites / 74 tests** rc=0.

✅ **A revoked node is handed NO new work, and the work is WITHHELD rather than dropped; decided and repaired under `.4.1.3.1` (REPAIR-0148).** `.4.1.3` bounded the tail to the remaining lease but left the server dispatching into it. **census of the delivery ladder BEFORE the rung was chosen:** 2 writers into `node_inbox`, 7 readers, exactly **1 that delivers rows to a node** — and that one has **two** callers, `poll` AND the handshake, so a filter written into `poll` alone would have been correct today and silently incomplete. ⛔ **The enqueue rung is REJECTED on that census:** refusing at dispatch destroys work for a node about to be REPLACED, and `node_replacement`'s ritual is the control that proves the withheld tail must survive. The delivery read now asks the question the RENEWAL asks, asked the same way, and it is a **filter, not a refusal** — `poll` still admits and answers the true cursor, `ack`/`events` untouched, so `.4.1.3`'s seam holds. 🔴 **A claim recorded in three live documents was STALE and a control caught it:** `node_presence.suspended` is not *ever revoked* — **migration 0017 supersedes 0012** — so the rejection stands on a narrower fact, that 0017 never asks whether the surviving certificate is IN DATE. Promoted to `docs/knowledge/a-schema-object-is-its-latest-migration.md`. ⚠️ A fixture was **completed, not deleted** (a credential-less poller is unreachable through the live routes). FALSIFIED in three separate neutralizations (2 / 2 / 1 failures — both halves load-bearing, the controls discriminate). **7 suites / 92 tests** rc=0.

🔴 **Revoking a node did NOTHING to a node that was running, and kept doing nothing; repaired under `.4.1.3` (REPAIR-0147).** MEASURED on every channel surface after a real revocation, not read: `heartbeat ALLOWED · poll ALLOWED · ack ALLOWED · events ALLOWED`, only `rotate`/`handshake` refused — the two operations that re-present a certificate. The fencing check reads no ledger fact by design, so a node heartbeating inside the 60 s lease TTL renewed for ever and never reached the check that would have refused it. ⭐ **`.1.3.1`'s decision was RIGHT and unfinished**: "suspension gates re-entry" has a bounded reading, and what shipped was the unbounded one. Lease RENEWAL now requires a certificate neither revoked nor expired; poll/ack/events keep their pure fencing check, which IS the intended tail. ⛔ **`node_presence.suspended` is the WRONG predicate** — ⚠️ **corrected 2026-09-13 by `.4.1.3.1`:** the rejection stands but *not* for the reason recorded here; **0017 supersedes 0012**, so a replaced node reads NOT suspended and was never at risk. The column is disqualified because it never asks whether the surviving certificate is IN DATE. ⚠️ **Two limits:** the tail is up to a full lease, and during it the server still hands the node NEWLY enqueued work (measured, deliberately not repaired here — ✅ **closed by `.4.1.3.1`/REPAIR-0148**); and nothing at the transport re-checks a certificate — the dev wire is plain HTTP, already named by `PHASE-7`. Registered as **R-REVOKE**. FALSIFIED (`30 passed; 1 failed`, exactly the one control). **5 suites / 53 tests** rc=0.

🔴 **A token that expired unused locked its node out for good; repaired under `.4.1.1` (REPAIR-0146).** Migration 0018's partial index keys `(node_id) WHERE used_at IS NULL`, and an EXPIRED token still satisfies that — so a token issued and never consumed held the index permanently and every later issuance for that node id answered `409`, while the lapsed token enrolled nothing. ⭐ **The 409's own message named the recovery and the recovery was the defect**: *"consume or expire it before issuing another"*. REPRODUCED against the live routes first, body captured. ⛔ **The obvious repair is UNAVAILABLE** — a partial index predicate must be IMMUTABLE and `now()` is not — so migration 0059 adds `superseded_at`, the index keys on it, and issuance stamps a lapsed row inside its own transaction at the admission's own database time. ⛔ **Not DELETE** (an audit row would name a token id resolving to nothing), ⛔ **not REUSE in place** (an audit row would resolve to a DIFFERENT token's facts — and it needs no migration, which is what makes it tempting), ⛔ **never `used_at`** (it was never redeemed). ⚠️ **Bound:** scoped to the admitted tenant, so another tenant's lapsed token still blocks — `.3.5`'s global-index territory, neither widened nor repaired here. FALSIFIED by neutralizing only the supersede predicate (`13 passed; 1 failed`, identical body) and restoring (`15 passed; 0 failed`); 0018's invariant falsified in the same test. **3 suites / 46 tests** rc=0.

⚠️ **`.4.1` censused five mechanisms before splitting, and REFUTED one of them (REPAIR-0146).** The rotation/revocation serialization worry is **not** a defect: `rotate` opens no transaction and takes no tenant guard, but `verify_rotate_proof` selects `(revoked_at IS NOT NULL OR expires_at <= now())` and answers `proof_refused` — **a revoked node cannot rotate**. Recorded as refuted so it is not re-raised. Two mechanisms stay OPEN and are named as unmeasured rather than assumed: `.4.1.2` (redemption never re-checks the issuer's authority — a bearer-semantics DECISION, not a presumed defect) and `.4.1.3` (`revoke_node` carries no lease write, so what a revoked node's live fencing token still authorises is READ, not measured — ⛔ measure what `.1.5.2`'s revocation epoch already fences first).

⚠️ **"Measure the population before proposing the rule" is 4 of 8, not a law (`.11.6`, REPAIR-0145).** The general population is NOT mechanically countable (`grep -c superseded` -> 49, nearly all about superseded designs), so the census ran where "a rule was proposed" is a mechanical fact: **11 checks across 9 leaves** shipped by this repository, dated per check. **4 of the 8 assessed had their rule revised by the census that preceded it**; twice the obvious rule was rejected outright. ⭐ **One was measured and shipped UNCHANGED** and is counted as such. ⛔ The leaf's running "ten instances" counts something DIFFERENT (severity reversals in repair leaves shipping no gate) and is not conflated; a leaf whose census was corrected but whose rule was not is counted as NOT revised. Decision: a `TOOLBOX.md` method statement, **no gate** — the shape is a judgement over prose, and the one detectable shape is already gated by `GAP-CLAIM-CENSUS`. ⚠️ One author, one repository, denominator 8: the statement is scoped to "the gates this project has shipped", not asserted generally.

⚠️ **The book does not name 80 of the server's 103 product routes (`.11.8`, REPAIR-0144).** Refined census, now a tracked instrument (`scripts/census_route_documentation.py`, `--self-test`, discovered by the `SELF-TEST` gate): **103 product routes — 22 described (named beside an HTTP method), 1 mentioned, 80 absent**, the naive `111/24/87` corrected by excluding 8 `#[cfg(test)]` fixture routes and by splitting "named" into contract-line vs passing mention. 🔴 **The instrument UNDERCOUNTED by 42 % first** — a per-line scan missed `.route(` calls whose path is on the next line (53 of api.rs's 92) — caught only by cross-checking an independently obtained count. ⛔ **No gate proposed:** "every route must appear in the book" would flag the console's static assets and accept a bare mention, wrong in both directions before it is written; the answer is a measured backlog the instrument re-derives. ⛔ **80 is not 80 defects** and "described" is a proxy — a contract line is not proof the description is correct. ⚠️ **This leaf documents NO route**; it measures.

🔴 **A node id enrolled by one tenant answered `500` to another, and was one arm away from answering with a CERTIFICATE; closed under `.3.5.1` (REPAIR-0143).** `nodes.node_id` is a GLOBAL primary key, but the redemption's existence check was `EXISTS(… node_id = $1 AND tenant_id = $2)` — tenant-scoped — so a node owned by someone else reported false, the insert ran, and the key aborted the transaction. ⚠️ Reachable today: the index blocks a second UNUSED token, not a second token, so once the first tenant enrols the second tenant's issuance succeeds. The query now reads the node's OWNER and refuses a foreign one, audited, reusing the same-tenant wording so nothing names the owner. 🔴 **The naive fix is a TAKEOVER, demonstrated:** the replacement branch swaps the node key and issues a certificate with NO tenant check once certs are revoked — with the refusal arm removed, tenant B's redemption returned `200` with `cert_der` for tenant A's node. FALSIFIED in both directions. ⛔ **The index is UNCHANGED — no migration**; this repairs the refusal, not the uniqueness, and ⛔ the leaf's own proposed per-tenant repair is REFUSED by the measurement. **3 suites / 50 tests** rc=0.

✅ **The enforcer now runs every instrument's own `--self-test` (`.11.4.3.1.7.2`, REPAIR-0142).** Nothing ran them before, which is how one sat broken from the commit that added it. Measured before proposing: **17 self-tests / 1.01 s** against a 3.15 s enforcer; registered, the gate is **4.27 s over 18 checks**. The check DISCOVERS its population, so a new instrument is covered the day it lands. ⛔ **It catches nothing today** — all 17 pass; its value is preventing a control from silently stopping. ⚠️ **11 of 28 check/census scripts have NO self-test** and this does not reach them; it also does not assert the 17 are GOOD, only that they still pass. 🔴 **The gate recursed while being built** — registering it put the flag's literal into the enforcer's description, discovery found the enforcer, and the enforcer ran every check again; killed at 400 s. Now guarded by an exported variable that makes any nested invocation exit immediately, with the enforcer also excluded by path. FALSIFIED end-to-end against the REAL defect: `make gate` prints `❌ SELF-TEST` and refuses the commit.

✅ **A revoked boundary no longer leaves the metrics surface open (`.3.5.2`, REPAIR-0141).** `GET /v1/admin/metrics` gated on the GRANT's status and never joined the boundary; revocation updates only `enrollment_boundaries` and does not cascade, so a withdrawn administrator kept the surface. REPRODUCED against the live route first — boundary `revoked`, grant still `active`, route still `200` — then repaired by joining the boundary and requiring it live. FALSIFIED at `left: 200, right: 403`, only that control. ⛔ Deliberately NOT the inspection carve-out: that exists to inspect AUTHORITY state during a revocation, and counters are not authority state. ⛔ **Two things confirmed UNREPAIRED:** the process-global width is unchanged (deliberate; 7 aggregate counters, no identities) and the read is still **unaudited** — both need the route to name a tenant, a breaking signature change, owned by `.3.5.2.1`. **37 passed / 0 failed.**

🔴 **Revoking a boundary does NOT remove access to the metrics surface, and nothing is repaired yet (`.3.5`, REPAIR-0140).** `GET /v1/admin/metrics` gates on a hand-rolled query over the GRANT's status and never joins the boundary; `revocation.rs` revokes a boundary with `UPDATE enrollment_boundaries` alone, leaving its grants `active`. The read also writes **no authorization record**, unlike the nine repaired tenant-admin inspection routes. ⚠️ Bounded: the payload is **7 aggregate counters, no tenant dimension, no identities** — an aggregate-volume side channel, NOT a data leak. ⚠️ The admission census classifies it "identity only", which means it reaches no RECOGNISED gate, not that it is unauthenticated. 🔴 **The one-unused-token index is still GLOBAL** (migration 0018, no tenant column), so the cross-tenant `409` oracle and hold-out reproduce unchanged. ⭐ The leaf's third surface is already closed by `.3.3.4.10` (five node mutations, guarded and tenant-bound); the residual 42 mutating identity-only routes stay owned and unrepaired. Split into `.3.5.2` (metrics) and `.3.5.1` (token index + the redemption-lineage question `.4.1` shares). ⛔ **NOTHING is repaired in this commit.**

🔴 **A node clock behind the server rotated its certificate five minutes AFTER it expired; reproduced and closed under `.3.4.3.1.3` (REPAIR-0139).** A leaf issued at server-time `T` expires at `T + 600` with rotation due from `T + 300`; with the node 600 s BEHIND, the check did not fire when due, nor with one second of validity left, nor at expiry — it first fired at `T + 900`, leaving the channel dead for five minutes. ⚠️ The OPPOSITE direction to the dispatch outage (a node AHEAD), which is why the warning now sits in the code at the point of use. The decision is extracted as a pure `rotation_due` and evaluated in the server's terms through `.3.4.3.1.2`'s offset; the channel keeps its own copy because the check runs inside `handshake`, before the journal is reachable. ⭐ FALSIFIED on the WIRING: dropping the offset leaves the three pure-function controls GREEN and fails only the wiring control — the decision was right, what fed it was not. ⚠️ One test-only dev-dependency (`time 0.3`, already resolved in the workspace; `cargo deny check bans` ok). ⛔ **No field failure is asserted** — reproduced in a driven control, not a deployment; the cert's `not_after` is the server's PROCESS clock and the offset is measured against its DATABASE clock, sound only to the 1 s tolerance `.3.4.3.1.1` declared. **69 tests** rc=0.

✅ **The dispatch outage is CLOSED; the node now tells the time in the server's terms (`.3.4.3.1.2`, REPAIR-0138).** A node whose clock ran >60 s AHEAD of the server refused EVERY dispatch — it did no work at all. Both responses now carry `server_time` (from `clock_timestamp()`, folded into the existing epoch query so a poll gains no round trip); the node stores `offset = server_time − midpoint(sent, received)` and evaluates `decided_at` through it, and REPORTS a disagreement ≥5 s. ⭐ **This SUPERSEDES `.3.4.3`'s clamp, measured before changing anything:** `min(decided_at, received_at)` mixes a LOCAL instant into a server-clock comparison, so a corrected node 600 s behind computed `expires 02:56:58` against `server now 03:05:58` and found everything stale — bounding a cross-clock comparison and removing it do not compose. Four controls asserting the clamp are superseded, each with a comment naming its replacement; the property is now asserted in BOTH directions. 🔴 **A THIRD cross-clock comparison was found here and not by the census:** the retry gate filters `updated_at >= decided_at` (local vs server), so a node behind filters every post-decision attempt out and **the retry bound never trips** — fail-open, now repaired with its own control. ⚠️ NOT clock security: the offset is the server's own statement. ⚠️ Coordinated wire change (`deny_unknown_fields`) — server and node ship together. **10 targets / 115 tests + 4 suites / 57 tests**, rc=0; falsified in both halves. ⛔ `.3.4.3.1.3` (the certificate's opposite direction) remains OPEN and unreproduced.

🔴 **A self-test searched for a string it contained, and had been failing since its own first commit; closed under `.11.4.3.1.7.1` (REPAIR-0137).** `census_pg_test_clusters.py`'s citation guard asserted that `run-selftest-no-such-cluster-name` appears in **zero** tracked files — while that literal was written in its own tracked source. The file's add-commit and the string's introducing commit are the SAME (`cb2f197`), so the arm passed exactly once, before the file was tracked. ⚠️ It survived because **NOTHING RUNS ANY `--self-test` here** — the 17 registered doctrine checks included (`grep -n "self-test" scripts/check_doctrines.sh` -> 0); that half is OPEN at `.11.4.3.1.7.2`. ⛔ The guard's other direction never broke, so cited clusters stayed protected — what was missing is proof the guard is not simply returning non-zero for everything, i.e. HALF-verified, which must not authorise a deletion. The probe is now generated per run (absent by construction, not by luck); FALSIFIED by restoring the literal. ⭐ With the control passing, the **§8 artifact review** it gates ran for the first time since `cb2f197`: 34 clusters / 1,797,794,711 bytes → **30 retired, 1,591,802,083 bytes freed**, residue verified at 4 / 205,992,628 with `retirable: 0`; the four survivors (three cited evidence, one under the one-hour floor) were confirmed present afterwards.

🔴 **The stored certificate expiry was never read from the certificate; closed under `.3.4.3.1.1` (REPAIR-0136).** Both issuance sites computed `Utc::now() + LEAF_TTL_SECS` independently of the `not_after` signed into the certificate — two derivations of one quantity that never agreed, because the certificate's instant is truncated to whole seconds and the stored one is not (falsified at `…12.973747Z` against `…12Z`), and the enrollment path samples its `now` before the transaction's database work. ⛔ **No certificate was ever accepted or refused wrongly** — nothing enforces the stored value; it was a record disagreeing with its artifact. `issue_node_leaf` now returns the signed instant and a public `ca::leaf_not_after` extracts it from a DER. The leaf's other option (unify on one clock) is UNAVAILABLE: a verifier enforces the signed `not_after`, which cannot move without changing what the CA signs — so the server's two clocks are **declared** coherent within 1 s and that assumption is now MEASURED by a control (⚠️ in CI, on one host; a remote database with a drifting clock is NOT covered). Recorded for `.3.4.3.1.2`: of the three instants the server sends, **only `decided_at` is ever read** by the node. **3 suites / 49 tests + lib 102 + mtls 1**, rc=0.

🔴 **Three clocks, two of them compared against each other in opposite directions, and NOTHING is repaired yet (`.3.4.3.1`, REPAIR-0135).** The census this leaf owed refuted the leaf's own framing. **Three** server instants reach the node (`decided_at`, `cert_expires_at`, `lease_expires_at`); **two** are compared against the node's own clock and they fail OPPOSITE ways — a node AHEAD refuses EVERY dispatch, a node BEHIND by >300 s makes `cert_expires_soon()` fire only after the workload certificate has expired, breaking the channel; the third is received and never read. ⛔ **And the SERVER has two clocks:** `decided_at` is PostgreSQL `clock_timestamp()` (22 sites) while the cert/lease instants are the server process (`Utc::now()`, 8 sites), with nothing reconciling them — so an offset correction built today would silently assume they agree. Split into `.3.4.3.1.1` (the server's own clock split, prerequisite), `.3.4.3.1.2` (the node evaluating server instants in the server's terms — closes the outage), `.3.4.3.1.3` (the certificate direction, ⛔ **arithmetic on a source reading, NOT reproduced**). ⛔ **The dispatch outage remains OPEN and unrepaired.** Documentation-only commit; no behaviour or qualification change.

🔴 **ADR-009 is titled "chain-in-envelope" and carries no chain; measured under `.3.4.5` (REPAIR-0134).** `AuthorityContext` holds one `on_behalf_of` string, so a depth-2 delegation has nowhere to go; a control pins it so the title cannot keep implying a capability the type lacks. ⚠️ Third independent measurement of the same absence, after `.3.4.1` (the grant-chain flags `delegable`/`max_delegation_depth` have no producer) and `.3.4.1.1` — **delegation is one hop deep everywhere, and the vocabulary around it is written for something larger.** The comparative wire-size measurement the withdrawn claim never had now exists as a re-runnable instrument asserting its own table: against a 308 B undelegated baseline, chain-in-envelope is 505/684/861 B at depths 1–3 against a token's 1,066/1,824/2,582 B (ES256), i.e. **177 B against a constant 758 B per hop, ~4.3×**. ⛔ No size ADVANTAGE is offered as the reason for the choice, which rests on the subtraction and the existing revocation lifecycle. ⚠️ Depth 1 encodes the shipped envelope; depths 2–3 are prototype-vs-prototype, and size is not the axis that decides the question — tokens buy offline verification and an unreachable delegator, which no byte count settles. Core suite **74 tests** rc=0.

🔴 **An untrusted HTTP body could say `join` while carrying a decline, and was recorded as a join; closed under `.3.4.4` (REPAIR-0133).** `POST /v1/calls/{call_id}/respond` and the MCP `join_call` seam decode `RecruitmentResponse` from client input. Measured against the shipped decoder: `{"kind":"join","reason":"I decline"}` **accepted as join**, `["join"]` accepted, same pair for `observe`; the members-carrying responses were already strict. Cause is the `.3.3.3.2.2.1` Serde branch — an internally tagged UNIT variant discards the rest of the map and accepts the sequence form, and the `deny_unknown_fields` this type declared switches off neither. ⭐ The census ranked by REACHABILITY, not resemblance: **28 tagged enums, 8 with the shape, exactly 1 decoded from untrusted input** — and it is not `Decision::Allowed`, the type the leaf was opened around, which has no untrusted producer. Repaired for `RecruitmentResponse`, `Decision` and `CachedDecisionKind`; ⚠️ `Decision` is deliberately TIGHTENED beyond its own declaration because the leniency read a denial and its evidence back as an allowance. ⛔ The four adapter types with the same shape are NOT repaired — none is deserialized. ⛔ No evidence any real client exercised it; this is what the endpoint ACCEPTS, not an incident. Core lib 53, `authorization_evaluation` 6, server lib 101, pg suites **3 / 96 tests** rc=0; FALSIFIED twice, each reversion isolating one crate.

⭐ **A latent authorization state is now unwritable, and removing it found two fixtures that had been producing it (`.3.4.1.1`, REPAIR-0132).** `CommandAuthz` held the delegated subject and the delegation scope as two independent `Option` fields; `selection.rs` read the scope under `if let Some(scope)`, so a subject-without-scope would have delegated with the subject's FULL grant selector — the §16.3 widening invariant skipped rather than failed. They are one `Delegation { subject, scope }` now. ⛔ **No defect is repaired and no reachable behaviour changes:** the only production producer always set both. ⚠️ But the "unreachable" claim held for production only — `tests/authority.rs` constructed the state twice, so the widening gate was being SKIPPED inside a test that reads as exercising delegation; both surfaced as compile errors. Giving those fixtures their scope makes the invariant RUN where it did not, which is a strengthening and is stated rather than absorbed. The unconstructibility is proved by the compiler (`error[E0063]: missing field `scope``), not by a test. **4 suites / 66 tests, zero failures**, every existing delegation control unchanged.

🔴 **A node's cached-decision freshness window compared two different clocks;
closed under `.3.4.3` (REPAIR-0131).** `decided_at` is the SERVER's database
clock, while `is_fresh` is evaluated against the NODE's process clock, so a
cached allow held by a node running behind the server stood for **`skew + 60 s`**
of observed time rather than 60 s. ⚠️ That window is load-bearing: the revocation
epoch is bumped only by explicit revocation, so a grant reaching its own
`expires_at` is bounded at the node by the freshness TTL **alone**. The window now
runs from `min(decided_at, received_at)` — the node's own receipt — which binds
only when the node received a decision before its clock says the server made it,
is byte-for-byte the shipped behaviour otherwise, and can never refuse work the
plain rule allowed. ⛔ The opposite direction is UNCHANGED and NOT repaired: a
node clock running *ahead* refuses every dispatch past 60 s of skew, fail-closed
and journaled, owned by `.3.4.3.1` with the question of whether the wire should
carry a duration instead of an instant. Skew is now bounded, **not detected** —
nothing reports that a node's clock disagrees with the server's. **10 targets /
116 tests, zero failures**; FALSIFIED three times, each reversion isolating one
part (no clamp → both skew controls fail with the dispatch allowed; no receipt
refresh → only the replay control fails; no guard → only the redelivery control
fails).

🔴 **A revoked delegation was answered with a success, and invisibly; reproduced
and closed under `.3.4.2` (REPAIR-0130).** `request_hash` covered the operation,
the actor and `envelope.body`, while `authority_context` is a SIBLING of `body`.
Because the idempotency claim is made BEFORE authorization, a second request with
the same key and body but a **revoked** subject answered **`200 replayed=true`,
`ok: true`** and wrote **zero authorization records**. ⛔ No new effect is applied
by a replay, so this was not an escalation of what was written — it was an
unauthorized request told it had succeeded, with no audit trace. The hash now
binds the authority context and such a request is `409 idempotency_mismatch`.
⚠️ A request with NO authority context hashes byte-identically to before, so every
historical undelegated key keeps replaying; a historical delegated key now
conflicts rather than replaying, which is the safe direction and is documented as
the wire change it is. **36 passed / 0 failed**; the affected set passes **6
suites / 62 tests**. FALSIFIED **35 passed / 1 failed** against the exact
pre-`.3.4.2` sources.

`SIGNOFF-REPAIR.3.4` is censused and split into five children (REPAIR-0128),
with two defects measured from source and not yet repaired. ⛔ **CORRECTED at `.3.4.1` (REPAIR-0129):** the census recorded `evaluate()`
never reading `grant.delegable` as a defect. The measurement is right and the
inference was wrong — `delegable` and `max_delegation_depth` govern grant CHAINS,
which have no producer (no grant's parent is ever another grant), and §16.3
conditions delegation on invariants rather than on a flag. A delegated request
must pass four gates, all enforced: the actor's own authority **for the same
target**, the subject's authority, the widening invariant, and the actor's own
thread participation — measured at `403 … is not a participant of this thread`,
naming the ACTOR. 🔴 **The replay hash does not bind the authority context**:
`request_hash` covers the operation, the actor and `envelope.body`, while
`authority_context` is a sibling of `body`, so two requests differing only in
`on_behalf_of` share an idempotency key and the second replays the first's result
without its own authority being evaluated (`.3.4.2`). Bounded depth is
deliberately unenforced in the dev profile and says so in a comment; the
delegate-without-scope pair is unreachable today and is recorded so it is not
rediscovered as a defect. ⭐ The cached decision's two halves fail in opposite
directions — a future EPOCH reads as stale, a future `decided_at` does not
(`.3.4.3`). No production source changed; nothing here is repaired yet.

**`SIGNOFF-REPAIR.3.3.4` is closed at `.13` (REPAIR-0127), and the closure states
its own limits.** Two tracked instruments re-derive the census rather than
re-reading it. The direct named-call surface fell **42 → 27** while the corpus
grew **101 → 118 files**; a new route census classifies all **118 registered
routes** by the gate each actually reaches. ✅ **18 routes run on a guarded
transaction**, all of them mutating: admission, mutation and final-effect evidence
in one commit under a declared guard, at database time sampled after the wait,
each mode derived and each repair falsified against its own superseded source.
⛔ **46 mutating routes do not**, and every one has a named repair owner —
policy 15, evaluations/routing 9, evidence 5, recruitment 3, resolvers/resources
3, deployment 3, regions 3, adapters 2, workflow 1, directory matching 1, plus
`PUT /v1/profiles/{role_id}` which is identity-gated by design. **Being owned is
not being repaired; this closure certifies none of the 46.** No obsolete
bypassing executor remains (84 authority functions, 10 unreferenced, all 10
`#[test]`). ⚠️ Both instruments were wrong before they were right, once in the
direction of reporting live code as dead; the corrections live in the scripts.
Broad step: **19 live suites / 262 tests / zero failures**, plus 99 library unit
tests. The instruments are lexical and bounded — no AST, no dynamic dispatch, no
runtime SQL — and classify which gate is REACHED, not whether it is the right one.

The card import's revocation race is now **closed on both sides** under
`.3.3.4.12.1` (REPAIR-0126). `.11.3` predeclared the limit, `.12` closed the
importing half by putting the direction verbs on their own tenant's exclusive
guard, and the import now declares BOTH tenants in one predeclared sorted set —
the importing tenant exclusive because it issues a grant, the origin tenant
shared because it reads the origin's agreement row. The set goes to the runner,
which sorts it; two acquisitions could not prevent inversion and the guard API
offers no later upgrade. An origin id that does not parse declares no second key,
because the set is built before the transaction and a refusal there would move a
card check ahead of the admission. **6 passed / 0 failed**; the affected set
passes **5 suites / 89 tests**. FALSIFIED **5 passed / 1 failed** at `the import
waits on the ORIGIN tenant's guard` — and the superseded import does not block at
all there, having never declared the key. ⛔ Still `.5.3`'s: what an effective
agreement guarantees a consumer ACROSS a change, since a re-proposal with
different terms still ends an agreement without a revoke.

The three federation direction verbs are now ONE guarded transaction each under
`.3.3.4.12` (REPAIR-0125): admission, mutation, the acceptance's cross-domain
receipt and the effect record share a commit under the LOCAL tenant's
**exclusive** guard. All three were the `.9` shape — an already-committed
shared-guard admission and then a pool mutation — with no guard, record or
receipt. ⭐ The mode is derived from the classify-then-write-over-a-possibly-absent-row
reason, not the epoch one: none advances the revocation epoch. 🔴 Proposing to a
tenant that does not exist RAISED a foreign-key violation and answered **`500`**;
it is a recorded `404 not_found` now. That defect was found by running the
enumerate-the-constraints check promoted one commit earlier, on its first use.
The wire is otherwise unchanged, with three previously indistinguishable idle
states now separated in the record (an unchanged re-proposal and an
already-accepted direction as `no_op`, a never-proposed one as `refused`).
**4 passed / 0 failed**; the affected set passes **4 suites / 72 tests**.
FALSIFIED **1 passed / 3 failed** against the exact pre-`.12` sources, the
ordering control failing at `the revocation is waiting`. ⚠️ **Half the card
import's revocation race is now closed and the leaf says which half**: the import
holds the importing tenant's key, so an importing-side revocation fences it and
an ORIGIN-side one still does not. `.3.3.4.12.1` owns the both-tenant guard set.

**`SIGNOFF-REPAIR.3.3.4.11` is closed** (REPAIR-0118/0119/0120/0122/0123/0124,
plus `.3.3.4.7.4` as REPAIR-0121). The profile/card family's three mutating
routes each run as one transaction with their evidence: the writer serializes at
the role's version anchor, the attestation holds its read AND write under that
anchor inside a shared-guard transaction, and the card import runs the whole
ladder — admission, rungs, grant, identity, quota, enrollment, receipt and the
PROFILE — inside one exclusive-guard transaction, with three unguarded bridges
deleted. ⭐ The family's last leaf was the book chapter the surface had never had,
and it produced two defects and three corrections: a repeated import answered
`500` (repaired as `.11.5`), an unknown field returns `422` rather than `400`,
and **an expired capability claim still satisfies a requirement** because the
eligibility check never reads `expires_at` (owned by `.5.1`). `docs/book/src/profiles.md`
documents all seven handlers, and
`docs/knowledge/writing-the-documentation-is-a-verification-pass.md` records the
method. ⛔ Still open and named rather than implied: card replay and provenance
(`.5.3`), capability expiry and ranking (`.5.1`), and the import's ordering
against a concurrent agreement revocation (`.3.3.4.12`, the next frontier row,
because the three federation direction verbs take no guard at all).

🔴 **A repeated card import answered `500` and recorded nothing; reproduced and
closed under `.3.3.4.11.5` (REPAIR-0123).** `agent_roles` carries
`UNIQUE (tenant_id, name)` and `enrollments` carries `UNIQUE (tenant_id, kind,
name)`, while the import writes the card's `display_label` into both, so the
`agent_roles` insert RAISED. ⚠️ Not only a re-import: any card whose label matches
an identity already in the importing tenant collides. ⭐ Making the import atomic
at `.11.3` is what turned this from a survivable mess into an unrecordable one —
a raised constraint aborts the transaction that now also carries the admission
and the effect record, which is exactly what
`docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md` said at
`.10.1` and `.11.3` did not apply to its own inserts. Both inserts now use
`ON CONFLICT … DO NOTHING RETURNING`; the answer is `400 invalid_command` naming
the label, with an effect record carrying the same sentence. ⛔ It says the label
is taken and deliberately does NOT decide what a repeated import ought to do —
card replay is `.5.3`'s. **4 passed / 0 failed**; the affected set passes **5
suites / 72 tests**. FALSIFIED **3 passed / 1 failed** against the exact
pre-`.11.5` sources. ⚠️ Found by running a documentation claim instead of
publishing it.

🔴 **An orphaned identity on the card-import path was reproduced and closed under
`.3.3.4.11.3` (REPAIR-0122).** The route committed the grant, the `agent_roles`
row, the quota row, the enrollment and the cross-domain receipt as one
transaction and then wrote the profile on the connection pool. Measured with
every new profile version row made to fail: the import answered **`500`** and
left the role behind (`left: 1, right: 0`) with its grant, quota, enrollment and
receipt durable. It also read its admission, its boundary and its federation
agreement outside the transaction that used them. The whole ladder now runs in
ONE transaction under the importing tenant's **exclusive** guard — derived,
because the import issues a grant and issuance asserts that scope. ⚠️ An existing
`cards.rs` assertion caught an unintended change to the grant-refusal message
during development; the response and the effect record now share ONE renderer, so
two descriptions of one refusal are the same string by construction. Three
superseded bridges are deleted: the unordered grant creator, the pool-taking
boundary loader and the pool-taking profile writer. **3 passed / 0 failed**; the
affected set passes **9 suites / 156 tests**. FALSIFIED **1 passed / 2 failed**
against the exact pre-`.11.3` sources. ⛔ Moving the agreement read inside buys
snapshot consistency and NOTHING more — `federation::{propose,accept,revoke}` take
no guard, so nothing here fences a concurrent revocation; `.3.3.4.12` owns it.
`.3.3.4.11`'s three transaction children have landed; `.11.4` still owes the
profile/card book chapter.

The administrative refusal vocabulary is corrected from three codes to four
under `.3.3.4.7.4` (REPAIR-0121), and the correction has the same shape as the
mistake it repairs. `.7.3` classified every `ControlApiError::unauthorized` in
the fourteen handlers as the admission's own denial — right **thirteen times out
of fourteen**. The fourteenth is the card import's allowlist rung, which refuses
an ADMITTED tenant administrator because the importing tenant holds no effective
federation agreement with the card's origin: a precondition about the two
tenants, not about the caller's grant. Re-measured across all fourteen handlers
with a re-runnable census: `handlers not found: none`, exactly one such site.
`AdministrativeRefusal::Unauthorized` is added so that refusal can be recorded
without the record and the response disagreeing — the one thing the type exists
to prevent. ⛔ A denied admission still writes no effect record at all, so a
`refused` outcome carrying this code always means the request was allowed and the
operation was not. No migration; migration 0058 pins only the outcome `kind`.
**68 core tests** and **25 live effect tests** pass; FALSIFIED **9 passed / 1
failed** by leaving the code on the fail-closed list. ⚠️ This is the seventh
instance of the pattern `.11.6` censuses and the second inside `.7`.

🔴 **A silent lost update on the audited attestation path was reproduced and
closed under `.3.3.4.11.2` (REPAIR-0120).** `attest_capability` read the current
profile on the pool and wrote it back through a different transaction. Measured
on the unchanged route: two administrators attesting two DIFFERENT capabilities
of one role both received **`200`**, and the published profile carried one
upgrade and not the other — an audited attestation dropped with no error to
either caller. ⭐ Serializing the writers, which `.11.1` had already done, would
not have closed it: the losing writer's version NUMBER was correct while its
CONTENT was stale, so the READ had to move under the same anchor lock as the
write. The verb now runs one shared-guard transaction from the admission through
the anchor lock, the read, the upgrade, the new version and the effect record;
the mode is shared because an attestation writes no authority and advances no
revocation epoch. The wire is unchanged — the single `404` still answers both
idle states and the record distinguishes them — plus the
`x-reasonbraid-authorization` receipt on every admitted answer. **38 passed /
0 failed**; the affected set passes **4 suites / 65 tests**. FALSIFIED **36
passed / 2 failed** against the exact pre-`.11.2` sources. ⚠️ The mode does not
change in this repair, so no lock-holding fixture discriminates it; the
discriminating properties are the lost update and atomicity, and the third new
control is labelled a regression control.

🔴 **A concurrency defect on the profile write surface was reproduced and closed
under `.3.3.4.11.1` (REPAIR-0119).** The writer computed its version as a
read-then-write against migration 0019's `UNIQUE (role_id, version)`. Measured on
the unchanged route with four concurrent writers for one role, run twice: **five
of the eight answered `500`**, logging `duplicate key value violates unique
constraint "profile_versions_role_id_version_key"`. A role rewriting its own
profile from two places at once lost writes to an internal error. ⭐ The
serialization now happens at the role's own anchor row, created INSIDE the
acquisition — `SELECT … FOR UPDATE` over a row that does not exist yet locks
nothing, so a lock-only fix would have protected every write except a role's
first. The `UNIQUE` constraint is kept as the backstop that proves the lock
works. ⭐ The route takes NO tenant authority guard, derived rather than skipped:
it is gated on identity, evaluates no grant and produces no authorization record,
so there is no authority decision to order it against. `written_at` is now the
transaction's own database time; status codes, response fields and the content
addressing are unchanged. **35 passed / 0 failed**; the affected set passes
**4 suites / 62 tests**. FALSIFIED **34 passed / 1 failed** against the exact
pre-`.11.1` sources. ⚠️ The discriminating control is probabilistic and labelled
as such — no lock-holding fixture can discriminate this repair, because both
shapes contend on the same anchor row at different points. ⚠️ Newly tracked, not
merely noted: the `/v1/profiles` surface has **no book chapter**, owned as
`.3.3.4.11.4`.

The profile/card surface `.3.3.4.11` (REPAIR-0118) is censused and split into
three children before any implementation, and the census changed two of the
parent's own assumptions. Seven handlers on six route entries, of which exactly
**three mutate**; `.7.1`'s closed vocabulary assigns this parent two operations
and deliberately no `ProfileWrite`, because `put_profile` is a self-declaration
gated by identity with no authorization record at all. ⭐ Unlike `.10`, **all
three mutations are already tenant-bound**, so there is no tenant-binding repair
in this family. What is missing is the transaction and the evidence, and three
defects are measured from source: `profiles::write_profile` computes its version
as a read-then-write against migration 0019's `UNIQUE (role_id, version)`, so two
concurrent writes for one role collide and one caller receives **500** instead of
a serialized second version; `attest_capability` reads the profile on the pool
and writes through a different transaction, so two administrators attesting
different capabilities of the same role **silently lose one of the two
attestations**; and `import_profile_card` commits the grant, role, quota,
enrollment and receipt and writes the profile only afterwards, so a failure there
answers 500 leaving an **orphan role with no profile**, having also read its
admission, its active boundary and its effective agreement outside the
transaction that uses them. ⚠️ One limit is recorded against the import child in
advance: `federation::revoke` takes no guard, so no guard set here can fence a
concurrent agreement revocation — that ordering arrives with `.3.3.4.12`. Guard
modes are not decided at the split; each child derives its own. Task-tree and
index only; no production source changed and no repair is claimed.

🔴 **A cross-tenant destructive defect was reproduced and closed under
`.3.3.4.10.3` (REPAIR-0117).** `node_inbox` has carried a `tenant_id` column
since migration 0003 and none of the three operator verbs used it. Measured on
the superseded routes, with an administrator of tenant A acting on a node whose
inbox rows belong to tenant B: quarantine answered **200** and quarantined the
foreign row, replay answered **200** and re-sequenced it, and prune answered
**200** with `{"deleted":2,"before":2,"after":0}` — **destroying both of the
other tenant's rows**. It was reachable by any ordinary tenant administrator
through the supported HTTP surface. ⚠️ Three of this project's own fixtures
depended on the defect, seeding a node into one tenant and administering it from
another, which is why nothing caught it; they are CORRECTED, with every original
feature assertion intact. All three verbs now select bound to the admitted tenant
inside one shared-guard transaction, the prune's before/after counts are bound
too, and each records its outcome. **7 passed / 0 failed**; the affected set
passes **6 suites / 72 tests**. FALSIFIED **3 passed / 4 failed** against the
exact pre-`.10.3` handlers. `.3.3.4.10` is now closed: all five node
administrative mutations the parent's census found are one guarded transaction
with a tenant-bound target and a final effect record.

Node certificate revocation `.3.3.4.10.2` (REPAIR-0116) is now ONE guarded
transaction, taking the **exclusive** guard — the opposite answer from `.10.1`
one commit earlier, and derived rather than alternated: it advances the tenant's
revocation epoch, so it is a revocation in the sense the guard contract means and
must fence admissions. Its superseded shape ran the tenant-bound existence probe
as a SEPARATE pool query and then mutated on `node_id` alone, so the check and the
act read two different snapshots; both are now one transaction, the node row is
selected `FOR UPDATE` bound to the admitted tenant, and the UPDATE carries its own
tenant predicate. The submitted reason is PERSISTED with the effect rather than
checked for blankness and discarded, and gains the 1 024-byte control-character-free
bounds; `revoked_at` is the transaction's own database time; every answer carries
the receipt. The single `409` still answers both idle states and the record
distinguishes a repeat (`no_op`) from a node that never had a certificate
(`refused`). **30 passed / 0 failed**; the affected set passes **5 suites / 74
tests**. FALSIFIED **25 passed / 5 failed** against the exact pre-`.10.2` handler,
where the request COMPLETED while a shared holder held the tenant's guard. ⚠️ The
discriminating fixture is a SHARED holder here and was ATOMICITY for `.10.1` —
same parent, same sequence, two different correct fixtures, because one repair
changed the lock mode and the other did not.

Node administration `.3.3.4.10` is censused and split (REPAIR-0114), and its
first child `.10.1` (REPAIR-0115) puts enrollment-token issuance onto the guarded
effect transaction. The census measured something worse than the parent's note
assumed: **four of the five node administrative mutations carry no tenant
predicate at all** — the three inbox verbs and the certificate revocation select
rows by `node_id` alone, and revocation's tenant-bound existence probe is a
separate pool query outside the mutation's transaction. That finding is annotated
at `.3.5`, which owns real target ownership; `.10` owns putting the verification
inside the mutating transaction. `.10.1` takes the **shared** guard, not the
exclusive one `.9` took, and the reasons are measured: the refusal is decided
atomically by one insert that does nothing on conflict, same-node contention is
already settled by the partial unique index, and issuance touches nothing the
reservation path reads. `expires_at` now runs from the transaction's own database
time; every admitted answer carries the receipt, and the pre-admission validation
refusal deliberately does not. **11 passed / 0 failed**; the affected set passes
**6 suites / 84 tests**. FALSIFIED **7 passed / 4 failed** against the exact
pre-`.10.1` handler, where the token COMMITS although its evidence could not be
written (`left: 200, right: 500`). ⚠️ A shared-guard repair cannot be falsified by
a lock-holding fixture in EITHER mode, because the superseded admission took the
shared guard too — the discriminator is atomicity, and the ordering property is
labelled a regression control rather than presented as proof. ⚠️ Reproduced and
routed to `.3.5`, not introduced here: the one-unused-token index is global rather
than per tenant, so one tenant's outstanding token both reveals itself to, and
blocks, another tenant's administrator for the same node identity.

Spend-breaker arm and reset are now ONE guarded transaction under `.3.3.4.9`
(REPAIR-0113) — the SECOND family onto the effect transaction, and previously the
weakest administrative path in the server: each verb admitted the caller under a
shared guard in its own transaction and then mutated **on the connection pool**,
outside any transaction, under no guard, recording nothing about what it did.
Both now run one exclusive-guard transaction holding database time sampled after
the wait, the admission, tenant-bound `FOR UPDATE` selection, the mutation and the
effect record. The exclusive mode is measured rather than copied: classifying an
arm against a row that MAY NOT EXIST cannot be done under a row lock, and the
reservation path holds the shared guard, so exclusive is what orders an arm
against every in-flight reservation in the tenant. Status codes, messages and
success bodies are byte-unchanged; the one addition is the
`x-reasonbraid-authorization` receipt on every answer, without which the effect
record is unreachable. The `409` that has always collapsed "no breaker armed" and
"armed but untripped" still does, and the record distinguishes them as
`refused`/`invalid_transition` against `no_op`. Neither verb takes a caller
reason, so neither invents one, and neither advances the revocation epoch —
asserted on the success path too, which is where a copied-from-`.8` mistake would
show. **25 passed / 0 failed** (16 from `.8` plus 9 new); the affected set passes
**5 suites / 94 tests**. FALSIFIED against the exact pre-`.9` handlers restored
from `HEAD`: **17 passed / 8 failed**, with an arm APPLYING while another
operation held the tenant's authority guard, and
`authority_that_ends_while_a_breaker_request_waits_refuses_it` reporting
`left: 200, right: 403`. ⚠️ That falsification also corrected the controls: an
EXCLUSIVE-holder fixture could not discriminate the repair, because the
superseded admission took a SHARED guard and waited behind it too — promoted to
`docs/knowledge/proving-a-race-is-closed.md`, and it applies to `.10`–`.12`.

Grant and boundary revocation are now ONE guarded transaction under `.3.3.4.8`
(REPAIR-0112), and it is the first route to produce a final effect record. The
route used to run two guarded transactions — a shared-guard admission, then an
exclusive-guard service — so a revocation could apply on authority that had
already stopped holding. One `transact` now holds database time sampled after the
guard wait, the admission, tenant-bound `FOR UPDATE` selection, the status change,
the epoch bump and the effect record. The two superseded services are DELETED,
not deprecated. Every answer including refusals carries
`x-reasonbraid-authorization`; a 403 or 400 records its admission only, while a
404 records `refused`/`not_found` in the caller's own tenant, preserving `.3.1`'s
one indistinguishable answer for missing and foreign. Two documented wire
changes: the reason gains the 1 024-byte control-character-free contract, and
`revoked_at` is the transaction's own database time. **16 passed / 0 failed** on
three independent runs; FALSIFIED **11 passed / 5 failed** — the pre-`.8` shape
makes `authority_that_ends_while_a_revocation_waits_refuses_it` report
`left: 200, right: 403`, the revocation applying after the caller's own
administration ended while it queued. `.9`–`.12` adopt the effect record next;
until each does, its operations have no effect row, which reads as an absence and
never as a success. `.9` is now done; `.10`–`.12` remain.

`.3.3.4.7.3` (REPAIR-0111) corrects a defect in `.7.1`'s own representation,
found by its first consumer. The stored refusal code was typed against the §9.8
`KnownReasonCode` registry so the record and the response could not disagree; the
registry has no `not_found`, which is exactly what a revocation's 404 returns. A
census rejected the obvious one-value patch: the registry publishes **20** codes,
the product emits **19** distinct ones, **10** of those are absent from the
registry and **11** registry codes are never emitted. A second census — parsing
all 14 administrative handler bodies — fixed the replacement at **three** domain
refusals of an admitted operation, `invalid_command`, `invalid_transition` and
`not_found`, whose wire names are literally the strings the response carries.
No migration: the `outcome` column's `CHECK` pins only the `kind` discriminant.
Core: **68 passed / 0 failed**, strict lint rc=0. The registry drift itself is
routed to `.11.7` with its two-way census attached.

The final administrative effect record now has durable storage under `.3.3.4.7.2`
(REPAIR-0110). Migration 0058 adds `administrative_effects`, keyed by the
admission's own `authz_…` id, with the `kind` discriminant of both JSON columns
constrained to the declared vocabularies and a COMPOSITE foreign key
`(record_id, tenant_id)` so an effect cannot cite another tenant's admission.
The writer runs on the caller's already-guarded connection and takes no guard of
its own, so evidence and mutation share one commit and an evidence failure rolls
the protected write back; the reader filters by tenant before decoding. The
layering is deliberate: the database constrains the discriminant and the core
codec enforces everything beneath it, asserted by five rows that pass the column
`CHECK` and fail the codec. `bash scripts/run_pg_tests.sh administrative_effects`
returns rc=0 with **8 passed / 0 failed**, cluster stopped and removed. FALSIFIED
**5 passed / 3 failed** against three individually attributable injections — a
dropped composite key admits a cross-tenant effect, a widened `CHECK` admits an
invented `grant_revoke_all`, and a writer swallowing its error stops reporting a
duplicate — while five controls passed at that same baseline. ⚠️ The FIRST
falsification attempt returned 0 passed / 8 failed on invalid injected SQL: a
negative build where controls the injection cannot touch also fail is a broken
fixture, not a falsification. The 26 fixture plans that delete
`authorization_records` were censused and updated before the constraint
shipped; the sweep also hit one NON-plan array (`migration_upgrade.rs`'s
pre-upgrade snapshot at version 55), which the adjacent-suite gate caught
with `relation "administrative_effects" does not exist` and which is
reverted — the re-audit is structural, parsing every `delete_tables(…)`
argument list rather than matching a line's shape.
No production route wrote an effect record at that leaf; `.3.3.4.8` is the first
producer and `.3.3.4.9` the second.

The final administrative effect representation is defined under `.3.3.4.7.1`
(REPAIR-0109), in `reasonbraid-core` only: no schema, no server change, and no
route writes an effect record yet. An admission record says a caller was allowed
to ASK; it says nothing about what the local mutation did, and `.7.2` then `.8`
close that. The closed operation set was measured rather than chosen — all **27**
guarded-admission call sites in `api.rs` classified as 9 reads and 18 mutations,
of which **14** belong to this family's `.8`–`.12` children and 4 are routed to
the top-level `SIGNOFF-REPAIR.5.2`/`.7.1`. The 14 partition 2+2+5+2+3 across those four children, reproducing
scopes they declared independently. Operation and target are one closed type, so
an operation carrying a target that cannot belong to it is unrepresentable; every
target is caller-supplied, so it exists at refusal as well as at success. The
outcome is `applied`/`no_op`/`refused`, where only `applied` asserts a protected
change and a refusal names a §9.8 `KnownReasonCode`. `cargo test -p
reasonbraid-core` returns **68 passed / 0 failed** including 10 new controls; the
object-only control is FALSIFIED against a permissive decoder (`["breaker_arm"]
decoded as an operation`, 9 passed / 1 failed) and the file restored. A control
also refused the leaf's own first draft: `Option<T>` does not make a serde field
required, so a dropped `submitted_reason` would have read back as "no reason
submitted".

Standalone read admissions are ordered against authority changes under
`.3.3.4.6` (REPAIR-0108). The eight frozen-tenant reads COMMIT a record naming
the selected parent's status and the grant's selector, and that evidence was
being selected with no ordering against the writer that changes it; so was the
ordinary standalone admission behind thread inspection, node-token issuance,
automatic thread creation, recruitment calls and every `tenant_admin` gate.
Baseline `command_ordering`: 4 passed / 3 failed — the admission committed under
a held exclusive guard (`admission rows 0 -> 1`), authority that ended during a
wait still returned 200, and the thread-inspection admission committed too. The
repair takes the SHARED guard before any authority row is read and samples
`clock_timestamp()` after that wait, in `authorize_tenant_admin_inspection` and
in the new `authorize_guarded`; `authorize` is unchanged as the explicit-time
standalone API. The nested-transaction hazard was censused first: all 28
guarded-admission call sites, 0 with an open transaction before them. The
approved frozen-boundary carve-out keeps its own control, so a boundary revoked
while a read waits still admits its eligible administrator. `command_ordering`
goes 4 passed / 3 failed → 7 passed / 0 failed and the affected set passes rc=0
with 5 suites / 71 tests / zero failures. Response queries still run after the
admission commits, so a receipt proves admission, not a shared snapshot or
delivery.


The overdue `CLAUDE.md` §8 artifact review ran under `.11.4.3.1.7`
(REPAIR-0107), with the judgement in a tracked instrument rather than in a
habit. `scripts/census_pg_test_clusters.py` censused **13 retained PostgreSQL
clusters / 676,882,885 bytes**, all `state: stopped`, with 0 live servers and 0
tracked citations, and retired **11 / 573,416,291 bytes** after re-checking
every condition immediately before each removal; the residue census verified the
2 survivors, both held back by the age floor because they are this session's own
baselines. Its `--self-test` fires six refusals plus the citation guard's two
directions. The measurement corrected the leaf's own starting guess:
`target/debug/deps` is 134,478,136 KiB against `incremental`'s 58,989,732 KiB,
so the cache Cargo does NOT collect is the larger one and `.11.4.3.1.6` retired
only the other. Nothing under `target/debug` was deleted — that directory is
what the linker resolves by hash — and `.11.4.3.1.8` owns the decision, with
"measured and not worth acting on" available as a legitimate outcome.


The ordering chapter's expiry-during-wait row is now a control rather than an
argument under `.3.3.4.4.1` (REPAIR-0106). `.3.3.4.4` published five contract
rows and shipped two controls; the census behind the finding is
`grep -n 'async fn a_\|async fn the_' crates/reasonbraid-server/tests/command_ordering.rs`
returning 2 test functions, and `grep -rn 'UPDATE authority_grants'
crates/*/tests/*.rs` returning 21 sites across four suites of which none runs
while a request is blocked. The new control holds the exclusive guard, ends the
caller's grant underneath the blocked command and asserts 403 with an unchanged
event count; the suite goes 2 → 3 controls at rc=0. It is falsified rather than
merely green: with `acquire_in_tx` removed, `command_ordering` returns 1 passed
/ 2 failed, and `api.rs` was then restored with an empty `git diff HEAD`. The
acceptance as opened was corrected by building it — the control separates the
WAIT, not a database clock from a process clock, because both are sampled after
the wait on this path. No production source changed.


Node results are ordered against authority changes under `.3.3.4.5`
(REPAIR-0105), and the defect was the same shape as `.3.3.4.4` one path
further on: `node_channel::events` opened a plain transaction, locked the
node's LEASE row, wrote the receipt and folded the result while authorizing on
the PROCESS clock, with no tenant guard anywhere. Baseline under a held
exclusive guard: the result ran to completion and wrote both rows
(`receipts 0 -> 1, contributions 0 -> 1`), authority that ended while a result
waited was still used to fold, and an exclusive guard could be acquired freely
while the handler sat blocked on the lease row — the inversion, observed in
`pg_stat_activity` rather than argued. A SECOND defect was found by these
controls and had not been suspected: `events` discarded the application's error
and committed regardless, so a SQL failure's COMMIT ran as a ROLLBACK while the
handler answered `200 {"accepted":true}` for a receipt that was never written
(measured with an injected, reverted `event_log` trigger: `node_events` rows 0).
The repair resolves the effect's tenant with a lock-free read, takes the shared
guard BEFORE the lease, binds every effect's SQL to that tenant, samples
`clock_timestamp()` after the guard and idempotency waits, and probes the
transaction with `SELECT 1` before COMMIT. An ordinary channel receipt takes no
guard, because it has no tenant-bound effect to order. `node_result_ordering`
goes **2 passed / 4 failed → 6 passed / 0 failed**; the affected family passes
at rc=0 with **7 suites, 46 tests, zero failures**
(`node_result_ordering node_work node_channel node_replacement node_inbox
quarantine command_ordering`), cluster stopped and removed, and `budget` keeps
its 7 settlement controls green. Node credential/lease proof, partial-result
handling and budget settlement guarantees keep their own owners `.4.1`–`.4.5`.


Thread commands are ordered against authority changes under `.3.3.4.4`
(REPAIR-0104). The defect was not a narrow race window: `run_thread_command`
opened a plain transaction, claimed its idempotency key and authorized on the
PROCESS clock with no tenant guard anywhere, so a command and a revocation had
no defined order at all. The reproduction is therefore deterministic rather than
a race — an exclusive guard is what a revocation holds, so the control holds one
and watches the command run underneath it: `event rows 0 -> 1` at baseline,
while the sibling control (shared guard, unrelated tenant) passed. A FALSE
reproduction was caught first and is recorded: `event rows 0 -> 0`, a request
rejected at deserialization that "completed" without reaching any lock and
looked exactly like the defect. Asserting the effect count beside the timing is
what separated them. The repair factors the guard runner's own per-key
acquisition into `acquire_in_tx`, so both entrypoints take the identical lock
rather than a reimplementation; the mode is Shared, so commands stay concurrent
with each other while a revocation's Exclusive mode fences them. Decision time
is now `clock_timestamp()` sampled after the guard and idempotency waits, so a
grant that expires while a command is queued is evaluated as expired.
`authorize_in_tx` keeps its explicit timestamp parameter, so the standalone
API's documented evaluation-time compatibility is untouched. The FULL owned PostgreSQL collection passes at **rc=0** — 41 commands, 42 suites, **295 tests, zero failures** — with `pg-tests: stopped and removed target/pg-tests/run-n7nqvz4b`. That breadth is the right gate here rather than a focused set: every HTTP and MCP command routes through the transaction this leaf changed, and the new `command_ordering` suite is registered in the runner so it travels with the collection and with CI instead of being run by hand.
Replay-hash and consent semantics remain `.3.4`; automatic-initiation preflight
remains `.5.2`.


Authority writer coverage is re-derived under `.3.3.4.3.4` (REPAIR-0103), and
the first finding was about the evidence rather than the code: `.3.3.4.1`'s
census existed as a table plus a corpus hash with **no tracked producer**, so it
could not be repeated and a rebuilt predicate would have looked comparable while
differing by a name. `scripts/census_authority_paths.py` is now that instrument,
with a `<sha>` mode, and its correctness check is exact reproduction of the
recorded baseline at `1ba6184` — 101 files, 1,749,975 bytes, SHA-256 `340c4af6…`
and 42 locations, all four matching. Only because they match is the new number
evidence. Re-run at the current commit: the corpus has grown to 111 files /
1,971,693 bytes while direct named-call locations fell 42 → 39, and a per-file
diff separates two events a total would have merged — three calls MOVED into the
new `authority/issuance.rs` (net zero, a module split), and three
`create_grant_in_tx` sites genuinely went away, that name now having zero
references including its declaration. Obsolete bridges: none remain — 86
functions declared across nine authority modules, 10 unreferenced, all 10
`#[test]` functions reached by the harness. The two unguarded families are
re-verified rather than assumed: `revoke_node` still calls
`bump_revocation_epoch` on a raw transaction and `import_profile_card` still has
no guard, both matching their recorded owners `.3.3.4.10` and `.3.3.4.11`, so
the ownership table needs no correction. The compatibility set passes live at
rc=0 — 109 tests across seven suites including `migration_upgrade`, cluster
stopped and removed. Limits are unchanged and not widened by re-running the
instrument: bounded, lexical, blind to dynamic dispatch and arbitrary SQL, and
not a security-boundary proof.


The CLI's configured endpoint is checked on every verb under
`.3.3.4.3.3.3.3.2.3.3` (REPAIR-0102), and the reproduction upgraded the finding
from reasoned to demonstrated: pointed at
`http://operator:secret@127.0.0.1:<port>`, an ordinary verb sent
`authorization: Basic b3BlcmF0b3I6c2VjcmV0` — the transport converts URL
userinfo into credentials on the wire rather than ignoring it.
`ApiClient::for_base` now canonicalises through the same `canonical_server` the
bootstrap path has always used, and `ApiClient::new` is crate-private, so the
only construction reachable from outside the crate is the checked one; all 22
`lib.rs` call sites are migrated. The control asserts on what the ORIGIN
received, because the claim is that nothing was sent, and that the refusal does
not echo the credential. A live base is normalised while a stored bootstrap
identity must already be canonical — configuration input and a durable binding
are deliberately different. The whole CLI crate passes `--all-targets` at **rc=0** — 12 library, 4 bootstrap-state, 5 end-to-end, 5 transport, 4 state-publication and 12 state-writers tests — with the run status captured BEFORE any pipe, after an earlier run in this session reported exit 0 through a `grep` while a test had failed. Severity is unchanged by the
measurement: the input is the operator's own configuration, so this is hardening
rather than a third-party escalation.


The CLI's transport is bounded under `.3.3.4.3.3.3.3.2.3.2` (REPAIR-0101). The
baseline is the defect stated precisely: against an origin that completes the
TCP handshake and never answers, `run_enroll` **did not return within 90
seconds**. That matters more here than in an ordinary client because the CLI
persists a bootstrap request key BEFORE dispatch and holds the state lock across
the response — both deliberate — so unbounded, one silent peer held the store for
the life of the process. The bounds are a 10 s connect ceiling, a 60 s
whole-request ceiling covering the body, an 8 MiB reply ceiling read in chunks,
and no redirect following. The 60 s figure is derived rather than chosen: the
server's whole-operation budget is 15 s and a caller can wait behind another
caller's operation first, so the worst legitimate case is about 30 s and the
bound is twice it. The reply ceiling matches the store's own 8 MiB limit,
because a reply the store could never hold cannot become a published outcome.
Redirects are refused because the base is operator-configured and a redirect
would carry the development principal header and a bootstrap key to a host
nobody named. What survives a refusal is what recovery needs: the ORIGINAL key,
a released exclusion, and a reused key on the next attempt. Four controls pass
plus the whole CLI crate at `--all-targets`, with two reverted injections and
three repeats plus one run under deliberate saturation. Repeat 1 FAILED — `the refusal took 75.032698709s` against a 60-second bound, with a clippy running alongside, because the assertion was `< 75s`. That failure IS the evidence for the margin: wall clock contains the runtime scheduling as well as the deadline, so a margin tight enough to separate 60 from 75 measures the host, not the client, and fails wherever saturation is normal. The margin was widened to 100 s, the declared 60 s value pinned deterministically against the number the book documents, and the reply control timing assertion replaced with a semantic one. Repeats 2 and 3 then passed, and the corrected controls passed again with all 12 cores deliberately saturated (load average 1.86 → 14.15, 134.46 s, 4 of 4) — heavier load than produced the original failure. A first attempt at that loaded run was discarded rather than counted: its clippy was cached, checked one crate and loaded nothing. This bounds ONE process's transport; process death mid-request,
server restart and filesystem failure remain `.3.3.4.3.3.3.3.3`, and a timeout
still says nothing about whether the server committed. One census routed out
rather than folded in: `canonical_server` guards only the bootstrap path while
`ApiClient::new` has 22 unvalidated call sites, so 1 of 23 construction paths
checks the configured endpoint — owned by `.3.3.4.3.3.3.3.2.3.3`.


The enforcer now runs **17 registered checks**. `INDEX-FRONTIER` (REPAIR-0100)
extends to the project's own index the rule `BOOK-FRONTIER` already applied to
the book: `docs/TASK_TREE.md`'s Frontier column may not name a leaf the owning
tree's row 1 does not. Measured rather than estimated — over the last 60
commits, 39 breach and 21 genuinely agree, and the drift begins at `1ebfebe`,
the commit that CLOSED the leaf the row then went on naming for 39 commits. The
rule is deliberately narrow because the census rejected the obvious one: 8
completed trees write a dash row and point at the NEXT tree's first leaf, and
`PHASE-8`'s row 1 is a cross-tree prerequisite its cell summarises accurately,
so blanket equality would flag legitimate rows. The generator that was this
leaf's provisional preference was rejected on the same census — it would destroy
accurate curated prose in 13 of 14 rows. Only an ACTIVE tree whose row 1 names
its own leaf is checked, which is also the only row that moves. The gate caught
its own author on its first live run: closing the leaf moved row 1 and the index
still pointed at the leaf being closed.

The enforcer previously reached 16 checks with `LOCKSTEP-CLAIM`. `LOCKSTEP-CLAIM` (REPAIR-0099)
closes the half `TASK-ACCEPTANCE` structurally cannot: that gate proves a leaf's
box is ticked and cites something re-runnable, never that the cited edit landed.
A ticked bold LOCKSTEP box may no longer name a core live document the commit
does not stage. It is keyed on the author's own claim rather than a blanket
requirement, because the census showed the blanket rule was wrong — over 25
commits, 10 closed a leaf and CHANGELOG.md was staged 10/10 but MEMORY.md only
6/10, so "closing a leaf must stage MEMORY" would have asserted something the
project does not do, flagged four innocent commits, and still missed the defect.
Re-measured with the finished gate over 30 commits: 11 add a claim and were
genuinely exercised, 10 pass, exactly one (6bf0c40) breaches, zero false
positives; the other 19 pass vacuously because they add no claim, and that
distinction is recorded so 30 green is not read as 30 tests. The honest escape
is `lockstep: <doc> unchanged (<why>)` in the claim's own section, and the
refusal says explicitly that deleting the document's name is not a discharge.
`scripts/check_lockstep_claim.sh --against <sha>` re-runs the predicate over any
past commit, which is what made the leaf's acceptance executable rather than
argued.


The full startup source read found open invariant failures and coverage gaps.
Historical phase closure does not establish current production qualification.
`docs/tasks/SIGNOFF-REPAIR.md` owns reproduction, fixes and requalification;
`docs/tasks/artifacts/signoff_review/INDEX.md` preserves the source evidence.
The runner cleanup and spawn/signal defects now have runtime controls and fixes;
foreign-target revocation and repeated-revoke epoch defects are reproduced; the
`.3.1` correction passed 34 focused tests and strict lint. Shared writes by frozen
tenant admins are now runtime-confirmed. The new site service passes ten live
controls and strict focused lint; the protected CLI passes its live controls.
All seven HTTP registry operations now use site authority and pass eight focused
HTTP controls plus adjacent checks; other repairs remain open.

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

The shared adapter/region registry design is now explicit site-operator authority.
The separate site service implements that design. HTTP handlers call that service under `.3.2.3`, with live cross-tenant/freeze,
actual-parent, audit/rollback, wire-input and revocation-race controls.

The scheduled pre-push checkpoint inventory is complete under .11.4.3.1.1. Workflow
locality/script coverage, publisher/browser fixture ownership and evidence-based
cleanup have concrete prerequisite owners .3–.6 before full execution .2. This
source/tool census changes no qualification category and claims no fresh full CI.
The shared CI environment prerequisite .11.4.3.1.3.1 now passes eight focused and
eighteen adjacent controls. Scanner setup .11.4.3.1.3.2 passes thirteen controls,
final affected checks, all eight archive identity/layout checks and native version
probes; the original version-format failure and corrected retry remain preserved.
Workflow wiring .11.4.3.1.3.3 now passes YAML/shell routing checks, five omission
controls, real synthetic Gitleaks redaction, fifty Python controls (including live
PostgreSQL ownership) and rendered book checks. Subsequent prerequisite and full
local/remote gate results are tracked separately; categories are unchanged.
Publisher fixture ownership .11.4.3.1.4 now passes five focused controls, independent
helper lifetimes, two concurrent executable runs, strict lint and rendered book
checks. Historical residue is preserved; all results are consumed. Browser and
compiler-artifact prerequisites remain before full execution; categories unchanged.

Browser test ownership .11.4.3.1.5.1 passes eight controls with real rendering,
strict lint and consumed process/source/residue checks. Native transient group
refusal is reproduced; bounded observation still requires actual absence and
preserves the original failed fixture. Production lifetime .5.2 now passes five
unit/thirteen integration controls, strict lint and native/source/book checks:
private runtime storage, owned cancellable launch and consumed process/task shutdown.
All twenty-one final groups are absent; failed startup evidence is retained and its
uncaptured wait mechanism remains .11.2. Parent transport/termination and aggregate
retention/container limits have concrete .7.3.1/.2 owners. Combined .5.3 now passes
fifteen integration controls, strict lint and native/source/book checks with unchanged
production bytes: real root relocation/refusal, gated overlap and exact listener
close receipts. Twenty-six final groups are absent and nine failed fixtures preserved.
Browser prerequisite .5 is complete. Compiler disposition .6 removes 645 obsolete
sessions under verified native locks; exact residue and preserved source/evidence
checks pass. The affected server build and final book checks pass; all results
are consumed and .6 is complete. Full local/remote execution .2 and push remain
pending. Qualification categories are unchanged.

Publication-precondition audit .11.4.3.1.2.1 established the public remote and
preserved the interrupted checkpoint. The director has resolved the question:
README's private instruction was wrong; the project is public and must remain
public. Correction .2.3 synchronizes README, ADR, security/companion guidance and
the book; no remote-setting change is needed. Format/cargo-deny passed; two
redacted history findings remain .2.2 and interrupted/unstarted gates remain
unqualified. Continue repairs and full checkpoint .2 before the authorized public
push. Qualification categories are unchanged. Policy:
docs/decisions/2026-09-09_public-repository-policy.md.

History-scan repair .11.4.3.1.2.2 classifies both original matches as predictable
metadata-only fixture literals. Two exact immutable fingerprints are qualified by
five native controls (2/1/1/0/2 findings), including detection of identical content
in a new commit. The actual pinned scanner passes with an empty report; eleven
control groups are independently absent and all results consumed. No file/rule
suppression or history rewrite. Full checkpoint .2 resumes next; qualification
categories remain unchanged.

The source-7e01097 full checkpoint passes format, strict lint, bins, fifty Python
controls, book/doctrines and both scanners, but workspace execution stops at two
browser timing witnesses. Repair .11.4.3.1.2.4 reproduces those assumptions and
qualifies explicit gated arrivals/overlap with two delayed controls and all sixteen
integration tests on a dedicated testing runtime. Native evidence identifies the
desktop browser's detached updater/crash-report stderr writers; that refusal stays
preserved. Production bytes are unchanged; strict focused lint and process/fixture
checks pass. Pinned local/CI runtime binding .2.5 precedes the next full checkpoint.
PostgreSQL/demo and remote CI are pending; qualification categories are unchanged.

Dedicated runtime prerequisite .11.4.3.1.2.5 is now complete. Local Make and CI pin
the same verified testing browser; four archive layouts and native setup/rendering
are qualified. All 67 Python controls and sixteen browser integration tests pass,
including a reproduced/repaired shared Python zombie-group shutdown race. Final
wiring/book/process/residue checks pass; failed evidence remains and production
Rust is unchanged by that prerequisite. Qualification categories remain unchanged.

The resumed source-b0cddfe checkpoint passes nine gates, including the pinned-browser
workspace run and 67 Python controls, then fails in identity_store fixture cleanup
after thirteen live PostgreSQL suites pass. Certificates left by node_work block
parent-node deletion; twenty-six later commands and the demo are unstarted. The
minimal test-only repair passes its new regression and all four identity tests on
fresh and node-work-populated databases; the production FK and deployment CA remain
intact. Expanded fixture review reproduces MCP-listener, CLI spend-breaker and
incarnation residue failures. Shared cleanup-plan check .11.4.3.1.2.7.1 passes
eight guard tests and strict lint. Fourteen node callers now use it and all cleanup
plans execute; real MCP→identity and breaker→CLI sequences pass. The affected
census has 135 passing assertions and two failures, both reproduced with original
fixtures: participant-removal authorization (.2.8) and a fixed retention-test date
(.2.9). The server target repair .2.8 now passes all six invitation tests, 22
authority and 33 command-API tests plus ten pure evaluator controls and strict
server lint. Refusals preserve domain state and historical denial replay; valid
tenant administrators can remove participants. CLI companion .2.10 now passes
all five real CLI tests: administrative removal requests tenant-wide scope and
ordinary delegated invitation retains its single-thread scope. Actual unchanged-
CLI and deliberately overbroad controls fail as expected. Retention fixture .2.9
now uses observed creation times and passes the focused test and all 31 profiles
tests, with strict boundaries, exact class effects and preserved audit/replay
state. Strict server lint and source/process/book checks pass. All six partial
fixtures under .2.7.3 now use explicit checked dependencies and pass both actual
node-work→consumer pairs and a consecutive consumer run, preserving deployment
CA rows and original assertions. All 25 original plans now use checked cleanup. Final five-plan adoption .2.7.4
passes its 22-test consumer sequence and all 169 distinct tests in the consecutive
affected collection, plus strict server/MCP/CLI lint and final verification.
Both successful databases are removed and prior failures preserved. Full
checkpoint resumption follows the state-lock prerequisite below; authorized public push and remote CI remain pending.
Preserve the stopped failed databases and startup diagnostics under
.11.2. Qualification categories remain unchanged. Evidence:
docs/tasks/artifacts/signoff_review/identity-fixture-cleanup.md and
docs/tasks/artifacts/signoff_review/fixture-cleanup-plan-check.md and
docs/tasks/artifacts/signoff_review/node-fixture-cleanup.md and
docs/tasks/artifacts/signoff_review/participant-removal-authority.md and
docs/tasks/artifacts/signoff_review/cli-removal-delegation.md and
docs/tasks/artifacts/signoff_review/retention-fixture-clock.md and
docs/tasks/artifacts/signoff_review/partial-fixture-cleanup.md and
docs/tasks/artifacts/signoff_review/fixture-plan-coverage.md.

The resumed source-8d1504d checkpoint passes eight gates, then workspace testing
fails one initial state-writer lock acquisition (eleven other writer tests pass).
PostgreSQL/demo never start. Diagnosis .11.4.3.1.2.11.1 proves close-only lock
retention across actual CLI success/error/cancellation when a forked child retains
the descriptor; all three no-child controls and later child-exit acquisitions
succeed. All 341 source hashes match and fifty recorded groups are absent. The
original holder was not captured; its unchanged reruns pass. Explicit-release
repair/permanent controls .2.11.2 precede checkpoint resumption. Qualification
categories remain unchanged. Evidence:
docs/tasks/artifacts/signoff_review/state-writer-lock-lifetime.md.

Inherited state-lock release is now repaired under .11.4.3.1.2.11.2: the private
guard explicitly unlocks before File close, including post-acquisition failures.
A permanent five-path child control fails on unchanged production and passes after
repair; all 32 selected CLI tests, final twelve-writer rerun, six raw-fork actual-API
scenarios and strict CLI lint/format pass. Two test probes release explicitly too.
Original assertions and 339 other non-Markdown sources remain unchanged. Broader
inherited-descriptor abrupt-owner-death qualification stays owned by the existing
restart leaf. Full checkpoint/public push/remote CI remain pending, with all
qualification categories unchanged. Evidence:
docs/tasks/artifacts/signoff_review/state-writer-lock-release.md.

The ec8df08 checkpoint passes eight gates and the lock controls, then stops at two
PDF tests; PostgreSQL/demo remain unstarted. An unchanged-binary wrong-ZIP result
and three exact-helper native-clock collisions establish unsafe fixture ownership.
Exclusive unit/stdio inputs are repaired under .11.4.3.1.2.12; all twelve selected
tests, strict lint/format and three independent locality cases pass. Original PDF paths
were not captured. The analogous production R2 path now has immediate next repair
owner .7.3.3 before the full checkpoint; Git scratch and remaining fixture names
have separate concrete owners. Qualification categories remain unchanged. Evidence:
docs/tasks/artifacts/signoff_review/extraction-fixture-ownership.md.


Production-boundary diagnosis .7.3.3.1 now reproduces eight wrong-owner extraction
responses through the exact input acquisition span and unchanged worker; two
own-input controls pass. No HTTP/database reproduction or production fix is
claimed. Direct-worker completion .7.3.3.2 and exclusive input/digest integration
.7.3.3.3 precede the full checkpoint; larger transport/retention limits stay .7.3.4.
Qualification categories remain unchanged. Evidence:
docs/tasks/artifacts/signoff_review/extraction-input-boundary.md.


Direct-worker completion .7.3.3.2 is repaired and qualified under REPAIR-0060.
The permanent controls first reproduced the defect on the unchanged spawner:
`7 passed; 1 failed`, with an early request failure leaving pid 39486 in the
process table after the spawner returned. Every exit path now passes through one
bounded stop and reap, and every return reports never-started, consumed or
explicitly unconfirmed completion; a failed stop request is recorded rather than
read as a termination. Sixteen process/evidence controls, six spawner controls
(four synthetic injections), 81 server library tests, twelve adjacent extractor
tests, strict server lint and workspace format pass; api.rs is byte-identical.
Exclusive same-volume inputs and digest-bound cleanup remain .7.3.3.3; pipes,
descendants and aggregate retained storage remain .7.3.4. No HTTP, database,
full-CI or push claim follows. Qualification categories remain unchanged.
Evidence: docs/tasks/artifacts/signoff_review/extraction-worker-completion.md.


The 2026-09-11 book read found the superseded private-visibility instruction
still live in the book introduction and the governance charter after two hand-run
corrections, and the book roadmap page naming a frontier seven committed leaves
stale. REPAIR-0061 corrects both statements against the unchanged director
ruling, removes the duplicated frontier in favour of the per-leaf maintained
qualification page, and registers two self-tested checks — VISIBILITY-POLICY and
BOOK-FRONTIER — each falsified by reintroducing the exact defect it exists for.
Documentation and enforcement only: no visibility, remote, production or
qualification-category change. Decision:
docs/decisions/2026-09-11_mechanized-document-invariants.md.

Exclusive extraction input ownership .7.3.3.3.1 is complete under REPAIR-0063.
One private 0600 file per request is created under a runtime-discovered
repository root with checked owned parents; occupied candidates are skipped
whole and removal requires both a finished reader and an unchanged (device,
inode, one link) identity, retaining anything else with its relative path named.
Eleven controls pass — 32 simultaneous creators hold 32 distinct documents, and
the real worker describes an owned input by its own digest. 90 server library
tests, 16 completion controls, 12 adjacent extractor tests, strict server lint
and format pass. One control was itself racy on ambient worker selection; a
widening probe and its own retained input identify it, and the serialized
controls pass 40 repeated runs. api.rs is unchanged and still carries the
superseded span until .7.3.3.3.2. Qualification categories are unchanged.
Evidence: docs/tasks/artifacts/signoff_review/extraction-owned-input.md.

The R2 wiring .7.3.3.3.2 is complete under REPAIR-0064, closing the production
input boundary .7.3.3. The handler's extraction leg is one bound call: the
acquired bytes become an owned private input, and a response whose parent digest
is not that input's digest is refused as `extraction_source_mismatch` before any
snapshot, derivation or receipt write. Seven integration controls pass, including
the mismatch refusal naming both digests and eight concurrent callers each
receiving their own document. 90 server library tests, 16 completion controls,
12 adjacent extractor tests, strict server lint and format pass, and the live
adjacent profiles suite passes 31 tests with its cluster stopped and removed.
One gap is explicit with its census: no live test drives a SUCCESSFUL R2
acquisition through to a snapshot and derivation, because the live R2 test
refuses at the loopback destination gate. Concrete owner: .7.3.3.4.
Qualification categories are unchanged; no full-CI or push claim follows.

That join is closed under .7.3.3.4.1 (REPAIR-0095). `ApiState::with_acquisition`
lets a deployment supply the R0 fetcher its acquisition legs use, while `new`,
`with_gate`, `api_router` and `api_router_gated` all keep building the shipped
https-only public-destination policy. A live control serves one Atom document
from a local origin and resolves ONE reference through three deployments that
differ only in that fetcher, so each production gate is measured on its own: the
shipped state refuses `scheme_not_allowed`, the shipped destination policy
refuses `destination_refused` naming `loopback`, and a loopback-admitting policy
acquires. The evidence is then read back and asserted against the served bytes —
the snapshot's `raw_digest` and `byte_length`, the resolver id and locator, and
the derivation rows' content and digests — with the refused reference asserted to
hold zero snapshots. The profiles suite passes 32 of 32 live and its cluster was
stopped and removed; 97 server library tests, 16 completion controls, seven
owned-input controls, strict all-target server lint and format pass. Three
injections, each reverted and re-run green, prove the control goes red: one extra
served byte (which left the chunk digests identical, so the raw-byte leg is the
one that caught it), a discarded supplied fetcher, and a derivation whose content
is not the worker's chunk.

That control also MEASURED a finding rather than inferring it: the R2 pack
advertises five media types its own acquisition leg refuses. The R0 sniff accepts
a declared content type only when it is `text/html`, `application/xhtml+xml` or
`text/*`, so the identical feed succeeds served as `text/xml` and is refused
`media_type_refused` served as `application/atom+xml`. A caller is therefore
ranked onto a resolver that cannot acquire its document and receives an
acquisition refusal instead of the explicit `resource_unresolvable_now` the §12.2
contract reserves for "no eligible resolver". The per-format census and the
repair decision are owned by .7.3.3.5. Neither the registry row nor the sniff
changes before that census exists.

The census is complete under .7.3.3.5.1 (REPAIR-0097), with NO production
behaviour change, and it changed the finding's shape. Two mechanical controls in
the fetcher's own module measure the predicate: a DECLARED content type is
accepted only from `text/html`, `application/xhtml+xml` and any `text/*` — three
arms, enumerated in both directions — and all five advertised types are refused.
UNTYPED, the verdict is a property of the bytes rather than of the format: an
all-printable body is accepted whatever format it belongs to, and the same body
with one non-text byte is refused; ZIP and tar cannot reach that branch by
construction. So "the five advertised formats are unacquirable" is true but the
wrong SHAPE — the leg's rule is not about formats, and no subset of the
advertisement satisfies it. Narrowing the registry row is rejected on that
measured ground; the accepted repair is that the acquisition leg admits the
RANKED resolver's own advertised media types, with the destination policy,
scheme list, byte ceiling, ratio brake, redirect policy and time ceiling all
explicitly outside the change. The controls pass on unchanged production, as a
census must, and were falsified by adding `application/pdf` to the accept set.
The census measures the predicate, not real files of each format; a given real
PDF's untyped verdict depends on that PDF and is not claimed. Decision:
docs/decisions/2026-09-12_r2-acquisition-accept-set.md.

The repair ships under .7.3.3.5.2 (REPAIR-0098). `sniff_kind` takes the ranked
pack's advertised types and returns the additive `SniffedKind::DeclaredType` for
a declared type in that set; `Fetcher::fetch_admitting` supplies it while
`fetch`, `fetch_head` and `fetch_authenticated` pass an empty slice, so R0's
shipped accept set is byte-for-byte unchanged; and
`resolvers::advertised_media_types` reads the row's own media_types per
resolution, returning an EMPTY set for a missing or malformed row so a bad
advertisement cannot widen a gate. Bound census: `git grep -n "fetch_admitting"
-- 'crates/**/*.rs'` returns exactly one call site, the R2 arm; the R0 arm, the
R5 authenticated arm and the R3 preflight pass no admitted set, and no
destination policy, scheme list, SSRF control or ceiling changed. The profiles
suite passes 33 of 33 live with its cluster removed, including the unchanged R0
and R1 resolver controls; 99 library tests, 23 extraction controls, strict
all-target server lint and format pass, and a whole-workspace `cargo check
--all-targets` returns 0. Three injections, each reverted and re-run green: an
admitted set that is not the ranked row's fails the acquisition, an empty set at
the call site fails it identically, and adding ONE unadvertised type makes the
unadvertised document acquire and fails the negative control — the decision's
bound, proved live. One measured fact is deliberately flipped: .7.3.3.4.1's
recorded `media_type_refused` for `application/atom+xml` was the defect, and it
stays recorded as the evidence the repair rests on.

The sibling join .7.3.3.4.2 is closed under REPAIR-0096, with no production
change: the gap was coverage. The seven mismatch controls .7.3.3.3.2 added all
call extract_acquired_bytes directly and never reach a database, so they prove
the refusal is RAISED and nothing proved the handler HONOURS it. A dishonest
worker injected through the existing R2_WORKER_BIN override now returns a
well-formed reply describing bytes the request never supplied; the handler
refuses it `extraction_source_mismatch` naming both digests, and the control
asserts the ABSENCE three ways — zero snapshots for the exact reference, zero
derivations joined to it through parent_snapshot_id, and the whole store's
snapshot and derivation counts unchanged across the request. The profiles suite
passes 33 of 33 live with its cluster stopped and removed; strict all-target
server lint and format pass. Two injections, each reverted and re-run green:
removing the digest binding makes the handler accept the foreign document, and
**writing a snapshot before reporting the same refusal leaves the
extraction_source_mismatch assertion PASSING while the count fails with left: 1**
— the evidence that this control measures the absence rather than the error
kind. Production source is byte-identical to REPAIR-0095 afterwards. The parent
.7.3.3.4 is complete. Evidence:
docs/tasks/artifacts/signoff_review/r2-acquisition-join.md.

The source-165cb3a full checkpoint STOPPED at its fourth command
(`04-pg-demo rc=101`, 3,137s): 36 of 40 suites started, 35 passed with 259 tests,
and `site_authority` returned `9 passed; 1 failed`. Four suites, the
demonstration and gates five to eight never ran. REPAIR-0065 root-causes it to
`migration_upgrade` recreating `public` with `DROP SCHEMA … CASCADE; CREATE
SCHEMA public`, which drops the PUBLIC `USAGE` grant a fresh database ships, so
every later non-owner role could not resolve a qualified name. The fixture now
restores the owner and the grant through one helper, and the site privilege
probe resolves the audit table by catalogue OID so an unprivileged caller is
refused `OperatorRequired` (403) rather than `Error::Sql` (500). This was a
misclassification, not an escalation: the forensic copy shows the outsider never
held operator membership. The reproduced sequence and the affected family of six
suites now pass; the new control is falsified against the unchanged query.
The full checkpoint has NOT passed and no push claim follows.
`02-check` took 3,922s with only 940s accounted; that gap is owned by
.11.4.3.1.2.15. Qualification categories are unchanged. Evidence:
docs/tasks/artifacts/signoff_review/site-operator-schema-usage.md.

The re-run checkpoint on source-5c8609e stops earlier, at `02-check` (rc=2,
2,608s): `pg_guard` panics creating a fixture directory named from the process
id and a clock reading that this host does not advance between concurrent
callers. REPAIR-0067 replaces the clock with a monotonic discriminator and skips
an occupied candidate; 0 failures in 60 parallel runs against 1 in 15 before.
The family census it triggered found two production instances with concrete
owners: evidence identifiers minted from the same shape, measured at about one
distinct value per twelve calls (.7.4.1), and four ambient Git scratch paths
built from the process id alone (.7.2.1). The full checkpoint has NOT passed.
Qualification categories are unchanged.

Evidence identity .7.4.1 is repaired under REPAIR-0068. Snapshot, derivation and
claim-assessment identifiers were minted from a clock and a process id, measured
at about one distinct value per twelve calls; all three now use one `evidence_id`
built on a v7 UUID, with controls measuring 400 concurrent and 1,000 rapid
sequential identifiers all distinct. Because the three columns are primary keys a
collision was always a refused insert, so no stored row can hold another's
identity and there is nothing to reconcile. The storage-failure misclassification
the diagnosis exposed — every fault reported as `ReferenceMissing` and mapped to
HTTP 400, with the R2 pipeline still reporting a successful acquisition — is
routed to .7.4.2. Qualification categories are unchanged.

**The full pre-push checkpoint PASSES at source 7233122** — the first complete
run in this project's recorded history. All eight commands return 0: build,
format/strict lint/workspace tests with the pinned browser, Python controls, the
full owned PostgreSQL collection with an explicit demonstration, thirteen
doctrines, pinned cargo-deny, pinned Gitleaks and the book. 40 of 40 registered
suites ran with 291 tests passed and 0 failed; the demonstration reports ALL
acceptance checks passed; both scanner receipts record scope `gate` with exit 0.
The pass was falsified before publication: no DATABASE_URL skips, 16 real browser
controls rather than an absent-browser early return, and the only ignored tests
are the env-gated live-provider dispatches. This satisfies the condition the
recorded policy places on the already-authorized push; remote CI has never run
and must be consumed after it. No external gate closes: G6/G7, name clearance and
the license decision remain open, historical phase closures remain under
corrective review, and .7.4.2, .7.2.1, .7.3.3.4, .11.4.3.1.2.15 and .11.5 remain
open. Evidence: docs/tasks/artifacts/signoff_review/checkpoint-7233122.md.

Evidence storage faults are now reported honestly under REPAIR-0071. Ten map_err
arms across the snapshot, derivation and claim modules collapsed every storage
failure into a caller error that the handlers rendered as HTTP 400; each enum now
carries a Storage variant preserving its SQLx source, the handlers use the
existing internal_with_log idiom, and the R2 pipeline records an
`evidence_unstored` acquisition error instead of discarding a failed snapshot
while reporting a successful acquisition. The control injects a storage fault,
restores the database before asserting, and requires 500 rather than 400 while a
genuinely absent reference stays 400; against the unrepaired source it fails
printing the defect verbatim. Live profiles/evaluation/cards, 92 library tests,
strict lint and format pass. The R2 leg is qualified at the handler boundary
only, because a live successful acquisition has no coverage (.7.3.3.4).

Remote CI ran for the first time on a626768: doctrines and supply-chain pass,
and the rust workflow FAILS at `codex_adapter_passes_the_conformance_suite`
("the lose trigger refused instead of dispatching"). That suite passes locally,
so it is environment-dependent and is exactly what remote execution exists to
find. It is the next repair; no remote-CI green claim is made.

The first remote CI failure is being diagnosed under .11.4.3.1.2.19. The
certification harness discarded the adapter's refusal reason at three arms, so
the CI log carried no cause at all; those arms now append it. The conformance
stubs also stopped naming their directory from the clock, on measured evidence
of their own. The remote cause remains UNPROVED and no remote-CI green claim is
made: the instrument was pushed so the next run names the cause rather than
having it guessed.

The first remote CI failure is root-caused and repaired under .11.4.3.1.2.19.
The instrument pushed as REPAIR-0072 made the next run name its cause: Linux
ETXTBSY, `execve` refusing a stub still open for writing while a parallel thread
forked to spawn. macOS does not enforce it, so every local run had passed.
REPAIR-0072's stub-naming change was NOT the cause and the evidence says so —
the failing path already carried that naming and the failure moved between
adapters. REPAIR-0073 writes one stub per adapter kind per process behind a
OnceLock, removing the race rather than retrying around it. All adapter targets
pass locally, which is explicitly not evidence about Linux; remote confirmation
is still required and no CI-green claim is made.

The conformance repair held on the runner and pg-tests SUCCEEDED remotely for
the first time — the full PostgreSQL collection with its demonstration, on a
Linux runner. The `check` job now fails in reasonbraid-browse: six tests report
browser_launch_failed, "browser exited before publishing a loopback endpoint",
with zero gated navigations observed. The pinned executable's version was
verified by the launcher and --no-sandbox/--headless are already passed, so
neither is the cause. REPAIR-0074 retains the worker's own stderr on failure so
the next run names it. The cause is UNPROVED and no repair of it is claimed.

The browse failure is repaired and remote-confirmed: relative TMPDIR keeps
Chrome's singleton socket inside the 108-byte limit and all browse controls pass
with render_succeeded true. The remaining known `check` failure was four
git::tests::* reporting AuthorMissing, whose cause is proved: gix resolves a
commit signature from git configuration and the fixtures borrowed the
developer's. REPAIR-0080 gives them their own identity. It is the first
remote-only failure reproduced LOCALLY — suppressing ambient git configuration
recreates the runner's condition — and that probe exposed a fourth commit site
the first pass missed. Six git tests pass both with ambient configuration
suppressed and with it present. Remote confirmation of the whole `check` job is
still required and no CI-green claim is made.

REPAIR-0080 is confirmed on the runner: the server library now reports 92
passed, 0 failed there, with the four AuthorMissing failures gone. The check job
still fails, on a different and newly exposed defect — a check-then-act in the
pg_guard fixture parent creation, which loses a race between parallel threads on
Linux and has never lost it on macOS. REPAIR-0082 owns the R1 acquisition
workspace: it is now exclusively created under .project-data/git on the
repository volume, proved by identity before removal, and released when the last
holder drops, which also closes a leak that ran on every SUCCESSFUL acquisition.
97 server lib tests, strict lint and the rendered book pass locally.

REPAIR-0083 repairs the check-then-act in the pg_guard fixture parent, the sole
instance of that shape in the workspace. It was never Linux-only: a probe lost
346 of 640 creations locally, and the suite reproduces the exact CI failure here
once the warm control directory is removed. Six consecutive cold-tree runs pass.
No remote-green claim is made until the runner confirms it.

REPAIR-0085 closes the §13 storage-locality family: 26 breaches repaired across
16 files and a STORAGE-LOCALITY gate registered so the policy is now checked
rather than merely stated. Remote CI is still red, on a DIFFERENT defect each
time rather than the same one recurring: the four AuthorMissing git tests are
confirmed fixed on the runner, the pg_guard race is fixed, and the current
failure is an intermittent ETXTBSY in the conformance stubs which PASSED in the
preceding run. No remote-green claim is made.

REMOTE CI IS GREEN. Run 34652116508 for c17841c passes rust (book, check,
pg-tests), doctrines and supply-chain, with 669 tests passed and 0 failed suites
in check. Verified by re-derivation from the API, falsified for hidden skips,
and durable at origin/main. Push cadence returns to ~300 commits per COMMIT.md.
That local-only navigation-deadline defect is now REPAIRED under .11.4.3.1.2.27
(REPAIR-0089), and it was a product defect rather than a flaky test. The browse
worker returned one error kind for two independent facts, so an unconfirmed
cleanup overwrote the render's own kind — eleven distinct kinds, not just the
budget — and left the caller with an operator's fact. A refusal now carries the
render's kind plus explicit cleanup_confirmed/cleanup_error fields; only a
SUCCESSFUL render under unconfirmed cleanup is still named for the cleanup. The
wire change is additive and the server-side spawner is unaffected. 25 browse
controls pass with the pinned browser; both strict lints and format pass; the
controls were falsified against the superseded expression. The underlying
escaped-writer condition remains real and unrepaired — it is now reported
honestly instead of overwriting a result. That defect is now FIXED and GATED under
.11.4.4 (REPAIR-0092): five leaf sections asserted two different statuses, which
is why .7.4.1 sat on the frontier seven commits after closing, and the census
that found them also found 29 headings at levels 7-11 -- which Markdown does not
treat as headings at all, so the tree's deepest leaves were invisible as
structure. Both are repaired without deleting a word and both are now enforced:
TASK-STATUS and HEADING-DEPTH, each fence-aware, each with a two-sided
self-test, each falsified against the unrepaired tree. The enforcer runs 15
checks.

The checkpoint's unexplained hour is now ACCOUNTED under .11.4.3.1.2.15
(REPAIR-0090), which unblocks .11.5's verification lanes. Of 02-check's 3,922
seconds, cargo reported 813 and the harnesses 127; the other 2,982 are macOS
first-execution validation of each newly written executable on the repository
volume at about 21.9 seconds apiece, fixed rather than size-proportional, cached
per file identity, against about 0.15 seconds on the boot volume. It is not
inherent to the gate: the same four commands on the Linux runner, from a cold
checkout, take 444 seconds with 18.7 (4.2%) unaccounted. The leaf's own leading
candidate, nine rustdoc doctest-harness builds, is at most 8% and is refuted.
Planning number: one more integration-test file costs about 22 seconds of every
future checkpoint. scripts/measure_check_phases.py makes this re-derivable by one
command. Two levers -- a macOS security setting and the repository's volume --
are now DECIDED and CLOSED under .11.4.3.1.2.28 (REPAIR-0093). The macOS
Developer Tools setting is rejected: it exempts the shell from validating
exactly the untrusted-content workers this project builds, and it cannot be
committed, so no other machine or runner would inherit it. The volume move is
rejected on measured capacity: the repository is 3.77 MiB but its build tree is
185 GB, against 249 GB free on the boot volume. The lever actually pulled was on
neither list -- the REMOTE run is now the authoritative pre-push gate, verified
from the workflow files as a superset in which two of the eight commands are
stricter remotely. Nothing is removed, weakened or skipped; only four cheap
local gates run before a push, and the full checkpoint becomes a deliberate
diagnostic. The named trade: gate latency is now bounded by the ~300-commit push
cadence, which remains the director's standing instruction and is unchanged.

Two defects introduced by this session's own work are now task-tree owned under
.11.4.5. The doctrine registry could contain shell expansion -- and the driver
EXECUTED it, on every commit and in CI -- which is fixed and falsified under
.11.4.5.1 (REPAIR-0094) by a self-guard that runs before the registry is
assigned. The second, a LOCKSTEP box claiming a document its commit did not
touch, is NOT fixed: its census is done and decisive (1 breach in 6 commits,
zero false positives, and the obvious blanket rule measured and rejected), and
it sits at frontier row 2 with its acceptance recorded.
