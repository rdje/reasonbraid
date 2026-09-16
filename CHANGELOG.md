# CHANGELOG.md

## 2026-09-16 — Anchor the LFS gate to the pointer's own grammar (`SIGNOFF-REPAIR.7.2.3`)

✅ **A repository that merely documented Git LFS could not be acquired, and the acquisition's LFS policy is now stated rather than implied.**

- 🔴 **Reproduced before the repair.** Two new controls fail against the unrepaired predicate — a documentation file whose first line quotes the version line in backticks, and a manifest for an unrelated specification — both `Refused { what: "Git LFS" }`, `test result: FAILED. 9 passed; 2 failed`.
- **Why it was possible:** the gate refused a blob when the ten-byte run `version ht` appeared anywhere in its first 64 bytes. A real pointer carries `version https://git-lfs.github.com/spec/v1` at offset 0, so the predicate was both too broad and unanchored.
- ⚠️ **The direction matters and is recorded:** this OVER-refused. An availability and correctness fault, never a bypass, and no bypass is claimed because none was measured.
- **Fix:** a named predicate, `is_lfs_pointer`, doing a 42-byte prefix test at offset 0 that must end at a newline or at the end of a blob carrying nothing else — the specification makes the `version` line the pointer's first line. `oid` and `size` are deliberately not required: a truncated pointer is still not the content.
- **Policy, decided rather than inherited: refuse.** A pointer's bytes were never transferred by the clone, so they were never classified by the destination policy and never counted against a budget; accepting one would report a stand-in as content. Now written in `docs/book/src/deployment.md`, where ROADMAP §12.5's "explicit Git LFS policy" can be checked against the code.
- ⭐ **Falsified in two directions.** Restoring the search fails both controls; an anchored-but-not-spec-matched `starts_with(b"version ht")` passes the prose control and fails the other — so the two controls bound different halves of the repair.
- ⚠️ **The leaf's own acceptance number was wrong and is corrected:** the version line measures 42 bytes, not the 41 it claimed.
- ⚠️ **Two residuals recorded rather than absorbed:** git-lfs's legacy `hawser`/`git-media` version URLs are not matched, and the refusal names the file rather than its path.
- **Verified:** 108 passed / 0 failed, with the pre-existing pointer fixture unchanged; clippy `-D warnings`, `cargo fmt --check`, `make gate`, `mdbook build` and the book link check all rc=0.

## 2026-09-16 — Classify every redirect hop, and prove the unrepaired one was dialed (`SIGNOFF-REPAIR.7.2.2`)

✅ **The last §16.12 line that was still an open defect is repaired — and it was reproduced at runtime first.**

- 🔴 **The unrepaired client did not merely fail to refuse; it dialed and was answered.** With `Policy::limited(5)` restored, the control fails with `the IP-literal hop must be refused by name: Ok(200)`: the redirect to `http://0.0.0.0:{port}/private` was followed and the refused destination served the request.
- **Why it was possible:** `git.rs` built its client with `Policy::limited(5)` and a DNS belt, hyper-util 0.1.20 skips the resolver when the host is already an IP address — in its own source — and `GitFetcher::classify` runs once, on the initial URL. A hostname hop was covered; an IP-literal hop reached nothing.
- ⭐ **The instrument is `0.0.0.0`.** It classifies as `reserved` and the kernel routes a connection to it at the local host — measured with a two-socket probe before the control was written — so the same origin answers and its hit counter reports whether the hop was actually dialed. A control that only asserted "an error happened" could not tell a classification from a connection that failed. `127.0.0.2` was the obvious first choice and does not bind on this host (`Errno 49`).
- **Fix:** `Policy::custom` applies the pre-flight's own policy to each hop whose host is an IP literal, re-states the five-hop cap that `limited` used to own, and carries a typed refusal through a slot `send` clears before each request — reqwest's redirect policy can answer only follow, stop or error.
- **The DNS belt can now say what it refused**: `ClassifiedDns::resolve` returned an empty address list when every address was refused, surfacing as a bare connect failure. ⚠️ Never a safety gap — it failed closed — and it is recorded as diagnosability.
- ✅ **The R1 module header is true again**, and now names the two mechanisms that make it so instead of claiming the other pack's property.
- ⚠️ **Two residuals recorded rather than absorbed:** the POST path has no error channel for a refusal, and an `https → http` hop is still unrefused.
- **Verified:** 105 passed / 0 failed, every pre-existing destination control unchanged. **Falsified** self-reversing, restored byte-for-byte. A second control keeps the repair a classification rather than a prohibition.

## 2026-09-16 — Split the acquisition lane into the three repairs its goal line was carrying (`SIGNOFF-REPAIR.7.2`)

⛔ **One leaf, one `- Status: pending.`, no acceptance of its own, and three repairs that share nothing but a subsystem.**

- **`.7.2.2` — classify every redirect hop, including an IP literal.** `Policy::limited(5)` auto-follows up to five hops with only the DNS belt behind them, and hyper-util 0.1.20 skips the resolver when the host is already an IP address — in its own comment. The pre-flight classifies the first destination only.
- **`.7.2.3` — the LFS gate refuses on a ten-byte run, not on a pointer.** `head.windows(10).any(|w| w == b"version ht")` over the first 64 bytes, where a real pointer carries that line at offset 0. ⚠️ It over-refuses: an availability defect, not a bypass, and the leaf says which direction it runs.
- **`.7.2.4` — the repository is opened with gix's default permissions** over an untrusted remote, while gix 0.87.1 ships the named remedy it does not use, `open::Options::isolated()`. Distinct from `.7.2.1`, which owned the directory: this is about what gix reads.
- ⭐ **The census narrowed the published finding.** A hostname redirect hop IS covered — hyper-util resolves it through `ClassifiedDns`, which retains only policy-allowed addresses — so the uncovered destination is precisely an IP-literal hop.
- ⚠️ **And turned up one nobody had recorded:** `ClassifiedDns::resolve` retains allowed addresses rather than erroring, so a hostname resolving only into a refused range gives a generic connect failure instead of a named `DestinationRefused`. It fails closed, so it is diagnosability rather than safety, and it is owned rather than reported.
- The parent's opening line is renamed `- Opened:` so the leaf carries exactly one `- Status:` — which `TASK-STATUS` refused the commit until it did.

## 2026-09-16 — Close the delegation lane on its goal line, three verdicts of which are not "done" (`SIGNOFF-REPAIR.3.4`)

✅ **Eight children, reconciled against the leaf's own goal line rather than against their statuses.**

- Closing on the children's statuses alone would have been the shape `.3.4.3.1` had just shown is wrong one commit earlier: a third of THAT leaf's acceptance turned out to be unmeetable, and nothing about its children said so.
- **Three of the six verdicts are not "done".** Delegability is **refused as a gate** — `delegable` governs grant chains that have no producer, and enforcing it would have broken a shipped, tested feature. Bounded depth is **deliberately unenforced** over a population of zero, said so in the book rather than repaired. Consent is **not a requirement**: §16.3 states six delegation invariants and none is consent.
- **Three are done and measured:** participation enforced (by a gate the census had not looked for), the replay hash bound to the authority context and to the target, with committed-replay semantics asserted in the same control as each binding, and cached expiry and future-clock behaviour made explicit.
- ⚠️ **Three residuals carried forward by name:** delegation chains and their depth bound have no producer and the wire cannot carry one; `lease_expires_at` is received on the handshake and the heartbeat and never read; a delegation's attribution claim names a subject that never agreed in an authorization record §16.9 makes high-impact evidence.
- ⛔ **The split bullet's wrong count is kept verbatim** — it says "five children along the four mechanisms the goal line names" while the goal line names five. That sentence hid two mechanisms for eleven commits, and correcting it in place would hide the hiding.

## 2026-09-16 — Close the clock split, and record the acceptance clause it could not meet (`SIGNOFF-REPAIR.3.4.3.1`)

⛔ **A container leaf's closure is its own acceptance, clause by clause — and one of these three could not be met.**

- The leaf read `active` while all three children read `done` (REPAIR-0136 / 0138 / 0139): `SIGNOFF-REPAIR.11.4.5.3`'s finding running the other way, the tree over-reporting remaining work instead of under-reporting it.
- 🔴 **Clause 3 is UNMET and superseded, not quietly satisfied.** *"The `.3.4.3` backward-skew controls pass unchanged"* — both assert the receipt-anchoring clamp that `.3.4.3.1.2` removes, and both drove a state production cannot reach. The supersession lived inside the child; it now sits at the parent whose acceptance it belongs to, so a reader need not open a child to learn that a third of this leaf's acceptance was withdrawn.
- ⭐ **The withdrawn property is stronger now**, which is what makes the supersession honest: "one TTL of real time whatever this node's clock says" is asserted in both skew directions, and the clamp could only do it for one.
- **Clause 2's margin, stated rather than implied:** the control drives the node 600 s ahead against a 60 s TTL — ten times the threshold that refused every dispatch — and proves the gate's allow by a channel error rather than by a success, because a refusal in that path returns `Ok`.
- ⚠️ **One residual carried forward:** `lease_expires_at` is received on the handshake and the heartbeat and never read — a latent third cross-clock comparison.
- **Verified:** 14 suites, 78 passed, 0 failed, 2 ignored, rc=0, all four clock controls passing by name. No source changed.

## 2026-09-16 — The subject is not asked, and the specification never asked for it (`SIGNOFF-REPAIR.3.4.7`)

⛔ **Consent was never a delegation requirement in this project**, which is a different finding from the one this leaf was opened to make.

- The leaf opened by measuring that ADR-009 never says "consent" and read that as a gap the ADR had left. Measuring the **specification** instead: `grep -ic consent ROADMAP.md` returns **8** and `awk 'NR>=1754 && NR<=1769' ROADMAP.md | grep -ic consent` returns **0**. §16.3, where the delegation invariants live, states six of them and subject consent is not among them.
- ⭐ The roadmap's consent is **§4.4's** — an enrollment and mandate-domain act, carried on the `EnrollmentAuthorityBoundary` as `target_disclosure_and_acknowledgement` and shown to the target owner before enrollment completes. A different mechanism in a different lane. ADR-009 was silent because there was nothing to decide.
- **The answer, in one sentence:** a delegation here is trusted impersonation inside one tenant; the subject is not asked and cannot refuse; the consequence is bounded to attribution.
- **The bound is re-derived at HEAD rather than inherited:** `authority.rs::authorize_in_tx` builds a caller authz with `delegation: None` and the same action and target, and a denial there replaces the outcome with `the caller's own authority failed`. Each of the four gates is pinned by a named control, so no new control was added for appearance.
- ⚠️ **The cost is accepted rather than dismissed:** an authorization record — §16.9 evidence — names a principal as the authority behind an act it never agreed to. The alternative is the capability-token machinery ADR-009 already subtracted.
- **Revisit trigger, mechanical rather than atmospheric:** a delegation crossing a tenant boundary, a subject that is not an enrolled principal of the same tenant, or any relaxation of the four gates — each a failing control.
- ⭐ Both mechanisms `.3.4`'s own split dropped are now carried. ⚠️ `.3.4` still cannot close: `.3.4.3.1` reads `active` with all three of its children `done`, which is the same shape running the other way, and closing it is that leaf's own commit.

## 2026-09-16 — Bind the idempotency hash to the command's target (`SIGNOFF-REPAIR.3.4.6`)

🔴 **A caller addressed one thread and was handed another thread's event as its own result.** `request_hash` covered the operation, the actor, the body and the authority context — never the target.

- **Reproduced before repairing.** The same actor, body and idempotency key against a *different thread in the same tenant* answered `200` with `"replayed":true`, carrying the first thread's `thread_id` and `event_id`. Thread two was never looked at: the idempotency claim is step 1 and authorization is step 2.
- **Why it was possible:** a thread command's thread arrives as a PATH segment, the typed bodies carry `tenant_id` but no `thread_id`, and `migrations/0001_atomic_transaction.sql:39` keys `idempotency` on `(tenant_id, idempotency_key)` — tenant-wide.
- **Fix:** `request_hash` gains `target: Option<&str>`, appended as `\ntarget={id}` only when present; all eleven operations of `POST /v1/threads/{thread_id}/commands` pass the path's thread.
- ⭐ **The target is bound exactly where a caller can vary it independently of the key.** Three callers pass none, each with a structural reason at its own call site: a creation has no thread and its target is the tenant (already the idempotency primary key's first column); the MCP respond tool fixes the thread inside its key, and binding it there would change the key and turn an old call's replay into a **duplicate contribution**; a node result is keyed by the server-assigned `command_id`.
- ⚠️ **The migration answer is not the previous one's.** Every historical key for those eleven operations now conflicts instead of replaying — the safe direction — while creation, MCP and node-result keys hash byte-identically and keep replaying. The leaf says which keys break rather than claiming none do.
- **Verified:** 6 suites, 66 tests, zero failures. **Falsified** against the exact unrepaired sources: 38 passed / 1 failed. The same control asserts the committed-replay contract in its last arm, so a hash that merely stopped matching would fail it.

## 2026-09-16 — The qualified run finishes, and the deadline now bounds what it is about (`SIGNOFF-REPAIR.11.4.8`)

✅ **The workspace is green in ONE completed run: 103 suites, 816 passed, 0 failed, 3 ignored** — 94 test binaries plus 9 doc-test targets, under the pinned Chrome for Testing runtime, rc=0, `real 30m15.757s`, with zero skips.

- ⭐ **The arithmetic against the previous measurement is exact.** `SIGNOFF-REPAIR.11.4.7.2.2` measured 815 passed / 1 failed over the same 94 binaries; this run measures 816 / 0. One test moved and nothing else did — the control REPAIR-0202 repaired.
- 🔴 **The first attempt was cut off** at 78 of 94 binaries by `ci_browser.py`'s own `COMMAND_SECONDS = 3600` cap, the first timeout in 16 receipts. The deadline was bounding a COMPILE: the same 94 binaries execute in **1,816.78 s** on a warm tree.
- **Fix, in two places rather than by raising a number:** `make test` and `.github/workflows/rust.yml` run `cargo test --all --locked --no-run` before entering the browser harness, and pass `--timeout 5400` for the workspace run. `COMMAND_SECONDS = 3600` stays the script default for the crate-scoped runs the book documents, which finish in 31 s.
- **The three acquisition phases, measured separately** by watching the harness's own fsynced receipt: download **51.54 s** (191,016,009 bytes, ~3.7 MB/s), extract + binary SHA-256 **0.63 s** (675 entries, 375,683,518 expanded bytes), version check **1.86 s** — **55.27 s** total.
- ⛔ **A cache was refused on that number.** Acquisition is **3.0%** of the run and 93% of it is the download. The per-call fetch provides *these bytes, verified by this process on this invocation*; a cache provides bytes verified earlier plus a re-hash. Nearly the same is not the same, and the price of the difference is 52 seconds.
- ⚠️ **The cold path is not re-measured.** 5400 rests on two observations, not a third run from a wiped tree; macOS's ~21.9 s first-execution validation per freshly written binary stays inside the harness because `--no-run` does not move it. The instrument that settles it is the remote runner — blocker **C1**.
- **Artifact retirement, censused first:** `target/ci-browser/` went from **2.2 G** to **542 M**. Three retained failure payloads were retired to their receipts alone (each carries `archive_sha256` and `binary_sha256`, so the payload is byte-reproducible from the pin); `mac-arm64-0281kcb0` is kept because a tracked evidence record says it remains preserved.

## 2026-09-15 — The suite had been picking its own browser (`SIGNOFF-REPAIR.11.4.7.2.4`)

⭐ **A pin a second path can bypass is not a pin.** The only failing test in 94 binaries was a control qualified against the pinned Chrome for Testing runtime, being evaluated against whatever browser the host happened to have installed.

- 🔴 **The leaf's own question offered two answers and the measurement refused both.** It asked whether the CONTROL over-asserts or the PRODUCT fails to confirm. Neither: `crates/reasonbraid-browse/tests/support/mod.rs::browser_binary()` carried a discovery list, so any run outside `scripts/ci_browser.py` — which is what `cargo test --all` is — silently selected `/Applications/Google Chrome.app/…`.
- **Two-site contrast, one variable, same machine and the same competing load:** desktop Google Chrome 152.0.7977.83 → cleanup unconfirmed, failed 4 of 4, worker elapsed **41.07 s**; pinned Chrome for Testing 153.0.8010.36 → cleanup confirmed, passed 2 of 2, elapsed **31.13 s**. The ten-second difference is the whole 10 s cleanup budget spent on an end-of-file that cannot arrive.
- **Pinpointed at the file descriptor, not inferred.** `lsof` prints the worker's fd 11 and `chrome_crashpad_handler`'s fd 2 as the two ends of one pipe; the handler runs at `PPID 1` in a process group the worker never owned and outlived the worker by **6.24 s**.
- 🔴 **The pinned runtime escapes identically** — two handlers, `PPID 1`, foreign process groups, the same pipe — and differs only in exit latency. The control's green is therefore a latency property of a third-party process, not containment evidence. Annotated at `.7.3.2` with a number rather than left as a passing test's implied claim.
- **Fix:** `browser_binary()` reads `R3_BROWSER_BIN` and nothing else. A run that names no runtime SKIPS the six real-browser controls; the other eleven, including the injected escaped-writer refusal, still run.
- ⚠️ **A skip must never read like a pass.** `libtest` captures `println!`/`eprintln!` and replays them only for FAILING tests. Measured with a one-test `rustc --test` probe: a write through the `std::io::stderr()` HANDLE escapes the capture. Each skip now prints on an ordinary captured run, naming what went unqualified and the command that qualifies it.
- ⛔ **No product code changed, deliberately.** The worker was telling the truth; loosening the assertion would have meant claiming a termination nobody observed, which REPAIR-0089 refused by name. The production worker's own browser discovery is untouched — a deployment runs what its host provides.
- ⭐ **`--no-fail-fast` adopted** in `make test` and `.github/workflows/rust.yml`, on `.11.4.7.2.2`'s census: default fail-fast reached 11 of 94 binaries and left four crates unknown, against 94 for 319.3 s of test execution. A command that answers *is the workspace green* cannot answer it from 11 binaries; a crate-scoped run keeps its early stop.
- **Falsified where it counts:** with the repair in place, naming the desktop browser explicitly still fails (rc=101, same assertion, 41.07 s), so the control still discriminates and the skip is not suppression.
- 🔴 **No completed workspace run is claimed.** `make test` under the pinned runtime reached **78 of ~94 test binaries with zero failures** and was cut off by `ci_browser.py`'s own 3600 s command cap — the first timeout in 16 receipts, 12 of which read `completed exit=0`. Stitching it to the earlier 94-binary run would be exactly what `an-aborted-run-is-not-a-partial-pass` forbids. New owner `SIGNOFF-REPAIR.11.4.8`, which also owns the 191 MB the qualified path re-downloads on every invocation.

## 2026-09-15 — Make B3's deferral trigger evaluable, and find the secret scan could not see the commit it was gating (`SIGNOFF-REPAIR.13.1.2`)

⭐ **A deferral with a trigger nobody checks is an omission with extra steps.** `ACTION-BOUNDARY` makes blocker B3's revisit trigger a script instead of a sentence.

- The gate pins the four facts B3's deferral rests on: `claude.rs::EXEC_ARGS` passes `--restricted` and `--tools` followed by the empty string; `codex.rs::EXEC_ARGS` passes `--sandbox read-only`; `threads::work_payload` dispatches exactly eight keys and no acquired bytes; and every `AdapterCapabilities` declares `tool_support: false` across 5 files and 4 `Adapter` implementers.
- ⭐ **The fourth is pinned here because nothing enforces it at runtime.** `.13.1.1` measured that `verify_ladder` — the five-rung ladder that would refuse a tool-declaring adapter — has no production caller, and `AllowedCapabilities::dev()` permits `tool_support: true` anyway. A weaker guarantee honestly placed beats a stronger one imagined.
- **Falsified five ways against the working tree**, each self-reversing: `--tools Bash`, `--restricted` removed, codex widened to `workspace-write`, an adapter declaring `tool_support: true`, and `work_payload` gaining an `acquired_evidence` field — all rc=1; `git diff --quiet -- crates` and `cmp -s` afterwards.
- 🔴 **The gate's first real run failed on my own parser while its self-test was green.** `payload_keys` ended the function at `\npub `, which does not match `pub(crate) async fn`, so the slice ran to the end of a 5,000-line file and reported 26 phantom fields. Fixed with brace matching; the real-file probe now asserts the EXACT key set instead of "parsed something".
- 🔴 **And registering it exposed a defect in the PREVIOUS commit — mine.** `SECRET-SCAN`, installed by REPAIR-0200, fired on REPAIR-0200's own fixture. ⛔ That leaf recorded `gitleaks detect` rc=0 as its evidence, measured against an **uncommitted** working tree, while the command scans **history** — so it verified the state before its own change, and the value it introduced surfaced at `b6445d1` the moment it was committed.
- ⭐ **The irony is exact:** that leaf's headline lesson was "a rename cannot reach a HISTORICAL finding, because `detect` scans every commit". The same fact cuts the other way — a history scan cannot reach an UNCOMMITTED change — and I took only the half in front of me.
- ⚠️ **The rename was also ineffective on its own terms.** `key-delegation-1` scores entropy 3.578 against a ~3.5 threshold it was never measured against. The value is now `idem-test-000001`: still 16 characters so `baseline == 308` holds, entropy 2.899 so it trips nothing. The fixture now says both properties are load-bearing.
- **Fix:** `gitleaks git --staged` added as a FIRST arm to `SECRET-SCAN` — 0.03 s, and falsified by staging a high-entropy literal (rc=1) then restoring (rc=0). ⚠️ `detect --no-git` was measured and rejected: it walks `target/` and did not finish in 120 s. ⭐ The staged arm means the next such value is one you FIX, not one you permanently annotate.

## 2026-09-15 — Clear the secret scan, and give each supply-chain gate the trigger it actually has (`SIGNOFF-REPAIR.11.4.7.2.3`)

✅ **All three red things are now green.** `gitleaks detect --source . --redact` returns `no leaks found`, rc=0 — and both supply-chain gates now RUN locally instead of living in a CI workflow that has never executed.

- 🔴 **A measurement corrected the plan mid-leaf.** The leaf preferred renaming the fixture over an allowlist entry. ⛔ **A rename cannot reach a historical finding**: `gitleaks detect` scans all 488 commits and attributes a finding to the commit that introduced the line, so after renaming, the scan still returned rc=1 with the same fingerprint. The exact-fingerprint entry is the only instrument that clears it — which is what `.gitleaksignore` exists for and what its two existing entries are.
- **Both repairs taken.** The fingerprint carries its reason inline (a verified test constant, referenced nowhere else); the path and the rule are not suppressed, which that file's header forbids. The fixture is also renamed to the house idiom — it was the lone high-entropy hex among ~10 readable idempotency keys, and the next author copies what they see.
- ⚠️ **The new name is exactly 16 characters, and that is load-bearing.** `delegation_representation.rs:172` asserts `baseline == 308` — a byte count over an envelope containing this field. A different length would have silently moved a measurement instrument's number. The file now says so.
- ⭐ **The commit-time-gate question is settled on what each check is TRIGGERED BY, not on cost.** A commit can introduce a secret, so `SECRET-SCAN` is change-triggered and joins the doctrine gate (1.07–1.15 s over three runs, against a 6.65 s enforcer). An advisory appears against code nobody touched, so `cargo deny` is **time-triggered** — gating it on commits is both too often and too rarely — and it goes in a new `.githooks/pre-push` (1.15 s, advisory DB cached repo-locally).
- ⛔ **Both skip LOUDLY when their tool is absent rather than failing closed**, with a notice naming what was not checked. Failing closed would block every contributor without `gitleaks` or `cargo-deny`; a skip is a weaker guarantee than a pass and must never read like one.
- **Falsified three ways, self-reversing**: fingerprint removed → rc=1; `PATH` stripped of `gitleaks` → rc=0 with `SECRET-SCAN: SKIPPED` on stderr; `rustls 0.23.43` restored → `pre-push` rc=1 printing the advisory and `REFUSED`. Every file proved restored with `cmp -s`.
- ⚠️ **Recorded, not fixed here:** `check_self_tests.sh`'s header publishes the enforcer's cost as "about 3.15 s". Measured today: **6.65 s**. It is another file's number, and correcting it in passing is how a leaf stops being bounded.

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

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated nineteen times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

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
