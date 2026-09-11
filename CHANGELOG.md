# CHANGELOG.md

## 2026-09-12 — Assess and sequence the verification strategy (`SIGNOFF-REPAIR.11.5`)

- All four proposals accepted in principle and sequenced rather than started, with one new lane added ahead of three of them: a clean-state lane, then the failed-first control doctrine, then the adversarial concurrency lane, then the pipeline-stage registry, then the deterministic simulator.
- The evidence strengthened after the proposal was written: ten defects repaired in total, none of them a wrong return value and none reachable by MCP. `REPAIR-0086`'s invariant control fails against the superseded design while every conformance scenario still passes, which is the failed-first proposal demonstrated.
- The new lane goes first because two of the three "remote-only" failures needed only a CLEAN machine, not a remote one, and each was reproduced by one command after costing CI round-trips.
- MCP's placement is unchanged and better evidenced: a conformance lane, never the correctness lane.
- Recorded as `docs/decisions/2026-09-12_verification-strategy-assessment.md`.

## 2026-09-12 — Write every conformance stub before any scenario can spawn (`SIGNOFF-REPAIR.11.4.3.1.2.26`)

- Remote CI reported `failed to spawn …/conformance-stubs/codex-14645/codex: Text file busy` — intermittently, since the same suite passed in the run immediately before.
- This corrects the earlier repair's own record. Giving each adapter kind its own `OnceLock` serialises each stub against itself and nothing else, so its claim of "no window at all" was false: `execve` refuses a file open for writing anywhere, and `fork` copies the descriptor table, so the thread forking to spawn one stub inherits the open write descriptor of another stub being written.
- One `OnceLock` now holds every stub. `get_or_init` blocks all other threads until it returns, so no scenario can hold any stub path — and therefore cannot spawn — until every write has finished.
- Verified by invariant rather than symptom, because macOS does not enforce `ETXTBSY` at any timing. A new control asserts that holding either stub path implies every stub is written and executable; it FAILS against the superseded design while all three scenarios still pass, which is exactly why the race shipped.

## 2026-09-11 — Enforce storage locality and prove fixture names (`SIGNOFF-REPAIR.11.4.3.1.2.21`)

- §13 says project-owned data lives on the repository's own volume and that a name must be proved, not proposed. It was prose for the life of the project and was breached in 26 places: 8 ambient temporary directories and 18 clock-derived paths across 16 files. The census found 18, not the "roughly ten" the finding estimated.
- All repaired. Fixtures now take a v7 UUID and create their leaf directory EXCLUSIVELY while the shared parent stays tolerant of an existing one. `backup_restore.rs` also had a FIXED directory name shared by every run, with a clock supplying the only uniqueness in the file name.
- Why it mattered beyond policy: these directories persist between runs (`target/journal-tests` held 954 MB of residue) and `create_dir_all` adopts. With a clock measured at 501 distinct values in 2000 calls, collision with a PRIOR run's directory was likely rather than remote — so a test could open a previous run's SQLite database and call the result a pass.
- New `STORAGE-LOCALITY` gate refuses both patterns across all tracked Rust, with a verbatim reasoned allowlist where a stale entry is also a breach. It reports 195 files clean with one reviewed exception — a duration measurement that reaches no path.
- 60 node tests, the release-tool suite, strict lint and all gates pass.

## 2026-09-11 — Stop racing for the guard fixture's parent (`SIGNOFF-REPAIR.11.4.3.1.2.25`)

- `pg_guard.rs` created its control parent only when the path was absent, then unwrapped the result. That is a check-then-act, and these fixtures run in parallel threads, so all but one loser panicked with `AlreadyExists`.
- It was never Linux-only. A probe of the exact shape lost 346 of 640 creations on the development machine. It had never fired in a local suite because `target/` stays warm between runs, making the racing branch dead code; a fresh checkout runs it every time. Removing the control directory reproduces the identical CI failure here.
- The fix accepts `AlreadyExists`, as every other shared-parent creation in the workspace already does. Six consecutive cold-tree runs pass; the unrepaired code fails the first.
- Promoted to `TOOLBOX.md`: "remote-only" is a hypothesis, not a category — two of today's three remote-only failures were reproduced locally by removing accumulated local state.

## 2026-09-11 — Own the Git acquisition workspace (`SIGNOFF-REPAIR.7.2.1`)

- The production R1 acquisition named its working directory from the process id and a nanosecond field in the ambient temporary directory, then created it with a call that adopts an occupied path. Reproduced verbatim: 2000 names gave 501 distinct values with every collision between adjacent calls, `create_dir_all` returned `Ok` over another acquisition's pack, and the error path then deleted it. Two concurrent acquisitions share the process id by construction.
- A second defect the census had not named: nothing removed the directory on SUCCESS, so every successful acquisition leaked a bare repository.
- New `project_storage` module holds the storage rules `extraction_input` had proved — repository-root discovery, `.project-data/<area>` at 0700, checked parents — now shared instead of duplicated, plus an `OwnedDirectory` that creates exclusively and proves identity before removing. The acquisition owns its workspace and releases it when the last holder drops.
- Measured rather than assumed: an open descriptor pins a directory's inode, but macOS keeps reporting two links after `rmdir`, so the file owner's link-count check is deliberately not reused for directories.
- 97 server lib tests pass including five new controls; `.project-data/git` holds no leftovers; strict lint and the rendered book pass.

## 2026-09-11 — Give the git fixtures their own commit identity (`SIGNOFF-REPAIR.11.4.3.1.2.24`)

- The last known remote `check` failure: four `git::tests::*` with `the commit writes: AuthorMissing`. `repo.commit` resolves its signature from git configuration, so the fixtures borrowed whatever identity the developer's machine carried. They could never have passed on a clean machine.
- Reproduced locally by suppressing ambient git configuration, which removes CI from this loop and immediately exposed a fourth commit site the first pass had missed. The probe is recorded in `TOOLBOX.md`.
- All four commits now pass an explicit `fixture_identity`, so the fixture owns its identity the way it owns its storage. Six git tests pass both with ambient configuration suppressed and with it present; the unrepaired fixtures fail under suppression.

## 2026-09-11 — Prove input identity against the descriptor (`SIGNOFF-REPAIR.11.4.3.1.2.23`)

- The browser repair landed on Linux: every `reasonbraid-browse` control passes, with `render_succeeded: true` and a worker reporting `stderr_bytes: 0` where it had reported 403.
- Linux CI then found a defect in REPAIR-0063's own repair. `OwnedInput::verify` compared a stored `(device, inode)` pair, and Linux reuses an inode number as soon as it is freed, so a file deleted and immediately replaced presented the same pair and `release` returned success for a successor it never created — the exact deletion the check exists to prevent.
- A probe confirms both halves on the development platform: an open file reports one link while linked and zero after unlink, and macOS hands the successor a different inode, which is why the defect was invisible locally.
- The owner now holds its open handle for life and refuses when the descriptor reports zero links, so identity no longer depends on a number the kernel may hand out again. Nine owner controls, strict lint and format pass.

## 2026-09-11 — Fit Chrome's singleton socket in its path budget (`SIGNOFF-REPAIR.11.4.3.1.2.22`)

- Chrome aborted on Linux with `FATAL:process_singleton_posix.cc:313] Socket path too long`: 228 bytes against a 108-byte `sun_path`. It places that socket under the temporary directory precisely to keep the path short, and the worker's absolute per-invocation TMPDIR defeated the mitigation.
- Arithmetic chose the fix before any code was written. The socket suffix is 45 bytes, leaving TMPDIR 63: today's absolute path measures 183, a short temp directory under the existing fixture 132, and a short temp directory under a shortened fixture 67 — still over. Shortening names cannot fix it, because the harness gives each command its own fake repository root and the worker derives storage from it.
- `TMPDIR`, `TMP` and `TEMP` become the relative `tmp`. The child's working directory is already the workspace, so it resolves where the absolute value did, at 3 bytes instead of 183. Every other variable stays absolute, so the isolation the controls assert is unchanged.
- It cannot regress: if Chrome canonicalises TMPDIR it resolves against that same working directory and reproduces today's exact absolute path. Sixteen browser integration controls, five production lifetime controls, strict lint and format pass locally — which is explicitly not evidence for the Linux singleton path, since macOS does not use it.

## 2026-09-11 — Root-cause the browser CI failure: a socket path 120 bytes over the limit

- Chrome's own stderr, once the instrument finally delivered it: `FATAL:process_singleton_posix.cc:313] Socket path too long`, then `Received signal 6`. The socket path measures 228 bytes against a `sun_path` capacity of 108.
- The missing-runtime-library hypothesis is DENIED. Every observation had been consistent with it; only Chrome's own words separated the two.
- Mechanism: the worker overrides TMPDIR into a per-invocation workspace nested under a per-command fixture, and Chrome creates its singleton socket under TMPDIR precisely to keep that path short. The isolation design defeated the vendor's mitigation. macOS never reaches that code path, so no local run could see it.
- Repair owned by `.11.4.3.1.2.22`: a short repository-derived temporary directory plus an explicit startup budget check that refuses by name instead of allowing a FATAL abort.

## 2026-09-11 — Correct the browser evidence instrument (`SIGNOFF-REPAIR.11.4.3.1.2.20`)

- REPAIR-0074's upload returned byte COUNTS and no bytes: the artifact held only `worker.json` per fixture, reporting `stderr_bytes: 403` while the 403 bytes themselves stayed on the runner.
- The glob named `browser.stderr`, `owner.json` and `completion.json` — filenames from the book's description of the PRODUCTION worker's storage. The test harness writes `stdout.log` and `stderr.log` instead. The filenames were inferred rather than read from the code that writes them.
- The upload now retains `target/browser-lifetime-controls/**` and `.project-data/browser/**` whole. The fixtures are a few hundred bytes each, so filtering bought nothing and cost the evidence.
- What the counts do establish: the worker exits 0, writes 403 bytes of stderr and 108 of stdout, and confirms group cleanup — so it ran correctly and reported `browser_launch_failed` itself. The browser's own reason remains unproved.

## 2026-09-11 — Retain the browser worker's own stderr in CI (`SIGNOFF-REPAIR.11.4.3.1.2.20`)

- The conformance repair held and `pg-tests` succeeded remotely for the first time — the full PostgreSQL collection on a runner. The `check` job now fails in `reasonbraid-browse`: six tests with `browser_launch_failed`, "browser exited before publishing a loopback endpoint".
- Two candidates are already ruled out by the same log: the launcher verified the pinned executable's exact version, so the binary runs, and `--no-sandbox` and `--headless` are already passed.
- The worker retains up to 64 KiB of Chrome's own stderr for a failed invocation, and the workflow was discarding it. One `if: failure()` upload now keeps it, so the next run carries the cause instead of the symptom. The cause remains unproved and no repair of it is claimed.

## 2026-09-11 — Write each conformance stub once (`SIGNOFF-REPAIR.11.4.3.1.2.19`)

- The instrument from REPAIR-0072 did its job: the next remote run named the cause — `failed to spawn .../conformance-stubs/lose-14317-1/claude: Text file busy (os error 26)`. Linux `ETXTBSY`: `execve` refuses a file still open for writing, and one thread writing a stub while another forks to spawn hands that child the open write descriptor.
- REPAIR-0072's stub-naming change was NOT the cause, and the evidence says so: the failing path already carried its pid-and-counter naming, and the failure moved from `codex` to `claude`. That leaf deliberately said "measured defect, not a proved cause", which is what makes this a correction rather than a retraction.
- The two stub scripts branch on the prompt, so the per-scenario copies carried no information and only created the window. `stub_once` now writes one stub per adapter kind per process, with a `OnceLock` publishing the path only after the write and chmod complete. The race is removed, not retried around.
- All adapter targets pass locally, which is explicitly not evidence about the Linux behaviour being repaired; the remote run is the measurement.

## 2026-09-11 — Make a conformance refusal name its cause (`SIGNOFF-REPAIR.11.4.3.1.2.19`)

- The project's first remote CI run failed: `doctrines` and `supply-chain` pass, `rust` fails at `codex_adapter_passes_the_conformance_suite` with "the lose trigger refused instead of dispatching". The suite passes locally and the Claude scenario passes on the same runner.
- The diagnosis was blocked by the harness itself: `InvokeOutcome::FailedBeforeDispatch` carries a `reason`, and the certification discarded it at three arms, emitting a fixed sentence. The whole CI log contained no cause, because none was ever produced. The three arms now carry the adapter's own reason.
- Separately and on its own evidence, the conformance stubs no longer name their directory from the clock: scenario names repeat across adapters, `create_dir_all` succeeds on an existing directory, and 2,446 local stub directories ending in `000` confirm the resolution. This is the fourth instance of that family today.
- The remote cause remains UNPROVED. The stub-naming repair is made because it is a measured defect, not because it is demonstrated to be the cause; the next remote run is the measurement.

## 2026-09-11 — Report evidence storage faults honestly (`SIGNOFF-REPAIR.7.4.2`)

- Ten `map_err` arms across `snapshots.rs`, `derivations.rs` and `claims.rs` collapsed every storage fault into a caller error, and `api.rs` rendered all of them as HTTP 400. A server-side failure told the caller its own reference did not exist.
- Each enum gains `Storage(sqlx::Error)` with the source reachable through `Error::source`, matching the `GrantCreateError` contract; the handlers use the existing `internal_with_log` idiom so the cause is logged server-side and the wire keeps its safe generic message.
- The R2 pipeline no longer discards a failed snapshot while still reporting a successful acquisition: it records an `evidence_unstored` acquisition error and returns no receipt.
- The control installs a trigger that raises on insert, drops it before asserting so a failure cannot leave the shared database rejecting snapshots, and requires 500 rather than 400 — with a companion assertion that an absent reference stays 400. Against the unrepaired source it fails printing the defect verbatim.

## 2026-09-11 — Repair three dead book links and check the rest (`SIGNOFF-REPAIR.11.4.3.1.2.18`)

- `docs/book/book/cli.html` rendered `href="docs/book/src/cli-state.html"`, a page that does not exist. A census of the whole book found 29 intra-book links with exactly 3 broken, all in the CLI chapters.
- The cause is worth recording: `DOCPATH` requires repo-root-relative references, an author applied that to intra-book navigation, and mdBook resolves a link relative to its own page. The doctrine's intent was satisfied and the navigation broke — in the surface the director reads.
- Register `BOOK-LINKS` so the book's own navigation is enforced rather than assumed. External URLs stay out of scope deliberately: a network call in a commit hook is a flake generator. The negative control reintroduces the exact original link and is detected.

## 2026-09-11 — The full pre-push checkpoint passes (`SIGNOFF-REPAIR.11.4.3.1.2`)

- All eight checkpoint commands return 0 at source `7233122`: build, format/strict lint/workspace tests with the pinned browser, Python controls, the full owned PostgreSQL collection with `--demo`, thirteen doctrines, pinned cargo-deny, pinned Gitleaks and the book. Every earlier attempt stopped somewhere.
- Re-derived from the receipts rather than the exit code: 40 of 40 registered suites, 291 tests passed and 0 failed, the demonstration's `ALL acceptance checks passed`, and both scanners recording `scope: gate` with `exit_code: 0`.
- Falsified before publishing: zero DATABASE_URL skips, 16 real browser controls instead of an absent-browser early return, and only the deliberately env-gated live-provider dispatches ignored.
- This satisfies the condition on the already-authorized push. It closes no external gate: G6/G7, name clearance and the license decision remain open, and five repair leaves remain open including the unexplained checkpoint wall time.

## 2026-09-11 — Mint evidence identifiers that are actually distinct (`SIGNOFF-REPAIR.7.4.1`)

- `snapshots.rs`, `claims.rs` and `derivations.rs` each minted durable evidence identifiers from `format!("{:x}{:x}", nanos, pid)` — a function literally named `uuid_like_suffix`, resembling a UUID in shape and not in the one property a UUID is for. A probe of that exact expression measured 8 collisions in 10 sequential calls, 918 in 1,000, and 269 among 400 across eight threads: about one distinct value per twelve calls.
- Establish the consequence rather than assume it: all three columns are `TEXT NOT NULL PRIMARY KEY`, so a collision is a refused insert and no stored row can hold another's identity. The damage is that every storage failure is then reported as `ReferenceMissing` and mapped to HTTP 400, blaming the caller's input, while the R2 pipeline discards the failed snapshot and still reports a successful acquisition.
- Replace all three with one `evidence_id` minting `uuid::Uuid::now_v7()` — time-ordered like the old shape, distinct by construction. Two controls measure the property directly: 400 concurrent and 1,000 rapid sequential identifiers, all distinct.
- The storage-failure misclassification is diagnosed and routed to `.7.4.2` rather than fixed in passing: it changes three error types and their wire mapping and deserves its own injected-fault qualification.

## 2026-09-11 — Name guard fixtures without a clock (`SIGNOFF-REPAIR.11.4.3.1.2.17`)

- The source-5c8609e checkpoint stops at `02-check`: `pg_guard` panics creating its fixture directory because `Fixture::new` names it `{pid}-{nanos}`, six consecutive `time_ns()` samples on this host are byte-identical, and three tests call it in parallel threads of one process. The PostgreSQL runner's `--test-threads=1` is why this suite always passed there and only fails under `make check`.
- Replace the clock with an `AtomicU64` discriminator and skip an occupied candidate instead of panicking on it: 0 failures in 60 parallel runs against 1 in 15 before, with `Drop` still removing every directory.
- Census the family, since this was its third instance. Two production findings, both with concrete owners: `snapshots.rs`/`claims.rs`/`derivations.rs` mint durable evidence identifiers from the same clock-and-pid shape — a probe of the exact expression measures 918 collisions in 1,000 calls and 269 among 400 across eight threads, about one distinct value per twelve calls (`.7.4.1`) — and `git.rs` builds four ambient temporary paths from the process id alone (`.7.2.1`).

## 2026-09-11 — Gate file termination (`SIGNOFF-REPAIR.11.4.3.1.2.16`)

- Register `FILE-TERMINATION`: every tracked text file ends with exactly one newline. Deliberately not a `git diff --check` wrapper — `ROADMAP.md` uses trailing double-spaces as Markdown hard line breaks, so that check's whitespace family has a legitimate use here and a blanket rule would teach bypass. A blank line at end of file does not, and is the defect that forced a correction commit.
- Census first: 642 tracked text files, 9 non-conforming. Two would have been damaged by a naive fix — the schema goldens are produced by a writer that emits no trailing newline, so the WRITER is corrected and the goldens regenerated from it; the benchmark corpus is hashed and published as `prompts_digest`, so it is the one reviewed exception with its reason recorded.
- Three negative controls each detect their defect (new blank line, stripped terminator, stale exception) and the restored tree passes; `--self-test` covers eight classifications including a Markdown hard break and a binary file.

## 2026-09-11 — Restore the recreated schema's grant and refuse non-operators honestly (`SIGNOFF-REPAIR.11.4.3.1.2.14`)

- Root-cause the full checkpoint's stop at `04-pg-demo`: `migration_upgrade` recreates `public` with `DROP SCHEMA … CASCADE; CREATE SCHEMA public`, and a manually created schema does not inherit the default ACL a fresh database ships. A direct catalogue comparison shows the failing database's `{postgres=UC/postgres}` against a pristine `{pg_database_owner=UC/…,=U/pg_database_owner}` — PUBLIC's `USAGE` is gone, so every later non-owner role cannot resolve a qualified name.
- Restore the owner and the PUBLIC grant through one helper used by all four recreate sites: a fixture that mutates database-wide privilege state owns restoring it, as the cleanup plans already do for rows.
- Resolve the site privilege probe's audit table by catalogue OID instead of a qualified name, so a caller without schema `USAGE` is refused `OperatorRequired` (403) rather than `Error::Sql` (500 `dependency_unavailable`). This was a misclassification, not an escalation — the forensic copy shows the outsider never held operator membership.
- Make three opaque refusal assertions report the value they received; the reproduction then named SQLSTATE 42501 immediately. The new control is falsified against the unchanged production query, restores the shared grant before asserting, and proves its own precondition held.
- The reproduced two-suite sequence and the affected family of six suites (30 tests) pass with the cluster stopped and removed. The full checkpoint has NOT passed: four suites, the demonstration and gates five to eight remain unrun.

## 2026-09-11 — Bind R2 responses to owned input bytes (`SIGNOFF-REPAIR.7.3.3.3.2`)

- Replace the R2 handler's ambient temporary-directory span with one bound call: the acquired bytes become a private owned input, the worker reads it, and the input is released only when no reader can still hold it. No `std::env::temp_dir()` use remains in the R2 path, including the adjacent spawner fixture.
- Refuse a response whose `parent_digest` is not the digest of the bytes this request supplied, with the named `ExtractionError::SourceMismatch` and the wire kind `extraction_source_mismatch`, before any snapshot, derivation or receipt is written. The enum stays exhaustively matched, so the handler could not compile without mapping it.
- Seven integration controls pass, including the mismatch refusal naming both digests, a preserved `feed_unreadable` refusal, a worker failure keeping its classification, and eight concurrent callers each receiving a receipt for their own document. 90 server library tests, 16 completion controls, 12 adjacent extractor tests, strict lint and format pass; the live adjacent `profiles` suite passes 31 tests.
- State one gap with its census and give it an owner: no live test drives a SUCCESSFUL R2 acquisition through to a snapshot and derivation, because the live R2 test refuses at the loopback destination gate. `.7.3.3.4` owns closing that join with a policy-allowed local origin.

## 2026-09-11 — Own the extraction input exclusively (`SIGNOFF-REPAIR.7.3.3.3.1`)

- Create one private 0600 extraction input per request under a runtime-discovered repository root, with checked owned same-volume parents, bounded exclusive candidate allocation and no temporary-directory or home fallback. An occupied candidate is skipped whole — never opened, truncated or adopted.
- Gate removal on both a finished reader (`WorkerCompletion::reader_finished`, added beside `is_consumed`) and an unchanged (device, inode, one link) identity; retain anything else with its repository-relative path named. The store removes one file it created, or nothing.
- Expose the written bytes' digest in the worker's own `sha256:<hex>` form so a response can be bound to its source. Eleven controls pass, including 32 simultaneous creators holding 32 distinct documents and the real worker describing an owned input by its own digest.
- One control was itself racy on ambient worker selection. A widening probe and the failed run's own retained input identify it; both controls now serialize selection and 40 repeated runs pass. `api.rs` is unchanged and still carries the superseded span until `.7.3.3.3.2`.

## 2026-09-11 — Mechanize the visibility policy and drop the duplicated book frontier (`SIGNOFF-REPAIR.11.4.3.1.2.13`)

- Correct the two places where the superseded private-visibility instruction survived two hand-run censuses: the book's introduction and the governance charter's single-owner clause. The accepted director correction is unchanged; no visibility, remote or release setting moves.
- Remove the book roadmap page's second copy of the corrective frontier — it named a leaf seven committed leaves after that leaf closed — and route the reader to the per-leaf maintained qualification page instead of re-synchronizing a duplicate.
- Add two registered checks with self-tests and negative controls: `VISIBILITY-POLICY` reviews every private-visibility sentence against a verbatim allowlist (a new, reworded or stale entry breaches), and `BOOK-FRONTIER` refuses a book page naming a frontier the task tree does not. Both were falsified by reintroducing the exact defects they exist for.

## 2026-09-11 — Own direct extraction worker completion (`SIGNOFF-REPAIR.7.3.3.2.2`)

- Permanent process-fact controls reproduce the defect on the unchanged spawner: `7 passed; 1 failed`, an early request failure returning while its direct worker was still in the process table. The controlled worker closes stdin and waits on an explicit release; an 8 MiB media type forces the real `EPIPE` write path.
- One bounded stop and reap now owns every exit path (500 ms grace, zero grace after a tripped budget, five-second cap). Every return reports never-started, consumed or explicitly unconfirmed completion, and a failed stop request is recorded rather than read as a termination. Two piped-handle panics became typed refusals and the timeout message dropped a kill it had not confirmed.
- 16 process/evidence controls, 6 spawner controls (four synthetic injections), 81 server library tests, 12 adjacent extractor tests, strict server lint and workspace format pass; `api.rs` is byte-identical and the error vocabulary is unchanged. Exclusive same-volume inputs and digest-bound cleanup remain `.7.3.3.3`; pipes, descendants and aggregate storage remain `.7.3.4`. No HTTP, database, full-CI or push claim.

## 2026-09-11 — Preserve the requested clean handoff (`SIGNOFF-REPAIR.7.3.3.2.1`)

- Record the direct-worker completion diagnostic plan and pending implementation child before any new probe or production edit. PNT is paused at the director's request; resume .7.3.3.2.2 from durable task/live pointers.
- Preserve all prior evidence and the private unsent OpenAI Support report. Source/book behavior and qualification categories remain unchanged; documentation checks and the native handoff census are the relevant gates.

## 2026-09-10 — Diagnose production extraction input interference (`SIGNOFF-REPAIR.7.3.3.1`)

- Exact production acquisition span and unchanged worker reproduce six shared paths and eight wrong-owner results among 32 callers; two own-input controls pass. Preserve all 24 files, original identities and failed diagnostic attempts. Production code is unchanged; HTTP/database effects remain untested.
- Own staged direct-worker completion and exclusive input/digest integration before the full checkpoint. Keep broader pipe, descendant and retention bounds explicit; annotate the historical Phase 4 claim and book.
- Prepare the director-requested private OpenAI Support report with available correlation metadata and clearly missing fields; preserve a redacted incident record without publishing session identifiers or submitting the report.

## 2026-09-10 — Own extraction fixture files exclusively (`SIGNOFF-REPAIR.11.4.3.1.2.12`)

- Preserve the failed PDF checkpoint and reproduce wrong-owner data with both the unchanged extractor executable and the exact original helper/native clock.
- Replace timestamp/truncating unit and stdio fixtures with exclusively created, repository-local inputs retained through assertions. Add concurrent ownership, existing-file/link refusal and panic/replacement controls without changing parser behavior or dependencies.
- All twelve selected tests, strict extractor lint/format and three independent locality cases pass; original assertions, parser and 339 other non-Markdown sources are unchanged. Keep historical attribution limits and concrete production/fixture follow-ups. Full checkpoint/public push remain pending.

## 2026-09-10 — Explicitly release state-writer locks (`SIGNOFF-REPAIR.11.4.3.1.2.11.2`)

- Install release ownership immediately after flock acquisition and explicitly unlock before File close, including errors/unwind. Preserve nonblocking exclusion, snapshots and synchronization; correct two successful test probes with the same lifetime.
- A permanent five-path inherited-descriptor regression fails on unchanged production and passes after correction. All 32 selected CLI tests, the final twelve-writer rerun and six actual-API raw-fork scenarios pass; strict CLI lint and format pass.
- Preserve all original assertions and 339 other non-Markdown sources. Abrupt owner death with surviving inherited references remains owned by broader restart qualification; full checkpoint/public push remain next.

## 2026-09-10 — Reproduce inherited state-lock retention (`SIGNOFF-REPAIR.11.4.3.1.2.11.1`)

- Preserve source-8d1504d's failed workspace gate: eight other gates pass, PostgreSQL/demo unstarted. Original sources and executable copies match; unchanged concurrent/isolated/serial reruns pass.
- A controlled actual-CLI probe proves close-only retention across success, HTTP error and cancellation when a forked child retains the descriptor. Three no-child controls release immediately; three child exits release retained locks. Production remains unchanged; .2.11.2 owns the fix and permanent regressions.
- Independent verification checks all 341 source files and fifty absent process groups. Preserve preliminary observer failure and original evidence. Scope original-cause attribution and inherited-descriptor process-loss limits explicitly.

## 2026-09-10 — Complete explicit fixture-plan coverage (`SIGNOFF-REPAIR.11.4.3.1.2.7.4`)

Migrate the final five fixtures to checked explicit cleanup, preserving each
original scope and assertion. All 25 original plans are covered; the first
consumer sequence passes 22 tests and the consecutive affected collection passes
169 distinct tests. Strict server/MCP/CLI lint, format, book and source/process
checks pass. Both successful databases are removed and historical failure
evidence preserved. Production/schema remain unchanged; the full checkpoint
is next before public push and remote CI.

## 2026-09-10 — Complete six partial fixture plans (`SIGNOFF-REPAIR.11.4.3.1.2.7.3`)

Reproduce each original cleanup failure after real node-work residue. Add only
its required FK dependencies and adopt checked static plans for cards, quota,
classification, mcp_listen, federation and quarantine. Preserve original table
order, feature assertions and deployment CA. All six repaired producer/consumer
pairs and the consecutive consumer run pass; preserve six failed baselines and
remove seven successful databases. Five remaining explicit plans retain separate
coverage ownership before the full checkpoint. Strict server lint, format, book
and source/process verification pass; production/schema stay unchanged.

## 2026-09-10 — Remove the retention fixture's calendar dependency (`SIGNOFF-REPAIR.11.4.3.1.2.9`)

Derive expiry instants from recorded snapshot creation time. Qualify strict
one-day/thirty-day boundaries, exact tombstone counts, repeated no-change behavior,
audit-class preservation and unchanged retention age on replay. The focused
retention test and all 31 profiles tests pass; preserve the original failures and
historical caller evidence. Strict server lint, format, book and independent
source/process verification pass. Production retention policies remain unchanged.

## 2026-09-10 — Match CLI removal delegation to tenant administration (`SIGNOFF-REPAIR.11.4.3.1.2.10`)

Correct participant removal's delegated scope while retaining single-thread scope
for ordinary existing-thread commands. Real rb controls reproduce the old failure
and reject a deliberately overbroad mutation before exact restoration. All five
CLI, six adjacent server invitation and eleven library tests pass, as do strict
CLI lint, format, book and source/process checks. Preserve expected failures and
historical caller evidence; retention and remaining fixture repairs stay pending.

## 2026-09-10 — Restore authorized participant removal (`SIGNOFF-REPAIR.11.4.3.1.2.8`)

Correct the handler’s authorization target to Tenant for its existing TenantAdmin
action. Keep tenant-wide authority and locked tenant/thread binding. New real HTTP
controls reproduce the old audit target, then verify allowed removal, exact
no-effect authority/domain refusals and preserved committed-denial replay. All
six invitation tests plus 22 authority and 33 command-API tests pass. Preserve
historical evidence and own the CLI delegation-scope companion under .2.10. Ten
evaluator controls, strict server lint, format, book and independent source/process
checks pass; three successful databases are removed and failure evidence retained.


## 2026-09-10 — Migrate fourteen checked cleanup callers (`SIGNOFF-REPAIR.11.4.3.1.2.7.2`)

Add missing MCP-listener/CLI-breaker dependencies and explicitly declare existing
resource-cascade children. Preserve original cleanup order, unrelated state and
feature assertions. Real predecessor→identity/CLI sequences pass; all fourteen
cleanup callers execute successfully. The affected census records 135 passing
assertions and two failures, both reproduced with original fixtures: participant
removal has a tenant-action/thread-target mismatch, and a retention test uses an
expired calendar assumption. Own their immediate repairs under .2.8/.2.9; retain
the four failed databases and all results. Strict server/CLI lint, format, book
and independent scope/process checks pass; production code remains unchanged.


## 2026-09-10 — Check complete fixture cleanup plans (`SIGNOFF-REPAIR.11.4.3.1.2.7.1`)

Reproduce MCP-listener, spend-breaker and incarnation residue failures in actual
producer/consumer suites. Add a shared test-only checker that validates canonical
table names, supported relations and complete dependency order before deletion,
including cascade/null/default effects. Preserve typed errors, unrelated rows and
honest late-error partial effects. All eight guard tests and strict server lint
pass. Existing fixture callers remain unchanged for the next migration children;
production code/schema and the failed full-checkpoint status remain unchanged.

## 2026-09-10 — Repair certified-node fixture residue (`SIGNOFF-REPAIR.11.4.3.1.2.6`)

The b0cddfe checkpoint passes nine gates, then fails at identity fixture cleanup
after thirteen live PostgreSQL suites pass. Reproduce the fresh-versus-node-work
ordering failure and the exact restrictive certificate FK. Remove certificate
children before nodes and add a regression that preserves the FK refusal, then
checks complete hierarchy cleanup. The regression, four fresh identity tests and
eight node-work plus four identity tests pass; strict server lint, format and book
pass. Preserve all failed databases/logs. Production behavior is unchanged; MCP
listener and CLI spend-breaker fixture dependencies have prerequisite owner .2.7.
Full checkpoint, public push and remote CI remain incomplete.

## 2026-09-10 — Pin the local and CI browser runtime (`SIGNOFF-REPAIR.11.4.3.1.2.5`)

Make test/check and the Rust workflow now use an exact verified Chrome for Testing
runtime with private local installation, bounded phases and retained failure
evidence. Four platform archives are qualified; fresh native setup and all sixteen
browser tests pass. Repair a reproduced shared Python shutdown race: transient
zombie-group denial requires bounded reaping and confirmed absence. All 67 Python
controls and final wiring/book/residue checks pass. Preserve the diagnostic failures;
full checkpoint and public push remain next. README shrinks; production Rust is unchanged.

## 2026-09-10 — Qualify browser phase witnesses (`SIGNOFF-REPAIR.11.4.3.1.2.4`)

Reproduce the full checkpoint's navigation/overlap timing failures with a controlled
startup delay. Require explicit gated arrivals, retain delayed simultaneous-profile
proof and record dispatch errors before observer panics. The stronger witness
exposes detached desktop Chrome updater/crash-report stderr writers; preserve the
real cleanup refusal and qualify a dedicated testing runtime without changing the
production worker. Two delayed controls, all sixteen integration controls and strict
lint pass. Pinning/CI binding is the next owned prerequisite; full CI remains pending.
Correct the CI guide's missed private-visibility sentence to the public policy.

## 2026-09-09 — Qualify exact history-fixture exceptions (`SIGNOFF-REPAIR.11.4.3.1.2.2`)

Trace both scanner matches to predictable metadata-only test literals. Exclude
only their immutable commit/file/rule/line fingerprints. Five native controls
recover each omitted finding and detect identical content in a new commit; the
actual pinned history scanner passes. Preserve the diagnosed additive-ignore
probe failure and all results; no file/rule exclusion, Rust change or history rewrite.

## 2026-09-09 — Keep the repository public (`SIGNOFF-REPAIR.11.4.3.1.2.3`)

Apply the director correction that README’s private instruction was wrong: the
project is public and must remain public. Synchronize ADR/companion/security/risk
guidance, live records and the book while retaining historical checkpoint evidence.
Public Git is not a confidential embargo channel. No visibility or production
change. Read-only remote, guidance, rendered-book and scope checks pass; README
shrinks to 2,033 bytes / 52 lines. Resume history repair and full checkpoint before push.

## 2026-09-09 — Contain the publication-precondition conflict (`SIGNOFF-REPAIR.11.4.3.1.2.1`)

Independently confirm the remote is public despite the private-repository policy.
Block publication and record the director decision plus exact history-scan repair
owners. Preserve passed format/dependency gates, two redacted scanner findings and
the intentionally interrupted Clippy result; all process cleanup is consumed.
Correct current-state documentation and retain the full checkpoint as incomplete.
No visibility change, push, scanner exemption or production change is made.

## 2026-09-09 — Retire verified obsolete compiler sessions (`SIGNOFF-REPAIR.11.4.3.1.6`)

Qualify the pinned macOS compiler locks with native protected-session and
regeneration controls. Remove 645 frozen, obsolete sessions / 1,984 files /
5,116,558,334 logical bytes under verified exclusive locks; preserve newest,
young, partial, hard-linked and evidence-bearing data. Exact residue, unchanged
retained source/evidence, affected server build and book checks pass. All results
are consumed; preserve the failed phase assumption and sampler timeout. Production
bytes are unchanged; the full local/remote checkpoint remains next.

## 2026-09-09 — Qualify browser storage and origin ownership (`SIGNOFF-REPAIR.11.4.3.1.5.3`)

Verify real rendering after moving the runtime root and refusal of linked storage.
Gate concurrent navigation explicitly instead of relying on a short timing window.
A native successor-listener control falsifies the old port-reachability assertion;
require the original listener's close receipt and consumed serving task. Fifteen
integration controls, strict lint and native/source/book checks pass. Production
bytes are unchanged; failed evidence and broader qualification owners remain open.

## 2026-09-09 — Own production browser lifetimes (`SIGNOFF-REPAIR.11.4.3.1.5.2`)

Reproduce a renderer outliving a successful worker. Give each invocation private
repository-derived profile/cache/diagnostic storage and explicit process/task
ownership through cancellable startup, rendering and bounded shutdown. Preserve
named render refusals; return browser_cleanup_unconfirmed when cleanup fails.
Five unit/thirteen integration controls, strict lint and native/source/book checks
pass. Keep failed-run evidence and explicit parent/container/retention follow-ups;
R3-enabled deployment now requires a repository working directory.

## 2026-09-09 — Bound browser verification lifetimes (`SIGNOFF-REPAIR.11.4.3.1.5.1`)

Give browser tests private local fixtures, bounded I/O/output/process groups and
consumed origin shutdown. Budget admission runs without a browser; rendering
keeps explicit qualification boundaries. Eight controls and strict lint pass.
A real cleanup refusal exposed transient Darwin zombie-group EPERM behavior;
native reproduction and bounded transient/persistent observation controls preserve
strict absence checks. Keep the original failed fixture. The worker still requires
supervisor assistance after rendering; production lifetime repair remains next.

## 2026-09-09 — Isolate publisher test directory ownership (`SIGNOFF-REPAIR.11.4.3.1.4`)

Reproduce a second test process deleting a still-live owner's fixture. Replace
counter-based removal with exclusive private creation, checked directory identity
and explicit cleanup after gix handle closure; retain incomplete evidence. Five
publisher/ownership controls, independent helper owners, two concurrent real test
executables, strict focused lint and book checks pass. Historical residue is
unchanged. The slow compiler's native-loader sample is retained under .11.2;
all results completed naturally and were consumed. Production behavior is unchanged.

## 2026-09-09 — Wire complete CI commands through local stores (`SIGNOFF-REPAIR.11.4.3.1.3.3`)

All six workflow command jobs now use the local launcher. Require workers/Chrome,
all Python controls, the full owned PG demo and a pinned local book build; use the
verified scanner drivers and retain only their report/log allowlists. Actual
synthetic Gitleaks redaction, YAML/shell checks, five omission controls, fifty
Python tests and the rendered book pass. Full local/remote CI remains pending
after publisher/browser/cleanup prerequisites; qualification categories unchanged.

## 2026-09-09 — Verify pinned CI scanners before execution (`SIGNOFF-REPAIR.11.4.3.1.3.2`)

Add exact archive/version pins, bounded verified extraction and supervised
scanner setup/execution in unique local directories. Preserve redacted reports,
nonzero results and failed evidence; retire only consumed run executables/archives.
Thirteen controls, final configuration checks, all eight archive identities/layouts
and native version-only probes pass. The real Gitleaks probe caught an incorrect
expected version format; its root cause and failure/retry remain recorded.
Workflow wiring and actual full security gates remain pending.

## 2026-09-09 — Establish CI stores before installation (`SIGNOFF-REPAIR.11.4.3.1.3.1`)

Add a repository-local CI launcher with optional exact pinned compiler setup.
Clear documented ambient gate overrides; reuse owned process supervision for
installer failure, timeout and terminal cancellation before command dispatch.
Eight new and eighteen adjacent controls, final syntax/source identity and book
checks pass; results consumed, fixtures absent. Installer controls use real
processes with instrumented tools. Scanner setup and workflow wiring remain next;
no actual compiler download or remote-CI pass is claimed.

## 2026-09-09 — Census the scheduled CI checkpoint (`SIGNOFF-REPAIR.11.4.3.1.1`)

Record actual workflow/gate/target/skip/tool/locality and artifact-pressure scope.
Correct stale CI guidance; own workflow coverage/locality, publisher/browser
lifetimes and safe cleanup before broad execution. Repeated metadata/source
identity, missing/extra registration controls, unchanged policy values and nine
rendered book markers pass. No runtime gate, deletion, push or qualification
advance is inferred from this inventory.

## 2026-09-09 — Check bootstrap completion capacity before HTTP (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.3.1`)

Reproduce a real CLI dispatch whose pending snapshot fits but whose completed
receipt is one byte too large. Validate the full prospective snapshot with the
actual state codec before publishing intent or sending HTTP. Preserve maps and
saved request identity on refusal; never publish the private sizing sample as
an outcome. Thirty-one selected controls, final three-scenario boundary matrix
and strict CLI lint pass; all results consumed, unique fixtures absent. The
exact-limit positive control succeeds; physical disk reservation is not claimed.

## 2026-09-09 — Keyed CLI bootstrap and explicit recovery (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.2`)

Persist one bootstrap request before HTTP and reuse matching pending intent with
its original key/actions. Strictly validate complete original-byte replies;
publish principal/completion and pending cleanup under the same guard. Add
--resume-bootstrap for explicit pending/latest-completion recovery; historical
receipts recover locally without HTTP and label their source. Normal no-pending
invocation remains intentionally fresh. Thirty-three selected controls, final output rerun and strict lint pass; all
results/shutdown consumed and unique fixtures/owned cluster absent. Capacity and HTTP bounds,
integration reconciliation and broader restart qualification remain next.

## 2026-09-09 — Strict bootstrap recovery snapshots (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.1`)

Add version-two pending/completed recovery records with exact identity/outcome
validation and preserved legacy wire shape. Retain a writer guard across multiple
synchronized snapshots; refuse stale saves that lose pending intent and ordinary
CLI dispatch while recovery is pending. Twenty-four selected controls, strict
CLI lint and book checks pass; all results consumed, unique fixtures absent.
Preserve the interrupted pre-main launch and successful unchanged-binary retry.
Keyed enrollment, explicit resume, HTTP deadlines and restart qualification remain
next; this schema prerequisite does not claim integrated client recovery.

## 2026-09-09 — Hold CLI state across HTTP writers (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.1.2`)

Acquire and validate state before enrollment/thread-create dispatch; retain the
same guard through fresh actor selection, merge and synchronized publication.
Busy writers now send zero HTTP requests. Preserve explicit-principal API behavior
and unrelated mappings; qualify both overlap orders and process-loss release.
Nineteen selected controls, final strict CLI lint and rendered book checks pass.
All results/shutdown consumed, unique fixtures and the owned PostgreSQL cluster
absent. Replace fixed live fixture deletion/unbounded child waits with owned
lifetimes. Pending bootstrap identity, bounded HTTP waits and restart remain next.

## 2026-09-09 — Durable bounded CLI snapshots (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.1.1`)

Repair in-place publication, linked-target overwrite and ignored process locks.
StateFile now validates bounded snapshots and repository-volume paths, holds an
OS lock, synchronizes private working bytes and atomically publishes before
acknowledging durability. Preserve invalid/ambiguous files and report uncertain
replacement honestly. Twelve selected controls, all-target CLI strict lint and
book verification pass; results consumed, unique fixtures absent. Document the
remaining whole-writer and pending-bootstrap integration explicitly. No full CI
or push; live progress category values and bounded README remain unchanged.

## 2026-09-09 — Keyed server bootstrap recovery (`SIGNOFF-REPAIR.3.3.4.3.3.3.2`)

Persist a canonical request key and complete creation outcome with guarded
bootstrap. Matching retries recover original tenant/principal/boundary/grant IDs;
conflicts and malformed storage refuse. Concurrent losers roll back before one
guarded replay within the original deadline. Actual lost acknowledgment and
unconfirmed-commit recovery are qualified; no-key semantics remain unchanged.
All 73 selected controls, final eleven-control fixture rerun and strict lint pass;
every result/shutdown consumed, four owned clusters absent. Preserve identity FKs,
update their 24 named fixture purges, document API examples and CLI limitations,
and capture the director's semantic introspection proposal with an assessment
owner. The observed native trust-store fixture wait has an immediate local repair
and broader .11.2 follow-up. Durable CLI persistence is next.

## 2026-09-09 — Bootstrap uncertainty and recovery contract (`SIGNOFF-REPAIR.3.3.4.3.3.3.1`)

Qualify a real bootstrap commit timeout followed by the original committed
readback and a distinct repeated no-key request. All 25 selected controls (24 live
/ one pure), focused strict lint and book checks pass; every result/shutdown is
consumed and the owned cluster is absent. Select an explicit client RequestId,
immutable guarded outcome and durable CLI pending-state contract. Server protocol
and CLI recovery implementation remain the next owned children; production
behavior is unchanged in this qualification/contract leaf.

## 2026-09-09 — Complete guarded development enrollment (`SIGNOFF-REPAIR.3.3.4.3.3.2`)

Development enrollment now holds one exclusive tenant guard from replay through
boundary, grant, identity, quota and enrollment commit. Same-context authority
helpers use database time and reject non-live parents. Concurrent same-name
requests return one new principal and an honest replay; typed errors roll back
all provisional rows, and commit failures retain their unconfirmed HTTP phase.
The book preserves validation/replay and dev issuer limits, and corrects the CLI
role defaults. All 97 selected controls (96 live / one pure), final focused strict lint
and rendered book checks pass. Every result/shutdown is consumed; all three
owned clusters are absent. New-bootstrap response-loss recovery is tracked as
the next child, with its current client/operator limitation explicit in the book.

## 2026-09-09 — Typed rollback for guarded grant refusals (`SIGNOFF-REPAIR.3.3.4.3.3.1`)

A private typed-error entrypoint now aborts provisional guarded work while
preserving domain errors, SQL causes and commit uncertainty. Standalone grant
refusals roll back a new coordination anchor and cannot be replaced by its
deferred commit fault; pre-existing anchors remain intact. Deliberately committed
refusal values keep their existing contract. All 89 selected controls (88 live / one
pure), focused strict lint and generated book contract checks pass. Every result
and shutdown is consumed; both owned clusters are absent. Complete enrollment
integration is the next bounded child.

## 2026-09-09 — Guarded standalone authority writers (`SIGNOFF-REPAIR.3.3.4.3.2`)

Boundary/grant creation and grant/boundary revocation now share the exclusive
tenant guard; active-boundary lookup uses its shared mode. Issuance checks the
actual own-tenant parent at fresh database time after the guard/read, preserving
scheduled grants under live parents. Malformed target statuses now refuse with
unchanged evidence/epoch. Public Rust errors and safe HTTP
commit_outcome_unconfirmed preserve commit uncertainty. All 85 selected controls
(84 live / one pure), focused strict lint and book checks pass; matched races,
deferred faults, exact recovery and the corrected observed contention chain are
qualified. All results/shutdown consumed and three owned clusters removed.
Complete enrollment/import and administrative admission/effect coupling remain
separately owned; the book documents the Rust/error-code compatibility changes.

## 2026-09-09 — Distinct grant creation failures (`SIGNOFF-REPAIR.3.3.4.3.1`)

GrantCreateError now distinguishes missing parents, actual structural refusals and
original SQLx storage failures. Enrollment and card import return safe HTTP 500
for storage failure; malformed active-boundary data no longer panics the handler.
Actual ceiling violations retain their contextual HTTP 400 responses. The book
documents the public Rust return-type migration. All 56 live authority/HTTP/card
controls and focused strict lint pass, including exact failure snapshots and
recovery. All results/shutdown consumed and four owned clusters removed. Guard
integration and complete import transaction work retain their following owners.

## 2026-09-09 — Qualified tenant guard foundation (`SIGNOFF-REPAIR.3.3.4.2`)

Migration 0056 preserves all legacy authority/identity rows and adds full-tenant
coordination anchors. The private runner owns the connection before BEGIN and
keeps shared/exclusive guards through a bounded callback and commit. Matched
controls reproduce and repair cancelled-BEGIN pooling; commit acknowledgment
timeouts retain uncertainty even when PostgreSQL later commits. All four crates
embedding migrations now track directory additions/removals, correcting a cached
executable that omitted the new schema. The 28-command dependency probe passes;
final qualification passes 35 controls (34 live / one pure) and four-crate focused
strict lint. Seven checker controls also repair recursive acceptance-owner
selection without letting nested evidence replace a real task tree. All results
consumed, clusters and temporary probes removed. This
qualifies primitives; application authority/effect integration starts next.

## 2026-09-09 — Tenant transaction repair contract (`SIGNOFF-REPAIR.3.3.4.1`)

Trace 42 direct authority call locations across 101 tracked Rust sources and
cross-check transitive effects, authority-table/epoch mutations and alternate
permission gates. A dedicated full-tenant-key guard preserves standalone authority
namespaces without identity rows; shared/exclusive modes, lock order, fresh
evaluation time, transaction ownership and distinct final-effect evidence are
specified in thirteen bounded census/implementation children. Independent location
re-derivation and omission/invention controls pass. This commit changes documentation
only; guard behavior and runtime race qualification remain pending.

## 2026-09-09 — Tenant-scoped authorization receipt readback (`SIGNOFF-REPAIR.3.3.3.2.2.3`)

GET /v1/admin/authorization-records/{record_id}?tenant_id=ten_… returns one complete
own-tenant record after a separately audited inspection admission. Tenant/record
filtering precedes strict decoding; foreign and missing records share a generic
404, including malformed foreign evidence. Human/role frozen access, denial and
legacy readback, invalid authority/input, audit failure and malformed-own-record
recovery pass. All 18 live authority tests and 30 HTTP tests pass, including the
final denied-record readback control; strict lint passes. All results consumed,
clusters removed. The inspection selection/provenance/readback children are now
complete; tenant authority/effect serialization remains next.

## 2026-09-09 — Committed administrative inspection receipts (`SIGNOFF-REPAIR.3.3.3.2.2.2`)

Seven administrative reads commit explicit allow/deny admissions with their actual
principal, named purpose, parent status and grant scope. Responses carry
x-reasonbraid-authorization while successful bodies retain their shape. Audit
failure refuses admission without protected data or an unconfirmed receipt; a
later response-query failure retains its real committed receipt. Ordinary records
remain boundary_checked. All 45 live authority/API tests, ten pure evaluator tests
and strict lint pass; all results and cluster shutdown consumed. No schema change,
delivery guarantee or revocation serialization is claimed. Scoped receipt lookup
remains the next child.

## 2026-09-09 — Explicit audit evaluation provenance (`SIGNOFF-REPAIR.3.3.3.2.2.1`)

Authorization records add a closed evaluation object. Migration 0055 preserves
legacy rows as legacy_unspecified; new ordinary writers explicitly record
boundary_checked, including ordinary reads. Exact and thread-audit readback
refuse malformed evidence without guessing fields or panicking. Core JSON without
provenance remains readable; old strict consumers of newly serialized records
must upgrade. Map-only evaluation/inspection and shared selector decoders reject
discarded fields, duplicates and sequence alternatives while preserving valid
JSON and schema. Final code passes 51 core units, seven metadata/subject controls,
44 live authority/HTTP/upgrade tests and strict lint. The existing digest format
is unchanged. All results and shutdown are consumed; the owned cluster is removed.
REPAIR-0017 closes implementation 305ed26. Frozen-inspection HTTP receipts and
scoped exact receipt lookup are separately owned next.

## 2026-09-09 — Bound frozen-tenant inspection (`SIGNOFF-REPAIR.3.3.3.2.1`)

Seven administrative GET routes retain inspection through active, suspended or
revoked actual boundary status while enforcing parent/tenant/subject binding,
whole-grant ceilings, tenant-wide selectors and nonempty half-open validity.
Usable older grants survive newer ineligible candidates. Response shapes remain
compatible; normal writes retain their boundary-status gate. All 40 live
authority/command API tests, ten pure evaluator controls and strict focused lint
pass. All results consumed and the owned cluster removed. Explicit inspection
audit provenance remains the next owned leaf.

## 2026-09-09 — Actual-parent command authority (`SIGNOFF-REPAIR.3.3.3.1`)

Normal commands select usable caller/delegated grants through deterministic
32-row pages and each grant's actual parent. Requested delegation scope participates
in selection. Denial records preserve the authority source; absent sources carry
no grant or parent. Malformed stored candidates fail with a storage error. Six
pure evaluator controls, all 37 live authority/command API tests and strict
focused lint pass. All results are consumed and the owned cluster removed. Frozen-admin reads and revocation serialization remain separate repairs.

## 2026-09-09 — Verified changelog rotation (`SIGNOFF-REPAIR.11.4.1`)

Retain the ten recent corrective-review records and rotate 120 older entries
through the existing Git-history terminal. The exact predecessor and whole-record
segments reconstruct all 95,038 source bytes; the 96,000-byte cap is unchanged.
History retrieval and historical qualification limits are documented below.

## 2026-09-09 — Bound tenant authority (`SIGNOFF-REPAIR.3.3.2`)

Bind grants to their named parent, tenant and evaluated subject; enforce nonempty
half-open validity and action-target selector coverage. Thread-scoped grants cannot
administer or list an entire tenant. Core 51 unit + 3 subject tests, six evaluator
controls, 32 live authority/command API tests and strict core/server lint pass.
All results are consumed and the owned runner stopped/removed its cluster. Book and decision
record document the contract and remaining loader/transaction repairs.

## 2026-09-09 — Canonical core subject JSON (`SIGNOFF-REPAIR.3.3.1`)

Core human/role subjects now serialize as kind/id objects and reject malformed,
duplicate, unknown or mismatched input through an object-only parser. Public
command-envelope strings and split database fields retain their contracts; only
the generated schema description changes. Direct and enclosing payload failures
are reproduced and corrected. Core tests pass 49 unit + 3 integration controls;
strict core/server lint and all 40 live authority/command API/site-receipt
compatibility controls pass. The owned cluster stopped and was removed. ADR-009's
synthetic token-size comparison is withdrawn and the missing evidence is owned
by the delegation repair.

## 2026-09-09 — Site authority enforced on registry HTTP (`SIGNOFF-REPAIR.3.2.3`)

All seven adapter/region operations now require explicit site grants and use the
atomic authority/effect/audit service. Mutations require bounded reasons; success
bodies retain their keys with a committed audit header. Refusals distinguish
malformed input, authority, domain and storage failures. Invalid UTF-8 paths now
return typed JSON after a regression exposed the extractor bypass. Tenant
enrollment confers no site authority. The live HTTP controls pass all eight tests;
strict focused lint passes. The selected security run completed with 58 passes;
the final corrected HTTP/registry run completed with 12 passes. Both supervised
clusters stopped and were removed; REPAIR-0009 closes the implementation commit's
verification-pending record.

## 2026-09-09 — Protected site operator CLI (`SIGNOFF-REPAIR.3.2.2`)

Added `rb-site` for explicit boundary/grant issuance, disabling and audited
paginated inventory. It requires a selected loopback database, verifies storage
on the repository volume and uses explicit credentials without home lookup.
The runner now supplies a matching private synthetic passfile after source
inspection exposed SQLx's fallback from the former missing placeholder.
Validation: 16 focused CLI/service/ownership tests; strengthened CLI controls
repeated with 3 passes; 13 runner controls; final strict CLI/server lint, format,
script syntax and rendered book checks. HTTP enforcement remains `.3.2.3`.

## 2026-09-09 — Explicit site-authority service (`SIGNOFF-REPAIR.3.2.1`)

Added separate site boundaries/grants, protected database-session issuance and
disabling, and a registry service that commits effects with attributable audit.
Actual-parent liveness, scope/window ceilings, usable-grant selection and a shared
transaction guard fence site revocation; tenant enrollment grants no site rights.
Native libpq controls reproduced stale prepared role membership after a wait;
fresh text-protocol checks close that path. Ten live service controls and strict
focused lint pass. The book documents capabilities, examples and integration
limits. Operator CLI and HTTP enforcement follow as separate committed leaves.

## 2026-09-09 — Bind revocation to the authorized tenant (`SIGNOFF-REPAIR.3.1`)

Grant/boundary revocation now selects and locks only a target in the authorized
tenant before changing status and epoch. Foreign targets remain 404 with victim
state unchanged. Rejected repeats no longer increment the epoch again, and two
requests forced to contend on the same grant produce one transition/epoch bump.
Validation: both foreign-target defects and the repeated-epoch defect reproduced;
34 corrected API/authority/escalation tests passed; format and focused strict
Clippy passed. Final documentation gates run in the commit workflow. Atomic final-effect auditing and
administrator-authority serialization remain explicitly owned by `.3.3`.

## 2026-09-09 — Disposable ownership at test connections (`SIGNOFF-REPAIR.2.2.2`)

Server, CLI and MCP database fixtures now validate a live runner receipt and
verify server identity on every new pool connection before fixture SQL. Missing
ownership changes from a reproduced two-row write to refusal with zero public
tables. Restore CREATE/DROP uses verified connections; CI uses the same runner
and repository-local compiler stores. Active command receipts discard stale exits.
Validation: 40 tests across eight selected suites, including forged ownership,
replacement connections, restore, migration and RLS; format, book and CI syntax
checks passed. Strict all-target/all-feature Clippy passed with warnings denied;
final restore and three malformed/missing/absent-environment controls passed.

## 2026-09-09 — Supervised focused PostgreSQL verification (`SIGNOFF-REPAIR.2.2.1`)

The runner now creates unique owned clusters, ignores caller database targets,
checks server identity before creation, and runs named suites serially. It records
process/command receipts and logs, verifies shutdown before deletion, and retains
failure evidence. A reproduced spawn/signal race is closed by deferred signals
and an exec trampoline. Test-side refusal and CI wiring remain `.2.2.2`.
Validation: 12 lifecycle controls, 4 live PostgreSQL controls, 9 existing authority
tests, syntax checks, book build and the staged doctrine gate.

## 2026-09-09 — Repository-local command environment (`SIGNOFF-REPAIR.2.1`)

Added a launcher and Makefile integration for repository-derived Cargo, build,
temporary, XDG and CLI stores. Locked cache seeding verifies archives and index
copies without deleting shared sources. Installed tools remain read-only inputs.
Validation: five focused controls; offline locked metadata (496 local packages);
core tests 49 passed, 1 intentional ignored schema writer; book built; staged
doctrine gate runs in the commit hook.

## 2026-09-09 — Corrective ownership and site-authority decision (`SIGNOFF-REPAIR.1`)

Completed the required roadmap, tracked-codebase and mdBook read before editing.
Recorded the source census and bounded repair leaves, selected explicit site-operator
authority for shared registries, and corrected progress pointers and qualification
limits. The former live status carried 42,374 bytes in 18 lines and omitted separate
Phases 5–7/9 rows; its history remains in git and phase records. This is documentation
and design work; implementation and runtime reproduction remain pending.
Validation: mdBook built; 13 doctrine checks passed; diff whitespace clean; owner/phase omission controls detected. Runtime tests remain pending.

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. Retrieve the complete pre-rotation
ledger, including its earlier rotation notice, from the repository root:

```bash
git show 25ed7d184203e2d8701800558b785b30c75bb4d0:CHANGELOG.md
```

That snapshot contains 130 dated entries, including the ten retained above.
Its Git blob is `0bc51d581f9158ebafcef94cfb6717464722c6cb`; exact byte/line counts,
SHA-256 identities and transition evidence are in
`docs/decisions/2026-09-09_changelog-rotation.md`. Use
`git log --follow -- CHANGELOG.md` for earlier versions. Keep the reachable Git
history when cloning or handing off; a shallow checkout may need the named commit
before retrieval. A missing object is a retrieval failure, never evidence that
history was empty.

Historical success statements describe the recorded revisions and assertions.
Current qualification is in `LIVE_STATUS.md` and the mdBook's qualification review;
open repairs remain tracked in `docs/tasks/SIGNOFF-REPAIR.md`.
