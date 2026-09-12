# CHANGELOG.md

## 2026-09-12 — Check the configured endpoint on every verb (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.3.3`)

- **Measuring it made it worse than the note that routed it here.** The previous leaf recorded this as "an inconsistency" reasoned from source. The reproduction points an ordinary verb at `http://operator:secret@127.0.0.1:<port>` with an origin that records what it receives, and the origin recorded `POST /v1/enrollments HTTP/1.1` carrying **`authorization: Basic b3BlcmF0b3I6c2VjcmV0`** — base64 of `operator:secret`. The transport does not ignore URL userinfo; it converts it into credentials on the wire.
- The control asserts on what the ORIGIN received rather than on the client's account of its own refusal, because the whole claim is that nothing was sent. It also asserts the refusal does not echo the credential it refused.
- **Fix:** `ApiClient::for_base` canonicalises through the same `canonical_server` the bootstrap path has always used, and `ApiClient::new` becomes crate-private — so the only construction reachable from outside the crate is the checked one, and the unchecked one survives exactly where the base was already validated. All **22** `lib.rs` call sites migrated.
- A deliberate difference, stated so it is not read as an oversight: a live configured base is NORMALISED (an uppercase scheme is accepted) while a bootstrap record's stored server identity must already be canonical. Configuration input and a durable binding that a later recovery compares against are different things.
- Severity unchanged by the measurement, and stated plainly: the base is the operator's own configuration, so this is hardening, not a third-party escalation. It is worth doing because a credential silently leaving for whatever host the base names is a poor failure mode even when the operator typed it.
- Validation: the reproduction control fails on the unrepaired code printing the recorded Basic header, and passes after. The whole CLI crate passes `--all-targets` at rc=0 (12 library, 4 bootstrap-state, 5 end-to-end, 5 transport, 4 state-publication, 12 state-writers), with the status captured BEFORE any pipe. Gate (17 checks), book.

## 2026-09-12 — Bound the CLI's transport without losing its recovery key (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.3.2`)

- **The baseline is the defect stated precisely:** against an origin that completes the TCP handshake and then never answers, `run_enroll` **did not return within 90 seconds**. The control does not FAIL on unrepaired production — it does not RETURN, and an outer bound is what turns the hang into a measurement. A refused connection fails fast on its own; only a silent ACCEPTED connection can hold a caller open.
- Why it matters more here than in an ordinary client: the CLI persists a bootstrap request key BEFORE dispatch and holds the state lock across the response. Both are deliberate and correct. Unbounded, that design means one silent peer holds the store for the life of the process.
- Bounds: **connect 10 s**, **whole request 60 s** covering the response body, **reply 8 MiB** read through `Response::chunk` instead of `bytes()`, and **no redirect following**.
- **The 60-second ceiling is derived, not chosen by feel.** The server's own whole-operation budget is 15 s, and a legitimate caller can wait behind ANOTHER caller's operation before running its own — so the worst legitimate case is about 30 s and the bound is twice it. It exists for a peer that never answers, not to second-guess a server that is working.
- The reply ceiling is matched to the store's own 8 MiB limit rather than picked independently: a reply the store could never hold cannot become a published outcome, so reading it to the end buys nothing. It is a NAMED refusal, never a truncation — a prefix of a JSON outcome is not an outcome.
- **Redirects are refused** because the base is operator-configured and reqwest's default chases up to ten hops silently; a redirect would carry a request bearing the development principal header, and a bootstrap request key, to a host nobody named.
- **What survives a refusal is what recovery needs:** the ORIGINAL key, a genuinely released exclusion, and a key that is reused rather than replaced on the next attempt — a new key would be a second logical bootstrap against a server that may already hold the first. A timeout says nothing about whether the server committed, which is exactly why the key is retained.
- Falsified twice, each reverted: removing the redirect policy makes that control fail with a transport error from the CHASED target instead of the configured endpoint's 302; restoring `bytes()` makes the oversized-reply control fail at the decoder rather than refusing the read — the precise difference the bound makes.
- three repeats plus one run under deliberate saturation. Repeat 1 FAILED — `the refusal took 75.032698709s` against a 60-second bound, with a clippy running alongside, because the assertion was `< 75s`. That failure IS the evidence for the margin: wall clock contains the runtime scheduling as well as the deadline, so a margin tight enough to separate 60 from 75 measures the host, not the client, and fails wherever saturation is normal. The margin was widened to 100 s, the declared 60 s value pinned deterministically against the number the book documents, and the reply control timing assertion replaced with a semantic one. Repeats 2 and 3 then passed, and the corrected controls passed again with all 12 cores deliberately saturated (load average 1.86 → 14.15, 134.46 s, 4 of 4) — heavier load than produced the original failure. A first attempt at that loaded run was discarded rather than counted: its clippy was cached, checked one crate and loaded nothing.
- Cost stated rather than hidden: the two deadline controls each wait out the real 60 s bound, adding about 130 s to a CLI test run. The bound is not lowered to make tests faster, and the controls are not `ignore`d — a gate that does not run in CI is how the defect returns.
- One census routed out rather than folded in: `canonical_server` guards only the bootstrap path while `ApiClient::new` has 22 unvalidated call sites, so **1 of 23** construction paths checks the configured endpoint. Owned by `.3.3.4.3.3.3.3.2.3.3`; closing it changes a constructor signature at 22 sites, and the input is the operator's own configuration.
- Validation: 4 transport controls, the whole CLI crate at `--all-targets` (12 library, 4 bootstrap-state, 5 end-to-end, 4 state-publication, 12 state-writers), strict all-target CLI lint, format, gate (17 checks), book.

## 2026-09-12 — Gate the tree index against its own trees (`SIGNOFF-REPAIR.11.4.5.3`)

- The third of this session's self-inflicted defects, and the one with the sharpest root cause. `docs/TASK_TREE.md`'s Frontier column is a SECOND copy of a fact each tree owns, nothing derived it, and `--against` over the last 60 commits finds **39 breaching and 21 genuinely agreeing** — with the drift beginning at `1ebfebe`, **the very commit that closed the leaf the row then kept naming for 39 commits**. The row was written from the leaf just completed rather than the next one, and a successful commit is precisely when it went stale.
- `BOOK-FRONTIER` fixed this exact shape for the book and was simply not extended to the project's own index. This is that extension.
- **The census rejected BOTH options the leaf had provisionally written down**, which is the reason it was required before any rule. Blanket "the cell equals row 1" is unsound: 8 completed trees write a dash row and point their cell at the NEXT tree's first leaf, `PHASE-8`'s row 1 is the cross-tree prerequisite `SIGNOFF-REPAIR.3.3` while its cell accurately summarises rows 1 and 2, and 2 rows claim no leaf — equality would flag legitimate rows. And the **generator**, which the leaf preferred, would destroy accurate curated prose in 13 of 14 rows.
- What survives is narrow and honest: an ACTIVE tree whose row 1 names a leaf OF THAT TREE must be named by its index cell. That is one row today — and it is the only row that moves. The rule extends itself the moment another tree becomes active.
- **The gate refused this very commit, on its first live run.** Closing `.11.4.5.3` moved the tree's row 1, and the index still pointed at the leaf being closed — the identical shape as the original defect. The LOCKSTEP box had already claimed the index was deliberately not restaged; that claim was wrong, and it is corrected in place rather than deleted.
- Validation: `--self-test` (numbered row 1 extracted, a completed tree's dash row correctly yields none, 4 cell shapes normalised including the dot-shorthand and the prerequisite cell), `--against 300de41` fails naming both leaves, the current tree passes, gate at 17 checks, book.

## 2026-09-12 — Gate a LOCKSTEP box against the commit it claims (`SIGNOFF-REPAIR.11.4.5.2`)

- The half of this session's self-inflicted pair that was recorded but not fixed is now fixed. A ticked `**LOCKSTEP**` box is a claim about ITS OWN commit, and nothing read it: `6bf0c40` shipped a box naming `MEMORY.md` and `LIVE_STATUS.md` and staged neither, after a scripted multi-edit hit a failed assertion and `git add -A && git commit` ran regardless.
- **`TASK-ACCEPTANCE` cannot catch this and never could.** It proves the box is ticked and cites something re-runnable; it cannot prove the cited edit landed. `LOCKSTEP-CLAIM` is the complementary half, and the enforcer now runs 16 checks.
- **Keyed on the author's own claim, because the census killed the obvious rule.** Over 25 commits, 10 closed a leaf and `CHANGELOG.md` was staged 10/10 but `MEMORY.md` only 6/10 — a blanket "closing a leaf must stage MEMORY" would have asserted something the project does not do, flagged four innocent commits, and still missed `6bf0c40`, which did stage `CHANGELOG.md`.
- The claim shape is the BOLD bullet, so narrative that reports on a past box — `c6a843f`'s own "its LOCKSTEP box overstated" lines — is a report, not a claim about this commit, and is deliberately not matched.
- The honest escape follows the `GAP-CLAIM-CENSUS` precedent instead of parsing intent out of prose: `lockstep: <doc> unchanged (<why>)` in the claim's own heading section. The refusal says explicitly that **deleting the document's name is not a discharge** — the name is the claim, and removing it hides the decision rather than recording it.
- `--against <sha>` re-runs the predicate over any past commit, which made the leaf's acceptance literally executable: `--against 6bf0c40` fails naming both documents and the exact line, `--against c6a843f` passes.
- **Re-measured with the finished gate, over a wider window than the rule was designed on:** across the last 30 commits, 11 add a claim and were genuinely exercised — 10 pass, exactly one breaches, zero false positives. The other 19 pass vacuously because they add no claim, and that is recorded so "30 green" is not mistaken for 30 tests.
- The document list is COMMIT.md's root live documents, and the `--self-test` re-checks that each still appears there, so the list cannot drift away from the document it was taken from.
- Validation: `--self-test` (2 claim shapes, 2 narrative lines ignored, 3 naming cases incl. a nested path and a longer filename, the escape both ways, 5 documents confirmed in COMMIT.md), the two acceptance runs, gate at 16 checks, book.
- Sequencing note: `README-STABILITY` refused this commit first, because the ledger had crossed its rotation threshold. The rotation is `DOC-0006` (leaf `.11.4.1.1`), committed separately and immediately before this one so neither commit carries the other's scope.

## 2026-09-12 — Admit the ranked pack's advertised media types (`SIGNOFF-REPAIR.7.3.3.5.2`)

- The repair the census chose. The R2 arm acquired through `Fetcher::fetch` and inherited R0's text-only accept set, while `resolvers::resolve` ranked the pack on an advertisement that accept set cannot satisfy. The pack exists for non-text documents in a sandboxed worker; the shared fetcher was one engine reused, not the architecture.
- Three pieces: `sniff_kind` takes the ranked pack's advertised types and returns the additive `SniffedKind::DeclaredType` for a declared type in that set; `Fetcher::fetch_admitting` supplies it while `fetch`, `fetch_head` and `fetch_authenticated` pass an empty slice; `resolvers::advertised_media_types` reads the row's own `media_types` **per resolution**, so a deployment that narrows a pack's advertisement narrows what it may acquire in the same act. A missing or malformed row yields an EMPTY set — a bad advertisement must never widen a gate.
- **The bounds are censused, not asserted.** `git grep -n "fetch_admitting" -- 'crates/**/*.rs'` returns exactly one call site, the R2 arm at `api.rs:2032`. The R0 arm, the R5 authenticated arm and the R3 preflight pass no admitted set. No destination policy, scheme list, SSRF control, byte ceiling, ratio brake, redirect policy or time ceiling changed.
- **And proved live.** Three injections, each reverted and re-run green: an admitted set that is not the ranked row's fails the advertised-type acquisition; an empty set at the call site fails it identically, so the repair is load-bearing; and adding ONE unadvertised type (`application/json`) makes the unadvertised document acquire and fails the negative control — the decision's bound, demonstrated rather than claimed.
- `SniffedKind::DeclaredType`'s `as_str` is `application/octet-stream` and cannot be reached through this path: the variant is only produced FROM a declared type, so the snapshot's `content_type.unwrap_or_else(sniffed)` fallback does not fire. The live control asserts the snapshot records `application/atom+xml`, not a stand-in.
- **One measured fact is deliberately flipped and kept.** `.7.3.3.4.1`'s control recorded `application/atom+xml` as `media_type_refused`. That refusal was the defect, so the control now asserts the acquisition and a new `application/json` path carries the negative case — but the superseded measurement stays recorded, because it is the evidence the repair rests on.
- Validation: profiles 33/33 live with its cluster removed (R0 and R1 resolver controls unchanged in the same run), 99 library tests, 23 extraction controls, strict all-target server lint, format, a whole-workspace `cargo check --all-targets` at exit 0, gate (15 checks), book.

## 2026-09-12 — Census the acquisition leg's accept set, and decide (`SIGNOFF-REPAIR.7.3.3.5.1`)

- The finding `.7.3.3.4.1` surfaced got its census before it got a repair, and **the census changed its shape** — which is the entire reason the leaf was decomposed rather than implemented.
- **Declared content type:** all five types migration 0027 advertises for `r2-extract-worker` are refused, and the complete accepted set is `text/html`, `application/xhtml+xml` and any `text/*`. Three arms, enumerated in both directions, so a fourth added later fails the census instead of passing unnoticed.
- **No content type:** the verdict is a property of the BYTES, not of the format. An all-printable body is accepted whatever format it belongs to; the same body with one non-text byte is refused. ZIP and tar cannot reach that branch at all, structurally — a ZIP local file header is `PK\x03\x04` plus nine little-endian integer fields, and a tar header is a 512-byte block with a NUL-padded 100-byte name.
- **So "the five advertised formats are unacquirable" is true but the wrong SHAPE.** The leg's rule is not about formats, which means no subset of the advertisement satisfies it. **Narrowing the registry row is rejected on that measured ground** — it cannot express the truth, and narrowing to what the leg does accept would advertise `text/*`, which is the R0 pack's own row.
- **Decision:** the acquisition leg admits the RANKED resolver's own advertised media types, read at resolution time. Explicitly outside the change: the destination policy, the scheme list, every SSRF control, any type the ranked pack does not advertise, R0-ranked acquisitions, and the byte/ratio/redirect/time ceilings. The risk it accepts — bytes that were never fetched now reaching a parser — is stated rather than buried, and `.7.3.4` still owns pipe bounds and descendant containment.
- No production behaviour changed: the leaf's own acceptance forbids it until the census exists, and the census is the deliverable. `.7.3.3.5.2` implements the repair.
- The controls pass on unchanged production, as a census must, and were falsified by adding `application/pdf` to the accept set — which fails the advertisement control and leaves the untyped one untouched, because the two measure different things.
- Stated limit: the census measures the PREDICATE, not real files. A given real PDF's untyped verdict depends on that PDF and is not claimed; the ZIP and tar statements are structural facts of those formats applied to a measured predicate.
- Validation: 19 fetcher controls, 97 library tests, strict all-target server lint, format, gate (15 checks), book.

## 2026-09-12 — Prove the R2 mismatch refusal persists nothing (`SIGNOFF-REPAIR.7.3.3.4.2`)

- The sibling join, and **no production change**: the gap was coverage. The seven mismatch controls `.7.3.3.3.2` added all call `extract_acquired_bytes` directly and never reach a database — `git grep -n "PgPool\|sqlx" -- crates/reasonbraid-server/tests/extraction_input.rs` returns nothing. They prove the refusal is RAISED; nothing proved the handler HONOURS it.
- A dishonest worker, injected through the `R2_WORKER_BIN` override the spawner already reads, returns a **well-formed** reply describing bytes the request never supplied — well-formed deliberately, because a malformed one is refused by the parser and never exercises the digest binding at all. The handler refuses it `extraction_source_mismatch` naming both digests.
- **The absence is asserted three ways, because each permits a different defect on its own:** zero `evidence_snapshots` for the exact reference; zero `derivations` joined to it through `parent_snapshot_id`; and the whole store's snapshot and derivation counts unchanged across the request, which catches a row written under any reference.
- **The decisive falsification kept the refusal and moved persistence in front of it.** The `extraction_source_mismatch` assertion still PASSED; the count failed with `left: 1`. That is the evidence that this control measures the absence rather than the error kind — a control asserting only the kind would have been green on a handler that persisted a foreign document and then complained about it. The second injection removed the digest binding entirely and failed with the stub's own `another document` chunk visible in the receipt.
- `git diff --quiet -- crates/reasonbraid-server/src/` confirms production source is byte-identical to REPAIR-0095 after both injections were reverted.
- `.4.1`'s control is refactored onto the helpers this needed, assertions unchanged. The parent `.7.3.3.4` is complete.
- Also opened: `.11.4.5.3`. `docs/TASK_TREE.md`'s frontier column had drifted to a leaf closed on 2026-09-11 — the same defect class as this session's other two. A census run before claiming it: `git grep -n "TASK_TREE" -- scripts/ .githooks/` hits five files, and classifying all five leaves **zero** reading that column (one asserts the file exists; the rest are a seed script, a README fragment, a comment and a spine file list). The row is corrected; the per-row census and the generate-or-check decision are the leaf's.
- Validation: profiles 33/33 live with its cluster removed, strict `-D warnings` all-target server lint, workspace format, gate (15 checks), book.

## 2026-09-12 — Drive an R2 acquisition to its persisted evidence (`SIGNOFF-REPAIR.7.3.3.4.1`)

- The gap `.7.3.3.3.2` stated rather than implied is closed: a SUCCESSFUL R2 acquisition now runs through the real HTTP handler to its snapshot and derivations, and the persisted evidence is asserted against the bytes the origin served — not against a 200.
- **The seam relaxes nothing.** `ApiState::with_acquisition` takes the R0 fetcher a deployment's acquisition legs use; `with_gate` delegates to it with the same `Fetcher::new(FetchLimits::default())` it built inline before, so `new`, `with_gate`, `api_router` and `api_router_gated` are behaviourally unchanged and every construction inside the server binary still gets the shipped https-only public-destination policy. A caller wanting another policy writes it in its own source.
- **Three deployments, one reference**, so each production gate is measured on its own instead of bundled into a single pass: the shipped state refuses `scheme_not_allowed`; the origin's scheme under the SHIPPED destination policy refuses `destination_refused` naming `loopback`; the origin's scheme under a loopback-admitting policy acquires. Then the readback: the snapshot's `raw_digest`, `byte_length`, resolver id and locator, the derivation rows' content AND digests, and zero snapshots for the refused reference.
- **The control was made to go red three times**, each injection reverted and re-run green. Serving ONE extra byte fails the digest assertions — and its chunk digests were *identical*, so the derivation leg alone would not have caught it; that is why both legs are present. Neutering the seam so it discards the supplied fetcher fails at the destination gate, proving the seam is what makes it pass. Persisting a derivation that is not the worker's chunk fails at the derivation assertion and nowhere earlier.
- **A design correction found by reading, not assumed:** `resolvers::resolve` filters on the registry row's `schemes @> [reference.scheme]`, and migration 0027 declares `["https"]`, so an honest `http` reference ranks no R2 pack. `resource_references.scheme` is caller-supplied and unvalidated against the locator — claiming `https` for an `http` locator would have made the control pass on false data. The registry row is a deployment fact and is configured as one: the baseline is asserted, widened for the three resolves, and restored before anything is asserted.
- **The finding this surfaced, measured rather than inferred, with an owner rather than a note (`.7.3.3.5`):** the R2 pack advertises five media types its own acquisition leg refuses. The R0 sniff accepts a declared content type only for `text/html`, `application/xhtml+xml` and `text/*`, so the identical feed succeeds served as `text/xml` and is refused `media_type_refused` served as `application/atom+xml`. A caller is ranked onto a resolver that cannot acquire its document and gets an acquisition refusal instead of the `resource_unresolvable_now` the §12.2 contract reserves for "no eligible resolver" — a wrong answer, not a missing feature. What an UNTYPED response of each format sniffs to is **not** measured and is **not** claimed; the per-format census comes first, then the decision to narrow the registry row or teach the leg the format set. Neither changes before that census exists.
- Validation: profiles 32/32 live with its cluster stopped and removed, 97 server library tests, 16 completion controls, seven owned-input controls, strict `-D warnings` all-target server lint, workspace format.

## 2026-09-12 — Fix the two defects this session's own work introduced (`SIGNOFF-REPAIR.11.4.5`)

- The director asked whether the two defects recorded in REPAIR-0092/0093 were being FIXED or merely written down. They were written down. Both are in the enforcement spine itself — the driver that runs on every commit and in CI — so both get a leaf.
- **Fixed (`.1`, REPAIR-0094):** the doctrine registry is data and must not be executable. Each entry is a bash double-quoted string, so backticks in a description are command substitution — the driver **executes** them, on every commit and in CI. REPAIR-0092 removed the instance and censused the rest but left the trap armed, and every other description in `DOCTRINE_ENFORCEMENT.md` uses backticks, so copying one in is the natural next move. A self-guard now reads the driver's own source text and refuses a backtick, `$(`, `${` or bare `$` in the registry block, **before the array is assigned** — the only placement that prevents the execution rather than reporting it afterwards. Falsified by reintroducing the exact original text.
- **Tracked, not yet fixed (`.2`):** a LOCKSTEP box may not claim a document the commit does not touch. The census is done and it disciplined the rule: over 25 commits, 10 close a leaf, and `CHANGELOG.md` is staged 10/10 but `MEMORY.md` only 6/10 — so a blanket "closing a leaf must stage MEMORY" would assert a rule the project does not follow, flag four pre-existing commits, and still miss the defect. The rule that holds is keyed on the author's own claim: of 6 commits adding a LOCKSTEP bullet naming a live document, exactly **one — this session's own `6bf0c40`** — named one it did not stage. Zero false positives. It sits at frontier row 2 with its acceptance and its known false-positive shape recorded.
- Also corrected: the frontier table's numbering had drifted to 1,2,5,6,7,8 after an earlier row removal in this session; it is rewritten wholesale.

## 2026-09-12 — The remote run is the authoritative pre-push gate (`SIGNOFF-REPAIR.11.4.3.1.2.28`)

- Both levers left open by the cost model are decided. **The macOS Developer Tools setting is rejected**: it exempts everything the shell runs from Gatekeeper assessment, on a project that deliberately executes untrusted content — the browser worker renders arbitrary pages, the extraction worker parses hostile documents — and it cannot be committed, so no other machine, fresh clone or runner would inherit it. A performance fix that lives in a system preference silently falsifies the published cost model for everyone else.
- **The volume move is rejected on measured capacity**, not on deference: `git count-objects -vH` puts the tracked content and full history at 3.77 MiB, but `du -sh target` is **185 GB** and §13 makes the build tree follow the checkout. The boot volume has 249 GB free, so it would take 74% of what remains. Note too that §13 requires project data on *the repository's own* volume, not a particular one — there was no policy defect to repair.
- **The lever actually pulled was on neither list: the remote run becomes the authoritative pre-push gate.** Verified from the workflow files rather than from prose — all eight checkpoint commands run remotely and **two run more strictly** there (the check job asserts the worker executables exist; the book job pins and asserts mdBook 0.5.4). It is a superset, and the remote is where the defects that escape have lived: all six in the recent remote-CI sequence were invisible locally.
- Nothing is removed, weakened, reordered or skipped — the change is *where*. Before a push, four cheap gates run (`make gate`, `make book`, `cargo fmt --all -- --check`, the Python controls); none links or executes a new binary, so none pays the ~21.9s constant. The full checkpoint stays available as a deliberate diagnostic with its >2h cost stated.
- The trade is named rather than hidden: gate latency is now bounded by the ~300-commit push cadence. That cadence is the director's standing instruction and is **not** changed here. Recorded for whenever it is revisited: GitHub-hosted runner minutes are free for public repositories.
- Also corrected here rather than left standing: `.11.4.4`'s LOCKSTEP box overstated — two scripted live-doc edits failed their anchor assertions while the commit proceeded, so `MEMORY.md` and `LIVE_STATUS.md` did not carry that leaf when its checklist said they did. Both are fixed, and `docs/ci.md`'s stale "the complete checkpoint still needs to pass before public push" is reconciled with the new gate section it contradicted.

## 2026-09-12 — Gate a leaf's status, and Markdown's heading ceiling (`SIGNOFF-REPAIR.11.4.4`)

- Five leaf sections asserted two different statuses, because the convention that grew was to APPEND a closing status and leave the opening line untouched. The first line is therefore stale by construction, and any first-match reader takes it — which is why `.7.4.1` sat at frontier row 2 for seven commits after it closed. 21 registered doctrine checks, **0** of which read a leaf's status line.
- The census that found them found a second defect nobody suspected: the tree encodes hierarchy in heading DEPTH, and **an ATX heading stops at level 6**. `#######` is a paragraph that starts with hashes, in CommonMark and GFM alike. **29 lines at levels 7–11**, whose content belongs — for any heading-aware reader, including this project's own generators — to the nearest real heading above them. The first census reported one section holding 18 status lines; it held one, and had swallowed five children.
- Fixed without deleting a word: the opening line becomes `- Opened:`, which is what it always meant, and every over-deep heading is capped at `######` because the leaf id already carries the depth — further than six levels ever could.
- Both rules are now **enforced rather than remembered**: `TASK-STATUS` and `HEADING-DEPTH`, each fence-aware, each with a two-sided `--self-test`, both falsified against the unrepaired tree before being trusted. The enforcer now runs 15 checks.
- A defect in this leaf's own work, caught by reading the enforcer's output: backticks inside a bash double-quoted registry string ran as command substitution, so the enforcer printed `line 36: pending: command not found` and the backticked words vanished from its description. Fixed, with a census confirming no other row carries one.

## 2026-09-12 — Decompose the R2 acquisition coverage gap (`SIGNOFF-REPAIR.7.3.3.4`)

- The leaf needed two unrelated fixtures — a destination-policy seam for the success path, a dishonest stub worker for the mismatch refusal — so it became `.4.1` and `.4.2` before any implementation, per the tree's execution contract.
- The design is established from measured source facts rather than left to the implementer: `FetcherConfig` and `Fetcher::from_config` are already public; `classify` returns through `allow_ip` for an IP-literal host **before** any resolver call, so a loopback origin needs only a policy plus `schemes: ["http"]` and its port; the R2 format set makes a minimal Atom feed the cleanest served document; and `extraction.rs` already reads `R2_WORKER_BIN` for the mismatch injection. The only missing piece is a seam on `ApiState`, whose `new`/`with_gate` hard-code the production `Fetcher::new`.
- No production rule is relaxed by either child: the seam constructs a different fetcher for a test profile and leaves the shipped https-only public-destination policy untouched.

## 2026-09-12 — Where the checkpoint's missing hour goes (`SIGNOFF-REPAIR.11.4.3.1.2.15`)

- The full checkpoint's `02-check` took 3,922s and reported 940s of it. The other 2,982s is now measured: **macOS first-execution validation of each newly written executable on the repository volume, ~21.9s apiece**, paid once per file identity and cached afterwards. The same bytes cost ~0.15s on the boot volume — about 150x, across four pairs with a spread under 0.8s.
- The cost is **fixed, not size-proportional** (81.6 MB costs 21.25s; 2.3 MB costs 21.6s), the process is blocked throughout (`user 0.00 sys 0.00`), only Apple's own XProtect/syspolicyd stack is present, and `com.apple.provenance` is kernel-managed so there is no file-level lever.
- **Two hypotheses were refuted by measurement rather than by argument.** The leaf's own leading candidate — nine rustdoc doctest-harness builds — is at most 8% of the run. And the duration is not inherent to the gate: the same four commands on the Linux runner, from a COLD checkout, take **444s with 18.7s (4.2%) unaccounted**, compiling 514 crates in less time than this machine compiles 12 warm.
- The number to plan with: **one more integration-test file costs about 22 seconds of every future checkpoint here**, whatever it tests.
- `scripts/measure_check_phases.py` is tracked, so this is re-derivable by one command instead of inferred from cargo's summaries. It also splits the doc/non-doc phases `cargo test --all` hides.
- Two levers exist and are deliberately NOT taken, because both are the director's and neither is a change this repository can make in its own sources: a macOS security setting, and the repository's volume — which is the first evidence that §13's storage-locality policy carries a large, previously invisible cost.
- No gate was weakened, skipped or reordered; the phase split runs strictly more than `make check` does. Recorded as `docs/decisions/2026-09-12_checkpoint-cost-model.md`, with evidence in `docs/tasks/artifacts/signoff_review/checkpoint-wall-time.md`.
- This unblocks `SIGNOFF-REPAIR.11.5`, which made explaining the gap the gate on every new verification lane.

## 2026-09-12 — A render refusal survives an unconfirmed cleanup (`SIGNOFF-REPAIR.11.4.3.1.2.27`)

- The browse worker returned ONE `kind` for two independent facts. When cleanup could not be confirmed it wrote `browser_cleanup_unconfirmed` into `kind` and kept the render's own kind only as a prefix inside the human message — so a caller reading `kind` lost the fact it had to act on (its own budget, its own selector) and received an operator's fact about a stray process instead.
- That is why the navigation-deadline control passed on the runner and failed here: it asserted `kind == "time_budget_exceeded"`, and the value of that field depended on whether the host was slow enough for a detached browser helper to outlive the ten-second cleanup budget. The control was not flaky — it was asserting a conjunction nobody intended, and it was reporting a real product defect.
- Two corrections to the finding as filed, both re-derived: the `group_cleanup_confirmed: true` / `cleanup_error: null` receipt quoted in it is the FIXTURE's, about the worker's own group, not the worker's receipt about the browser; and "it fails identically warm" did not hold — the unchanged control passed here in 31.18s. The trigger is load-dependent, so the repair deliberately does not depend on observing it.
- **Eleven** distinct render kinds were being relabelled, not one. The budget is simply the kind that exposed it.
- Fix: a pure `settle(render, cleanup)` combinator, plus `cleanup_confirmed` and an omitted-when-absent `cleanup_error` on the error object. `kind` now names the render's own outcome; the cleanup fact rides beside it. The one corner where the old replacement was doing real work is unchanged — a SUCCESSFUL render under unconfirmed cleanup stays `browser_cleanup_unconfirmed`, because it has no failure of its own to name and must not read as complete while a browser may still be running.
- Reproduced DETERMINISTICALLY instead of waiting for a slow host: an injected script forks a child, calls `setsid` to leave the browser's process group, and holds the inherited stderr, so no EOF arrives — the exact shape the desktop-runtime diagnosis proved for `chrome_crashpad_handler` and `GoogleUpdater`. Twelve seconds, any host, no browser. Against unchanged production it reproduces the reported symptom; against the repair it passes.
- Verified: 25 browse controls pass (8 unit + 17 integration, 0 skipped) against the pinned Chrome for Testing runtime; both strict lints and workspace format pass. Falsified both ways — restoring the superseded collapse turns all three new controls RED while the six unrelated ones stay green.
- The escaped-writer condition itself remains real and unrepaired. It is now reported honestly rather than overwriting the caller's result.
- Recorded as `docs/decisions/2026-09-12_browse-refusal-carries-two-facts.md`; promoted as `docs/knowledge/one-field-cannot-carry-two-facts.md`.
- Also found while updating the frontier, and routed rather than fixed inline: a leaf's first `Status:` line can be contradicted by a later one in its own section — `.7.4.1` sat at frontier row 2 seven commits after it closed. Owner `SIGNOFF-REPAIR.11.4.4`.

## 2026-09-12 — Remote CI is green for the first time

- Run 34652116508 for `c17841c`: `rust` success with all three jobs green (`book`, `check`, `pg-tests`), alongside `doctrines` and `supply-chain`. The `check` job reports 669 tests passed and 0 failed suites.
- Verified three ways: re-derived from the API rather than a notification, falsified for hidden skips (the one skipped step is `if: failure()` and correctly does not run on a green build), and durability confirmed with `origin/main` still at `c17841c`.
- It closes a repair sequence in which every push cleared a real defect and exposed the next, none a repeat: `ETXTBSY`, the 108-byte `sun_path` limit, inode reuse, `AuthorMissing`, a `pg_guard` check-then-act, and a conformance stub race.
- **Push cadence returns to ~300 commits**, per `COMMIT.md`. The exception that made each push a measurement no longer applies.
- The clean-state lane's first full local run immediately found its own defect, and it is the mirror image of the others: a browse navigation-deadline control that passes on the runner and fails here, reporting `browser_cleanup_unconfirmed` where it expects `time_budget_exceeded`. Owned by `SIGNOFF-REPAIR.11.4.3.1.2.27`, unrepaired and not claimed as fixed.

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

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated twice; each
rotation names the commit holding the ledger immediately before it, so the chain
walks back without guessing.

Retrieve the ledger immediately before the SECOND rotation (2026-09-12) from the
repository root:

```bash
git show f75106915ff1f1171b332451257388905b05d815:CHANGELOG.md
```

That snapshot is 93,956 bytes and contains 91 dated entries — the 61 retained
above plus the 30 rotated out of it, the newest of which is
`2026-09-09 — Census the scheduled CI checkpoint`. Its Git blob is
`b9aacfc467e1729cae1a5e76fe1d0adee6398c1d`.

That snapshot in turn carries the FIRST rotation's notice, which names the
pre-rotation ledger before it:

```bash
git show 25ed7d184203e2d8701800558b785b30c75bb4d0:CHANGELOG.md
```

That earlier snapshot contains 130 dated entries. Its Git blob is
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
