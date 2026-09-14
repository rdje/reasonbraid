# CHANGELOG.md

## 2026-09-14 — Ask the renderer, not the specification (`SIGNOFF-REPAIR.11.9.1.1.3`, tranche 2 complete)

- **Tranche 2c reconciled the last four source-census records — `R-76-77-2`, `R-87-1`, `R-88-1`, `R-90-1` — into 38 clause rows**: 21 `owned`, 11 `attach`, 6 `handled`, **0 `unowned`**. Tranche 2 is complete at 14 records and 84 clauses.
- **No new repair leaf was opened, and that is the measurement.** Tranches 2a and 2b each found a clause no leaf covered and opened `.11.10` and `.4.1.6`. Every one of these 38 reached an existing owner, because `.11.2` and `.11.3` already own the bootstrap, the doctrine gates and the backup/restore/dev/demo/load scripts.
- 🔴 **`check_table_arity.sh` disagrees with the renderer that publishes this project's book, and its own self-test locks the error in.** The gate exists to catch GFM's silent cell-dropping; its splitter treats a pipe inside an inline code span as part of the cell. Asked of mdbook/pulldown-cmark directly, a two-column row reading `` \| `x \| y` \| 2 \| `` renders as **two** cells and the `2` is discarded. Census over 322 tracked markdown files: the gate finds **0** defective rows, the renderer's rule finds **2** — both confirmed by rendering them.
- 🔴 **One of the two is the doctrine registry's own row.** `DOCTRINE_ENFORCEMENT.md:39` renders its third cell as a bare `—`, so the published page never names `scripts/check_tree_index_frontier.sh` as `INDEX-FRONTIER`'s enforcer. It is deliberately left unrepaired as the last real-world specimen, so `.11.2` can falsify its parser repair against a row it did not write.
- 🔴 **`RECONCILIATION.md:111` was the other, in this leaf's own deliverable**, truncating the `R-53-2` Evidence cell at "three `.map_err(". Repaired here, with the superseded text named.
- 🔴 **`pipefail` does not fix the demonstration's negative controls**, which is the record's own proposed remedy refuted by measurement: the four-way truth table is rc=0 in every cell, because `!` negates a pipeline whose status is 1 whether the CLI failed or simply matched nothing.
- 🔴 **The load harness reports `PASS: every command committed` for a run that issued zero requests** (`--commands 0` and `--commands -5` both satisfy its exit gates), and issues 104 for a requested 100.
- 🔴 **7 of the 9 doctrine checks that consume the staged file list then read the worktree**, `check_task_tree_ownership` among them; **`check_docpaths.sh` cannot see `/Volumes`**, this checkout's own prefix; and **the handoff census prints `handoff: OK` at rc=0 when both its process censuses fail**, while its `ps -Ao` arm covers 620 rows across 40 uids where its documentation claims one.
- ⛔ **Refuted and recorded rather than dropped:** `R-87-1`'s premise that this project retains `MAINTAINING.md` is false, and was false at the census baseline. The sentinel finding stands; the framing that placed this repository in the blast radius does not.
- **11 `attach` clauses written into `.5.2`, `.11.2` and `.11.3` in this commit**, per the ledger's rule; 26 of 26 `attach` rows verified present in their owners' sections by hand. **Promoted:** "ask the renderer, not the specification" (`TOOLBOX.md`).
- Validation: `--classified` 110 rows clean, `--self-test` 42 controls, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema, test or script changed.

## 2026-09-14 — Graded on the question, and one claim failed (`SIGNOFF-REPAIR.11.9.1.1`, `.11.11` opened)

- The director asked whether I trusted the session's findings. The claim standard treats that as the grading event and its pass condition is answering yes in one word, with no keyboard. I could not, so I re-derived twelve published claims by routes different from the ones that produced them.
- **Eleven held.** The certification count (3 of 6 invariants asserted per run) re-derives from the arm structure — three early-return pushes are mutually exclusive. The reconciler never reads `git.effective`: a grep for the field finds only comments and its declaration. `filter_profile` emits exactly fourteen fields, neither of the two the record named. `regions::route` still has no production caller. The sizing re-derives at 2,554 / 7,192 / 2.82×. All **15** `attach` rows are present in their owner leaves — I did not repeat tranche 1's mistake. The first `declined` row holds: the module carries exactly one wire-absence claim and it is about hidden fields.
- 🔴 **One failed, and it was mine.** The eight uncited-but-named records were published with LINE NUMBERS. Two of the eight were stale before the session ended, having moved as this tree grew above them. A line number into a growing file is the `LIVE-DOC-CURRENCY` failure in another costume: true when written, false the next commit, and nothing checks it. All eight are now located by the heading they sit under, and the superseded form is named rather than quietly replaced.
- ⚠️ A second, softer correction: the reproduction for that number reports **13** today rather than 8, because a `## Commit acceptance` heading is not a leaf heading either and five of this session's own blocks name records. The 8 is a true statement about the population at leaf-open, not a stable quantity — now said so.
- The grading also produced a tracked gate candidate rather than a note. `SIGNOFF-REPAIR.11.11`: `attach` is the only ledger state whose next action is a sentence in someone else's leaf, and `--classified` cannot see whether it was written. Censused before proposing — 15 rows, **0 breaching today, 3 of 3 at tranche 1's close** — which is the shape this project allows a gate to have.
- Validation: 18 doctrines green, `mdbook` rc=0. Documentation only.

## 2026-09-14 — A host claim is checked where a human typed it (`SIGNOFF-REPAIR.4.1.6`)

- Reproduced through the supported routes: a non-ASCII host claim was accepted at token issuance and panicked when the node redeemed it. The node received a dropped connection rather than an answer; the server kept serving; no node row was written; and the token was left unconsumed.
- That last fact is the one the opening inference had not reached. An outstanding unused token refuses a second issuance for the same node id, so the node id could not be enrolled until the token lapsed — bounded to the token's lifetime (an hour by default, a day at most), and bounded only because `SIGNOFF-REPAIR.4.1.1` already supersedes a lapsed token.
- Two changes. `issue_node_leaf` returns a result instead of unwinding — a function whose only way to report a bad caller string is to panic leaves its caller's typed error path unreachable — and the issuance route checks the claim beside the node-id shape and the lifetime range, because that is where an operator typed it. The first makes the failure answerable; the second makes it unreachable.
- The accepted set is deliberately unchanged: the check asks the certificate library rather than restating a grammar, so it narrows nothing and cannot drift from what issuance would have done. Whether a stricter host-name grammar should bind is deferred with its compatibility question named, in `docs/decisions/2026-09-14_host-claim-checked-at-issuance.md`.
- Validation: 82 live tests across five suites and 103 library tests, rc=0; strict all-target lint on the server rc=0. Falsified in two halves, each hitting a disjoint control: removing the issuance check gave 18 passed / 1 failed with `left: 200, right: 400`; restoring the panic under the unchanged signature gave 19 passed / 2 failed, reproducing the transport error a second time.
- One documented wire narrowing on `POST /v1/nodes/enroll-tokens`; no migration and no schema change.

## 2026-09-14 — The first declined clause, and asking the library instead of the call site (`SIGNOFF-REPAIR.11.9.1.1.2`)

- Tranche 2b classified 19 clauses over four records: 1 handled, 8 owned, 8 attach, 1 unowned, and the ledger's **first `declined`** row. That state was put in the vocabulary before any instance existed, because a deliberate rejection is invisible to every search; the first one is a clause whose mechanism is real and whose asserted contradiction is not.
- One clause could not be settled by reading — `expect panic if malformed host` — because what the certificate library calls malformed is not knowable from the call site. A probe asked it: of eight host claims, rcgen 0.14.10 refused exactly one, the non-ASCII one. An empty string, 300 characters, `not a dns name!!`, `*.example.com` and `..` were all accepted.
- That produced two findings pointing opposite ways. The panic is real and reachable — the host claim travels unvalidated from token issuance to enrolment and is handed to an `expect` — and it is much narrower than "malformed" suggests. And the accept set is far wider than a DNS name, so the certificate's host binding is effectively unvalidated, which is the larger finding and not the one the record named. New owner `.4.1.6`, opened with the sample explicitly labelled a sample and the blast radius explicitly not claimed.
- All eight `attach` clauses were written into the leaves that own them in this same commit, under the rule added one commit ago: a completion invariant that accepts a failure as completion and an unbounded drain (`.10.2`); a certificate authority signed for a year with no renewal and a leaf never compared against its issuer (`.4.1`); a visibility default that publishes eleven of fourteen fields while its documentation promises self-only, a reversed doc example beside correct code, and a "full profile" reader that never receives two fields (`.5.1`); and a publication commit whose object id changes every second while its own documentation calls it idempotent (`.9.2`).
- Validation: `--classified` 72 clause rows, ledger clean; `--self-test` 42 controls; uncited 106 → 102; 18 doctrines green; `mdbook build` rc=0. No product code, test or script changed — the probe was an untracked file, run once and removed.

## 2026-09-14 — A failed read is not a verdict about the site (`SIGNOFF-REPAIR.11.10`)

- `regions::route` converted any `sqlx::Error` into a routing refusal. Reproduced with the probe declaring its own premise first: `dev-local` is a seeded declaration, and with the pool closed the routing answered ``the region `dev-local` is undeclared — the routing refuses``. A statement about the site's configuration, produced by a failure that touched no configuration.
- The same repair already existed one module away, with its reason in its own source: `SIGNOFF-REPAIR.3.2.1` put the `pair` verb on the site-authority service because "an unavailable database must never masquerade as an undeclared region". The review record named `pair` **and** `route`; only `pair` was reached, and the clause reconciliation found the other half.
- A new `RouteError` keeps the two families apart — `Refused(RegionRefusal)` for a verdict reached by reading the configuration, `Storage(sqlx::Error)` for a decision that could not be made. `RegionRefusal` is byte-identical, variants and wording, because the phase acceptance and the operator contract rest on those names. The storage message names neither the region nor "undeclared".
- Reachability stated honestly: `route` has no production caller today. It was repaired now rather than left to the store-and-forward lane that will inherit it, because an inheriting leaf censuses its own goal line.
- Validation: `run_pg_tests.sh regions` 3 passed / 0 failed, rc=0; strict clippy on the crate's lib and this test rc=0 with no warnings; `cargo fmt --all -- --check` rc=0. Falsified against the exact superseded mechanism — the `map_err` restored under the new signature gives `2 passed; 1 failed`, the single failure this leaf's control naming `Refused(UndeclaredRegion { region: "dev-local" })`, with the other two tests still green.

## 2026-09-14 — The ledger's own next action was never taken (`SIGNOFF-REPAIR.11.9.1.1`, `.11.9.1.1.1`)

- Tranche 2 of the clause reconciliation was **2.8x** tranche 1 by record text — 14 records, 7,192 characters, ~57 sentence-clauses against 7 / 2,554 / ~19 — so it was **sized before it was attempted** and split three ways on each record's own narrowest candidate, the same measurement that formed the tranche.
- 🔴 **The finding that matters is about the instrument, not the records.** Tranche 1 classified three clauses `attach` — the state that means *this leaf owns the surface but its text does not mention the clause, so its next census will drop it* — and attached none of them. `SIGNOFF-REPAIR.11.9`'s defect was running inside the ledger built to stop it. All seven `attach` clauses are now written into `.7.1`, `.8.1` and `.11.4`, and the vocabulary requires the attachment **in the commit that classifies it**.
- ⛔ **A published number is corrected rather than withdrawn.** Eight of the 112 uncited records — the population when this leaf opened; the command now reads 106, because this commit's seven attachments made three leaves name six of them — are named in the task tree, in text that belongs to no leaf — six of them in one historical-dispositions table written by the leaf that did the work. The instrument's rule (a leaf's *own* section) stands; what does not is the conclusion that "114 uncited" meant "114 nobody re-read".
- Tranche 2a classified **27 clauses of 6 records** against the source: 9 `handled`, 13 `owned`, 4 `attach`, 1 `unowned`. Four are live defects already named by their owning leaf's goal line — the resolver registry's tenant-admin write, the workflow registry's enrolment-only write, the reconciler that never reads the effective channel ref, and the decision close that consults only the caller's unresolved list.
- New owner `SIGNOFF-REPAIR.11.10`: `regions::route` reports any database error as an undeclared-region verdict. The same record named `pair` and `route`; `pair` was repaired and carries the rule as a comment, `route` was never touched. No production caller today — the store-and-forward lane is the one that would inherit it.
- Validation: `--classified` -> 53 clause rows, ledger clean; `--self-test` -> 42 controls pass; `check_doctrines.sh` -> all 18 green; `mdbook build` rc=0. Documentation only; no product code, schema, test or script changed.

## 2026-09-14 — Three claims graded on a challenge, and three corrected (`SIGNOFF-REPAIR.11.2.2.1`)

- The director asked whether I still stood by the day's findings. The claim-verification standard treats that question as the moment a published claim is graded, and its acceptance test is that the author answers **yes in one word, with no keyboard**. I could not, for three of them.
- ⭐ **Everything measured held.** Seven sites, zero firing today, 36 against 46 pending leaves, 265 files scanned, 22 self-test classifications, the two device ids, the seven file-and-line locations — all reproduce.
- 🔴 **What did not hold were three verbs and attributions, every one added while summarising rather than while reading.** The probe scripts do not *clone* throwaway git repositories, they `git init` them — which is the source record's own word, "create". The script that genuinely clones is a third one, and it was in the seven all along. The storage-locality checker's header does not *cite* the self-test doctrine's founding incident; it names its own `--self-test` flag, and the true version is sharper: the file had a self-test of its own and that self-test still could not see the breach. And the source record names its scripts by role, not by path, so saying my filter "excluded the directory the record was pointing at" made the misreading that followed look less available than it was.
- ⚠️ A fourth correction never reached a tracked file and is recorded so the count survives: I described "a description of a check being ahead of the check" as happening three times in one session. It happened **twice**. The third case belongs to a different family — an instrument matching a literal inside its own source.
- ⛔ No number moved, no decision reversed, no gate behaviour changed. Nine wording sites across seven files, with each superseded phrase named rather than silently replaced.
- ⭐ The pattern in the corrections is the useful part: every claim I could not affirm in one word was one I had written from memory of the source instead of from the source. The numbers, which came from commands, all survived.

## 2026-09-13 — The gates' own scratch comes back onto the repository volume (`SIGNOFF-REPAIR.11.2.2`)

- 🔴 **Three of the scripts that enforce doctrine were writing their own temporary files to an ambient directory**, and two probe scripts `git init` whole throwaway git repositories there (corrected: they create rather than clone; the script that clones is `update_scaffold.sh`). Measured, not assumed: the checkout and the repaired scratch are on device `16777244`; the ambient directory is on `16777232` — a different volume, so that data left the repository's storage accounting entirely.
- 🔴 **The gate written for exactly this could not see any of it**, because it enumerated Rust files. The shell family was outside the scan by construction rather than by exception, so the reviewed-exceptions list did not mention them either: invisible, not waived.
- 🔴 **This leaf's own published number was wrong, and the correction is the more useful finding.** It opened with five sites. The real count is **seven** — its census had been scoped with a `grep -v '^docs/'` filter that excluded the very directory the source record was pointing at. The record says "task-acceptance and waiver-routing **probe scripts**", and both of those live under `docs/tasks/artifacts/`. A path filter that excludes a directory is the same defect as a glob that matches nothing: the census reports a smaller world and raises no error.
- **The gate is extended** to tracked shell and Python. It fires on **zero** sites today and would have fired on all seven — the shape a gate should have, catching the next drift rather than presenting a backlog.
- ⛔ **The rule is about the argument, not the call.** Both `mktemp` and `tempfile` name by exclusive creation, which is precisely the half of this doctrine a clock-derived name gets wrong. They are the right tools pointed at the wrong volume. A pattern flagging the call itself would have condemned fifteen conforming sites and taught people to route around the gate.
- ⭐ One Python site is a deliberate control — a probe that exists to *measure* where a child process's temporary file lands, which pinning a directory would turn into an assertion. It is a reviewed exception with that reason recorded.
- 🔴 The extended gate flagged **itself** on its first run: its own diagnostic message spelled the command it matches. That is the founding incident of the self-test doctrine repeating, in a file that carries a `--self-test` of its own and still could not see it (corrected: its header names that flag, not that incident). Fixed by re-wording rather than by excluding the file's path, so spelling the literal again turns the scan red immediately instead of silently passing.
- Validation: 265 files scan clean with 2 reviewed exceptions; the self-test grows from 8 classifications to 22 across all three languages; falsified against the exact pre-repair sources, which the gate names all seven of. Both repaired probe scripts pass (10/0 and 5/0) and each repaired guard runs green.
- ⛔ Not claimed: a Makefile recipe, a CI workflow step or a Rust `tempfile` call reaches none of the three patterns, and no census of those was run.

## 2026-09-13 — The inbox inspection reads only the tenant it was admitted for (`SIGNOFF-REPAIR.3.5.3`)

- 🔴 **Reproduced before anything was changed.** An administrator of one tenant, over the supported HTTP surface, naming its own tenant and another tenant's node id, received that tenant's inbox rows in full — command ids, thread ids, delivery state and the command payloads.
- ⭐ **Why it outlived a repair that named its three siblings.** The record behind this family named **four** verbs. The census that drove the earlier repair scoped itself to the node administrative *mutations* and said so; it bound quarantine, replay and prune. The inspection is a read, so it fell outside a census that was correct about its own scope and silent about the record's. Nothing had ever cited that record, which is why the clause-level reconciliation found this and no test did.
- ⭐ **The shape worth carrying: the caller supplies two independent identifiers and the handler checked one of them.** The admission proved the caller administers the tenant they *named*; the select then asked only for the node id. Nothing tied the two together, so the tenant parameter was decoration.
- **All 24 read routes were censused before the repair, and the three hits classified down to one.** One was this defect. One was the metrics surface, already owned and deliberately process-wide. The third was a false positive — and it produced the better rule: `GET /v1/calls/{call_id}` loads the call first and authorises against the call's *own* tenant, so its later queries need no predicate at all. Derived versus accepted is the difference.
- ⚠️ The census could not finish, and says so: ten of the twenty-four handlers hold no SQL of their own and delegate to a module the scan does not follow. They are **unmeasured, not clean**, and now have their own owner.
- ⭐ Falsified in the strongest form available: the control was written *before* the fix, so the pre-repair run **is** the neutralised build — nothing reverted, nothing reconstructed. It asserts the absence twice (an empty row list, and no victim command id anywhere in the response text, because an id discloses as much as a payload) and carries a positive arm: the owning administrator must still see every row.
- ⚠️ **A third finding, in prose rather than code.** The sibling control's comment claimed it covered "read, move or destroy … all three" and asserted two of the three — and the same sentence had propagated into two book chapters, one of which said "all three verbs" directly beneath a block showing four. The read stayed unbound while three chapters discussed its neighbours. All three corrected.
- Validation: 4 suites / 55 tests rc=0; strict server lint rc=0; gate 18 checks; book built and link-checked. No migration — the column has existed since the table did; only the query changed.

## 2026-09-13 — Census the 114, correct the ranking the leaf proposed, and build the ledger (`SIGNOFF-REPAIR.11.9.1`)

- ⛔ **The leaf's own instruction was rejected by its own census.** It said to start from the records that name only one or two candidate leaves, "there the routing was a genuine assignment". Ranking instead by each record's **narrowest** candidate — the candidate leaf fewest other records also name — the two orders come out anti-correlated. All five fan-out-1 records point at a container: `SIGNOFF-REPAIR.11.4` is named by **99 of the 131 records**, `.3.3` by 41, `.4.1` by 37. Every record with a fan-out of nine or more reaches a leaf named by ten or fewer.
- ⭐ **One record settles it.** `R-6-27-1` has a fan-out of one. Its entire body is the artifact's own boilerplate — "Source-review candidates only; runtime reproduction and task-tree ownership MUST follow full-read prerequisite." It carries **no finding at all**, and it still received a candidate leaf. A fan-out of one is the reviewer reaching for the default bucket, not precision.
- ⚠️ The earlier statement is **not** called wrong. Fan-out measures how sure the reviewer was about where a finding goes; whether an uncited routing is a real miss depends on whether the *target* needed the record. Two real measures, two different questions, and this activity needs the second.
- **The ledger**: `docs/tasks/artifacts/signoff_review/RECONCILIATION.md`, one row per **clause** — the unit the previous leaf established — with a closed six-state vocabulary and an instrument that re-reads it (`--classified`) and refuses an unknown record, an out-of-set state, an owner that is no leaf, or a duplicated clause.
- ⭐ **The vocabulary gained a state the acceptance did not have: `attach`** — a clause whose owning leaf's own text does not make it visible. That is exactly the mechanism that dropped clauses before, caught while it still costs one sentence to fix rather than a retrospective reconciliation. Three clauses carry it.
- 🔴 **Classifying the first seven records found two live defects, each with a new owner.** `inspect_node_inbox` is the fourth verb its record named; it admits on the caller's tenant and then selects the inbox by node id alone, so an administrator of one tenant reads another tenant's command ids, thread ids and payloads. The earlier repair covered the three *mutations* and its census said so. Owner: `SIGNOFF-REPAIR.3.5.3`.
- 🔴 **And three of the doctrine enforcers write their own scratch off the repository volume**, which the storage-locality gate cannot see because it enumerates Rust files only. Owner: `SIGNOFF-REPAIR.11.2.2`, opened with the raw 19 matches classified down to the 5 that actually breach — the twelve Python sites all pass a repository-derived directory and conform.
- 🔴 The reconciliation instrument was wrong a fifth time, and this leaf's own prose is what exposed it: its elided-citation rule matched any `:N`, so source line numbers written after a census citation inherited the census filename. Now anchored to the backtick the form is always written with, with two controls that go red without it.
- 🔴 The task tree's frontier caption published a pending-leaf count from a command that missed every leaf using the newer `- Opened:` spelling — **11** of them. It said 36; the tree holds 46.
- Validation: `--self-test` 42 controls, rc=0 (21 before); `--classified` reports 26 clause rows with a clean ledger; the doctrine enforcer's 18 checks pass. No product code changed.

## 2026-09-13 — Rotate the changelog a fifteenth time (`SIGNOFF-REPAIR.11.4.1.2`)

- The rotation threshold refused a commit at 96,102 bytes. That is the containment doing its job: the cap decides when to rotate, not a judgement about length.
- 14 whole records stay; 16 move into reachable Git history behind an exact pointer. Nothing is deleted and the threshold is unchanged.
- Validation: header plus retained plus retired plus the original footer reconstruct the 92,548-byte predecessor **byte for byte**, with a matching SHA-256 — and altering a single byte of the retained segment breaks both. Both chain pointers were re-measured rather than copied: the predecessor blob really is 92,548 bytes with the recorded digest, and the link behind it really is the 95,013 bytes its own notice claims. New digest 44,559 bytes.
- ⚠️ Sequenced as its own commit, with the entry that triggered it set aside first, so the snapshot this rotation names holds only entries that were already committed.

## 2026-09-13 — A replacement ends the old machine's session, host and incarnation (`SIGNOFF-REPAIR.4.1.5`)

- 🔴 **All three clauses confirmed, each reproduced in its own run** before it was repaired — not asserted together against one green suite at the end.
- 🔴 **The severe one: a replacement RESTORED the replaced machine's session.** Measured — its heartbeat answered `200` after the replacement. The mechanism is what makes it worth carrying: an earlier repair stopped a revoked node renewing by requiring *a usable certificate for this node id*, and a replacement issues a fresh one **for that same id**, so the predicate becomes true again and the old process gets its lease back. ⚠️ Bounded to a replacement landing within the lease TTL of the revocation — which is the ordinary operational case.
- 🔴 **The host reverted.** `nodes.host_id` kept the old machine while the response echoed the new one, and certificate rotation reads the SAN's host from that row — so the first automatic rotation, within half a leaf lifetime, silently renamed the certificate back to the machine the node no longer runs on. The control drives a real rotation and reads the SAN.
- 🔴 **The incarnation never ended**, and the comment claiming it could not duplicate — *"the `nodes` primary key refuses a second enroll for the id"* — is the sentence that hid it. A replacement is precisely a second enroll for an existing id. ⚠️ Its consequence was measured too and is narrower than it sounds: both selectors already order by `valid_from DESC LIMIT 1`, so the defect is the **ledger**, not the selection — an incarnation that never ends cannot be attributed against.
- ⛔ **No property was traded.** The withheld-work decision is about inbox rows, not the lease; the control asserts in the same run that the replacement handshakes and takes its own lease, and the replacement ritual passes unchanged.
- ⭐ The lease epoch is **bumped**, not deleted: deleting lets the next handshake reset the epoch to 1, breaking the per-node monotonicity a previous leaf's argument rests on. Bumping is also this codebase's own way of saying "that session is over".
- Validation: falsified each part separately with the full repair restored between runs — `1 passed; 1 failed` three times, each naming its own clause. **72 tests** across five live suites, rc=0; clippy `-D warnings` rc=0.
- 🔎 Worth carrying: a guard written as *"does a usable credential exist for this node id?"* is satisfied by **any** such credential, including one issued after the event the guard was defending against. Every predicate of that shape deserves re-reading against this.

## 2026-09-13 — The wake gate is a drain switch, and every state it has now has a control (`SIGNOFF-REPAIR.4.2.10`)

- **census taken from the filter rather than from the tests**: the delivery gate has FIVE reachable states and the existing control covered two — zero, then two.
- ⭐ **The three uncovered states all mean "deliver", and each reaches that answer by a different route through the SQL** — a missing `concurrency` key makes `NULL = 0` yield NULL, a missing `availability` block makes the whole path NULL, and a negative number is simply not zero. One of them does not stand for the others, so all three now have controls.
- ⭐ **The drain state is re-asserted LAST, after the four delivering ones.** Without it, every positive arm could be passing because the fixture had quietly stopped being deliverable for an unrelated reason. A control whose positive arms can all be vacuous is not coverage.
- 🔴 **The substantive answer is a claim narrowed, not a defect repaired: the gate is a DRAIN SWITCH, not a concurrency limiter.** It tests for exactly zero and never compares a declared concurrency against an active count, so a node declaring `concurrency: 2` will receive a third row. That is the design — presence already names zero `draining` — and the book now says so, including what an operator must therefore enforce at the node.
- ⛔ **A hypothesis of my own, measured and refuted, recorded because it would have been serious**: the `::bigint` cast on that JSON field appears at six query sites and a non-numeric value would raise at every one. It is unreachable — the field is typed `Option<i64>` and the writer stores the re-serialized typed struct, not the raw body.
- Validation: falsified by removing the gate — the held role receives its row, `36 passed; 1 failed`, only this control. `37 passed; 0 failed` restored. No production code changed.

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

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated fifteen times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

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
