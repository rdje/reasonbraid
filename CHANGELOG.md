# CHANGELOG.md

## 2026-09-13 — A rotated identity is written where the next start looks for it (`SIGNOFF-REPAIR.4.2.9`)

- 🔴 **Confirmed and live, not latent.** The node rotates its workload certificate automatically when the leaf nears expiry, and the fresh identity lived only in memory. Nothing had ever rewritten `cert.der`/`key.der` after enrollment — the only writer takes an *enroll response* and runs once.
- ⭐ **The bound would have been wrong in both directions if it had been guessed.** A restart usually RECOVERS: the stale certificate loads, the rotation window is still open, and the next handshake rotates again. But a node that stays DOWN until that certificate expires — at most 300 s later on a 600 s leaf — cannot rotate its way out, because rotation requires a usable certificate. That node is permanently locked out and needs an operator-issued enrollment token: the same lockout class as an enrollment token that expires unused, reached through a different door.
- **Decision: the persistence belongs to the caller.** The channel does no file I/O anywhere and does not start now; it gains an optional sink, and `Node::open` — which already has the journal path — wires it to the exact two files the loader reads. ⭐ That pairing is the point: a control asserting only "a sink fires" would not catch a sink writing to the wrong place, so the control asserts the filenames.
- ⭐ **Persist first, then swap memory**, and the order is reasoned: a crash between them leaves the NEW identity on disk and the OLD in memory, which the next start corrects. The reverse would strand a node that had rotated and lost the record of it — this leaf's own defect, reintroduced by its fix. A control asserts the order by observing memory from inside the sink.
- A sink failure is reported and not fatal: the node already holds a working identity, and refusing to continue would turn a durability problem into an outage.
- Validation: falsified by removing the sink call — the superseded shape exactly — `29 passed; 3 failed`, precisely the three new controls, the node-level one reporting `cert.der was written: NotFound`. 76 node tests plus the live channel, replacement and work suites, rc=0. ⚠️ clippy refused twice for a complex type; answered with the type alias it asked for rather than an `allow`.

## 2026-09-13 — The fencing token and its epoch become one fact (`SIGNOFF-REPAIR.4.2.8`)

- 🔴 **Reproduced with a driven interleaving: `242 torn of 40,000` reads**, the first carrying a token from generation 444 with the epoch of 455. The control installs self-describing generations — generation `n` is token `fnc_n` with epoch `n` — so a torn pair checks against itself with no bookkeeping.
- ⭐ **Why it looked safe: the WRITE was atomic.** `handshake` took both locks in one scope, so two writers could never interleave. But all four fenced request paths READ the pair through two separate acquisitions, and a reader straddles a complete write. The asymmetry is the whole defect, and it is invisible if you only ask whether the writes are correct.
- ⚠️ **The bound, stated before the work and unchanged by it: an availability defect, not a fencing bypass.** The server refuses a mismatched pair because no lease row matches both, so the cost was a spurious `401` and a reconnect. It is repaired because a request that cannot possibly succeed should not be constructible.
- ⭐ And the measurement explains why it never surfaced: **0.6 % of reads under maximal contention**, against a real node that reads the pair a few times a second and handshakes on reconnect. Rare enough to look like a network blip and be retried away — the class of defect that survives.
- Fix: one field, `lease: Arc<Mutex<Option<Lease>>>`, read in one acquisition. The mixed state is now **unrepresentable** rather than merely unlikely. A "take both locks" helper was rejected (nothing stops the next caller taking one) and so was accepting the window (the fix costs one struct and removes the question).
- 🔴 **The first falsification was the wrong one, and that is recorded rather than hidden.** It split the WRITE — which the superseded code never did — and failed more loudly, `4193 torn`. Redone against the actual superseded mechanism, the split READ, it gives `242 torn`. A neutralization that exaggerates the defect is not evidence for the repair, and the louder number is the tell.
- Validation: node crate 72 tests across 8 targets, plus the live channel suites (37 + 8 + 7 + 1), rc=0 — every handshake, heartbeat, ack, poll and events control unchanged. clippy `-D warnings` rc=0. No wire field, route or documented behaviour changed.

## 2026-09-13 — Reconcile the fifteen routed records, and find six clauses the split dropped (`SIGNOFF-REPAIR.4.2`)

- ⭐ **The parent's citation claim is measured for the first time**: all **15** records routing to this family are cited by it or a child — **15 cited, 0 uncited** — against `.4.1`'s **36 uncited**, the leaf whose dropped clause started the whole reconciliation question. Reading the routed records is now visible in an instrument rather than asserted as diligence.
- 🔴 **But citation is not accounting, and a clause-by-clause pass found SIX clauses this family's split did not carry** — in the very leaf that had avoided the citation form of the same defect. That is why the unit was reframed as *a clause with an owner* rather than *a record with a citation*: a leaf can read every record it is sent and still drop a sentence inside one.
- All six are now owned. Two were confirmed at the source while reconciling: the node reads its fencing token and lease epoch under **two separate mutexes**, so a handshake landing between the reads yields a request carrying a token from one generation and an epoch from another; and `install_identity` writes only an in-memory value, so whether a rotation survives a restart is an open question rather than a known-good.
- The other four are recorded as the REVIEWER's claims under the reproduce-before-writing-up rule, not as defects: the wake gate's coverage at positive concurrency, and three clauses about what a replacement enrollment leaves behind — the old `host_id`, unclosed incarnations, and an unfenced old lease. The replacement three are grouped because they are one question, and flagged against the earlier decision that deliberately preserves a revoked node's tail so its work reaches its replacement.
- Every remaining clause of all 15 records is dispositioned: to a closed child, to a named other leaf, or to the plain-HTTP transport bound the roadmap already records.
- ⛔ The tree's pending count ROSE by four, deliberately. A reconciliation that finds work makes the tree larger, and hiding that would be the defect. Documentation only; no code touched.

## 2026-09-13 — Publish the codes the product emits, and gate them (`SIGNOFF-REPAIR.11.7`)

- ⛔ **The leaf's own numbers were wrong, and it is the project's own lesson turned on the leaf: the emitted set is 18, not 19, and the unregistered set is 9, not 10.** `unknown` is a CLIENT-side sentinel built when a response body will not parse; it never travels from the server. A search's hits were published as a defect count without classifying them — in a leaf whose subject is a published set nobody re-derived.
- ⛔ **Half the finding is refuted by the code's own documentation.** The registry says, above the enum, that it is "the *complete* §9.8 list, not a Phase-0 subset … codes the demo never emits are still part of the registry". So the 11 unemitted codes are deliberate and already explained: **all 11 kept.**
- ⛔ **Most of the other half too.** The next sentence: codes beyond the list "are handled by `ReasonCode::Unknown`", which preserves the string verbatim. The 9 unregistered codes reach a client intact — the forward-compatibility path working as designed, not drift.
- 🔴 **The real gap was a different one: the book had no error page at all**, so nothing published which codes the product actually emits. A client author reads §9.8, sees 20, and learns about `quota_unconfigured` only by receiving one.
- **Delivered:** `docs/book/src/errors.md` — all 18 with HTTP status, registry status and meaning, derived from the sources; the 11 unemitted with the reason they are kept; and four rules for a client, including that `commit_outcome_unconfirmed` means *unknown*, not *failed*.
- ⭐ **A gate IS registered here — `REASON-CODE-DOC` — and the contrast with the previous commit is the point.** It fires on zero breaches today and would have fired on all nine: it catches the next drift rather than presenting a backlog. `SIGNOFF-REPAIR.11.9`'s citation gate would have fired on 114 of 131 and was rejected on exactly this test. Same question, measured, answered oppositely.
- ⛔ **The registry is not extended and no emitter is changed**: extending it would break its stated contract of mirroring §9.8, which lives in a roadmap frozen at v0.4.1; changing emitters is a wire change that would collapse distinctions the product really makes. Whether §9.8 gains the nine at v0.5.0 is routed to `.11.7.1` with the measurement, because the freeze says a version bump must cite exactly this kind of evidence.
- Validation: `--check` rc=0 (`all 18 emitted codes are documented`); `--self-test` 14 controls pass; falsified by introducing an undocumented code at a real emission site, which the gate names along with its file. No product code changed.

## 2026-09-13 — An agent role may issue and revoke, and the documentation now says so (`SIGNOFF-REPAIR.4.1.4`)

- 🔴 **Driven at the live route rather than read: an agent role granted `tenant_admin` issues a node enrollment token, `200`**, and the ledger records the role as the grant subject. Two documentation sites said "an authorized **human**"; the code never checked.
- **Decision: narrow the claim, not the code**, on four independent lines of evidence. §16.2 says nothing about who may issue; §16.4 specifies authorization over typed actions and resources; §16.3 states outright that "a human, service, or agent may delegate a strict subset of its own authority". And across the server's 117 `resolve_principal` call sites, authorization never depends on the principal's KIND — every production branch on it selects which identity TABLE to read.
- ⭐ **The strongest evidence was an oracle nobody built for this question.** Adding the human-kind check the old sentence implied fails THREE controls, two of which predate the leaf: they already drive an agent role at this route and require it to be adjudicated by the grant. One fails for the reason that matters most — a kind check refuses `401` *before* the authorization that writes the denial record, destroying the audit evidence the denial path exists to produce.
- ⚠️ **Scope widened by the leaf's own census**: `git grep "authorized human"` found six sites, not the two it was opened on — including `POST /v1/nodes/revoke`, the same sentence with the same gate. All six dispositioned; two fixture helpers now say a human is what THAT fixture uses rather than what the route requires.
- ⭐ **The governance consequence is now stated where a reader meets it**: the book says plainly that granting `tenant_admin` to an agent lets it extend the node population and revoke nodes, and that a deployment which does not want that must WITHHOLD the grant — narrowing the route would be a change to the grant model, not a check on one endpoint.
- Validation: falsified by adding the kind check (`15 passed; 3 failed`, naming the governance change); restored `18 passed; 0 failed`. **5 suites / 91 tests, 0 failed**; clippy `-D warnings` rc=0; `mdbook build` ok. No product behaviour changed. Recorded as `docs/decisions/2026-09-13_issuance-is-a-grant-not-a-kind-of-principal.md`.

## 2026-09-13 — The routed records censused, and the gate the leaf proposed rejected (`SIGNOFF-REPAIR.11.9`)

- ⛔ **The leaf's own census-owed line had a wrong number in it: 131 review records, not 53.** 53 is the artifact DIRECTORY's file count. `MEMORY.md` has carried 131 correctly since the first leaf, so two live documents disagreed and nothing compared them — which is the failure this leaf exists to catch, committed by the leaf.
- **The census:** 131 records, 614 record→candidate-leaf routings, 301 of them to a leaf that has since been SPLIT, 16 cited, 285 uncited. At record granularity — the level the claim is about — **17 of 131 cited, 114 by none**.
- 🔴 **The gate is REJECTED on the measurement: it would fire on 114 of 131 records the day it was registered.** That is a backlog wearing a gate's clothes, and a gate people route around is a gate that lies.
- ⭐ **The census changed the QUESTION, not just the answer.** The fan-out is a median of 4–5 candidate leaves per record, so a reviewer naming five leaves wrote a suggestion list rather than five assignments. "A leaf must account for every record routed to it" is therefore unsound at the population level, and the biggest uncited counts are the broad family containers that were never going to cite a record each.
- 🔴 **Which reframes the original defect.** The failure was not that a leaf omitted a citation; it is that one record named THREE findings and only ONE found an owner anywhere. The reconcilable unit is a clause with an owner, not a record cited by a leaf — and "accounted for" includes deliberately declined, which no search can see.
- 🔴 **The instrument was wrong four times, each caught by a different control**: 70 of 131 (one record-id shape of two); a parent section swallowing its children, so no split leaf could ever read as uncited; 3 citations of 19 (heading-line matching instead of a line RANGE); and fifteen real citations still reading UNCITED, because the tree elides the filename in a run — `` `census-2.md:28`, `:49`, `:56` ``. The last was caught only by a control run against the real tree. One control was itself wrong before the code was.
- The one concrete obligation is discharged: the "no human restriction" clause now owns `.4.1.4`, measured at the source — `resolve_principal` returns `Human` **or** `Role`, nothing on the issuance path asks which, and two doc sites say "an authorized human". The 114-record backlog is routed to `.11.9.1` with its measured size rather than absorbed or dropped.
- Validation: `--self-test` -> 21 controls pass, rc=0; gate green. ⚠️ It first refused the commit because a stray `python3` without `-B` wrote `scripts/__pycache__/`, which `git add -A` staged and the SELF-TEST gate then tried to run as a shell script; ignored now, with the reason. No product code touched.

## 2026-09-13 — The MEMORY warnings censused, and the worry refuted (`SIGNOFF-REPAIR.11.4.2.1`)

- ⛔ **The measurement refutes the worry the leaf was opened on.** `MEMORY.md` holds **26** standing warnings and **zero** exist only there — every one is also recorded in a durable layer. The rule that worry implies, "every MEMORY warning must first exist in a durable layer", would flag nothing, ever.
- 🔴 **The instrument was wrong twice before it was right, and the sequence is the evidence.** 31 (split at every marker — `🔴 ONE HOP DEEP: ⛔ do not fix it` counted as two); then 21 (split only at a sentence terminator — but this project writes `**… clamp.**`, so the terminator sits inside the markup and three pairs of distinct warnings merged); then 26. ⚠️ Its own `--self-test` passed throughout, because the fixtures were written in the same idiom as the bug. What caught the under-count was reading the output against the file it measured.
- ⛔ **Then the tool's own verdict was a false claim.** It printed `ORPHAN (only in MEMORY.md): 13`; hand-classifying all 13 found **13 of 13 recorded**. A search's population published as a defect count — the project's own lesson turned on its author. The verdict is renamed `UNCITED` and the output now states it is a population to classify.
- 🔴 **What the census did find is narrower and real: the pointer is missing, not the record.** 13 of 26 warnings name no leaf, so evicting one costs the next reader the path back rather than the fact.
- ⛔ **Not mechanized, and the census is why.** "A standing warning must name its leaf" would flag three legitimate ones today — the derived frontier note, the `docs/knowledge/` navigational pointer, and the `project_env.py` environment rule. The obvious rule destroys accurate content, for the second time in this tree's history.
- ⭐ What ships instead is `scripts/census_memory_warnings.py`, tracked with a 16-control `--self-test` and registered in `TOOLBOX.md`, to be run at the moment of the decision: prefer evicting a warning whose line names its leaf. That turns "whatever the author judges least costly" into something derived.
- Pressure, measured: seven recorded cap crossings, four of them in this session — one per commit. ⛔ The cap was not raised; containment stays with `.11.4.2`.
- Validation: `--self-test` -> 16 controls pass, rc=0; `check_doctrines.sh` green with the instrument inside the SELF-TEST gate's population (21, up from 20). No product code touched.

## 2026-09-13 — The secret store's claim is narrowed to the one read it routes (`SIGNOFF-REPAIR.4.2.5`)

- ⛔ **The census refuted the leaf's own opening sentence, and the refutation is recorded rather than edited into agreement.** "Node keys are read directly from their tables" is false in production: there are exactly two `SELECT`s of key material in the workspace, one of which IS the store, and the other a test fixture. The server performs ONE read of key material and it is routed.
- 🔴 **But the census found what the record did not: the seam is READ-ONLY.** The CA bootstrap reads through the store and then `INSERT`s a fresh CA into `server_ca` directly, before reading back through the store with an `expect`. That holds only while both ends are the same database — so with any external profile, first boot writes in one place, reads from another, and aborts the server.
- ⭐ Which makes the most confident published sentence the false one: *"the external store arrives as a configuration change, not a code migration"* is exactly backwards.
- Three published sentences, all dispositioned: the key-reads claim is TRUE over a smaller surface than it reads as; "every secret read" is FALSE (the enrollment token is read straight from its table); "a configuration change" is FALSE. The book carried no copy — measured, not assumed.
- **Decision: narrow the claim, not widen the code**, and the reason is falsifiability rather than effort. A write path for stores that do not exist, behind a registry with one profile, is speculative generality no control could test.
- ⭐ **The narrowing is a TRIPWIRE, not a warning comment** — a test pins the profile count with the reason in its assertion message, so a second profile cannot be added without confronting the unrouted write. A prose note asks to be remembered; a test makes it impossible to skip.
- ⛔ No runtime behaviour changed: the `expect` is deliberately not converted to a typed error, because with one profile it is unreachable and the change would be untestable.
- Validation: falsified by declaring a second profile — `3 passed; 1 failed`, only this control, printing the message the next implementer needs. `103 passed; 0 failed` for the crate's lib tests; clippy `-D warnings` rc=0.

## 2026-09-13 — A hex decoder returns its typed error instead of aborting the node (`SIGNOFF-REPAIR.4.2.7`)

- 🔴 **Reproduced with a driven decode**: `end byte index 2 is not a char boundary; it is inside 'é' (bytes 1..3 of string)` — a panic from a function whose signature returns `Result`.
- ⭐ **The condition is not "non-ASCII", and getting that exact is what made the control honest.** A multi-byte character starting at an even offset always decoded to a clean `Err`. The panic needs an EVEN byte length — which clears the odd-length guard — whose chunk boundary falls INSIDE a character. The control asserts both structural facts about its input before decoding, so it cannot quietly stop testing the case it names.
- ⭐ **The call site already handled the error the decoder never produced**: `from_hex(&parsed.cert_der).map_err(ChannelError::Malformed)?`. The typed refusal was written one line from the abort. This was never missing error handling — it was a `Result` the body did not honour.
- 🔴 **The census found the real story, and the grep found it rather than the reasoning.** Seven hand-rolled hex decoders exist; **six** use the panicking byte-index form and **one** is safe — and the safe one is the only one on the untrusted wire path, where someone was worried. Every copy nobody worried about kept the naive form.
- ⛔ **The server's copy is DELETED, not repaired.** It was `pub` with no caller; making it private turned that into `error: function from_hex is never used` under `-D warnings` — an independent oracle agreeing with the grep. A repaired-but-dead decoder is a path no control can reach, and a dead, well-named public one beside a private correct one is what the next caller reaches for.
- ⛔ **The four TEST-local copies are measured and deliberately unchanged**, with the reason recorded so a later census does not re-raise them: each decodes hex the server produced in the same test, so a panic there is a test failure rather than a product defect.
- ⚠️ Trust boundary stated rather than inflated: these strings arrive in the server's rotate response, so reaching this needs a malicious or faulty control plane, not a network attacker. The cost is the whole node process, not one request.
- Validation: falsified by reverting only the indexing while keeping the length guard — both new controls fail, the third stays green, `24 passed; 2 failed`. **Node 70 tests + rotation/enrollment 55 tests, 0 failed**; clippy `-D warnings` rc=0. Promoted to `docs/knowledge/a-signature-is-a-promise-the-body-must-keep.md` as the second instance of the shape.

## 2026-09-13 — A fenced session's acknowledgement cannot mark another session's delivery (`SIGNOFF-REPAIR.4.2.4`)

- 🔴 **Reproduced against the live route.** With the lease row held, the unrepaired `ack` completed without ever asking about the lease it was writing under; the session was then fenced, and the ack still marked **3 rows acknowledged** — rows that now belong to whoever holds the lease.
- 🔴 **The severity is the PRUNE, and it was measured rather than asserted.** Nine sites touch `acknowledged_at`; classifying all nine finds exactly one MUTATING consumer — the retention prune's `DELETE … AND acknowledged_at IS NOT NULL AND acknowledged_at <= cutoff`. So a stale ack does not merely record a wrong fact: it makes work the live session is still holding eligible for deletion, leaving the node's ledger and the server's permanently disagreed, which is precisely what the channel's duplicate-safety contract exists to prevent.
- Fix: `ack` becomes ONE transaction that re-verifies fencing with the lease row locked — `events`' existing shape, not a second one — and commits the acknowledgement with it.
- ⛔ **`poll` is decided, not assumed, and the decision is NO.** All three of its statements are reads, so a fenced session that receives a stale tail leaves no trace — the rows stay in the inbox and replay to whoever holds the lease. Re-verifying in a transaction there would take a row lock on the channel's most frequent call and buy no invariant.
- ⛔ **No tenant guard on `ack`, deliberately.** `events` takes one because it applies domain effects; an acknowledgement touches only this node's inbox, and adding a guard would order acknowledgements against revocation and cut the tail `.4.1.3.1` chose to preserve.
- ⭐ **The in-transaction cursor read ships with its limit named rather than a control that could only be green.** No concurrency fixture discriminates it — the bound only grows, so a stale read is permissive. Its real justification is mechanical: calling the pool method inside an open transaction would take a second connection per acknowledgement.
- Validation: falsified with the transaction KEPT and only the re-verification removed, so the neutralization isolates the right part — arm 1 fails `left: 3, right: 0`. Arm 2 was then run under the same neutralization with arm 1 softened and fails independently with `nothing ever blocked on FOR UPDATE`. **8 suites / 117 tests, 0 failed**; clippy `-D warnings` rc=0; `mdbook build` ok.

## 2026-09-13 — A lapsed lease cannot be renewed by a heartbeat already in flight (`SIGNOFF-REPAIR.4.2.3`)

- 🔴 **Reproduced against the live route, and the unrepaired product revived a dead lease.** With the renewal stalled at its `UPDATE` — its admission check having already passed — the lease reached its expiry and committed; the renewal then resumed, was **ALLOWED**, answered `200` with a fresh expiry, and the store held one live lease for a node whose session was over. Since presence, delivery and every fenced write are functions of that clock, the node came back from the dead.
- ⚠️ **The pause is artificial; the ordering is not — and the fixture proves the ordering rather than assuming it.** The lease row is held by a control transaction: `verify_fencing` is a plain `SELECT` and walks straight through it, the renewal's `UPDATE` cannot, and the control observes it WAITING in `pg_stat_activity` before the lapse is committed. In production the same gap is scheduler latency, a saturated pool or a slow statement.
- Fix, two parts: the expiry joins the epoch and the credential INSIDE the write (`AND lease_expires_at > now()`), and the refusal classifier gains the third reason a renewal can now match no row.
- ⭐ **The refusal's WORDING is a repair part, not tidying, because the wire answer must not depend on the timing.** A heartbeat arriving just after the expiry is refused as `lease_expired`; before this, the same request whose write was overtaken by the expiry was told its fencing token was refused — blaming a credential that is perfectly good. Both facts are now read in one statement so the answer comes from one snapshot.
- **The clock was derived, not inherited.** The predicate reads the DATABASE clock: one statement must not mix two clocks, and `node_presence.online` — the product's published answer to "is this node online?" — reads the same column the same way. The write and the fact an operator sees now agree by construction, so a renewal can never succeed for a node the API simultaneously reports `offline`.
- ⚠️ **The residual is named rather than bundled:** `lease_expires_at` is still WRITTEN from the server process clock and read from the database clock, so the published 60 s TTL is nominal and carries the skew. That predates this repair and is not widened by it — the renewal now merely agrees with the presence view that already had it — and it is tracked as `.4.2.3.1` with the book's honest-limits list saying so.
- ⛔ **The pre-existing expiry control does not discriminate this repair, which is why the race shipped.** "A heartbeat cannot resurrect an expired lease" passes against the unrepaired code, because it only exercises the sequential order where the admission check refuses first. It is a regression control here and is labelled as one.
- Validation: falsified twice, each part alone — the expiry conjunct removed gives `ALLOWED` and `left: 1, right: 0`; the classifier arm removed gives `the fencing token was refused`; `35 passed; 1 failed` both times, only this control. **7 suites / 79 tests, 0 failed**; clippy `-D warnings` rc=0; `mdbook build` ok.

## 2026-09-13 — A captured proof cannot be replayed (`SIGNOFF-REPAIR.4.2.2`)

- 🔴 **Reproduced byte for byte, with the node crate's own public helpers so the bytes are the real canonicalization.** The unrepaired product answered `200` to a replayed handshake — the replay took a **new lease**, and **the legitimate node's next heartbeat answered `401`**, fenced out of its own session by a replay of its own bytes. A replayed rotation answered `200` and returned a **second private key** for the same identity; the node finished holding three live certificates.
- **THE DECISION: a per-request nonce, consumed once — not a server challenge.** It costs no extra round trip where a challenge costs one on every handshake and rotation; it needs no clock, and a timestamp-and-skew design would put a clock dependency on the authentication path of a repository that has just repaired three separate cross-clock defects; and it is already this codebase's shape, since enrollment tokens carry a nonce echoed at use.
- ⛔ **Consuming the CERTIFICATE instead was considered and rejected.** A node whose rotate response was lost retries with the same certificate, and that retry is indistinguishable from a replay — the rule would have broken the §17.4 recovery path. A retry carries a fresh nonce, so this design serves it, and a control asserts exactly that: it refuses replays, not reconnects.
- ⚠️ The nonce is consumed only **after** the signature verifies, so an unauthenticated caller can never burn a value a legitimate node was about to use.
- 🔴 **A deadlock introduced in the first draft, found by the suite HANGING rather than failing.** Giving the nonce table a foreign key to `nodes` made the rotation wait on itself: it holds `nodes … FOR UPDATE`, and the key check on the nonce insert needs a KEY SHARE lock on that same row. ⭐ Worse than the deadlock was what the key implied when it did *not* deadlock — every handshake would have queued behind any in-flight revocation of that node, a coupling nothing asked for. The key is dropped and the consume moved inside the rotation's own transaction, so a failed rotation burns no nonce.
- ⚠️ **A hanging suite is a third failure mode beside red and green**, and neither the test output nor a timeout says which. It was diagnosed by asking PostgreSQL — the waiting `INSERT` and the `idle in transaction` beside it were visible in the process table — rather than by re-reading the code.
- ⚠️ **Wire contract narrowed deliberately:** `nonce` is required on both requests, so one omitting it is `422` rather than `401` — the same treatment `cert_der` and `proof_signature` already get. Server and node ship together.
- Validation: falsified by recording the nonce without enforcing it — the replay is handed a fresh fencing token and `lease_epoch: 2`, and only that control fails. **Broad PostgreSQL run: 44 suites, 374 tests, 0 failed**; `mdbook build` ok.

## 2026-09-13 — Every rung of the proof ladder gets its own negative (`SIGNOFF-REPAIR.4.2.6`)

- **The measured gap.** The control named "a handshake without a valid certificate proof is refused" presents `cert_der: "00"`. That decodes, fails the chain check, and stops — so it covers "an unparseable certificate is refused", not "a bad proof is refused". The signature rung, the one a forger actually has to beat, had **no negative at all**.
- ⚠️ **Every rung answers the same `401` by design**, because a node with no certificate and a node with a bad proof must fail identically. So the wire cannot say which rung refused, and the fixtures prove it by CONSTRUCTION: six negatives, each satisfying every rung except the one it targets — unparseable; well-formed but chaining to nobody *while registered and live*; real-CA but unregistered; another node's live certificate; this node's own but revoked; and real-CA, registered, live, this node, signed with someone else's key.
- ⭐ **A positive control differs from the signature negative in exactly one factor — the signing key.** Its success is what proves the negative reached the signature rung rather than failing earlier for a reason the identical `401` would have hidden. Without it the ladder would be six fixtures that might all be failing at rung 1.
- 🔴 **The falsification is the entire justification.** With the server's signature check neutralised, the forged fixture is handed `Ok(HandshakeResponse { fencing_token: "fnc_…", lease_expires_at: … })` — a live lease issued to a caller holding someone else's certificate and none of its key — and **only this new control fails**. The pre-existing bad-proof control stays green. The project could have shipped a completely disabled signature check and its own negative would not have noticed. The chain rung behaves the same way: neutralised, the self-signed foreign certificate is handed a lease, and again only this control fails.
- ⚠️ **Scope correction, recorded rather than made silently:** the captured-valid-proof fixture the acceptance named is NOT built here. It needs the exact canonical bytes of a signed request, which the node client does not expose; reconstructing them in a test would duplicate a wire contract nothing checks, and would go stale silently. It belongs with the replay leaf, which must assert its own decision anyway.
- ⛔ No production code changes. This is control coverage, and its worth is measured by what the falsifications show the previous coverage could not see. `node_channel` 34/34; clippy `-D warnings` rc=0.

## 2026-09-13 — A rotation in flight can no longer outlive a revocation (`SIGNOFF-REPAIR.4.2.1`)

- 🔴 **Reproduced against both live routes.** With the rotation stalled at its host lookup — after its proof check had already decided the old certificate was good — a complete `POST /v1/nodes/revoke` ran and answered `200`; the rotation resumed, was ALLOWED, and the node finished with **one live certificate**. Since the lease renewal and the work delivery both ask whether such a certificate exists, a node the operator had just withdrawn got its lease and its work back.
- ⭐ **The first reproduction produced an ARTEFACT, and chasing it found the real mechanism.** Holding the old certificate's row made the revocation answer `500 canceling statement due to lock timeout`, because the rotation's insert was itself blocked behind the revocation's `SELECT … FROM nodes … FOR UPDATE` through the foreign key. The reason that mattered: **the foreign key does not prevent the race, it makes it deterministic in the wrong direction** — the insert is forced to land after the revocation commits, so a rotation overlapping one does not merely sometimes survive it, it reliably does.
- Fix: rotation becomes ONE transaction that takes the node's row FIRST and only then re-reads the certificate it was shown. ⭐ **No new lock and no new guard were invented** — it is the same row, in the same mode, the revocation already takes before it changes anything, so the two serialize. A rotation that wins the row commits a certificate the revocation then revokes; one that loses it is refused.
- ⛔ **Folding the liveness into the `INSERT` — the shape used for the lease renewal — was considered and rejected because it does not work here.** READ COMMITTED takes a statement's snapshot at statement start and the foreign-key wait happens after the row is formed, so the condition would be evaluated against a snapshot predating the revocation's commit. The window lies between two statements, so only a lock held across both closes it.
- 🔴 **The first control did NOT discriminate the repair, and the falsification is what revealed it.** Removing the row lock left the suite green, because the reproduction arm stalls the rotation at the statement the repair made first. No fixture can stall it in the real gap — nothing between the proof check and the insert touches the database — so a third arm asserts the lock directly: hold the node row, and the rotation must be observed waiting in `pg_stat_activity`. Falsified separately afterwards: the lock removed gives `nothing ever blocked on FOR UPDATE OF n`; the proof check moved back in front of it gives `live certificates after the revocation: 1`.
- A second arm asserts the opposite order rather than assuming it, because the repair claims both are correct: a completed rotation leaves two live certificates and the revocation takes both to zero.
- Validation: `node_channel` 33/33, `node_replacement` 1/1, `node_enrollment` 17/17, `node_work` 8/8, `identity_store` 4/4, `administrative_effects` 25/25, `authority` 22/22, `node_inbox` 7/7; clippy `-D warnings` rc=0; `mdbook build` ok. Lock order checked rather than assumed — both paths now take `nodes` then `node_certificates`, so no new deadlock edge.

## 2026-09-13 — The handshake and fencing surface, censused and split (`SIGNOFF-REPAIR.4.2`)

- ⭐ **This census read the ROUTED SOURCE-CENSUS RECORDS as well as the leaf's own goal line — `.11.9`'s lesson applied the same day it was written.** `.4.1` censused its goal line rigorously and carried one of three clauses from the record that pointed at it, which is how a reachable panic went unopened for nine leaves. **15 records** route here; the clauses naming this leaf's surfaces are censused, and the ones that belong to the outbox, publications, policy and fixture-isolation leaves are named as *someone else's* rather than silently dropped.
- 🔴 **A rotation can outrun a revocation and leave a LIVE certificate behind, and it would undo the two repairs that shipped hours earlier.** `rotate` verifies the proof, looks up the host and inserts the new certificate as three separate statements with no transaction and no tenant guard; the revocation updates every unrevoked certificate under the tenant's exclusive guard. An insert that lands after that update is not seen by it. ⛔ **This is not the claim `.4.1`'s census refuted** — that one asked whether a revoked node can rotate (it cannot); this asks whether a rotation already in flight can commit across a revocation. Since `.4.1.3` and `.4.1.3.1` both gate on a certificate that is neither revoked nor expired, a post-revocation certificate restores lease renewal AND work delivery.
- 🔴 **The rotate proof covers `{channel_version, node_id, cert_der}` and nothing else** — entirely static — so a captured rotate request replays verbatim and **each replay answers with a fresh private key**. The handshake proof carries no nonce, timestamp or server challenge either, so a captured handshake takes the lease and fences the legitimate node. ⚠️ Bounded: both die with the certificate, the dev profile escrows keys server-side anyway, and the wire is plain HTTP so capture needs no TLS break.
- **Four more confirmed at the source:** `renew_lease`'s `UPDATE` never names `lease_expires_at`, so a heartbeat can revive a lease that lapsed between the check and the write; `ack` writes on a pre-check the way `events` used to before `.2.2`; `secret_store` presents itself as the route for key reads while only the CA loader uses it; and the node's `from_hex` byte-slices a `&str`, so a non-ASCII response panics it rather than returning the error its signature promises.
- **The bad-proof control refuses BEFORE it reaches the signature** — the fixture presents a malformed certificate, which fails the chain check, so what looks like coverage of a refused bad proof is really coverage of a refused unparseable certificate. That gap is a prerequisite: a replay defence cannot be falsified by a fixture that never examines a signature.
- ⛔ **Nothing is repaired in this commit.** Seven children are opened, ordered so the one that undoes an existing repair goes first and the control-coverage prerequisite precedes the leaf that needs it. Each carries the `.3.4.3.1.3` prohibition: a source reading of a race is a hypothesis about scheduling, not a defect.

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

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated thirteen times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

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
